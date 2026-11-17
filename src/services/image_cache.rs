use directories::ProjectDirs;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::DecodedImageBuffer;

pub struct ImageCacheService {
    cache_dir: PathBuf,
    client: reqwest::Client,
    enabled: bool,
    memory_cache: Arc<RwLock<HashMap<String, DecodedImageBuffer>>>,
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
            client: reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            enabled,
            memory_cache: Arc::new(RwLock::new(HashMap::with_capacity(128))),
        }
    }

    fn url_to_hash(&self, url: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    }

    fn url_to_filename(&self, url: &str) -> String {
        let hash = self.url_to_hash(url);
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

        let hash = self.url_to_hash(url);
        let thumb_filename = format!("{:x}_{}x{}.jpg", hash, max_w, max_h);
        let thumb_path = self.cache_dir.join(&thumb_filename);

        // 1. If exact size thumbnail already exists, return immediately
        if thumb_path.exists() {
            return Ok(thumb_path);
        }

        let raw_filename = self.url_to_filename(url);
        let raw_path = self.cache_dir.join(&raw_filename);

        // 2. If raw file exists on disk, transcode it
        if raw_path.exists() {
            let p = raw_path.clone();
            let out_p = thumb_path.clone();
            let res = tokio::task::spawn_blocking(move || -> Result<PathBuf, String> {
                if let Ok(dyn_img) = image::open(&p) {
                    let resized = if dyn_img.width() > max_w || dyn_img.height() > max_h {
                        dyn_img.thumbnail(max_w, max_h)
                    } else {
                        dyn_img
                    };
                    if resized.save_with_format(&out_p, image::ImageFormat::Jpeg).is_ok() {
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

        let out_p = thumb_path.clone();
        let fallback_p = raw_path.clone();
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

    /// High performance fetch + optimize + decode directly to DecodedImageBuffer in background
    /// with two-tier cache (In-Memory LRU + Local SSD/HDD Disk Cache)
    pub async fn fetch_and_decode(&self, url: &str, max_w: u32, max_h: u32) -> Result<DecodedImageBuffer, String> {
        if !self.enabled || url.is_empty() {
            return Err("Image caching disabled or empty URL".to_string());
        }

        let mem_key = format!("{}_{}x{}", url, max_w, max_h);

        // Tier 1: Check Memory Cache (0ms latency, zero disk I/O, zero decoding)
        {
            let cache_read = self.memory_cache.read().await;
            if let Some(decoded) = cache_read.get(&mem_key) {
                return Ok(decoded.clone());
            }
        }

        let hash = self.url_to_hash(url);
        let thumb_filename = format!("{:x}_{}x{}.jpg", hash, max_w, max_h);
        let thumb_path = self.cache_dir.join(&thumb_filename);

        // Tier 2: Check Disk Cache (read file & decode on background worker)
        if thumb_path.exists() {
            let p = thumb_path.clone();
            let decoded_opt = tokio::task::spawn_blocking(move || {
                DecodedImageBuffer::decode_from_path(&p, max_w, max_h)
            })
            .await
            .map_err(|e| e.to_string())?;

            if let Some(decoded) = decoded_opt {
                let mut cache_write = self.memory_cache.write().await;
                if cache_write.len() > 120 {
                    cache_write.clear();
                }
                cache_write.insert(mem_key, decoded.clone());
                return Ok(decoded);
            }
        }

        // Tier 3: Fetch over HTTP, decode, thumbnail, persist to disk, and populate memory cache
        tracing::debug!("Fetching image over network: {}", url);
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
        let out_p = thumb_path.clone();

        let decoded = tokio::task::spawn_blocking(move || -> Result<DecodedImageBuffer, String> {
            let dyn_img = image::load_from_memory(&bytes)
                .map_err(|e| format!("Failed to decode image from memory: {}", e))?;

            let resized = if dyn_img.width() > max_w || dyn_img.height() > max_h {
                dyn_img.resize(max_w, max_h, image::imageops::FilterType::Triangle)
            } else {
                dyn_img
            };

            // Save thumbnail to disk for future launches
            let _ = resized.save_with_format(&out_p, image::ImageFormat::Jpeg);

            let rgba = resized.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            Ok(DecodedImageBuffer::new(w, h, rgba.into_raw()))
        })
        .await
        .map_err(|e| e.to_string())??;

        // Save into memory cache
        {
            let mut cache_write = self.memory_cache.write().await;
            if cache_write.len() > 120 {
                cache_write.clear();
            }
            cache_write.insert(mem_key, decoded.clone());
        }

        Ok(decoded)
    }
}
