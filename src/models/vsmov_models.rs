use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsmovMovieItem {
    pub id: Option<serde_json::Value>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub origin_name: Option<String>,
    pub thumb_url: Option<String>,
    pub poster_url: Option<String>,
    pub year: Option<serde_json::Value>,
    pub time: Option<String>,
    pub quality: Option<String>,
    pub lang: Option<String>,
    pub content: Option<String>,
    pub director: Option<serde_json::Value>,
    pub actor: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsmovListResponse {
    pub status: Option<serde_json::Value>,
    pub items: Option<Vec<VsmovMovieItem>>,
    pub data: Option<Vec<VsmovMovieItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsmovDetailResponse {
    pub status: Option<serde_json::Value>,
    pub movie: Option<VsmovMovieItem>,
    pub item: Option<VsmovMovieItem>,
}
