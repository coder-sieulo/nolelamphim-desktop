use slint::SharedString;
use crate::{MovieItem, PersonItem, EpisodeItem, GenreFilterItem};
use super::movie::{UnifiedCategoryItem, UnifiedEpisode, UnifiedMovieSummary, UnifiedPerson};

pub fn to_slint_movie_item(u: &UnifiedMovieSummary) -> MovieItem {
    MovieItem {
        id: SharedString::from(&u.id),
        slug: SharedString::from(&u.slug),
        title: SharedString::from(&u.title),
        origin_title: SharedString::from(&u.origin_title),
        poster_url: SharedString::from(&u.poster_url),
        backdrop_url: SharedString::from(&u.backdrop_url),
        poster: slint::Image::default(),
        has_poster: false,
        genre: SharedString::from(&u.genre),
        year: SharedString::from(&u.year),
        rating: SharedString::from(&u.rating),
        rank: u.rank,
        quality: SharedString::from(&u.quality),
        progress: u.progress,
        is_saved: u.is_saved,
    }
}

pub fn to_slint_movie_item_with_poster(u: &UnifiedMovieSummary, poster_path: Option<&std::path::Path>) -> MovieItem {
    let mut item = to_slint_movie_item(u);
    if let Some(path) = poster_path {
        if let Ok(img) = slint::Image::load_from_path(path) {
            item.poster = img;
            item.has_poster = true;
        }
    }
    item
}

pub fn to_slint_person_item(p: &UnifiedPerson) -> PersonItem {
    PersonItem {
        name: SharedString::from(&p.name),
        role: SharedString::from(&p.role),
        avatar_url: SharedString::from(&p.avatar_url),
    }
}

pub fn to_slint_episode_item(e: &UnifiedEpisode) -> EpisodeItem {
    EpisodeItem {
        server_name: SharedString::from(&e.server_name),
        name: SharedString::from(&e.name),
        slug: SharedString::from(&e.slug),
        link_embed: SharedString::from(&e.link_embed),
        link_m3u8: SharedString::from(&e.link_m3u8),
    }
}

pub fn to_slint_genre_item(g: &UnifiedCategoryItem, active: bool) -> GenreFilterItem {
    GenreFilterItem {
        name: SharedString::from(&g.name),
        slug: SharedString::from(&g.slug),
        active,
    }
}
