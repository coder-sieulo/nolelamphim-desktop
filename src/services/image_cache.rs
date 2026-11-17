use directories::ProjectDirs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;
use std::path::PathBuf;

pub struct ImageCacheService {
    cache_dir: PathBuf,
    client: reqwest::Client,
    enabled: bool,
}

impl ImageCacheService {
    pub fn new(enabled: bool) -> Self {
        let cache_dir = ProjectDirs::from("com", "nolelamphim", "desktop")
            .map(|dirs| dirs.cache_dir().join("images"))
            .unwrap_or_else(|| PathBuf::from("./.cache/images"));

        if enabled {
            let _ = std::fs::create_dir_all(&cache_dir);
        }

        Self {
            cache_dir,
            client: reqwest::Client::new(),
            enabled,
        }
    }

    fn url_to_filename(&self, url: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        let ext = if url.ends_with(".webp") {
            "webp"
        } else if url.ends_with(".png") {
            "png"
        } else {
            "jpg"
        };

        format!("{:x}.{}", hash, ext)
    }

    pub fn get_cached_path(&self, url: &str) -> Option<PathBuf> {
        if !self.enabled || url.is_empty() {
            return None;
        }

        let file_path = self.cache_dir.join(self.url_to_filename(url));
        if file_path.exists() {
            return Some(file_path);
        }

        // Check alternate jpg/webp file if transcoded or cached under alternate extension
        let mut alt_file = file_path.clone();
        if file_path.extension().and_then(|s| s.to_str()) == Some("webp") {
            alt_file.set_extension("jpg");
            if alt_file.exists() {
                return Some(alt_file);
            }
        } else if file_path.extension().and_then(|s| s.to_str()) == Some("jpg") {
            alt_file.set_extension("webp");
            if alt_file.exists() {
                return Some(alt_file);
            }
        }

        None
    }

    pub async fn fetch_image(&self, url: &str) -> Result<PathBuf, String> {
        if !self.enabled || url.is_empty() {
            return Err("Image caching disabled or empty URL".to_string());
        }

        if let Some(existing) = self.get_cached_path(url) {
            return Ok(existing);
        }

        let target_path = self.cache_dir.join(self.url_to_filename(url));

        tracing::debug!("Downloading image to cache: {}", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Network error downloading image: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP error {}: {}", resp.status(), url));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {}", e))?;

        let _ = fs::create_dir_all(&self.cache_dir);

        // Transcode and downsample to max 720x1080 to prevent huge 2000x3000 files from eating RAM and disk
        if let Ok(dyn_img) = image::load_from_memory(&bytes) {
            let resized = if dyn_img.width() > 720 || dyn_img.height() > 1080 {
                dyn_img.thumbnail(720, 1080)
            } else {
                dyn_img
            };
            let mut jpg_path = target_path.clone();
            jpg_path.set_extension("jpg");
            if resized.save_with_format(&jpg_path, image::ImageFormat::Jpeg).is_ok() {
                return Ok(jpg_path);
            }
        }

        fs::write(&target_path, &bytes)
            .map_err(|e| format!("Failed to write cached image to disk: {}", e))?;

        Ok(target_path)
    }

    pub async fn fetch_and_optimize(&self, url: &str, max_w: u32, max_h: u32) -> Result<PathBuf, String> {
        if !self.enabled || url.is_empty() {
            return Err("Image caching disabled or empty URL".to_string());
        }

        let target_path = self.cache_dir.join(self.url_to_filename(url));
        let mut jpg_path = target_path.clone();
        jpg_path.set_extension("jpg");

        // 1. If optimized jpg already exists, return immediately
        if jpg_path.exists() {
            return Ok(jpg_path);
        }

        // 2. If an existing raw/webp file exists on disk, transcode it on blocking thread
        if target_path.exists() {
            let p = target_path.clone();
            let out_p = jpg_path.clone();
            let res = tokio::task::spawn_blocking(move || -> Result<PathBuf, String> {
                if let Ok(dyn_img) = image::open(&p) {
                    let resized = if dyn_img.width() > max_w || dyn_img.height() > max_h {
                        dyn_img.thumbnail(max_w, max_h)
                    } else {
                        dyn_img
                    };
                    if resized.save_with_format(&out_p, image::ImageFormat::Jpeg).is_ok() {
                        let _ = fs::remove_file(&p);
                        return Ok(out_p);
                    }
                }
                Ok(p)
            })
            .await
            .map_err(|e| e.to_string())?;

            return res;
        }

        // 3. Otherwise fetch from network
        tracing::debug!("Downloading image to cache: {}", url);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Network error downloading image: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP error {}: {}", resp.status(), url));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read image bytes: {}", e))?;

        let _ = fs::create_dir_all(&self.cache_dir);

        let out_p = jpg_path.clone();
        let fallback_p = target_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                let resized = if dyn_img.width() > max_w || dyn_img.height() > max_h {
                    dyn_img.thumbnail(max_w, max_h)
                } else {
                    dyn_img
                };
                if resized.save_with_format(&out_p, image::ImageFormat::Jpeg).is_ok() {
                    return Ok(out_p);
                }
            }
            fs::write(&fallback_p, &bytes).map_err(|e| e.to_string())?;
            Ok(fallback_p)
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
