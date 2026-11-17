use async_trait::async_trait;
use crate::api::client::HttpClient;
use crate::api::traits::MovieProvider;
use crate::models::phimapi_models::*;
use crate::models::{
    UnifiedCategoryItem, UnifiedEpisode, UnifiedMovieDetail, UnifiedMovieListResponse,
    UnifiedMovieSummary, UnifiedPagination, UnifiedPerson,
};

pub struct PhimApiEngine {
    base_url: String,
    http: HttpClient,
}

impl PhimApiEngine {
    pub fn new(base_url: String, timeout_secs: u64) -> Self {
        Self {
            base_url,
            http: HttpClient::new(timeout_secs),
        }
    }

    fn convert_item(&self, item: PhimApiMovieItem, rank: i32) -> UnifiedMovieSummary {
        let genre_str = item
            .category
            .as_ref()
            .map(|cats| {
                cats.iter()
                    .filter_map(|c| c.name.clone())
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_default();

        let country_str = item
            .country
            .as_ref()
            .and_then(|cs| cs.first().and_then(|c| c.name.clone()))
            .unwrap_or_default();

        let year_str = match &item.year {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => String::new(),
        };

        let rating_str = item
            .tmdb
            .as_ref()
            .and_then(|t| t.vote_average)
            .map(|v| format!("{:.1}", v))
            .unwrap_or_else(|| "N/A".to_string());

        let poster = item.poster_url.clone().unwrap_or_default();
        let thumb = item.thumb_url.clone().unwrap_or_default();

        // If poster URL is relative, prepend domain / cdn if needed
        let full_poster = if poster.starts_with("http") {
            poster
        } else if !poster.is_empty() {
            format!("https://phimimg.com/{}", poster.trim_start_matches('/'))
        } else {
            thumb.clone()
        };

        let full_thumb = if thumb.starts_with("http") {
            thumb
        } else if !thumb.is_empty() {
            format!("https://phimimg.com/{}", thumb.trim_start_matches('/'))
        } else {
            full_poster.clone()
        };

        UnifiedMovieSummary {
            id: item._id.unwrap_or_default(),
            slug: item.slug.unwrap_or_default(),
            title: item.name.unwrap_or_default(),
            origin_title: item.origin_name.unwrap_or_default(),
            poster_url: full_poster,
            backdrop_url: full_thumb,
            genre: if genre_str.is_empty() { "Phim".to_string() } else { genre_str },
            year: year_str,
            rating: rating_str,
            rank,
            quality: item.quality.unwrap_or_else(|| "HD".to_string()),
            time: item.time.unwrap_or_default(),
            category: item.movie_type.unwrap_or_default(),
            country: country_str,
            is_saved: false,
            progress: 0,
        }
    }
}

#[async_trait]
impl MovieProvider for PhimApiEngine {
    fn name(&self) -> &'static str {
        "KKPhim / PhimAPI"
    }

    fn id(&self) -> &'static str {
        "phimapi"
    }

    async fn get_home_trending(&self) -> Result<Vec<UnifiedMovieSummary>, String> {
        // Try v1 home endpoint first, fallback to phim-moi-cap-nhat
        let url = format!("{}/danh-sach/phim-moi-cap-nhat?page=1", self.base_url);
        let resp: PhimApiListResponse = self.http.get_json(&url).await?;

        let items = resp.items.or_else(|| resp.data.and_then(|d| d.items)).unwrap_or_default();
        let mut summaries = Vec::new();
        for (idx, it) in items.into_iter().enumerate() {
            summaries.push(self.convert_item(it, (idx + 1) as i32));
        }

        Ok(summaries)
    }

    async fn get_movies_by_category(&self, category: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let cat_slug = match category {
            "phim-le" | "phim-bo" | "hoat-hinh" | "tv-shows" | "phim-chieu-rap" => category,
            _ => "phim-moi",
        };

        let url = format!("{}/v1/api/danh-sach/{}?page={}", self.base_url, cat_slug, page);
        let resp: PhimApiListResponse = self.http.get_json(&url).await?;

        let items = resp.data.as_ref().and_then(|d| d.items.clone())
            .or(resp.items)
            .unwrap_or_default();

        let pagination_info = resp.data.as_ref()
            .and_then(|d| d.params.as_ref())
            .and_then(|p| p.pagination.clone())
            .or(resp.pagination);

        let pagination = UnifiedPagination {
            current_page: pagination_info.as_ref().and_then(|p| p.currentPage).unwrap_or(page),
            total_pages: pagination_info.as_ref().and_then(|p| p.pageRanges).unwrap_or(10),
            total_items: pagination_info.as_ref().and_then(|p| p.totalItems).unwrap_or(100),
            items_per_page: pagination_info.as_ref().and_then(|p| p.totalItemsPerPage).unwrap_or(24),
        };

        let summaries = items.into_iter().map(|it| self.convert_item(it, 0)).collect();

        Ok(UnifiedMovieListResponse {
            items: summaries,
            pagination,
        })
    }

    async fn get_movies_by_genre(&self, genre_slug: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let url = if genre_slug == "All" || genre_slug.is_empty() {
            format!("{}/v1/api/danh-sach/phim-moi?page={}", self.base_url, page)
        } else {
            format!("{}/v1/api/the-loai/{}?page={}", self.base_url, genre_slug, page)
        };

        let resp: PhimApiListResponse = self.http.get_json(&url).await?;
        let items = resp.data.as_ref().and_then(|d| d.items.clone()).or(resp.items).unwrap_or_default();

        let pagination = UnifiedPagination {
            current_page: page,
            total_pages: 10,
            total_items: items.len() as u32,
            items_per_page: 24,
        };

        let summaries = items.into_iter().map(|it| self.convert_item(it, 0)).collect();

        Ok(UnifiedMovieListResponse {
            items: summaries,
            pagination,
        })
    }

    async fn search_movies(&self, query: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let url = format!("{}/v1/api/tim-kiem", self.base_url);
        let page_str = page.to_string();
        let query_params = [
            ("keyword", query),
            ("page", &page_str),
            ("limit", "24"),
        ];

        let resp: PhimApiListResponse = self.http.get_json_with_query(&url, &query_params).await?;
        let items = resp.data.as_ref().and_then(|d| d.items.clone()).or(resp.items).unwrap_or_default();

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
        let url = format!("{}/v1/api/phim/{}", self.base_url, slug);
        let resp: PhimApiDetailResponse = self.http.get_json(&url).await?;

        let detail = resp.movie
            .or_else(|| resp.data.and_then(|d| d.item))
            .ok_or_else(|| format!("Movie not found: {}", slug))?;

        let servers = resp.episodes.or(detail.episodes).unwrap_or_default();
        let mut unified_episodes = Vec::new();

        for s in servers {
            let s_name = s.server_name.unwrap_or_else(|| "Server 1".to_string());
            if let Some(items) = s.server_data {
                for ep in items {
                    unified_episodes.push(UnifiedEpisode {
                        server_name: s_name.clone(),
                        name: ep.name.unwrap_or_else(|| "Tập 01".to_string()),
                        slug: ep.slug.unwrap_or_default(),
                        link_embed: ep.link_embed.unwrap_or_default(),
                        link_m3u8: ep.link_m3u8.unwrap_or_default(),
                    });
                }
            }
        }

        let director_str = match &detail.director {
            Some(serde_json::Value::Array(arr)) => arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "Đang cập nhật".to_string(),
        };

        let summary = UnifiedMovieSummary {
            id: detail._id.unwrap_or_default(),
            slug: detail.slug.unwrap_or_default(),
            title: detail.name.unwrap_or_default(),
            origin_title: detail.origin_name.unwrap_or_default(),
            poster_url: detail.poster_url.unwrap_or_default(),
            backdrop_url: detail.thumb_url.unwrap_or_default(),
            genre: detail.category.as_ref()
                .map(|cs| cs.iter().filter_map(|c| c.name.clone()).collect::<Vec<_>>().join(" / "))
                .unwrap_or_default(),
            year: match &detail.year {
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::String(s)) => s.clone(),
                _ => String::new(),
            },
            rating: "8.5".to_string(),
            rank: 0,
            quality: detail.quality.unwrap_or_else(|| "HD".to_string()),
            time: detail.time.unwrap_or_default(),
            category: detail.movie_type.unwrap_or_default(),
            country: detail.country.as_ref()
                .and_then(|cs| cs.first().and_then(|c| c.name.clone()))
                .unwrap_or_default(),
            is_saved: false,
            progress: 0,
        };

        Ok(UnifiedMovieDetail {
            summary,
            content: detail.content.unwrap_or_else(|| "Đang cập nhật nội dung...".to_string()),
            trailer_url: detail.trailer_url.unwrap_or_default(),
            director: if director_str.is_empty() { "Đang cập nhật".to_string() } else { director_str },
            actors: Vec::new(),
            episodes: unified_episodes,
            keywords: Vec::new(),
            tmdb_id: None,
            imdb_id: None,
        })
    }

    async fn get_genres(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        let url = format!("{}/the-loai", self.base_url);
        let resp: Vec<PhimApiCategory> = self.http.get_json(&url).await?;
        Ok(resp.into_iter().filter_map(|c| {
            Some(UnifiedCategoryItem {
                name: c.name?,
                slug: c.slug?,
            })
        }).collect())
    }

    async fn get_countries(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        let url = format!("{}/quoc-gia", self.base_url);
        let resp: Vec<PhimApiCountry> = self.http.get_json(&url).await?;
        Ok(resp.into_iter().filter_map(|c| {
            Some(UnifiedCategoryItem {
                name: c.name?,
                slug: c.slug?,
            })
        }).collect())
    }

    async fn get_peoples(&self, slug: &str) -> Result<Vec<UnifiedPerson>, String> {
        let url = format!("{}/v1/api/phim/{}/peoples", self.base_url, slug);
        let resp: Result<PhimApiPeoplesResponse, String> = self.http.get_json(&url).await;
        match resp {
            Ok(peoples) => {
                let persons = peoples.data.unwrap_or_default().into_iter().map(|p| {
                    UnifiedPerson {
                        name: p.name.or(p.original_name).unwrap_or_default(),
                        role: p.character.or(p.known_for_department).unwrap_or_else(|| "Diễn viên".to_string()),
                        avatar_url: p.profile_path.unwrap_or_default(),
                    }
                }).collect();
                Ok(persons)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn get_keywords(&self, slug: &str) -> Result<Vec<String>, String> {
        let url = format!("{}/v1/api/phim/{}/keywords", self.base_url, slug);
        let resp: Result<PhimApiKeywordsResponse, String> = self.http.get_json(&url).await;
        match resp {
            Ok(kw) => {
                let list = kw.data.unwrap_or_default().into_iter().filter_map(|k| k.name).collect();
                Ok(list)
            }
            Err(_) => Ok(Vec::new()),
        }
    }
}
