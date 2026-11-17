use std::rc::Rc;
use slint::{ModelRc, SharedString, VecModel};

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
        genre: SharedString::from(genre),
        year: SharedString::from(year),
        rating: SharedString::from(rating),
        rank,
        quality: SharedString::from("4K HDR"),
        progress,
        is_saved,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Nolelamphim Desktop Application...");

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

    // Hero Movie
    main_window.set_hero_movie(dune.clone());
    main_window.set_current_playing_movie(dune.clone());

    // Trending Movies (6 items)
    let trending_vec = vec![
        dune.clone(),
        interstellar.clone(),
        the_batman.clone(),
        spiderman.clone(),
        inception.clone(),
        fight_club.clone(),
    ];
    main_window.set_trending_movies(ModelRc::new(VecModel::from(trending_vec.clone())));

    // Continue Watching (3 items)
    let continue_vec = vec![
        interstellar.clone(),
        inception.clone(),
        the_batman.clone(),
    ];
    main_window.set_continue_movies(ModelRc::new(VecModel::from(continue_vec)));

    // Mood Movies
    let mood_vec = vec![
        eeaao.clone(),
        whiplash.clone(),
        la_la_land.clone(),
        dune.clone(),
        interstellar.clone(),
        the_batman.clone(),
    ];
    main_window.set_mood_movies(ModelRc::new(VecModel::from(mood_vec)));

    // Discover Movies (All items)
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
    main_window.set_discover_movies(ModelRc::new(VecModel::from(all_movies.clone())));

    // Discover Genres
    let genres = vec![
        GenreFilterItem { name: SharedString::from("Tất cả"), slug: SharedString::from("All"), active: true },
        GenreFilterItem { name: SharedString::from("Hành Động"), slug: SharedString::from("Action"), active: false },
        GenreFilterItem { name: SharedString::from("Viễn Tưởng"), slug: SharedString::from("Sci-Fi"), active: false },
        GenreFilterItem { name: SharedString::from("Chính Kịch"), slug: SharedString::from("Drama"), active: false },
        GenreFilterItem { name: SharedString::from("Kinh Dị"), slug: SharedString::from("Thriller"), active: false },
        GenreFilterItem { name: SharedString::from("Hài Hước"), slug: SharedString::from("Comedy"), active: false },
        GenreFilterItem { name: SharedString::from("Tình Cảm"), slug: SharedString::from("Romance"), active: false },
        GenreFilterItem { name: SharedString::from("Hoạt Hình"), slug: SharedString::from("Anime"), active: false },
    ];
    main_window.set_discover_genres(ModelRc::new(VecModel::from(genres)));

    // Library Data
    let saved_vec = vec![dune.clone(), interstellar.clone(), inception.clone()];
    main_window.set_library_saved_movies(ModelRc::new(VecModel::from(saved_vec)));
    let history_vec = vec![interstellar.clone(), the_batman.clone()];
    main_window.set_library_history_movies(ModelRc::new(VecModel::from(history_vec)));

    // Right Panel Suggestions
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

    // Callbacks Handling
    let window_weak = main_window.as_weak();
    main_window.on_navigate(move |page| {
        tracing::info!("Navigating to: {}", page);
        if let Some(window) = window_weak.upgrade() {
            window.set_active_page(page);
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_select_movie(move |slug| {
        tracing::info!("Selected movie slug: {}", slug);
        if let Some(window) = window_weak.upgrade() {
            window.set_active_page(SharedString::from("Detail"));
        }
    });

    main_window.on_play_movie(move |slug| {
        tracing::info!("Playing movie slug: {}", slug);
    });

    main_window.on_toggle_watchlist(move |slug| {
        tracing::info!("Toggled watchlist for: {}", slug);
    });

    let window_weak = main_window.as_weak();
    main_window.on_change_provider(move |provider| {
        tracing::info!("Switched API provider to: {}", provider);
        if let Some(window) = window_weak.upgrade() {
            window.set_current_provider(provider);
        }
    });

    main_window.on_search_query(move |query| {
        tracing::info!("Search query: {}", query);
    });

    tracing::info!("Nolelamphim Desktop UI ready. Launching window...");
    main_window.run()?;
    Ok(())
}
