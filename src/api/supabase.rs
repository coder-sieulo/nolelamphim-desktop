use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::traits::MovieProvider;
use crate::models::{
    UnifiedCategoryItem, UnifiedEpisode, UnifiedMovieDetail, UnifiedMovieListResponse,
    UnifiedMovieSummary, UnifiedPagination, UnifiedPerson,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SupabaseMovieRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub origin_name: Option<String>,
    pub poster_url: Option<String>,
    pub thumb_url: Option<String>,
    pub year: Option<i32>,
    pub quality: Option<String>,
    pub lang: Option<String>,
    pub time: Option<String>,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub episode_current: Option<String>,
    pub episode_total: Option<String>,
    pub view: Option<i32>,
    pub content: Option<String>,
    pub actor: Option<Vec<String>>,
    pub director: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub tmdb_vote_avg: Option<String>,
    pub categories: Option<serde_json::Value>,
    pub countries: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SupabaseEpisodeRow {
    pub id: i64,
    pub movie_slug: String,
    pub server_name: String,
    pub episode_name: String,
    pub episode_slug: String,
    pub embed: String,
    pub m3u8: Option<String>,
    pub sort: Option<i32>,
}

pub struct SupabaseEngine {
    client: reqwest::Client,
    base_url: String,
    anon_key: String,
}

impl SupabaseEngine {
    pub fn new(base_url: String, anon_key: String, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let clean_base = base_url.trim_end_matches('/').to_string();

        Self {
            client,
            base_url: clean_base,
            anon_key,
        }
    }

    fn row_to_summary(row: &SupabaseMovieRow) -> UnifiedMovieSummary {
        let genre = if let Some(serde_json::Value::Array(cats)) = &row.categories {
            cats.iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<&str>>()
                .join(" / ")
        } else {
            String::new()
        };

        // Standard thumb for cards, poster for hero/backdrop
        let poster = row
            .thumb_url
            .as_deref()
            .or(row.poster_url.as_deref())
            .unwrap_or("")
            .to_string();

        let backdrop = row
            .poster_url
            .as_deref()
            .or(row.thumb_url.as_deref())
            .unwrap_or("")
            .to_string();

        UnifiedMovieSummary {
            id: row.id.to_string(),
            slug: row.slug.clone(),
            title: row.name.clone(),
            origin_title: row.origin_name.clone().unwrap_or_default(),
            poster_url: poster,
            backdrop_url: backdrop,
            genre,
            year: row.year.map(|y| y.to_string()).unwrap_or_default(),
            rating: row.tmdb_vote_avg.clone().unwrap_or_else(|| "8.0".to_string()),
            rank: 0,
            quality: row.quality.clone().unwrap_or_else(|| "FHD".to_string()),
            time: row.time.clone().unwrap_or_default(),
            category: row.r#type.clone().unwrap_or_default(),
            country: String::new(),
            is_saved: false,
            progress: 0,
        }
    }

    async fn execute_get<T: serde::de::DeserializeOwned>(&self, path_and_query: &str) -> Result<T, String> {
        let url = format!("{}/rest/v1/{}", self.base_url, path_and_query);
        tracing::debug!("Supabase REST request: {}", url);

        let resp = self
            .client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase API error ({}): {}", status, body));
        }

        resp.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse Supabase JSON response: {}", e))
    }
}

#[async_trait]
impl MovieProvider for SupabaseEngine {
    fn name(&self) -> &'static str {
        "Supabase"
    }

    fn id(&self) -> &'static str {
        "supabase"
    }

    async fn get_home_trending(&self) -> Result<Vec<UnifiedMovieSummary>, String> {
        let query = "movies?select=id,slug,name,origin_name,poster_url,thumb_url,year,quality,time,type,status,episode_current,view,tmdb_vote_avg,categories,countries&order=updated_at.desc&limit=24";
        let rows: Vec<SupabaseMovieRow> = self.execute_get(query).await?;
        let summaries = rows.iter().map(Self::row_to_summary).collect();
        Ok(summaries)
    }

    async fn get_movies_by_category(&self, category: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let offset = page.saturating_sub(1) * 24;
        let type_filter = match category {
            "phim-le" | "single" => "type=eq.single",
            "phim-bo" | "series" => "type=eq.series",
            "hoat-hinh" | "hoathinh" => "type=in.(hoathinh,animation)",
            "tv-shows" | "tvshows" => "type=eq.tvshows",
            _ => "type=not.is.null",
        };

        let query = format!(
            "movies?select=id,slug,name,origin_name,poster_url,thumb_url,year,quality,time,type,status,episode_current,view,tmdb_vote_avg,categories,countries&{}&order=updated_at.desc&limit=24&offset={}",
            type_filter, offset
        );

        let rows: Vec<SupabaseMovieRow> = self.execute_get(&query).await?;
        let items: Vec<UnifiedMovieSummary> = rows.iter().map(Self::row_to_summary).collect();

        Ok(UnifiedMovieListResponse {
            items,
            pagination: UnifiedPagination {
                total_items: 12000,
                total_pages: 500,
                current_page: page,
                items_per_page: 24,
            },
        })
    }

    async fn get_movies_by_genre(&self, genre_slug: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let offset = page.saturating_sub(1) * 24;
        let query = if genre_slug == "All" || genre_slug.is_empty() {
            format!("movies?select=id,slug,name,origin_name,poster_url,thumb_url,year,quality,time,type,status,episode_current,view,tmdb_vote_avg,categories,countries&order=updated_at.desc&limit=24&offset={}", offset)
        } else {
            format!(
                "movies?select=id,slug,name,origin_name,poster_url,thumb_url,year,quality,time,type,status,episode_current,view,tmdb_vote_avg,categories,countries&categories=cs.%5B%7B%22slug%22%3A%22{}%22%7D%5D&order=updated_at.desc&limit=24&offset={}",
                genre_slug, offset
            )
        };

        let rows: Vec<SupabaseMovieRow> = self.execute_get(&query).await?;
        let items: Vec<UnifiedMovieSummary> = rows.iter().map(Self::row_to_summary).collect();

        Ok(UnifiedMovieListResponse {
            items,
            pagination: UnifiedPagination {
                total_items: 5000,
                total_pages: 200,
                current_page: page,
                items_per_page: 24,
            },
        })
    }

    async fn search_movies(&self, query: &str, page: u32) -> Result<UnifiedMovieListResponse, String> {
        let offset = page.saturating_sub(1) * 24;
        let mut encoded_query = String::with_capacity(query.len() * 2);
        for b in query.bytes() {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                encoded_query.push(b as char);
            } else {
                encoded_query.push_str(&format!("%{:02X}", b));
            }
        }
        let path = format!(
            "movies?select=id,slug,name,origin_name,poster_url,thumb_url,year,quality,time,type,status,episode_current,view,tmdb_vote_avg,categories,countries&name=ilike.*{}*&order=updated_at.desc&limit=24&offset={}",
            encoded_query, offset
        );

        let rows: Vec<SupabaseMovieRow> = self.execute_get(&path).await?;
        let items: Vec<UnifiedMovieSummary> = rows.iter().map(Self::row_to_summary).collect();

        Ok(UnifiedMovieListResponse {
            items,
            pagination: UnifiedPagination {
                total_items: 100,
                total_pages: 5,
                current_page: page,
                items_per_page: 24,
            },
        })
    }

    async fn get_movie_detail(&self, slug: &str) -> Result<UnifiedMovieDetail, String> {
        let movie_query = format!("movies?slug=eq.{}&limit=1", slug);
        let rows: Vec<SupabaseMovieRow> = self.execute_get(&movie_query).await?;
        let movie = rows.into_iter().next().ok_or_else(|| format!("Movie not found: {}", slug))?;

        let episode_query = format!("movie_episodes?movie_slug=eq.{}&order=sort.asc", slug);
        let ep_rows: Vec<SupabaseEpisodeRow> = self.execute_get(&episode_query).await.unwrap_or_default();

        let episodes: Vec<UnifiedEpisode> = ep_rows
            .into_iter()
            .map(|ep| UnifiedEpisode {
                server_name: ep.server_name,
                name: ep.episode_name,
                slug: ep.episode_slug,
                link_embed: ep.embed,
                link_m3u8: ep.m3u8.unwrap_or_default(),
            })
            .collect();

        let summary = Self::row_to_summary(&movie);

        let actors = movie
            .actor
            .unwrap_or_default()
            .into_iter()
            .map(|name| UnifiedPerson {
                name,
                role: "Diễn viên".to_string(),
                avatar_url: String::new(),
            })
            .collect();

        let director = movie
            .director
            .as_ref()
            .and_then(|d| d.first())
            .cloned()
            .unwrap_or_default();

        let keywords = movie
            .keywords
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        Ok(UnifiedMovieDetail {
            summary,
            content: movie.content.unwrap_or_default(),
            trailer_url: String::new(),
            director,
            actors,
            episodes,
            keywords,
            tmdb_id: None,
            imdb_id: None,
        })
    }

    async fn get_genres(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        Ok(vec![
            UnifiedCategoryItem { name: "Hành Động".into(), slug: "hanh-dong".into() },
            UnifiedCategoryItem { name: "Viễn Tưởng".into(), slug: "vien-tuong".into() },
            UnifiedCategoryItem { name: "Chính Kịch".into(), slug: "chinh-kich".into() },
            UnifiedCategoryItem { name: "Kinh Dị".into(), slug: "kinh-di".into() },
            UnifiedCategoryItem { name: "Hài Hước".into(), slug: "hai-huoc".into() },
            UnifiedCategoryItem { name: "Tình Cảm".into(), slug: "tinh-cam".into() },
            UnifiedCategoryItem { name: "Hoạt Hình".into(), slug: "hoat-hinh".into() },
        ])
    }

    async fn get_countries(&self) -> Result<Vec<UnifiedCategoryItem>, String> {
        Ok(vec![
            UnifiedCategoryItem { name: "Việt Nam".into(), slug: "viet-nam".into() },
            UnifiedCategoryItem { name: "Âu Mỹ".into(), slug: "au-my".into() },
            UnifiedCategoryItem { name: "Hàn Quốc".into(), slug: "han-quoc".into() },
            UnifiedCategoryItem { name: "Trung Quốc".into(), slug: "trung-quoc".into() },
            UnifiedCategoryItem { name: "Nhật Bản".into(), slug: "nhat-ban".into() },
        ])
    }

    async fn get_peoples(&self, _slug: &str) -> Result<Vec<UnifiedPerson>, String> {
        Ok(vec![])
    }

    async fn get_keywords(&self, _slug: &str) -> Result<Vec<String>, String> {
        Ok(vec![])
    }
}
