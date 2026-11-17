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
