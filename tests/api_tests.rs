use nolelamphim_desktop::models::phimapi_models::*;
use nolelamphim_desktop::models::nguonc_models::*;
use nolelamphim_desktop::models::vsmov_models::*;
use nolelamphim_desktop::models::{
    to_slint_episode_item, to_slint_movie_item, to_slint_movie_item_with_poster, to_slint_person_item,
    UnifiedEpisode, UnifiedMovieSummary, UnifiedPerson,
};

#[test]
fn test_phimapi_list_deserialization() {
    let json_data = r#"
    {
        "status": "success",
        "items": [
            {
                "_id": "665f0a1b2c3d4e5f",
                "name": "Dune: Hành Tinh Cát",
                "slug": "dune-part-two",
                "origin_name": "Dune: Part Two",
                "type": "single",
                "thumb_url": "thumb.jpg",
                "poster_url": "poster.jpg",
                "year": 2024,
                "time": "166 phút",
                "quality": "4K",
                "lang": "Vietsub"
            }
        ]
    }
    "#;

    let res: Result<PhimApiListResponse, _> = serde_json::from_str(json_data);
    assert!(res.is_ok(), "Failed to deserialize PhimApiListResponse");
    let resp = res.unwrap();
    let items = resp.items.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].slug.as_deref(), Some("dune-part-two"));
    assert_eq!(items[0].name.as_deref(), Some("Dune: Hành Tinh Cát"));
}

#[test]
fn test_nguonc_list_deserialization() {
    let json_data = r#"
    {
        "status": "success",
        "paginate": {
            "current_page": 1,
            "total_page": 5,
            "total_items": 50,
            "items_per_page": 10
        },
        "items": [
            {
                "id": "1001",
                "name": "Người Dơi",
                "slug": "the-batman",
                "original_name": "The Batman",
                "thumb_url": "batman_thumb.jpg",
                "poster_url": "batman_poster.jpg",
                "time": "176 phút",
                "quality": "HD"
            }
        ]
    }
    "#;

    let res: Result<NguonCListResponse, _> = serde_json::from_str(json_data);
    assert!(res.is_ok(), "Failed to deserialize NguonCListResponse");
    let resp = res.unwrap();
    assert_eq!(resp.status.as_deref(), Some("success"));
    let items = resp.items.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].slug.as_deref(), Some("the-batman"));
}

#[test]
fn test_vsmov_list_deserialization() {
    let json_data = r#"
    {
        "status": true,
        "items": [
            {
                "name": "Hố Đen Tử Thần",
                "slug": "interstellar",
                "origin_name": "Interstellar",
                "thumb_url": "interstellar_thumb.jpg",
                "poster_url": "interstellar_poster.jpg",
                "year": 2014,
                "time": "169 phút",
                "quality": "HD"
            }
        ]
    }
    "#;

    let res: Result<VsmovListResponse, _> = serde_json::from_str(json_data);
    assert!(res.is_ok(), "Failed to deserialize VsmovListResponse");
    let resp = res.unwrap();
    let items = resp.items.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].slug.as_deref(), Some("interstellar"));
}

#[test]
fn test_models_converter() {
    let unified = UnifiedMovieSummary {
        id: "test-1".to_string(),
        slug: "test-slug".to_string(),
        title: "Test Movie".to_string(),
        origin_title: "Test Original".to_string(),
        poster_url: "https://example.com/poster.jpg".to_string(),
        backdrop_url: "https://example.com/backdrop.jpg".to_string(),
        genre: "Sci-Fi".to_string(),
        year: "2024".to_string(),
        rating: "8.5".to_string(),
        rank: 1,
        quality: "4K".to_string(),
        time: "120m".to_string(),
        category: "Movie".to_string(),
        country: "USA".to_string(),
        is_saved: true,
        progress: 50,
    };

    let slint_item = to_slint_movie_item(&unified);
    assert_eq!(slint_item.title.as_str(), "Test Movie");
    assert_eq!(slint_item.slug.as_str(), "test-slug");
    assert_eq!(slint_item.rank, 1);
    assert_eq!(slint_item.progress, 50);
    assert!(slint_item.is_saved);
    assert!(!slint_item.has_poster);

    let item_none = to_slint_movie_item_with_poster(&unified, None);
    assert_eq!(item_none.title.as_str(), "Test Movie");
    assert!(!item_none.has_poster);

    let person = UnifiedPerson {
        name: "Christopher Nolan".to_string(),
        role: "Director".to_string(),
        avatar_url: "".to_string(),
    };
    let slint_person = to_slint_person_item(&person);
    assert_eq!(slint_person.name.as_str(), "Christopher Nolan");
    assert_eq!(slint_person.role.as_str(), "Director");

    let ep = UnifiedEpisode {
        server_name: "VIP 1".to_string(),
        name: "Episode 1".to_string(),
        slug: "ep-1".to_string(),
        link_embed: "".to_string(),
        link_m3u8: "https://example.com/stream.m3u8".to_string(),
    };
    let slint_ep = to_slint_episode_item(&ep);
    assert_eq!(slint_ep.name.as_str(), "Episode 1");
    assert_eq!(slint_ep.link_m3u8.as_str(), "https://example.com/stream.m3u8");
}

#[test]
fn test_supabase_movie_deserialization() {
    let json_data = r#"
    [
        {
            "id": 153232,
            "slug": "la-phieu-giai-cuu",
            "name": "Lá Phiếu Giải Cứu",
            "origin_name": "The 2nd",
            "year": 2020,
            "poster_url": "https://phimimg.com/poster.jpg",
            "thumb_url": "https://phimimg.com/thumb.jpg",
            "quality": "FHD",
            "episode_current": "Full",
            "categories": [{"id": 1, "name": "Hành Động", "slug": "hanh-dong"}]
        }
    ]
    "#;

    let res: Result<Vec<nolelamphim_desktop::api::supabase::SupabaseMovieRow>, _> = serde_json::from_str(json_data);
    assert!(res.is_ok(), "Failed to deserialize SupabaseMovieRow: {:?}", res.err());
    let rows = res.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].slug, "la-phieu-giai-cuu");
    assert_eq!(rows[0].name, "Lá Phiếu Giải Cứu");
    assert_eq!(rows[0].year, Some(2020));
}

#[test]
fn test_supabase_episode_deserialization() {
    let json_data = r#"
    [
        {
            "id": 902010,
            "movie_slug": "la-phieu-giai-cuu",
            "server_name": "Vietsub",
            "episode_name": "Full",
            "episode_slug": "full",
            "embed": "https://player.phimapi.com/player/?url=https://stream.m3u8",
            "m3u8": "https://stream.m3u8",
            "sort": 0
        }
    ]
    "#;

    let res: Result<Vec<nolelamphim_desktop::api::supabase::SupabaseEpisodeRow>, _> = serde_json::from_str(json_data);
    assert!(res.is_ok(), "Failed to deserialize SupabaseEpisodeRow: {:?}", res.err());
    let rows = res.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].movie_slug, "la-phieu-giai-cuu");
    assert_eq!(rows[0].episode_name, "Full");
    assert_eq!(rows[0].m3u8.as_deref(), Some("https://stream.m3u8"));
}

#[test]
fn test_decoded_image_buffer_creation() {
    let rgba_data = vec![255, 0, 0, 255, 0, 255, 0, 255]; // 2 pixels: red, green
    let buffer = nolelamphim_desktop::models::DecodedImageBuffer::new(2, 1, rgba_data);
    assert_eq!(buffer.width, 2);
    assert_eq!(buffer.height, 1);
    let slint_img = buffer.to_slint_image();
    let size = slint_img.size();
    assert_eq!(size.width, 2);
    assert_eq!(size.height, 1);
}

#[test]
fn test_cinema_duration_formatter() {
    use nolelamphim_desktop::models::format_cinema_duration;
    let formatted = format_cinema_duration(2110, 8040);
    assert_eq!(formatted, "00:35:10 / 02:14:00");

    let zero = format_cinema_duration(0, 7200);
    assert_eq!(zero, "00:00:00 / 02:00:00");
}

#[test]
fn test_duration_parser() {
    use nolelamphim_desktop::models::parse_duration_seconds;
    assert_eq!(parse_duration_seconds("166 phút"), 166 * 60);
    assert_eq!(parse_duration_seconds("45 min"), 45 * 60);
    assert_eq!(parse_duration_seconds(""), 7200);
    assert_eq!(parse_duration_seconds("unknown"), 7200);
}
#[tokio::test]
#[ignore]
async fn test_live_search() {
    use nolelamphim_desktop::api::traits::MovieProvider;
    use nolelamphim_desktop::api::phimapi::PhimApiEngine;
    let engine = PhimApiEngine::new("https://phimapi.com".to_string(), 10);
    let res = engine.search_movies("batman", 1).await;
    println!("Search result: {:?}", res);
    assert!(res.is_ok(), "Search failed: {:?}", res.err());
    let r = res.unwrap();
    println!("Found {} movies", r.items.len());
    assert!(!r.items.is_empty(), "Items should not be empty");
}

