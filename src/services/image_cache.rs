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
            Some(file_path)
        } else {
            None
        }
    }

    pub async fn fetch_image(&self, url: &str) -> Result<PathBuf, String> {
        if !self.enabled || url.is_empty() {
            return Err("Image caching disabled or empty URL".to_string());
        }

        let target_path = self.cache_dir.join(self.url_to_filename(url));
        if target_path.exists() {
            return Ok(target_path);
        }

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
        fs::write(&target_path, &bytes)
            .map_err(|e| format!("Failed to write cached image to disk: {}", e))?;

        Ok(target_path)
    }
}
