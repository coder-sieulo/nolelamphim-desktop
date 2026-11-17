use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::UnifiedMovieSummary;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalStoreData {
    pub watchlist: Vec<UnifiedMovieSummary>,
    pub history: Vec<UnifiedMovieSummary>,
}

#[derive(Clone)]
pub struct StorageService {
    file_path: PathBuf,
    data: Arc<RwLock<LocalStoreData>>,
}

impl StorageService {
    pub fn new() -> Self {
        let config_dir = ProjectDirs::from("com", "nolelamphim", "desktop")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./.config"));

        let _ = std::fs::create_dir_all(&config_dir);
        let file_path = config_dir.join("watchlist.json");

        // Load existing data from file if present
        let initial_data = if file_path.exists() {
            match std::fs::read_to_string(&file_path) {
                Ok(content) => serde_json::from_str::<LocalStoreData>(&content).unwrap_or_default(),
                Err(_) => LocalStoreData::default(),
            }
        } else {
            LocalStoreData::default()
        };

        Self {
            file_path,
            data: Arc::new(RwLock::new(initial_data)),
        }
    }

    async fn persist(&self) {
        let data = self.data.read().await;
        if let Ok(json) = serde_json::to_string_pretty(&*data) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }

    pub async fn get_watchlist(&self) -> Vec<UnifiedMovieSummary> {
        self.data.read().await.watchlist.clone()
    }

    pub async fn is_saved(&self, slug: &str) -> bool {
        self.data
            .read()
            .await
            .watchlist
            .iter()
            .any(|m| m.slug == slug)
    }

    /// Toggle movie in watchlist. Returns (is_now_saved, updated_watchlist)
    pub async fn toggle_watchlist(&self, mut movie: UnifiedMovieSummary) -> (bool, Vec<UnifiedMovieSummary>) {
        let mut data = self.data.write().await;
        let is_saved = if let Some(pos) = data.watchlist.iter().position(|m| m.slug == movie.slug) {
            data.watchlist.remove(pos);
            tracing::info!("Removed movie from watchlist: {}", movie.slug);
            false
        } else {
            movie.is_saved = true;
            data.watchlist.insert(0, movie.clone());
            tracing::info!("Added movie to watchlist: {}", movie.slug);
            true
        };

        let updated_list = data.watchlist.clone();
        drop(data);

        self.persist().await;
        (is_saved, updated_list)
    }

    pub async fn add_to_history(&self, mut movie: UnifiedMovieSummary, progress: i32) -> Vec<UnifiedMovieSummary> {
        let mut data = self.data.write().await;
        movie.progress = progress;

        // Remove if already in history to move to top
        if let Some(pos) = data.history.iter().position(|m| m.slug == movie.slug) {
            data.history.remove(pos);
        }

        data.history.insert(0, movie);
        if data.history.len() > 30 {
            data.history.truncate(30);
        }

        let updated_history = data.history.clone();
        drop(data);

        self.persist().await;
        updated_history
    }

    pub async fn get_history(&self) -> Vec<UnifiedMovieSummary> {
        self.data.read().await.history.clone()
    }
}
