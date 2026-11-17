# Implementation Plan — Nolelamphim-desktop

A high-performance, cross-platform native desktop movie discovery application built with **Rust** and **Slint**, styled with a **Cinematic Dark Liquid Glass** design inspired by Cinefy.

---

## 1. Executive Summary & Goals

### 1.1 Objectives
- Build a native, fluid desktop application for Windows, Linux, and macOS using a single Rust codebase.
- Implement a **Cinematic Dark Liquid Glass UI** reflecting the provided HTML/CSS mockup (`mockup/index.html`).
- Integrate seamlessly with the provided public Vietnamese movie APIs (prioritizing `https://phimapi.com` / KKPhim with compatibility for `vsmov.com` and `nguonc.com`).
- Ensure UI responsiveness through asynchronous request processing (Tokio runtime) and non-blocking image loading.
- Provide a robust local watchlist / library without requiring a separate backend database.

---

## 2. API Analysis & Data Mapping

From the provided documentation (`API-phimV3.md`, `API phim.md`, `API-phimv2.md`), we identify the primary API endpoints:

| Feature | Primary Endpoint (`https://phimapi.com`) | Alternate (`vsmov.com` / `nguonc.com`) | Response Handling |
|---|---|---|---|
| **Home Featured / Trending** | `GET /v1/api/home` or `/danh-sach/phim-moi-cap-nhat?page=1` | `/api/danh-sach/phim-moi-cap-nhat` | Extract hero banners & trending 6-card row |
| **Movie List by Category** | `GET /v1/api/danh-sach/{type}?page={p}` (`phim-le`, `phim-bo`, `hoat-hinh`) | `/films/danh-sach/{slug}` | Paginated list with pagination metadata |
| **Search Movies** | `GET /v1/api/tim-kiem?keyword={kw}&limit=20` | `/api/tim-kiem?keyword={kw}` | Instant / debounced search results |
| **Genre Listing & Filter** | `GET /the-loai` & `GET /v1/api/the-loai/{slug}?page=1` | `/api/the-loai/{slug}` | Dynamic genre tab buttons & filtered grid |
| **Movie Details & Episodes** | `GET /v1/api/phim/{slug}` or `/phim/{slug}` | `/api/phim/{slug}` | Full synopsis, poster, actors, episodes & stream links |

### Key Serde Data Models:
1. `MovieSummary`: `id`, `name`, `origin_name`, `slug`, `thumb_url`, `poster_url`, `year`, `quality`, `lang`, `time`, `category`, `country`.
2. `MovieDetail`: Extends summary with `content` (synopsis), `trailer_url`, `director`, `actor`, `episodes` (servers + episode stream URLs).
3. `ApiResponse<T>`: Wrapper for `status`, `message`, `data` (with `items`, `params.pagination`).
4. `Pagination`: `totalItems`, `totalItemsPerPage`, `currentPage`, `totalPages`.

---

## 3. UI/UX Architecture (Slint + Liquid Glass)

Based on `mockup/index.html`, the desktop layout uses a **3-column responsive desktop structure**:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                Nolelamphim Desktop Window                              │
├──────────────┬──────────────────────────────────────────────────────────┬──────────────┤
│   SIDEBAR    │                      MAIN CONTENT AREA                   │ RIGHT PANEL  │
│   (224px)    ├──────────────────────────────────────────────────────────┤   (300px)    │
│              │ TOPBAR: [Search Movie... Ctrl+K]      [Bell] [User Profile] │              │
│ • Logo       ├──────────────────────────────────────────────────────────┤ • Picked     │
│ • Navigation │ HERO SECTION:                                            │   For You    │
│   - Home     │   [Large Movie Backdrop]                                 │              │
│   - Discover │   "TRENDING #1" | Title | Watch Now Button | Watchlist   │ • Top Genres │
│   - Library  ├──────────────────────────────────────────────────────────┤   Cloud      │
│ • Playlists  │ SECTION 1: Trending Now (6-col movie poster grid)        │              │
│ • Pro Badge  ├──────────────────────────────────────────────────────────┤ • Now Playing│
│ • Settings   │ SECTION 2: Continue Watching (3-col cards with progress) │   Mini Player│
│              ├──────────────────────────────────────────────────────────┤   (Track &   │
│              │ SECTION 3: Made for Your Mood (6-col grid)               │    Controls) │
└──────────────┴──────────────────────────────────────────────────────────┴──────────────┘
```

### Slint Component Tree:
```text
ui/
├── app-window.slint             <-- Root window & page switcher
├── themes/
│   └── theme.slint              <-- Color tokens, fonts, spacing, glass styles
├── components/
│   ├── sidebar.slint            <-- Left navigation, brand, playlist items
│   ├── topbar.slint             <-- Search input (Ctrl+K), avatar, notifications
│   ├── hero-banner.slint        <-- Featured movie backdrop with call-to-actions
│   ├── movie-card.slint         <-- Card with poster, rank badge, hover play/heart
│   ├── continue-card.slint      <-- Cards with progress bar
│   ├── right-panel.slint        <-- Recommendations, genre cloud, mini-player
│   ├── search-modal.slint       <-- Quick search overlay / dropdown
│   └── loading.slint            <-- Skeleton loaders & spinners
└── pages/
    ├── home.slint               <-- Hero + Trending + Continue + Mood rows
    ├── discover.slint           <-- Genre filter tabs + 5-column paginated grid
    ├── movie-detail.slint       <-- Backdrop, synopsis, genres, episode picker
    └── library.slint            <-- Watchlist & favorites with empty state
```

### Design Tokens:
- **Background**: `#090A0D` with radial orange/violet glow
- **Panel Surface (Glass)**: `rgba(255, 255, 255, 0.055)`
- **Border**: `rgba(255, 255, 255, 0.105)`
- **Primary Accent**: `#FF781F` (Orange), Secondary `#FFAD65`
- **Text**: `#F7F6F3` (Primary), `#9899A2` (Muted)
- **Typography**: DM Sans / Space Grotesk styling

---

## 4. Technical Architecture & Rust Implementation

### Key Technical Considerations:
1. **Thread Separation**: UI thread remains strictly 60+ FPS responsive; all network requests and image downloads are spawned on the Tokio thread pool.
2. **Image Loading in Slint**:
   - Slint accepts `slint::Image`.
   - We implement an asynchronous image loader that downloads poster bytes in background, decodes via the `image` crate, and updates the Slint `Model` on completion.
3. **Data Caching**:
   - Cache API responses in memory (LRU or TTL) to prevent repeated network hits when switching between Home and Discover tabs.
   - Cache posters in a local cache directory (`~/.cache/nolelamphim-desktop/images/`).
4. **Local Watchlist**:
   - Store saved movies and playback progress in `watchlist.json` under the user configuration directory (`directories` crate).

---

## 5. Phased Roadmap

### Phase 1: Project Setup & Dependencies
- Initialize `Cargo.toml` with `slint`, `slint-build`, `tokio`, `reqwest`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `image`, `directories`.
- Create `build.rs` to compile `ui/app-window.slint`.
- Configure `.env.example` and `.gitignore`.
- Verify minimal Slint window opens.

### Phase 2: Design System & Shared Components (Slint)
- Implement `ui/themes/theme.slint` with color palette, dimensions, and typography.
- Implement `ui/components/sidebar.slint` (navigation items, playlist list, upgrade card).
- Implement `ui/components/topbar.slint` (search input, keyboard shortcut badge, avatar).
- Implement `ui/components/movie-card.slint` (aspect ratio 2:3, hover overlay, rank badge, rating).
- Implement `ui/components/continue-card.slint` (backdrop, progress bar).
- Implement `ui/components/right-panel.slint` (picked for you, genre cloud, mini player).

### Phase 3: Page Views (Slint Mockup Alignment)
- Implement `ui/pages/home.slint`: Hero section, Trending Now row, Continue Watching row, Mood row.
- Implement `ui/pages/discover.slint`: Genre pills, search query binding, 5-column grid.
- Implement `ui/pages/movie-detail.slint`: Backdrop header, movie metadata, server tabs, episode grid.
- Implement `ui/pages/library.slint`: Watchlist grid and empty state placeholder.
- Assemble all views into `ui/app-window.slint` with active tab switching.

### Phase 4: Data Models & API Client (Rust)
- Define Rust structs for PhimAPI / KKPhim response schemas in `src/models/`.
- Build HTTP client in `src/api/client.rs` with timeout, headers, and error handling.
- Build query builders in `src/api/endpoints.rs`.
- Write unit tests for API deserialization.

### Phase 5: State Bridge & UI Integration
- Bridge Slint callbacks: `navigate_to(page)`, `search_input(text)`, `select_movie(slug)`, `toggle_watchlist(id)`.
- Connect `MovieService` to fetch live data from `phimapi.com` on startup.
- Populate Home page (Hero, Trending, Recommended) from live API.
- Implement Discover search and genre filtering with debounced input.
- Implement Movie Detail modal / screen with live episode list.

### Phase 6: Image Caching & Local Watchlist
- Implement async image downloading and disk cache.
- Implement `WatchlistService` to save favorites and continue-watching timestamps to local disk.
- Reflect saved status immediately across Home, Discover, and Detail pages.

### Phase 7: Polish & Cross-Platform Packaging
- Add loading skeleton animations and network error / retry toasts.
- Verify hotkey support (`Ctrl+K` for search, `Esc` to close detail).
- Test on Linux and prepare build scripts for Windows (`.exe` via MSVC) and macOS (`.app`).
