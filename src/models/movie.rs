use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMovieSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub origin_title: String,
    pub poster_url: String,
    pub backdrop_url: String,
    pub genre: String,
    pub year: String,
    pub rating: String,
    pub rank: i32,
    pub quality: String,
    pub time: String,
    pub category: String,
    pub country: String,
    pub is_saved: bool,
    pub progress: i32,
}

impl Default for UnifiedMovieSummary {
    fn default() -> Self {
        Self {
            id: String::new(),
            slug: String::new(),
            title: String::new(),
            origin_title: String::new(),
            poster_url: String::new(),
            backdrop_url: String::new(),
            genre: String::new(),
            year: String::new(),
            rating: "N/A".to_string(),
            rank: 0,
            quality: "HD".to_string(),
            time: String::new(),
            category: String::new(),
            country: String::new(),
            is_saved: false,
            progress: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPerson {
    pub name: String,
    pub role: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEpisode {
    pub server_name: String,
    pub name: String,
    pub slug: String,
    pub link_embed: String,
    pub link_m3u8: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMovieDetail {
    pub summary: UnifiedMovieSummary,
    pub content: String,
    pub trailer_url: String,
    pub director: String,
    pub actors: Vec<UnifiedPerson>,
    pub episodes: Vec<UnifiedEpisode>,
    pub keywords: Vec<String>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPagination {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_items: u32,
    pub items_per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMovieListResponse {
    pub items: Vec<UnifiedMovieSummary>,
    pub pagination: UnifiedPagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCategoryItem {
    pub name: String,
    pub slug: String,
}
