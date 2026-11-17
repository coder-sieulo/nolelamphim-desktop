pub mod api;
pub mod config;
pub mod models;
pub mod services;

use std::rc::Rc;
use std::sync::Arc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use config::AppConfig;
use models::{to_slint_episode_item, to_slint_movie_item, to_slint_person_item};
use services::{ImageCacheService, MovieAggregatorService, StorageService};

slint::include_modules!();

fn create_sample_movie(
    id: &str,
    slug: &str,
    title: &str,
    origin_title: &str,
    genre: &str,
    year: &str,
    rating: &str,
    rank: i32,
    progress: i32,
    is_saved: bool,
) -> MovieItem {
    MovieItem {
        id: SharedString::from(id),
        slug: SharedString::from(slug),
        title: SharedString::from(title),
        origin_title: SharedString::from(origin_title),
        poster_url: SharedString::from(""),
        backdrop_url: SharedString::from(""),
        poster: slint::Image::default(),
        has_poster: false,
        genre: SharedString::from(genre),
        year: SharedString::from(year),
        rating: SharedString::from(rating),
        rank,
        quality: SharedString::from("4K HDR"),
        progress,
        is_saved,
    }
}

pub fn open_target_url(url: &str) {
    if url.is_empty() {
        return;
    }
    tracing::info!("Opening external stream/player URL: {}", url);
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Nolelamphim Desktop Application...");

    let config = AppConfig::from_env();
    let aggregator = Arc::new(MovieAggregatorService::new(&config));
    let storage = Arc::new(StorageService::new());
    let image_cache = Arc::new(ImageCacheService::new(config.enable_image_cache));

    let main_window = AppWindow::new()?;

    // Populate Initial Mock Data matching Cinefy Mockup
    let dune = create_sample_movie(
        "1",
        "dune-part-two",
        "Dune: Part Two",
        "Hành Tinh Cát: Phần Hai",
        "Sci-Fi / Phiêu Lưu",
        "2024",
        "8.5",
        1,
        0,
        true,
    );

    let interstellar = create_sample_movie(
        "2",
        "interstellar",
        "Interstellar",
        "Hố Đen Tử Thần",
        "Sci-Fi",
        "2014",
        "8.7",
        2,
        72,
        true,
    );

    let the_batman = create_sample_movie(
        "3",
        "the-batman",
        "The Batman",
        "Người Dơi",
        "Action / Tội Phạm",
        "2022",
        "7.8",
        3,
        18,
        false,
    );

    let spiderman = create_sample_movie(
        "4",
        "spider-man-no-way-home",
        "Spider-Man: No Way Home",
        "Người Nhện: Không Còn Nhà",
        "Action",
        "2021",
        "8.2",
        4,
        0,
        false,
    );

    let inception = create_sample_movie(
        "5",
        "inception",
        "Inception",
        "Kẻ Đánh Cắp Giấc Mơ",
        "Thriller",
        "2010",
        "8.8",
        5,
        38,
        true,
    );

    let fight_club = create_sample_movie(
        "6",
        "fight-club",
        "Fight Club",
        "Sàn Đấu Sinh Tử",
        "Drama",
        "1999",
        "8.8",
        6,
        0,
        false,
    );

    let eeaao = create_sample_movie(
        "7",
        "everything-everywhere-all-at-once",
        "Everything Everywhere All at Once",
        "Cuộc Chiến Đa Vũ Trụ",
        "Comedy / Viễn Tưởng",
        "2022",
        "7.8",
        0,
        0,
        false,
    );

    let whiplash = create_sample_movie(
        "8",
        "whiplash",
        "Whiplash",
        "Khát Vọng Nhịp Điệu",
        "Drama / Âm Nhạc",
        "2014",
        "8.5",
        0,
        0,
        false,
    );

    let la_la_land = create_sample_movie(
        "9",
        "la-la-land",
        "La La Land",
        "Những Kẻ Khờ Mộng Mơ",
        "Romance / Ca Nhạc",
        "2016",
        "8.0",
        0,
        0,
        false,
    );

    // Initial Views
    main_window.set_hero_movie(dune.clone());
    main_window.set_current_playing_movie(dune.clone());

    let trending_vec = vec![
        dune.clone(),
        interstellar.clone(),
        the_batman.clone(),
        spiderman.clone(),
        inception.clone(),
        fight_club.clone(),
    ];
    main_window.set_trending_movies(ModelRc::new(VecModel::from(trending_vec)));

    let continue_vec = vec![
        interstellar.clone(),
        inception.clone(),
        the_batman.clone(),
    ];
    main_window.set_continue_movies(ModelRc::new(VecModel::from(continue_vec)));

    let mood_vec = vec![
        eeaao.clone(),
        whiplash.clone(),
        la_la_land.clone(),
        dune.clone(),
        interstellar.clone(),
        the_batman.clone(),
    ];
    main_window.set_mood_movies(ModelRc::new(VecModel::from(mood_vec)));

    let all_movies = vec![
        dune.clone(),
        interstellar.clone(),
        the_batman.clone(),
        spiderman.clone(),
        inception.clone(),
        fight_club.clone(),
        eeaao.clone(),
        whiplash.clone(),
        la_la_land.clone(),
    ];
    main_window.set_discover_movies(ModelRc::new(VecModel::from(all_movies)));

    let genres = vec![
        GenreFilterItem { name: SharedString::from("Tất cả"), slug: SharedString::from("All"), active: true },
        GenreFilterItem { name: SharedString::from("Hành Động"), slug: SharedString::from("hanh-dong"), active: false },
        GenreFilterItem { name: SharedString::from("Viễn Tưởng"), slug: SharedString::from("vien-tuong"), active: false },
        GenreFilterItem { name: SharedString::from("Chính Kịch"), slug: SharedString::from("chinh-kich"), active: false },
        GenreFilterItem { name: SharedString::from("Kinh Dị"), slug: SharedString::from("kinh-di"), active: false },
        GenreFilterItem { name: SharedString::from("Hài Hước"), slug: SharedString::from("hai-huoc"), active: false },
        GenreFilterItem { name: SharedString::from("Tình Cảm"), slug: SharedString::from("tinh-cam"), active: false },
        GenreFilterItem { name: SharedString::from("Hoạt Hình"), slug: SharedString::from("hoat-hinh"), active: false },
    ];
    main_window.set_discover_genres(ModelRc::new(VecModel::from(genres)));

    // Load Persistent Watchlist & History from Storage
    {
        let saved = storage.get_watchlist().await;
        if !saved.is_empty() {
            let slint_saved: Vec<MovieItem> = saved.iter().map(to_slint_movie_item).collect();
            main_window.set_library_saved_movies(ModelRc::new(VecModel::from(slint_saved)));
        } else {
            let saved_vec = vec![dune.clone(), interstellar.clone(), inception.clone()];
            main_window.set_library_saved_movies(ModelRc::new(VecModel::from(saved_vec)));
        }

        let history = storage.get_history().await;
        if !history.is_empty() {
            let slint_hist: Vec<MovieItem> = history.iter().map(to_slint_movie_item).collect();
            main_window.set_continue_movies(ModelRc::new(VecModel::from(slint_hist.clone())));
            main_window.set_library_history_movies(ModelRc::new(VecModel::from(slint_hist)));
        } else {
            let history_vec = vec![interstellar.clone(), the_batman.clone()];
            main_window.set_library_history_movies(ModelRc::new(VecModel::from(history_vec)));
        }
    }

    let suggestions_vec = vec![eeaao.clone(), whiplash.clone(), fight_club.clone(), la_la_land.clone()];
    main_window.set_right_panel_suggestions(ModelRc::new(VecModel::from(suggestions_vec)));

    // Detail Movie Data
    main_window.set_detail_movie(dune.clone());
    main_window.set_detail_synopsis(SharedString::from(
        "Dune: Part Two sẽ khám phá hành trình thần thoại của Paul Atreides khi anh hợp nhất với Chani và người Fremen trên con đường trả thù những kẻ đã hủy diệt gia tộc mình. Đứng trước sự lựa chọn giữa tình yêu của đời mình và số phận của vũ trụ đã biết, anh cố gắng ngăn chặn một tương lai khủng khiếp mà chỉ anh mới có thể thấy trước."
    ));
    main_window.set_detail_director(SharedString::from("Denis Villeneuve"));

    let actors = vec![
        PersonItem { name: SharedString::from("Timothée Chalamet"), role: SharedString::from("Paul Atreides"), avatar_url: SharedString::from("") },
        PersonItem { name: SharedString::from("Zendaya"), role: SharedString::from("Chani"), avatar_url: SharedString::from("") },
        PersonItem { name: SharedString::from("Rebecca Ferguson"), role: SharedString::from("Lady Jessica"), avatar_url: SharedString::from("") },
        PersonItem { name: SharedString::from("Javier Bardem"), role: SharedString::from("Stilgar"), avatar_url: SharedString::from("") },
    ];
    main_window.set_detail_actors(ModelRc::new(VecModel::from(actors)));

    let episodes = vec![
        EpisodeItem { server_name: SharedString::from("Server VIP #1"), name: SharedString::from("Tập 01 (Full)"), slug: SharedString::from("tap-01"), link_embed: SharedString::from(""), link_m3u8: SharedString::from("") },
        EpisodeItem { server_name: SharedString::from("Server Dự Phòng #2"), name: SharedString::from("Tập 01 (Dự phòng)"), slug: SharedString::from("tap-01-bak"), link_embed: SharedString::from(""), link_m3u8: SharedString::from("") },
    ];
    main_window.set_detail_episodes(ModelRc::new(VecModel::from(episodes)));

    let keywords = vec![
        SharedString::from("Sci-Fi"),
        SharedString::from("Space Opera"),
        SharedString::from("Desert Planet"),
        SharedString::from("Frank Herbert"),
        SharedString::from("Epic"),
    ];
    let keywords_model = Rc::new(VecModel::from(keywords));
    main_window.set_detail_keywords(ModelRc::from(keywords_model));

    // Spawn Background Async Task to Fetch Live Trending Movies from API
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        tokio::spawn(async move {
            tracing::info!("Fetching live trending movies from primary API provider...");
            match agg.get_home_trending().await {
                Ok(live_movies) if !live_movies.is_empty() => {
                    tracing::info!("Successfully fetched {} live movies from API!", live_movies.len());
                    
                    // Background image caching
                    for m in &live_movies {
                        if !m.poster_url.is_empty() {
                            let url = m.poster_url.clone();
                            let c = cache.clone();
                            tokio::spawn(async move {
                                let _ = c.fetch_image(&url).await;
                            });
                        }
                    }

                    let c_ui = cache.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = window_weak.upgrade() {
                            let slint_movies: Vec<MovieItem> = live_movies
                                .iter()
                                .map(|m| {
                                    let cached = c_ui.get_cached_path(&m.poster_url);
                                    models::to_slint_movie_item_with_poster(m, cached.as_deref())
                                })
                                .collect();
                            if let Some(featured) = slint_movies.first().cloned() {
                                win.set_hero_movie(featured.clone());
                                win.set_current_playing_movie(featured);
                            }
                            win.set_trending_movies(ModelRc::new(VecModel::from(slint_movies)));
                        }
                    });
                }
                Ok(_) => tracing::info!("API returned empty movies list."),
                Err(e) => tracing::warn!("Live API fetch error (using fallback data): {}", e),
            }
        });
    }

    // Callbacks
    {
        let window_weak = main_window.as_weak();
        main_window.on_navigate(move |page| {
            tracing::info!("Navigating to: {}", page);
            if let Some(window) = window_weak.upgrade() {
                window.set_active_page(page);
            }
        });
    }

    // Select Movie -> Fetch Details from Live API
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        main_window.on_select_movie(move |slug| {
            let slug_str = slug.to_string();
            tracing::info!("User selected movie: {}", slug_str);
            if let Some(window) = window_weak.upgrade() {
                window.set_active_page(SharedString::from("Detail"));
            }

            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            let c = cache.clone();
            tokio::spawn(async move {
                if let Ok(detail) = agg_clone.get_movie_detail(&slug_str).await {
                    if !detail.summary.poster_url.is_empty() {
                        let url = detail.summary.poster_url.clone();
                        let c_fetch = c.clone();
                        tokio::spawn(async move {
                            let _ = c_fetch.fetch_image(&url).await;
                        });
                    }

                    let synopsis = SharedString::from(&detail.content);
                    let director = SharedString::from(&detail.director);
                    let actors: Vec<PersonItem> = detail.actors.iter().map(to_slint_person_item).collect();
                    let episodes: Vec<EpisodeItem> = detail.episodes.iter().map(to_slint_episode_item).collect();
                    let keywords: Vec<SharedString> = detail.keywords.iter().map(SharedString::from).collect();

                    let c_ui = c.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak.upgrade() {
                            let cached = c_ui.get_cached_path(&detail.summary.poster_url);
                            let slint_detail_movie = models::to_slint_movie_item_with_poster(&detail.summary, cached.as_deref());
                            win.set_detail_movie(slint_detail_movie);
                            win.set_detail_synopsis(synopsis);
                            win.set_detail_director(director);
                            win.set_detail_actors(ModelRc::new(VecModel::from(actors)));
                            win.set_detail_episodes(ModelRc::new(VecModel::from(episodes)));
                            win.set_detail_keywords(ModelRc::new(VecModel::from(keywords)));
                        }
                    });
                }
            });
        });
    }

    // Search Query -> Fetch Live Search Results
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        main_window.on_search_query(move |query| {
            let q = query.to_string();
            tracing::info!("Search query: {}", q);
            if q.is_empty() { return; }

            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            tokio::spawn(async move {
                if let Ok(results) = agg_clone.search_movies(&q, 1).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak.upgrade() {
                            let slint_results: Vec<MovieItem> = results.items.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_results)));
                        }
                    });
                }
            });
        });
    }

    // Change Provider Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        main_window.on_change_provider(move |provider| {
            let prov = provider.to_string();
            tracing::info!("Switched API provider to: {}", prov);
            if let Some(window) = window_weak.upgrade() {
                window.set_current_provider(provider);
            }

            let agg_clone = agg.clone();
            let win_weak = window_weak.clone();
            tokio::spawn(async move {
                agg_clone.set_active_provider(&prov).await;
                if let Ok(movies) = agg_clone.get_home_trending().await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak.upgrade() {
                            let slint_movies: Vec<MovieItem> = movies.iter().map(to_slint_movie_item).collect();
                            if let Some(feat) = slint_movies.first().cloned() {
                                win.set_hero_movie(feat.clone());
                                win.set_current_playing_movie(feat);
                            }
                            win.set_trending_movies(ModelRc::new(VecModel::from(slint_movies)));
                        }
                    });
                }
            });
        });
    }

    // Filter Genre Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        main_window.on_filter_genre(move |genre_slug| {
            let slug = genre_slug.to_string();
            tracing::info!("Filter by genre: {}", slug);
            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            tokio::spawn(async move {
                if let Ok(results) = agg_clone.get_movies_by_genre(&slug, 1).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak.upgrade() {
                            let slint_movies: Vec<MovieItem> = results.items.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_movies)));
                        }
                    });
                }
            });
        });
    }

    // Filter Type Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        main_window.on_filter_type(move |movie_type| {
            let t = movie_type.to_string();
            tracing::info!("Filter by movie type: {}", t);
            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            tokio::spawn(async move {
                if let Ok(results) = agg_clone.get_movies_by_category(&t, 1).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak.upgrade() {
                            let slint_movies: Vec<MovieItem> = results.items.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_movies)));
                        }
                    });
                }
            });
        });
    }

    // Toggle Watchlist Persistent Callback
    {
        let window_weak = main_window.as_weak();
        let stor = storage.clone();
        main_window.on_toggle_watchlist(move |slug| {
            let slug_str = slug.to_string();
            tracing::info!("Toggle watchlist for movie: {}", slug_str);
            let stor_clone = stor.clone();
            let win_weak = window_weak.clone();

            tokio::spawn(async move {
                let movie_summary = models::UnifiedMovieSummary {
                    id: slug_str.clone(),
                    slug: slug_str.clone(),
                    title: slug_str.clone(),
                    origin_title: String::new(),
                    poster_url: String::new(),
                    backdrop_url: String::new(),
                    genre: "Watchlist".to_string(),
                    year: "2024".to_string(),
                    rating: "8.5".to_string(),
                    rank: 0,
                    quality: "HD".to_string(),
                    time: String::new(),
                    category: String::new(),
                    country: String::new(),
                    is_saved: true,
                    progress: 0,
                };

                let (_is_saved, updated_list) = stor_clone.toggle_watchlist(movie_summary).await;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = win_weak.upgrade() {
                        let slint_saved: Vec<MovieItem> = updated_list.iter().map(to_slint_movie_item).collect();
                        win.set_library_saved_movies(ModelRc::new(VecModel::from(slint_saved)));
                    }
                });
            });
        });
    }

    // Play Movie -> Add to Watch History
    {
        let window_weak = main_window.as_weak();
        let stor = storage.clone();
        main_window.on_play_movie(move |slug| {
            let slug_str = slug.to_string();
            tracing::info!("Playing movie: {}", slug_str);
            let stor_clone = stor.clone();
            let win_weak = window_weak.clone();

            tokio::spawn(async move {
                let movie_summary = models::UnifiedMovieSummary {
                    id: slug_str.clone(),
                    slug: slug_str.clone(),
                    title: slug_str.clone(),
                    origin_title: String::new(),
                    poster_url: String::new(),
                    backdrop_url: String::new(),
                    genre: "Recently Played".to_string(),
                    year: "2024".to_string(),
                    rating: "8.5".to_string(),
                    rank: 0,
                    quality: "HD".to_string(),
                    time: String::new(),
                    category: String::new(),
                    country: String::new(),
                    is_saved: false,
                    progress: 25,
                };

                let updated_history = stor_clone.add_to_history(movie_summary, 25).await;
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = win_weak.upgrade() {
                        let slint_hist: Vec<MovieItem> = updated_history.iter().map(to_slint_movie_item).collect();
                        win.set_continue_movies(ModelRc::new(VecModel::from(slint_hist.clone())));
                        win.set_library_history_movies(ModelRc::new(VecModel::from(slint_hist)));
                        win.set_is_playing(true);
                    }
                });
            });
        });
    }

    // Play Episode Callback -> Open Stream URL or Web Embed Player
    {
        main_window.on_play_episode(move |ep| {
            let target_url = if !ep.link_embed.is_empty() {
                ep.link_embed.to_string()
            } else {
                ep.link_m3u8.to_string()
            };
            tracing::info!("Playing episode '{}' on server '{}': {}", ep.name, ep.server_name, target_url);
            open_target_url(&target_url);
        });
    }

    tracing::info!("Nolelamphim Desktop UI ready. Launching window...");
    main_window.run()?;
    Ok(())
}
