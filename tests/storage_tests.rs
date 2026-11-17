use nolelamphim_desktop::models::UnifiedMovieSummary;
use nolelamphim_desktop::services::StorageService;

#[tokio::test]
async fn test_storage_watchlist_toggle() {
    let storage = StorageService::new();
    let sample = UnifiedMovieSummary {
        id: "unit-test-1".to_string(),
        slug: "unit-test-slug".to_string(),
        title: "Unit Test Movie".to_string(),
        origin_title: "Unit Test Movie Origin".to_string(),
        poster_url: "".to_string(),
        backdrop_url: "".to_string(),
        genre: "Action".to_string(),
        year: "2024".to_string(),
        rating: "9.0".to_string(),
        rank: 1,
        quality: "HD".to_string(),
        time: "100m".to_string(),
        category: "Movie".to_string(),
        country: "VN".to_string(),
        is_saved: false,
        progress: 0,
    };

    // Toggle once -> should be added
    let (saved, list) = storage.toggle_watchlist(sample.clone()).await;
    assert!(saved);
    assert!(list.iter().any(|m| m.slug == "unit-test-slug"));
    assert!(storage.is_saved("unit-test-slug").await);

    // Toggle again -> should be removed
    let (saved_again, list_after) = storage.toggle_watchlist(sample.clone()).await;
    assert!(!saved_again);
    assert!(!list_after.iter().any(|m| m.slug == "unit-test-slug"));
    assert!(!storage.is_saved("unit-test-slug").await);
}

#[tokio::test]
async fn test_storage_history() {
    let storage = StorageService::new();
    let mut m1 = UnifiedMovieSummary::default();
    m1.slug = "movie-1".to_string();
    m1.title = "Movie 1".to_string();

    let mut m2 = UnifiedMovieSummary::default();
    m2.slug = "movie-2".to_string();
    m2.title = "Movie 2".to_string();

    let hist1 = storage.add_to_history(m1.clone(), 30).await;
    assert_eq!(hist1.first().unwrap().slug, "movie-1");
    assert_eq!(hist1.first().unwrap().progress, 30);

    let hist2 = storage.add_to_history(m2.clone(), 65).await;
    assert_eq!(hist2.first().unwrap().slug, "movie-2");
    assert_eq!(hist2.first().unwrap().progress, 65);
}
