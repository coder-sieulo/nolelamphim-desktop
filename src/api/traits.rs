use async_trait::async_trait;
use crate::models::{
    UnifiedCategoryItem, UnifiedMovieDetail, UnifiedMovieListResponse, UnifiedMovieSummary,
    UnifiedPerson,
};

#[async_trait]
pub trait MovieProvider: Send + Sync {
    /// Return the user-friendly name of the API provider
    fn name(&self) -> &'static str;

    /// Return provider identifier code (e.g. "phimapi", "nguonc", "vsmov")
    fn id(&self) -> &'static str;

    /// Fetch featured and trending movies for Home view
    async fn get_home_trending(&self) -> Result<Vec<UnifiedMovieSummary>, String>;

    /// Fetch paginated movies by category (e.g., phim-le, phim-bo, hoat-hinh, tv-shows)
    async fn get_movies_by_category(&self, category: &str, page: u32) -> Result<UnifiedMovieListResponse, String>;

    /// Fetch paginated movies by genre slug (e.g., hanh-dong, vien-tuong, tinh-cam)
    async fn get_movies_by_genre(&self, genre_slug: &str, page: u32) -> Result<UnifiedMovieListResponse, String>;

    /// Search movies by keyword
    async fn search_movies(&self, query: &str, page: u32) -> Result<UnifiedMovieListResponse, String>;

    /// Get complete movie details including servers and episodes
    async fn get_movie_detail(&self, slug: &str) -> Result<UnifiedMovieDetail, String>;

    /// Fetch available genres list
    async fn get_genres(&self) -> Result<Vec<UnifiedCategoryItem>, String>;

    /// Fetch available countries list
    async fn get_countries(&self) -> Result<Vec<UnifiedCategoryItem>, String>;

    /// Fetch actors & crew members for a movie
    async fn get_peoples(&self, slug: &str) -> Result<Vec<UnifiedPerson>, String>;

    /// Fetch keywords / tags for a movie
    async fn get_keywords(&self, slug: &str) -> Result<Vec<String>, String>;
}
