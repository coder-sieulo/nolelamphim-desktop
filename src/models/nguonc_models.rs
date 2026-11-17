use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCPaginate {
    pub current_page: Option<u32>,
    pub total_page: Option<u32>,
    pub total_items: Option<u32>,
    pub items_per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCTmdb {
    pub id: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub tmdb_type: Option<String>,
    pub season: Option<serde_json::Value>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCImdb {
    pub id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCMovieItem {
    pub id: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub original_name: Option<String>,
    pub thumb_url: Option<String>,
    pub poster_url: Option<String>,
    pub description: Option<String>,
    pub total_episodes: Option<serde_json::Value>,
    pub current_episode: Option<String>,
    pub time: Option<String>,
    pub quality: Option<String>,
    pub language: Option<String>,
    pub director: Option<String>,
    pub casts: Option<String>,
    pub category: Option<serde_json::Value>,
    pub tmdb: Option<NguonCTmdb>,
    pub imdb: Option<NguonCImdb>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCEpisodeItem {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub embed: Option<String>,
    pub m3u8: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCServerItem {
    pub server_name: Option<String>,
    pub items: Option<Vec<NguonCEpisodeItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCMovieDetail {
    #[serde(flatten)]
    pub base: NguonCMovieItem,
    pub episodes: Option<Vec<NguonCServerItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCListResponse {
    pub status: Option<String>,
    pub paginate: Option<NguonCPaginate>,
    pub items: Option<Vec<NguonCMovieItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NguonCDetailResponse {
    pub status: Option<String>,
    pub movie: Option<NguonCMovieDetail>,
}
