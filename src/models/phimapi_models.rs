use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiCategory {
    pub id: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiCountry {
    pub id: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiTmdb {
    pub id: Option<serde_json::Value>,
    pub season: Option<serde_json::Value>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiImdb {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiMovieItem {
    pub _id: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub origin_name: Option<String>,
    #[serde(rename = "type")]
    pub movie_type: Option<String>,
    pub thumb_url: Option<String>,
    pub poster_url: Option<String>,
    pub year: Option<serde_json::Value>,
    pub time: Option<String>,
    pub quality: Option<String>,
    pub lang: Option<String>,
    pub episode_current: Option<String>,
    pub category: Option<Vec<PhimApiCategory>>,
    pub country: Option<Vec<PhimApiCountry>>,
    pub tmdb: Option<PhimApiTmdb>,
    pub imdb: Option<PhimApiImdb>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PhimApiPagination {
    pub totalItems: Option<u32>,
    pub totalItemsPerPage: Option<u32>,
    pub currentPage: Option<u32>,
    pub pageRanges: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiListResponse {
    pub status: Option<serde_json::Value>,
    pub items: Option<Vec<PhimApiMovieItem>>,
    pub pagination: Option<PhimApiPagination>,
    pub data: Option<PhimApiListData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiListData {
    pub items: Option<Vec<PhimApiMovieItem>>,
    pub params: Option<PhimApiListParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiListParams {
    pub pagination: Option<PhimApiPagination>,
}

// Detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiEpisodeServerItem {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub filename: Option<String>,
    pub link_embed: Option<String>,
    pub link_m3u8: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiEpisodeServer {
    pub server_name: Option<String>,
    pub server_data: Option<Vec<PhimApiEpisodeServerItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiMovieDetail {
    pub _id: Option<String>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub origin_name: Option<String>,
    pub content: Option<String>,
    #[serde(rename = "type")]
    pub movie_type: Option<String>,
    pub thumb_url: Option<String>,
    pub poster_url: Option<String>,
    pub trailer_url: Option<String>,
    pub time: Option<String>,
    pub episode_current: Option<String>,
    pub episode_total: Option<String>,
    pub quality: Option<String>,
    pub lang: Option<String>,
    pub year: Option<serde_json::Value>,
    pub director: Option<serde_json::Value>,
    pub actor: Option<serde_json::Value>,
    pub category: Option<Vec<PhimApiCategory>>,
    pub country: Option<Vec<PhimApiCountry>>,
    pub episodes: Option<Vec<PhimApiEpisodeServer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiDetailResponse {
    pub status: Option<serde_json::Value>,
    pub msg: Option<String>,
    pub movie: Option<PhimApiMovieDetail>,
    pub episodes: Option<Vec<PhimApiEpisodeServer>>,
    pub data: Option<PhimApiDetailDataWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiDetailDataWrapper {
    pub item: Option<PhimApiMovieDetail>,
}

// Cast & Peoples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiPerson {
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub character: Option<String>,
    pub profile_path: Option<String>,
    pub known_for_department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiPeoplesResponse {
    pub status: Option<serde_json::Value>,
    pub data: Option<Vec<PhimApiPerson>>,
}

// Images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiImageItem {
    pub file_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiImagesResponse {
    pub status: Option<serde_json::Value>,
    pub posters: Option<Vec<PhimApiImageItem>>,
    pub backdrops: Option<Vec<PhimApiImageItem>>,
}

// Keywords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiKeywordItem {
    pub id: Option<serde_json::Value>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhimApiKeywordsResponse {
    pub status: Option<serde_json::Value>,
    pub data: Option<Vec<PhimApiKeywordItem>>,
}
