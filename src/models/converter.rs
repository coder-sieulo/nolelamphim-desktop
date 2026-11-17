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

pub fn load_slint_image_from_path(path: &std::path::Path) -> Option<slint::Image> {
    if !path.exists() {
        return None;
    }

    let is_webp = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("webp"))
        .unwrap_or(false);

    // If WebP, decode directly with `image` crate to prevent Slint format errors
    if is_webp {
        if let Ok(dyn_img) = image::open(path) {
            let rgba = dyn_img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
            let raw_bytes = rgba.as_raw();
            let slice = buffer.make_mut_slice();
            for (i, pixel) in slice.iter_mut().enumerate() {
                let idx = i * 4;
                pixel.r = raw_bytes[idx];
                pixel.g = raw_bytes[idx + 1];
                pixel.b = raw_bytes[idx + 2];
                pixel.a = raw_bytes[idx + 3];
            }
            return Some(slint::Image::from_rgba8(buffer));
        }
    }

    // Default Slint loader for PNG / JPEG
    match slint::Image::load_from_path(path) {
        Ok(img) => Some(img),
        Err(_) => {
            // Fallback: try decoding with `image` crate for any other format
            if let Ok(dyn_img) = image::open(path) {
                let rgba = dyn_img.to_rgba8();
                let (w, h) = (rgba.width(), rgba.height());
                let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
                let raw_bytes = rgba.as_raw();
                let slice = buffer.make_mut_slice();
                for (i, pixel) in slice.iter_mut().enumerate() {
                    let idx = i * 4;
                    pixel.r = raw_bytes[idx];
                    pixel.g = raw_bytes[idx + 1];
                    pixel.b = raw_bytes[idx + 2];
                    pixel.a = raw_bytes[idx + 3];
                }
                Some(slint::Image::from_rgba8(buffer))
            } else {
                None
            }
        }
    }
}

pub fn to_slint_movie_item_with_poster(u: &UnifiedMovieSummary, poster_path: Option<&std::path::Path>) -> MovieItem {
    let mut item = to_slint_movie_item(u);
    if let Some(path) = poster_path {
        if let Some(img) = load_slint_image_from_path(path) {
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
