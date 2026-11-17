use async_trait::async_trait;
use crate::api::client::HttpClient;
use crate::api::traits::MovieProvider;
use crate::models::vsmov_models::*;
use crate::models::{
    UnifiedCategoryItem, UnifiedEpisode, UnifiedMovieDetail, UnifiedMovieListResponse,
    UnifiedMovieSummary, UnifiedPagination, UnifiedPerson,
};

pub struct VsmovEngine {
    base_url: String,
    http: HttpClient,
}

impl VsmovEngine {
    pub fn new(base_url: String, timeout_secs: u64) -> Self {
        Self {
            base_url,
            http: HttpClient::new(timeout_secs),
        }
    }

    fn convert_item(&self, item: VsmovMovieItem, rank: i32) -> UnifiedMovieSummary {
        let poster = item.poster_url.clone().unwrap_or_default();
        let thumb = item.thumb_url.clone().unwrap_or_default();

        let full_poster = if poster.is_empty() { thumb.clone() } else { poster };
        let full_thumb = if thumb.is_empty() { full_poster.clone() } else { thumb };

        let year_str = match &item.year {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "2024".to_string(),
        };

        UnifiedMovieSummary {
            id: item.slug.clone().unwrap_or_default(),
            slug: item.slug.unwrap_or_default(),
            title: item.name.unwrap_or_default(),
            origin_title: item.origin_name.unwrap_or_default(),
            poster_url: full_poster,
            backdrop_url: full_thumb,
            genre: "Vsmov".to_string(),
            year: year_str,
            rating: "8.2".to_string(),
            rank,
            quality: item.quality.unwrap_or_else(|| "HD".to_string()),
            time: item.time.unwrap_or_default(),
            category: "Phim".to_string(),
            country: "Quốc tế".to_string(),
            is_saved: false,
            progress: 0,
        }
    }
}

#[async_trait]
impl MovieProvider for VsmovEngine {
    fn name(&self) -> &'static str {
        "Vsmov API"
    }

    fn id(&self) -> &'static str {
        "vsmov"
    }

    async fn get_home_trending(&self) -> Result<Vec<UnifiedMovieSummary>, String> {
        let url = format!("{}/danh-sach/phim-moi-cap-nhat?page=1", self.base_url);
        let resp: VsmovListResponse = self.http.get_json(&url).await?;

        let items = resp.items.or(resp.data).unwrap_or_default();
        let summaries = items
            .into_iter()
            .enumerate()
            .map(|(idx, it)| self.convert_item(it, (idx + 1) as i32))
            .collect();

        Ok(summaries)
    }

    async fn get_movies_by_category(&self, _category: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let url = format!("{}/danh-sach/subteam?page={}&limit=24", self.base_url, page);
        let resp: VsmovListResponse = self.http.get_json(&url).await?;

        let items = resp.items.or(resp.data).unwrap_or_default();
        let summaries = items.into_iter().map(|it| self.convert_item(it, 0)).collect();

        Ok(UnifiedMovieListResponse {
            items: summaries,
            pagination: UnifiedPagination {
                current_page: page,
                total_pages: 10,
                total_items: 100,
                items_per_page: 24,
            },
        })
    }

    async fn get_movies_by_genre(&self, genre_slug: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let slug = if genre_slug == "All" || genre_slug.is_empty() {
            "hanh-dong"
        } else {
            genre_slug
        };

        let url = format!("{}/the-loai/{}?limit=24&page={}", self.base_url, slug, page);
        let resp: VsmovListResponse = self.http.get_json(&url).await?;

        let items = resp.items.or(resp.data).unwrap_or_default();
        let summaries = items.into_iter().map(|it| self.convert_item(it, 0)).collect();

        Ok(UnifiedMovieListResponse {
            items: summaries,
            pagination: UnifiedPagination {
                current_page: page,
                total_pages: 10,
                total_items: 100,
                items_per_page: 24,
            },
        })
    }

    async fn search_movies(&self, query: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let url = format!("{}/tim-kiem", self.base_url);
        let page_str = page.to_string();
        let query_params = [
            ("keyword", query),
            ("page", &page_str),
            ("limit", "24"),
        ];

        let resp: VsmovListResponse = self.http.get_json_with_query(&url, &query_params).await?;
        let items = resp.items.or(resp.data).unwrap_or_default();
        let summaries = items.into_iter().map(|it| self.convert_item(it, 0)).collect();

        Ok(UnifiedMovieListResponse {
            items: summaries,
            pagination: UnifiedPagination {
                current_page: page,
                total_pages: 1,
                total_items: 24,
                items_per_page: 24,
            },
        })
    }

    async fn get_movie_detail(&self, slug: &str) -> Result<UnifiedMovieDetail, String> {
        let url = format!("{}/phim/{}", self.base_url, slug);
        let resp: VsmovDetailResponse = self.http.get_json(&url).await?;

        let item = resp.movie.or(resp.item).ok_or_else(|| format!("Movie not found: {}", slug))?;
        let summary = self.convert_item(item.clone(), 0);

        let director_str = match &item.director {
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "Đang cập nhật".to_string(),
        };

        let actors = match &item.actor {
            Some(serde_json::Value::String(s)) => {
                s.split(',')
                    .map(|name| UnifiedPerson {
                        name: name.trim().to_string(),
                        role: "Diễn viên".to_string(),
                        avatar_url: String::new(),
                    })
                    .collect()
            }
            _ => Vec::new(),
        };

        let default_episode = vec![
            UnifiedEpisode {
                server_name: "Server Vsmov #1".to_string(),
                name: "Tập 01 (Full)".to_string(),
                slug: "tap-01".to_string(),
                link_embed: String::new(),
                link_m3u8: String::new(),
            }
        ];

        Ok(UnifiedMovieDetail {
            summary,
            content: item.content.unwrap_or_else(|| "Đang cập nhật nội dung...".to_string()),
            trailer_url: String::new(),
            director: director_str,
            actors,
            episodes: default_episode,
            keywords: Vec::new(),
            tmdb_id: None,
            imdb_id: None,
        })
    }

    async fn get_genres(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        Ok(vec![
            UnifiedCategoryItem { name: "Hành Động".to_string(), slug: "hanh-dong".to_string() },
            UnifiedCategoryItem { name: "Hoạt Hình".to_string(), slug: "hoat-hinh".to_string() },
            UnifiedCategoryItem { name: "Tình Cảm".to_string(), slug: "tinh-cam".to_string() },
        ])
    }

    async fn get_countries(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        Ok(vec![
            UnifiedCategoryItem { name: "Việt Nam".to_string(), slug: "viet-nam".to_string() },
            UnifiedCategoryItem { name: "Hàn Quốc".to_string(), slug: "han-quoc".to_string() },
            UnifiedCategoryItem { name: "Nhật Bản".to_string(), slug: "nhat-ban".to_string() },
        ])
    }

    async fn get_peoples(&self, _slug: &str) -> Result<Vec<UnifiedPerson>, String> {
        Ok(Vec::new())
    }

    async fn get_keywords(&self, _slug: &str) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
}
