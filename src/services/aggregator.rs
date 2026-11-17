use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::traits::MovieProvider;
use crate::api::{NguonCEngine, PhimApiEngine, VsmovEngine};
use crate::config::AppConfig;
use crate::models::{
    UnifiedCategoryItem, UnifiedMovieDetail, UnifiedMovieListResponse, UnifiedMovieSummary,
};

pub struct MovieAggregatorService {
    phimapi: Arc<PhimApiEngine>,
    nguonc: Arc<NguonCEngine>,
    vsmov: Arc<VsmovEngine>,
    active_provider: Arc<RwLock<String>>,
}

impl MovieAggregatorService {
    pub fn new(config: &AppConfig) -> Self {
        let phimapi = Arc::new(PhimApiEngine::new(
            config.phimapi_base_url.clone(),
            config.request_timeout_secs,
        ));
        let nguonc = Arc::new(NguonCEngine::new(
            config.nguonc_base_url.clone(),
            config.request_timeout_secs,
        ));
        let vsmov = Arc::new(VsmovEngine::new(
            config.vsmov_base_url.clone(),
            config.request_timeout_secs,
        ));

        Self {
            phimapi,
            nguonc,
            vsmov,
            active_provider: Arc::new(RwLock::new(config.default_provider.clone())),
        }
    }

    pub async fn set_active_provider(&self, provider_id: &str) {
        let mut active = self.active_provider.write().await;
        *active = provider_id.to_lowercase();
        tracing::info!("Aggregator active provider switched to: {}", *active);
    }

    pub async fn get_active_provider_id(&self) -> String {
        self.active_provider.read().await.clone()
    }

    fn get_engine(&self, id: &str) -> Arc<dyn MovieProvider> {
        match id {
            "nguonc" => self.nguonc.clone() as Arc<dyn MovieProvider>,
            "vsmov" => self.vsmov.clone() as Arc<dyn MovieProvider>,
            _ => self.phimapi.clone() as Arc<dyn MovieProvider>,
        }
    }

    /// Fetch Home Trending with Automatic Fallback
    pub async fn get_home_trending(&self) -> Result<Vec<UnifiedMovieSummary>, String> {
        let active_id = self.get_active_provider_id().await;
        let primary = self.get_engine(&active_id);

        match primary.get_home_trending().await {
            Ok(movies) if !movies.is_empty() => Ok(movies),
            Err(e) => {
                tracing::warn!("Primary provider '{}' failed for home trending: {}. Falling back...", active_id, e);
                // Fallback to PhimAPI or NguonC
                if active_id != "phimapi" {
                    if let Ok(movies) = self.phimapi.get_home_trending().await {
                        return Ok(movies);
                    }
                }
                if active_id != "nguonc" {
                    if let Ok(movies) = self.nguonc.get_home_trending().await {
                        return Ok(movies);
                    }
                }
                Err(e)
            }
            Ok(empty) => Ok(empty),
        }
    }

    /// Fetch movies by category with fallback
    pub async fn get_movies_by_category(&self, category: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        engine.get_movies_by_category(category, page).await
    }

    /// Fetch movies by genre
    pub async fn get_movies_by_genre(&self, genre_slug: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        engine.get_movies_by_genre(genre_slug, page).await
    }

    /// Search movies
    pub async fn search_movies(&self, query: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        engine.search_movies(query, page).await
    }

    /// Get complete movie details
    pub async fn get_movie_detail(&self, slug: &str) -> Result<UnifiedMovieDetail, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        
        match engine.get_movie_detail(slug).await {
            Ok(mut detail) => {
                // Enrich with peoples and keywords if provider supports them
                if let Ok(peoples) = engine.get_peoples(slug).await {
                    if !peoples.is_empty() {
                        detail.actors = peoples;
                    }
                }
                if let Ok(keywords) = engine.get_keywords(slug).await {
                    if !keywords.is_empty() {
                        detail.keywords = keywords;
                    }
                }
                Ok(detail)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch movie detail for '{}': {}", slug, e);
                Err(e)
            }
        }
    }

    /// Get genres list
    pub async fn get_genres(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        engine.get_genres().await
    }

    /// Get countries list
    pub async fn get_countries(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        let active_id = self.get_active_provider_id().await;
        let engine = self.get_engine(&active_id);
        engine.get_countries().await
    }
}
