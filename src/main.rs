pub mod api;
pub mod config;
pub mod models;
pub mod services;

use std::rc::Rc;
use std::sync::Arc;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use config::AppConfig;
use models::{format_cinema_duration, parse_duration_seconds, to_slint_episode_item, to_slint_movie_item, to_slint_person_item};
use services::{ImageCacheService, MediaPlayerService, MovieAggregatorService, StorageService};

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


fn find_movie_in_models(win: &AppWindow, slug: &str) -> Option<MovieItem> {
    if win.get_hero_movie().slug == slug {
        return Some(win.get_hero_movie());
    }
    if win.get_current_playing_movie().slug == slug {
        return Some(win.get_current_playing_movie());
    }
    let trending = win.get_trending_movies();
    for i in 0..trending.row_count() {
        if let Some(m) = trending.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    let discover = win.get_discover_movies();
    for i in 0..discover.row_count() {
        if let Some(m) = discover.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    let mood = win.get_mood_movies();
    for i in 0..mood.row_count() {
        if let Some(m) = mood.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    let cont = win.get_continue_movies();
    for i in 0..cont.row_count() {
        if let Some(m) = cont.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    let saved = win.get_library_saved_movies();
    for i in 0..saved.row_count() {
        if let Some(m) = saved.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    let sug = win.get_right_panel_suggestions();
    for i in 0..sug.row_count() {
        if let Some(m) = sug.row_data(i) {
            if m.slug == slug {
                return Some(m);
            }
        }
    }
    None
}

fn update_movie_saved_status(win: &AppWindow, slug: &str, is_saved: bool) {
    if win.get_hero_movie().slug == slug {
        let mut hero = win.get_hero_movie();
        hero.is_saved = is_saved;
        win.set_hero_movie(hero);
    }
    if win.get_current_playing_movie().slug == slug {
        let mut cp = win.get_current_playing_movie();
        cp.is_saved = is_saved;
        win.set_current_playing_movie(cp);
    }
    let trending = win.get_trending_movies();
    for i in 0..trending.row_count() {
        if let Some(mut m) = trending.row_data(i) {
            if m.slug == slug {
                m.is_saved = is_saved;
                trending.set_row_data(i, m);
            }
        }
    }
    let discover = win.get_discover_movies();
    for i in 0..discover.row_count() {
        if let Some(mut m) = discover.row_data(i) {
            if m.slug == slug {
                m.is_saved = is_saved;
                discover.set_row_data(i, m);
            }
        }
    }
    let mood = win.get_mood_movies();
    for i in 0..mood.row_count() {
        if let Some(mut m) = mood.row_data(i) {
            if m.slug == slug {
                m.is_saved = is_saved;
                mood.set_row_data(i, m);
            }
        }
    }
    let cont = win.get_continue_movies();
    for i in 0..cont.row_count() {
        if let Some(mut m) = cont.row_data(i) {
            if m.slug == slug {
                m.is_saved = is_saved;
                cont.set_row_data(i, m);
            }
        }
    }
    let sug = win.get_right_panel_suggestions();
    for i in 0..sug.row_count() {
        if let Some(mut m) = sug.row_data(i) {
            if m.slug == slug {
                m.is_saved = is_saved;
                sug.set_row_data(i, m);
            }
        }
    }
}

fn trigger_play_movie(
    slug_str: String,
    window_weak: slint::Weak<AppWindow>,
    agg: Arc<MovieAggregatorService>,
    stor: Arc<StorageService>,
    player: Arc<MediaPlayerService>,
) {
    let win_weak_ui = window_weak.clone();
    let slug_ui = slug_str.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(win) = win_weak_ui.upgrade() {
            if let Some(movie) = find_movie_in_models(&win, &slug_ui) {
                win.set_current_playing_movie(movie);
            }
            win.set_is_playing(true);
            win.set_active_page(SharedString::from("Player"));
            win.set_player_progress_seconds(0);
            win.set_player_progress_percent(0);
            win.set_player_duration_seconds(7200);
            win.set_player_time_display(SharedString::from("00:00:00 / 02:00:00"));

            // Instant fallback episodes so player is active in 0ms with zero lag
            let default_episodes = vec![
                EpisodeItem {
                    server_name: SharedString::from("Server VIP #1"),
                    name: SharedString::from("Tập 01 (Full HD)"),
                    slug: SharedString::from("tap-01"),
                    link_embed: SharedString::from(""),
                    link_m3u8: SharedString::from(""),
                },
                EpisodeItem {
                    server_name: SharedString::from("Server VIP #2"),
                    name: SharedString::from("Tập 02"),
                    slug: SharedString::from("tap-02"),
                    link_embed: SharedString::from(""),
                    link_m3u8: SharedString::from(""),
                },
                EpisodeItem {
                    server_name: SharedString::from("Server Dự Phòng"),
                    name: SharedString::from("Tập 03"),
                    slug: SharedString::from("tap-03"),
                    link_embed: SharedString::from(""),
                    link_m3u8: SharedString::from(""),
                },
            ];
            win.set_current_episodes(ModelRc::new(VecModel::from(default_episodes.clone())));
            if let Some(ep) = default_episodes.first() {
                win.set_current_episode(ep.clone());
            }
        }
    });

    let win_weak_player = window_weak.clone();
    tokio::spawn(async move {
        if let Ok(detail) = agg.get_movie_detail(&slug_str).await {
            let episodes: Vec<EpisodeItem> = detail.episodes.iter().map(to_slint_episode_item).collect();
            let first_ep = episodes.first().cloned();
            let first_ep_for_player = first_ep.clone();
            let duration_secs = parse_duration_seconds(&detail.summary.time);
            let time_display = format_cinema_duration(0, duration_secs as i64);

            let updated_hist = stor.add_to_history(detail.summary.clone(), 25).await;
            let win_weak_hist = window_weak.clone();
            let summary_clone = detail.summary.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(win) = win_weak_hist.upgrade() {
                    let slint_hist: Vec<MovieItem> = updated_hist.iter().map(to_slint_movie_item).collect();
                    win.set_continue_movies(ModelRc::new(VecModel::from(slint_hist.clone())));
                    win.set_library_history_movies(ModelRc::new(VecModel::from(slint_hist)));
                    if !episodes.is_empty() {
                        win.set_current_episodes(ModelRc::new(VecModel::from(episodes.clone())));
                        win.set_detail_episodes(ModelRc::new(VecModel::from(episodes)));
                    }
                    if let Some(ref ep) = first_ep {
                        win.set_current_episode(ep.clone());
                        if !ep.server_name.is_empty() {
                            win.set_player_server(ep.server_name.clone());
                        }
                    }
                    win.set_player_duration_seconds(duration_secs);
                    win.set_player_progress_seconds(0);
                    win.set_player_progress_percent(0);
                    win.set_player_time_display(SharedString::from(time_display));
                    let movie_item = to_slint_movie_item(&summary_clone);
                    win.set_current_playing_movie(movie_item);
                }
            });

            // Automatically launch real stream in Cinema Media Player!
            if let Some(ep) = first_ep_for_player {
                let stream_url = if !ep.link_m3u8.is_empty() {
                    ep.link_m3u8.to_string()
                } else {
                    ep.link_embed.to_string()
                };
                if !stream_url.is_empty() {
                    tracing::info!("Auto-launching real stream in Cinema Media Player: {}", stream_url);
                    let _ = player.play(&stream_url, &detail.summary.title, &ep.name, win_weak_player).await;
                }
            }
        }
    });
}

fn load_trending_images_concurrent(
    live_movies: Vec<models::UnifiedMovieSummary>,
    cache: Arc<ImageCacheService>,
    window_weak: slint::Weak<AppWindow>,
) {
    tokio::spawn(async move {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(2));
        for (idx, m) in live_movies.into_iter().enumerate() {
            if m.poster_url.is_empty() {
                continue;
            }
            let c = cache.clone();
            let win_weak_row = window_weak.clone();
            let url = m.poster_url;
            let sem = semaphore.clone();
            tokio::spawn(async move {
                let _permit = sem.acquire().await;
                if let Ok(decoded) = c.fetch_and_decode(&url, 320, 480).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_row.upgrade() {
                            let model = win.get_trending_movies();
                            if let Some(mut item) = model.row_data(idx) {
                                item.poster = decoded.to_slint_image();
                                item.has_poster = true;
                                model.set_row_data(idx, item.clone());
                                if idx == 0 {
                                    win.set_hero_movie(item.clone());
                                    win.set_current_playing_movie(item);
                                }
                            }
                        }
                    });
                }
            });
        }
    });
}

fn load_discover_images_concurrent(
    movies: Vec<models::UnifiedMovieSummary>,
    cache: Arc<ImageCacheService>,
    window_weak: slint::Weak<AppWindow>,
) {
    tokio::spawn(async move {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(2));
        for (idx, m) in movies.into_iter().enumerate() {
            if m.poster_url.is_empty() {
                continue;
            }
            let c = cache.clone();
            let win_weak_row = window_weak.clone();
            let url = m.poster_url;
            let sem = semaphore.clone();
            tokio::spawn(async move {
                let _permit = sem.acquire().await;
                if let Ok(decoded) = c.fetch_and_decode(&url, 320, 480).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_row.upgrade() {
                            let model = win.get_discover_movies();
                            if let Some(mut item) = model.row_data(idx) {
                                item.poster = decoded.to_slint_image();
                                item.has_poster = true;
                                model.set_row_data(idx, item);
                            }
                        }
                    });
                }
            });
        }
    });
}

#[derive(Clone, Debug)]
enum DiscoverFilter {
    Trending,
    Genre(String),
    Category(String),
    Search(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Nolelamphim Desktop Application...");

    let config = AppConfig::from_env();
    let aggregator = Arc::new(MovieAggregatorService::new(&config));
    let storage = Arc::new(StorageService::new());
    let image_cache = Arc::new(ImageCacheService::new(config.enable_image_cache));
    let media_player = Arc::new(MediaPlayerService::new());
    let discover_filter = Arc::new(tokio::sync::Mutex::new(DiscoverFilter::Trending));

    let main_window = AppWindow::new()?;

    let raw_handle_opt = {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = main_window.window().window_handle();
        if let Ok(h) = handle.window_handle() {
            match h.as_raw() {
                RawWindowHandle::Xlib(xlib) => {
                    tracing::info!("Detected Xlib window ID: {}", xlib.window);
                    Some(xlib.window)
                }
                RawWindowHandle::Xcb(xcb) => {
                    tracing::info!("Detected Xcb window ID: {}", xcb.window.get());
                    Some(xcb.window.get() as u64)
                }
                _ => None,
            }
        } else {
            None
        }
    };
    if let Some(xid) = raw_handle_opt {
        media_player.set_parent_window_id(xid).await;
    }

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
        EpisodeItem { server_name: SharedString::from("Server VIP #1"), name: SharedString::from("Tập 01 (Full)"), slug: SharedString::from("tap-01"), link_embed: SharedString::from("https://www.youtube.com/embed/Way9Dexny3w"), link_m3u8: SharedString::from("") },
        EpisodeItem { server_name: SharedString::from("Server VIP #2"), name: SharedString::from("Tập 02"), slug: SharedString::from("tap-02"), link_embed: SharedString::from("https://www.youtube.com/embed/Way9Dexny3w"), link_m3u8: SharedString::from("") },
        EpisodeItem { server_name: SharedString::from("Server Dự Phòng #3"), name: SharedString::from("Tập 03"), slug: SharedString::from("tap-03"), link_embed: SharedString::from("https://www.youtube.com/embed/Way9Dexny3w"), link_m3u8: SharedString::from("") },
    ];
    main_window.set_detail_episodes(ModelRc::new(VecModel::from(episodes.clone())));
    main_window.set_current_episodes(ModelRc::new(VecModel::from(episodes.clone())));
    if let Some(first_ep) = episodes.first() {
        main_window.set_current_episode(first_ep.clone());
    }
    main_window.set_player_time_display(SharedString::from("00:35:10 / 02:14:00"));
    main_window.set_player_progress_percent(26);
    main_window.set_player_progress_seconds(2110);
    main_window.set_player_duration_seconds(8040);
    main_window.set_player_server(SharedString::from("Server VIP #1"));

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
                    
                    // Step 1: Immediate zero-latency UI display with placeholder
                    let win_weak_init = window_weak.clone();
                    let live_movies_init = live_movies.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_init.upgrade() {
                            let slint_initial: Vec<MovieItem> = live_movies_init.iter().map(to_slint_movie_item).collect();
                            if let Some(featured) = slint_initial.first().cloned() {
                                win.set_hero_movie(featured.clone());
                                win.set_current_playing_movie(featured);
                            }
                            win.set_trending_movies(ModelRc::new(VecModel::from(slint_initial.clone())));
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_initial.clone())));

                            if slint_initial.len() >= 6 {
                                let mood_slice: Vec<MovieItem> = slint_initial.iter().skip(1).take(6).cloned().collect();
                                win.set_mood_movies(ModelRc::new(VecModel::from(mood_slice)));
                            }
                            if slint_initial.len() >= 4 {
                                let sug_slice: Vec<MovieItem> = slint_initial.iter().rev().take(4).cloned().collect();
                                win.set_right_panel_suggestions(ModelRc::new(VecModel::from(sug_slice)));
                            }
                            win.set_discover_current_page(1);
                            win.set_discover_total_pages(10);
                        }
                    });

                    // Step 2: Background parallel image loading with non-blocking row-by-row updates
                    load_trending_images_concurrent(live_movies.clone(), cache.clone(), window_weak.clone());
                    load_discover_images_concurrent(live_movies, cache.clone(), window_weak.clone());
                }
                Ok(_) => tracing::info!("API returned empty movies list."),
                Err(e) => tracing::warn!("Live API fetch error (using fallback data): {}", e),
            }
        });
    }

    // Callbacks
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_navigate(move |page| {
            tracing::info!("Navigating to: {}", page);
            if let Some(window) = window_weak.upgrade() {
                window.set_active_page(page.clone());
                let p = player.clone();
                let page_str = page.to_string();
                tokio::spawn(async move {
                    if page_str == "Player" {
                        p.show_embedded().await;
                    } else {
                        p.hide_embedded().await;
                    }
                });
            }
        });
    }

    // Responsive dynamic viewport geometry synchronization with native subwindow
    {
        let player = media_player.clone();
        let win_weak = main_window.as_weak();
        main_window.on_update_viewport_geometry(move |x_len, y_len, w_len, h_len| {
            let p = player.clone();
            if let Some(win) = win_weak.upgrade() {
                let scale = win.window().scale_factor();
                let x = (x_len * scale) as i32;
                let y = (y_len * scale) as i32;
                let w = (w_len * scale).max(100.0) as u32;
                let h = (h_len * scale).max(100.0) as u32;
                tokio::spawn(async move {
                    p.update_viewport_rect(x, y, w, h).await;
                });
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
                    let synopsis = SharedString::from(&detail.content);
                    let director = SharedString::from(&detail.director);
                    let actors: Vec<PersonItem> = detail.actors.iter().map(to_slint_person_item).collect();
                    let episodes: Vec<EpisodeItem> = detail.episodes.iter().map(to_slint_episode_item).collect();
                    let keywords: Vec<SharedString> = detail.keywords.iter().map(SharedString::from).collect();
                    // Show detail immediately with info & placeholder (0ms)
                    let summary_init = detail.summary.clone();
                    let win_weak_ui = win_weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_detail_initial = to_slint_movie_item(&summary_init);
                            win.set_detail_movie(slint_detail_initial);
                            win.set_detail_synopsis(synopsis);
                            win.set_detail_director(director);
                            win.set_detail_actors(ModelRc::new(VecModel::from(actors)));
                            win.set_detail_episodes(ModelRc::new(VecModel::from(episodes)));
                            win.set_detail_keywords(ModelRc::new(VecModel::from(keywords)));
                        }
                    });

                    // Asynchronously fetch & decode poster in background with zero UI freeze
                    if !detail.summary.poster_url.is_empty() {
                        let url = detail.summary.poster_url.clone();
                        let c_fetch = c.clone();
                        let win_weak_detail = win_weak.clone();
                        let summary_clone = detail.summary.clone();
                        tokio::spawn(async move {
                            if let Ok(decoded) = c_fetch.fetch_and_decode(&url, 360, 540).await {
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(win) = win_weak_detail.upgrade() {
                                        let updated = models::to_slint_movie_item_with_decoded_poster(&summary_clone, Some(&decoded));
                                        win.set_detail_movie(updated);
                                    }
                                });
                            }
                        });
                    }
                }
            });
        });
    }

    // Search Query -> Fetch Live Search Results
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        let disc_filter = discover_filter.clone();
        main_window.on_search_query(move |query| {
            let q = query.to_string();
            tracing::info!("Search query: {}", q);
            if q.is_empty() { return; }

            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            let c_fetch = cache.clone();
            let df = disc_filter.clone();
            tokio::spawn(async move {
                {
                    let mut lock = df.lock().await;
                    *lock = DiscoverFilter::Search(q.clone());
                }
                if let Ok(results) = agg_clone.search_movies(&q, 1).await {
                    let win_weak_ui = win_weak.clone();
                    let items_clone = results.items.clone();
                    let total_pages = results.pagination.total_pages.max(1) as i32;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_results: Vec<MovieItem> = items_clone.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_results)));
                            win.set_discover_current_page(1);
                            win.set_discover_total_pages(total_pages);
                        }
                    });
                    load_discover_images_concurrent(results.items, c_fetch, win_weak);
                }
            });
        });
    }

    // Change Provider Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        main_window.on_change_provider(move |provider| {
            let prov = provider.to_string();
            tracing::info!("Switched API provider to: {}", prov);
            if let Some(window) = window_weak.upgrade() {
                window.set_current_provider(provider);
            }

            let agg_clone = agg.clone();
            let win_weak = window_weak.clone();
            let c_fetch = cache.clone();
            tokio::spawn(async move {
                agg_clone.set_active_provider(&prov).await;
                if let Ok(movies) = agg_clone.get_home_trending().await {
                    let win_weak_ui = win_weak.clone();
                    let movies_clone = movies.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_movies: Vec<MovieItem> = movies_clone.iter().map(to_slint_movie_item).collect();
                            if let Some(feat) = slint_movies.first().cloned() {
                                win.set_hero_movie(feat.clone());
                                win.set_current_playing_movie(feat);
                            }
                            win.set_trending_movies(ModelRc::new(VecModel::from(slint_movies)));
                        }
                    });
                    load_trending_images_concurrent(movies, c_fetch, win_weak);
                }
            });
        });
    }

    // Filter Genre Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        let disc_filter = discover_filter.clone();
        main_window.on_filter_genre(move |genre_slug| {
            let slug = genre_slug.to_string();
            tracing::info!("Filter by genre: {}", slug);
            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            let c_fetch = cache.clone();
            let slug_for_active = slug.clone();
            let df = disc_filter.clone();

            if let Some(win) = window_weak.upgrade() {
                let genres_model = win.get_discover_genres();
                for i in 0..genres_model.row_count() {
                    if let Some(mut g) = genres_model.row_data(i) {
                        g.active = g.slug == slug_for_active;
                        genres_model.set_row_data(i, g);
                    }
                }
            }

            tokio::spawn(async move {
                {
                    let mut lock = df.lock().await;
                    *lock = DiscoverFilter::Genre(slug.clone());
                }
                let fetch_res = if slug == "All" || slug.is_empty() {
                    agg_clone.get_home_trending().await.map(|items| models::UnifiedMovieListResponse {
                        items,
                        pagination: models::UnifiedPagination {
                            current_page: 1,
                            total_items: 20,
                            total_pages: 10,
                            items_per_page: 20,
                        }
                    })
                } else if slug == "phim-chieu-rap" || slug == "hoat-hinh" || slug == "phim-le" || slug == "phim-bo" || slug == "tv-shows" {
                    agg_clone.get_movies_by_category(&slug, 1).await
                } else {
                    agg_clone.get_movies_by_genre(&slug, 1).await
                };

                if let Ok(results) = fetch_res {
                    let win_weak_ui = win_weak.clone();
                    let items_clone = results.items.clone();
                    let total_pages = results.pagination.total_pages.max(1) as i32;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_movies: Vec<MovieItem> = items_clone.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_movies)));
                            win.set_discover_current_page(1);
                            win.set_discover_total_pages(total_pages);
                        }
                    });
                    load_discover_images_concurrent(results.items, c_fetch, win_weak);
                }
            });
        });
    }

    // Filter Type Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        let disc_filter = discover_filter.clone();
        main_window.on_filter_type(move |movie_type| {
            let t = movie_type.to_string();
            tracing::info!("Filter by movie type: {}", t);
            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            let c_fetch = cache.clone();
            let df = disc_filter.clone();
            tokio::spawn(async move {
                {
                    let mut lock = df.lock().await;
                    *lock = DiscoverFilter::Category(t.clone());
                }
                let fetch_res = if t == "all" || t.is_empty() {
                    agg_clone.get_home_trending().await.map(|items| models::UnifiedMovieListResponse {
                        items,
                        pagination: models::UnifiedPagination {
                            current_page: 1,
                            total_items: 20,
                            total_pages: 10,
                            items_per_page: 20,
                        }
                    })
                } else {
                    agg_clone.get_movies_by_category(&t, 1).await
                };

                if let Ok(results) = fetch_res {
                    let win_weak_ui = win_weak.clone();
                    let items_clone = results.items.clone();
                    let total_pages = results.pagination.total_pages.max(1) as i32;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_movies: Vec<MovieItem> = items_clone.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_movies)));
                            win.set_discover_current_page(1);
                            win.set_discover_total_pages(total_pages);
                        }
                    });
                    load_discover_images_concurrent(results.items, c_fetch, win_weak);
                }
            });
        });
    }

    // Discover Pagination Callback (Next / Prev Page)
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let cache = image_cache.clone();
        let disc_filter = discover_filter.clone();
        main_window.on_change_page(move |page| {
            let page_num = page.max(1) as u32;
            tracing::info!("Changing Discover page to: {}", page_num);
            let win_weak = window_weak.clone();
            let agg_clone = agg.clone();
            let c_fetch = cache.clone();
            let df = disc_filter.clone();
            tokio::spawn(async move {
                let current_filter = {
                    let lock = df.lock().await;
                    lock.clone()
                };

                let fetch_res = match current_filter {
                    DiscoverFilter::Trending => {
                        agg_clone.get_home_trending().await.map(|items| models::UnifiedMovieListResponse {
                            items,
                            pagination: models::UnifiedPagination {
                                current_page: page_num,
                                total_items: 20,
                                total_pages: 10,
                                items_per_page: 20,
                            }
                        })
                    }
                    DiscoverFilter::Genre(ref slug) => {
                        if slug == "All" || slug.is_empty() {
                            agg_clone.get_home_trending().await.map(|items| models::UnifiedMovieListResponse {
                                items,
                                pagination: models::UnifiedPagination {
                                    current_page: page_num,
                                    total_items: 20,
                                    total_pages: 10,
                                    items_per_page: 20,
                                }
                            })
                        } else if slug == "phim-chieu-rap" || slug == "hoat-hinh" || slug == "phim-le" || slug == "phim-bo" || slug == "tv-shows" {
                            agg_clone.get_movies_by_category(slug, page_num).await
                        } else {
                            agg_clone.get_movies_by_genre(slug, page_num).await
                        }
                    }
                    DiscoverFilter::Category(ref cat) => {
                        if cat == "all" || cat.is_empty() {
                            agg_clone.get_home_trending().await.map(|items| models::UnifiedMovieListResponse {
                                items,
                                pagination: models::UnifiedPagination {
                                    current_page: page_num,
                                    total_items: 20,
                                    total_pages: 10,
                                    items_per_page: 20,
                                }
                            })
                        } else {
                            agg_clone.get_movies_by_category(cat, page_num).await
                        }
                    }
                    DiscoverFilter::Search(ref kw) => {
                        agg_clone.search_movies(kw, page_num).await
                    }
                };

                if let Ok(results) = fetch_res {
                    let win_weak_ui = win_weak.clone();
                    let items_clone = results.items.clone();
                    let total_pages = results.pagination.total_pages.max(1) as i32;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(win) = win_weak_ui.upgrade() {
                            let slint_movies: Vec<MovieItem> = items_clone.iter().map(to_slint_movie_item).collect();
                            win.set_discover_movies(ModelRc::new(VecModel::from(slint_movies)));
                            win.set_discover_current_page(page_num as i32);
                            win.set_discover_total_pages(total_pages);
                        }
                    });
                    load_discover_images_concurrent(results.items, c_fetch, win_weak);
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

            if let Some(win) = window_weak.upgrade() {
                let maybe_movie = find_movie_in_models(&win, &slug_str);
                let current_saved = maybe_movie.as_ref().map(|m| m.is_saved).unwrap_or(false);
                let new_saved = !current_saved;

                // 1. Immediately toggle visual heart state across all views
                update_movie_saved_status(&win, &slug_str, new_saved);

                // 2. Prepare real metadata for persistent storage
                let movie_summary = if let Some(m) = maybe_movie {
                    models::UnifiedMovieSummary {
                        id: m.id.to_string(),
                        slug: m.slug.to_string(),
                        title: m.title.to_string(),
                        origin_title: m.origin_title.to_string(),
                        poster_url: m.poster_url.to_string(),
                        backdrop_url: m.backdrop_url.to_string(),
                        genre: m.genre.to_string(),
                        year: m.year.to_string(),
                        rating: m.rating.to_string(),
                        rank: m.rank,
                        quality: m.quality.to_string(),
                        time: String::new(),
                        category: String::new(),
                        country: String::new(),
                        is_saved: new_saved,
                        progress: m.progress,
                    }
                } else {
                    models::UnifiedMovieSummary {
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
                        is_saved: new_saved,
                        progress: 0,
                    }
                };

                // 3. Persist and refresh saved movies list in Library
                tokio::spawn(async move {
                    let (_saved, updated_list) = stor_clone.toggle_watchlist(movie_summary).await;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(w) = win_weak.upgrade() {
                            let slint_saved: Vec<MovieItem> = updated_list.iter().map(to_slint_movie_item).collect();
                            w.set_library_saved_movies(ModelRc::new(VecModel::from(slint_saved)));
                        }
                    });
                });
            }
        });
    }

    // Play Movie -> Add to Watch History & Launch Stream
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let stor = storage.clone();
        let pl = media_player.clone();
        main_window.on_play_movie(move |slug| {
            let slug_str = slug.to_string();
            tracing::info!("Playing movie: {}", slug_str);
            trigger_play_movie(slug_str, window_weak.clone(), agg.clone(), stor.clone(), pl.clone());
        });
    }

    // Prev Movie Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let stor = storage.clone();
        let pl = media_player.clone();
        main_window.on_prev_movie(move | | {
            if let Some(win) = window_weak.upgrade() {
                let current_slug = win.get_current_playing_movie().slug.to_string();
                let trending = win.get_trending_movies();
                let count = trending.row_count();
                if count > 0 {
                    let mut current_idx = 0;
                    for i in 0..count {
                        if let Some(m) = trending.row_data(i) {
                            if m.slug == current_slug {
                                current_idx = i;
                                break;
                            }
                        }
                    }
                    let prev_idx = if current_idx == 0 { count - 1 } else { current_idx - 1 };
                    if let Some(m) = trending.row_data(prev_idx) {
                        let slug = m.slug.to_string();
                        win.set_current_playing_movie(m);
                        win.set_is_playing(true);
                        trigger_play_movie(slug, window_weak.clone(), agg.clone(), stor.clone(), pl.clone());
                    }
                }
            }
        });
    }

    // Next Movie Callback
    {
        let window_weak = main_window.as_weak();
        let agg = aggregator.clone();
        let stor = storage.clone();
        let pl = media_player.clone();
        main_window.on_next_movie(move | | {
            if let Some(win) = window_weak.upgrade() {
                let current_slug = win.get_current_playing_movie().slug.to_string();
                let trending = win.get_trending_movies();
                let count = trending.row_count();
                if count > 0 {
                    let mut current_idx = 0;
                    for i in 0..count {
                        if let Some(m) = trending.row_data(i) {
                            if m.slug == current_slug {
                                current_idx = i;
                                break;
                            }
                        }
                    }
                    let next_idx = (current_idx + 1) % count;
                    if let Some(m) = trending.row_data(next_idx) {
                        let slug = m.slug.to_string();
                        win.set_current_playing_movie(m);
                        win.set_is_playing(true);
                        trigger_play_movie(slug, window_weak.clone(), agg.clone(), stor.clone(), pl.clone());
                    }
                }
            }
        });
    }

    // Play/Pause Toggle -> Control Native Cinema Media Player
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_toggle_play(move || {
            if let Some(win) = window_weak.upgrade() {
                let playing = win.get_is_playing();
                tracing::info!("Cinema player play state toggled: {}", playing);
                let p = player.clone();
                tokio::spawn(async move {
                    let _ = p.toggle_pause().await;
                });
            }
        });
    }

    // Change Volume
    {
        let player = media_player.clone();
        main_window.on_change_volume(move |vol| {
            let p = player.clone();
            tokio::spawn(async move {
                let _ = p.set_volume(vol).await;
            });
        });
    }

    // Play Episode Callback -> Launch Stream via Cinema Media Player
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_play_episode(move |ep| {
            let target_url = if !ep.link_m3u8.is_empty() {
                ep.link_m3u8.to_string()
            } else {
                ep.link_embed.to_string()
            };
            tracing::info!("Playing episode '{}' on server '{}': {}", ep.name, ep.server_name, target_url);

            let movie_title = if let Some(win) = window_weak.upgrade() {
                win.set_current_episode(ep.clone());
                win.set_is_playing(true);
                win.set_active_page(SharedString::from("Player"));
                win.set_player_progress_seconds(0);
                win.set_player_progress_percent(0);
                let duration = win.get_player_duration_seconds();
                win.set_player_time_display(SharedString::from(format_cinema_duration(0, duration as i64)));
                if !ep.server_name.is_empty() {
                    win.set_player_server(ep.server_name.clone());
                }
                win.get_current_playing_movie().title.to_string()
            } else {
                String::new()
            };

            if !target_url.is_empty() {
                let p = player.clone();
                let ep_name = ep.name.to_string();
                let ww = window_weak.clone();
                tokio::spawn(async move {
                    let _ = p.play(&target_url, &movie_title, &ep_name, ww).await;
                });
            }
        });
    }

    // Skip 10 Seconds Forward / Backward
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_skip_10(move |forward| {
            if let Some(win) = window_weak.upgrade() {
                let current_sec = win.get_player_progress_seconds();
                let duration = win.get_player_duration_seconds().max(1);
                let new_sec = if forward {
                    (current_sec + 10).min(duration)
                } else {
                    (current_sec - 10).max(0)
                };
                let percent = (new_sec * 100) / duration;
                win.set_player_progress_seconds(new_sec);
                win.set_player_progress_percent(percent);
                win.set_player_time_display(SharedString::from(format_cinema_duration(new_sec as i64, duration as i64)));
                let p = player.clone();
                let secs = if forward { 10.0 } else { -10.0 };
                tokio::spawn(async move {
                    let _ = p.skip_seconds(secs).await;
                });
            }
        });
    }

    // Seek to Percentage (0..100)
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_seek_percent(move |p| {
            if let Some(win) = window_weak.upgrade() {
                let clamped = p.clamp(0, 100);
                let duration = win.get_player_duration_seconds().max(1);
                let new_sec = (clamped as i64 * duration as i64 / 100) as i32;
                win.set_player_progress_percent(clamped);
                win.set_player_progress_seconds(new_sec);
                win.set_player_time_display(SharedString::from(format_cinema_duration(new_sec as i64, duration as i64)));
                let pl = player.clone();
                tokio::spawn(async move {
                    let _ = pl.seek_percent(clamped).await;
                });
            }
        });
    }

    // Cycle Playback Speed (1.0x -> 1.25x -> 1.5x -> 2.0x -> 0.75x -> 1.0x)
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_cycle_speed(move || {
            if let Some(win) = window_weak.upgrade() {
                let current_speed = win.get_player_speed().to_string();
                let (next_speed, speed_val) = match current_speed.as_str() {
                    "1.0x" => ("1.25x", 1.25),
                    "1.25x" => ("1.5x", 1.5),
                    "1.5x" => ("2.0x", 2.0),
                    "2.0x" => ("0.75x", 0.75),
                    _ => ("1.0x", 1.0),
                };
                win.set_player_speed(SharedString::from(next_speed));
                let pl = player.clone();
                tokio::spawn(async move {
                    let _ = pl.set_speed(speed_val).await;
                });
            }
        });
    }

    // Open Cinema Theater Mode (100% In-App)
    {
        let window_weak = main_window.as_weak();
        main_window.on_open_fullscreen(move || {
            if let Some(win) = window_weak.upgrade() {
                let is_theater = win.get_theater_mode();
                tracing::info!("Cinema player theater mode toggled: {}", is_theater);
                win.invoke_recompute_viewport();
            }
        });
    }

    // Next Episode Callback
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_next_episode(move || {
            if let Some(win) = window_weak.upgrade() {
                let current_ep = win.get_current_episode();
                let episodes = win.get_current_episodes();
                let count = episodes.row_count();
                if count > 0 {
                    let mut current_idx = 0;
                    for i in 0..count {
                        if let Some(ep) = episodes.row_data(i) {
                            if ep.slug == current_ep.slug && ep.server_name == current_ep.server_name {
                                current_idx = i;
                                break;
                            }
                        }
                    }
                    let next_idx = (current_idx + 1) % count;
                    if let Some(next_ep) = episodes.row_data(next_idx) {
                        tracing::info!("Cinema player next episode: {} ({})", next_ep.name, next_ep.server_name);
                        win.set_current_episode(next_ep.clone());
                        win.set_player_progress_seconds(0);
                        win.set_player_progress_percent(0);
                        let duration = win.get_player_duration_seconds();
                        win.set_player_time_display(SharedString::from(format_cinema_duration(0, duration as i64)));
                        win.set_is_playing(true);

                        if !next_ep.server_name.is_empty() {
                            win.set_player_server(next_ep.server_name.clone());
                        }

                        let target_url = if !next_ep.link_m3u8.is_empty() {
                            next_ep.link_m3u8.to_string()
                        } else {
                            next_ep.link_embed.to_string()
                        };
                        let movie_title = win.get_current_playing_movie().title.to_string();
                        if !target_url.is_empty() {
                            let pl = player.clone();
                            let ep_name = next_ep.name.to_string();
                            let ww = window_weak.clone();
                            tokio::spawn(async move {
                                let _ = pl.play(&target_url, &movie_title, &ep_name, ww).await;
                            });
                        }
                    }
                }
            }
        });
    }

    // Switch Streaming Server in Cinema Player
    {
        let window_weak = main_window.as_weak();
        let player = media_player.clone();
        main_window.on_switch_server(move |server_name| {
            let srv = server_name.to_string();
            tracing::info!("Switching cinema streaming server to: {}", srv);
            if let Some(win) = window_weak.upgrade() {
                win.set_player_server(SharedString::from(&srv));
                let current_ep = win.get_current_episode();
                let all_episodes = win.get_detail_episodes();
                let count = all_episodes.row_count();
                let mut found_ep = None;
                for i in 0..count {
                    if let Some(ep) = all_episodes.row_data(i) {
                        if (ep.server_name == srv || ep.server_name.contains(&srv))
                            && (ep.name == current_ep.name || ep.slug == current_ep.slug)
                        {
                            found_ep = Some(ep);
                            break;
                        }
                    }
                }
                if found_ep.is_none() {
                    for i in 0..count {
                        if let Some(ep) = all_episodes.row_data(i) {
                            if ep.server_name == srv || ep.server_name.contains(&srv) {
                                found_ep = Some(ep);
                                break;
                            }
                        }
                    }
                }

                if let Some(ep) = found_ep {
                    win.set_current_episode(ep.clone());
                    let target_url = if !ep.link_m3u8.is_empty() {
                        ep.link_m3u8.to_string()
                    } else {
                        ep.link_embed.to_string()
                    };
                    if !target_url.is_empty() {
                        let p = player.clone();
                        let title = win.get_current_playing_movie().title.to_string();
                        let ep_name = ep.name.to_string();
                        let ww = window_weak.clone();
                        tokio::spawn(async move {
                            let _ = p.play(&target_url, &title, &ep_name, ww).await;
                        });
                    }
                }
            }
        });
    }

    // Cinema Playback Progress Ticker (1-second tick when is_playing == true)
    {
        let window_weak = main_window.as_weak();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let win_weak_tick = window_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = win_weak_tick.upgrade() {
                        if win.get_is_playing() {
                            let curr = win.get_player_progress_seconds();
                            let dur = win.get_player_duration_seconds().max(1);
                            if curr < dur {
                                let new_sec = curr + 1;
                                let pct = (new_sec * 100) / dur;
                                win.set_player_progress_seconds(new_sec);
                                win.set_player_progress_percent(pct);
                                win.set_player_time_display(SharedString::from(format_cinema_duration(new_sec as i64, dur as i64)));
                            }
                        }
                    }
                });
            }
        });
    }

    tracing::info!("Nolelamphim Desktop UI ready. Launching window...");
    main_window.run()?;
    Ok(())
}
