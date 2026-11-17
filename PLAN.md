# Implementation Plan — Nolelamphim-desktop (Comprehensive Multi-API Integration)

A high-performance, cross-platform native desktop movie discovery application built with **Rust** and **Slint**, styled with a **Cinematic Dark Liquid Glass** design inspired by Cinefy.

---

## 1. Executive Summary & Architecture Principles

### 1.1 Core Principles
- **Toàn diện API (All-in-One API Integration)**: Không chỉ dùng một API đơn lẻ, ứng dụng tích hợp **toàn bộ các API và endpoint** được cung cấp trong tài liệu (`API-phimV3.md`, `API phim.md`, `API-phimv2.md`), bao gồm cả 3 nguồn:
  1. **KKPhim / PhimAPI** (`https://phimapi.com`)
  2. **Nguồn C** (`https://phim.nguonc.com/api`)
  3. **Vsmov** (`https://vsmov.com/api`)
- **Multi-Source Engine & Fallback**: Thiết kế kiến trúc `MovieProvider` trait trong Rust, cho phép:
  - Chọn nguồn dữ liệu trực tiếp từ giao diện (Settings hoặc Header switcher).
  - Tự động fallback (dự phòng) khi một server gặp sự cố hoặc thiếu tập/hình ảnh.
  - Tìm kiếm hợp nhất (Unified Search) đa nguồn.
- **Tận dụng 100% các endpoint chuyên sâu**:
  - Dữ liệu phim trang chủ & phim mới.
  - Phân loại theo loại phim (Phim lẻ, Phim bộ, Hoạt hình, TV Shows, Chiếu rạp).
  - Lọc theo thể loại, quốc gia, năm phát hành, ngôn ngữ (Vietsub, Thuyết minh, Lồng tiếng).
  - Chi tiết diễn viên & ê-kíp làm phim (`peoples` / `dien-vien`).
  - Bộ sưu tập hình ảnh, poster, backdrop (`images`).
  - Từ khóa & tag phim (`keywords`).
  - Tra cứu mã mở rộng TMDB & IMDb (`tmdb`, `imdb`).
  - Danh sách server phát và tập phim (`episodes`).

---

## 2. Full API Mapping & Endpoint Matrix

### 2.1 Ma trận Endpoint chi tiết

| Nhóm tính năng | PhimAPI (KKPhim) | Nguồn C | Vsmov | Mục đích sử dụng trên Desktop UI |
|---|---|---|---|---|
| **Trang chủ & Mới nhất** | `/v1/api/home`<br>`/danh-sach/phim-moi-cap-nhat` | `/films/phim-moi-cap-nhat` | `/api/danh-sach/phim-moi-cap-nhat` | Hero banner, Trending Now row, New Updates row |
| **Phân loại Phim** | `/v1/api/danh-sach/{type}`<br>(`phim-le`, `phim-bo`, `hoat-hinh`, `tv-shows`, `phim-chieu-rap`) | `/films/danh-sach/{slug}`<br>(`phim-bo`, `phim-le`, `tv-shows`) | `/api/danh-sach/subteam` | Tab điều hướng danh mục phim ở Sidebar & Discover |
| **Tìm kiếm phim** | `/v1/api/tim-kiem?keyword={kw}` | `/films/search?keyword={kw}` | `/api/tim-kiem?keyword={kw}` | Search bar (Ctrl+K), tìm kiếm đa nguồn với gợi ý tức thì |
| **Bộ lọc Thể loại** | `/the-loai`<br>`/v1/api/the-loai/{slug}` | `/films/the-loai/{slug}` | `/api/the-loai`<br>`/api/the-loai/{slug}` | Genre tabs trên Discover, Genre Cloud ở Right Panel |
| **Bộ lọc Quốc gia** | `/quoc-gia`<br>`/v1/api/quoc-gia/{slug}` | `/films/quoc-gia/{slug}` | `/api/quoc-gia`<br>`/api/quoc-gia/{slug}` | Filter bar chọn quốc gia (Hàn Quốc, Âu Mỹ, Trung Quốc...) |
| **Bộ lọc Năm** | `/nam-phat-hanh`<br>`/v1/api/nam/{year}` | `/films/nam-phat-hanh/{year}` | `/api/nam`<br>`/api/nam/{year}` | Filter dropdown duyệt phim theo năm |
| **Bộ lọc Ngôn ngữ** | Tham số `lang` trong bộ lọc | `/films/ngon-ngu/{slug}` (`vietsub`, `thuyet-minh`, `long-tieng`) | Tham số query | Bộ lọc phiên bản lồng tiếng / vietsub |
| **Chi tiết Phim & Tập** | `/v1/api/phim/{slug}`<br>`/phim/{slug}`<br>`/phim/id/{id}` | `/film/{slug}` | `/api/phim/{slug}` | Trang Movie Detail: tóm tắt, server phát, danh sách tập |
| **Diễn viên & Ê-kíp** | `/v1/api/phim/{slug}/peoples` | Thông tin trong `movie.casts` | `/api/dien-vien` | Section "Cast & Crew" với danh sách diễn viên & đạo diễn |
| **Bộ sưu tập Ảnh** | `/v1/api/phim/{slug}/images` | Poster / Thumb từ detail | TMDB image links | Gallery hình ảnh, poster chất lượng cao, backdrop viewer |
| **Từ khóa (Keywords)** | `/v1/api/phim/{slug}/keywords` | Tags | Tags | Tag pills để tìm kiếm các phim có chủ đề liên quan |
| **Tra cứu TMDB / IMDb** | `/tmdb/{type}/{id}`<br>`/imdb/title/{id}` | `tmdb.id`, `imdb.id` | Dữ liệu TMDB | Lấy điểm đánh giá, thông tin bổ sung chuẩn quốc tế |

---

## 3. Rust Multi-Provider Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                   Slint UI Layer                        │
└───────────────────────────┬─────────────────────────────┘
                            │ (Events & Callbacks)
┌───────────────────────────▼─────────────────────────────┐
│                 Application Service Layer               │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │                MovieAggregatorService             │  │
│  │  • Active Provider State (PhimAPI / NguonC / Vsmov)│ │
│  │  • Fallback Handler & Unified Cache               │  │
│  │  • Multi-Source Search Executor                   │  │
│  └─────────────────────────┬─────────────────────────┘  │
└────────────────────────────┼────────────────────────────┘
                             │
            ┌────────────────┴────────────────┐
            ▼                                 ▼
┌───────────────────────┐         ┌───────────────────────┐
│  trait MovieProvider  │         │   Image & Data Cache  │
└───────────┬───────────┘         └───────────────────────┘
            │
    ┌───────┼─────────────────────────┐
    ▼       ▼                         ▼
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ PhimApiEngine│         │ NguonCEngine │         │ VsmovEngine  │
│ (phimapi.com)│         │ (nguonc.com) │         │ (vsmov.com)  │
└──────────────┘         └──────────────┘         └──────────────┘
```

### 3.1 Định nghĩa Trait `MovieProvider`
```rust
#[async_trait]
pub trait MovieProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    
    // Trang chủ & Danh mục
    async fn fetch_home_feed(&self) -> Result<HomeFeedData, AppError>;
    async fn fetch_by_category(&self, category: &str, page: u32) -> Result<MovieListResponse, AppError>;
    
    // Chi tiết & Tập phim
    async fn fetch_movie_detail(&self, slug: &str) -> Result<MovieDetailData, AppError>;
    async fn fetch_peoples(&self, slug: &str) -> Result<Vec<PersonItem>, AppError>;
    async fn fetch_images(&self, slug: &str) -> Result<MovieGallery, AppError>;
    async fn fetch_keywords(&self, slug: &str) -> Result<Vec<String>, AppError>;
    
    // Tìm kiếm & Bộ lọc
    async fn search_movies(&self, query: &str, page: u32) -> Result<MovieListResponse, AppError>;
    async fn fetch_genres(&self) -> Result<Vec<GenreItem>, AppError>;
    async fn fetch_movies_by_genre(&self, slug: &str, page: u32) -> Result<MovieListResponse, AppError>;
    async fn fetch_countries(&self) -> Result<Vec<CountryItem>, AppError>;
    async fn fetch_movies_by_country(&self, slug: &str, page: u32) -> Result<MovieListResponse, AppError>;
    async fn fetch_years(&self) -> Result<Vec<u32>, AppError>;
    async fn fetch_movies_by_year(&self, year: u32, page: u32) -> Result<MovieListResponse, AppError>;
    
    // TMDB / IMDb tra cứu ngoài
    async fn lookup_tmdb(&self, media_type: &str, tmdb_id: &str) -> Result<TmdbMeta, AppError>;
    async fn lookup_imdb(&self, imdb_id: &str) -> Result<ImdbMeta, AppError>;
}
```

---

## 4. Slint UI / UX Alignment (Liquid Glass & Cinefy)

### 4.1 Bố cục 3 Cột (Responsive)
- **Cột Trái (Sidebar - 224px)**:
  - Brand Logo + Cinematic Tagline.
  - Nút chuyển nguồn API đang hoạt động (PhimAPI / Nguồn C / Vsmov).
  - Navigation chính: **Home**, **Discover**, **Library**.
  - Playlists cá nhân: Chill Movies, Action Bangers, Feel Good Vibes, Anime Corner.
  - Pro Badge / Upgrade Card.
  - Utilities: Settings, Notifications, Theme Toggle.
- **Cột Giữa (Main Content - Fluid)**:
  - **Topbar**: Search Bar tích hợp phím tắt `Ctrl + K`, Avatar người dùng, Chuông thông báo, Bộ lọc nhanh nguồn dữ liệu.
  - **Trang Home**:
    - Hero Section (phim nổi bật lớn, tag `TRENDING #1`, Watch Now, Add to Watchlist).
    - Section "Trending Now" (dãy 6 phim, badge rank số 1..6, điểm rating).
    - Section "Continue Watching" (dãy 3 phim kèm thanh tiến độ %).
    - Section "Made for your mood" (dãy 6 phim theo thể loại gợi ý).
    - Section "Phim Chiếu Rạp Mới" & "Hoạt Hình Nổi Bật".
  - **Trang Discover**:
    - Bộ lọc đa tầng: Thể loại (Genre Tabs), Quốc gia (Country Filter), Năm phát hành, Phân loại (Phim lẻ / Phim bộ / Chiếu rạp).
    - Lưới phim 5 cột chuẩn tỷ lệ 2:3 với lazy loading & phân trang mượt mà.
  - **Trang Movie Detail (Dedicated View / Modal)**:
    - Backdrop cỡ lớn với hiệu ứng làm mờ Cinematic.
    - Poster, Tiêu đề gốc, Năm, Thời lượng, Quốc gia, Điểm TMDB/IMDb.
    - Danh sách diễn viên (`peoples`) kèm ảnh đại diện & vai diễn.
    - Bộ sưu tập hình ảnh (`images`) có thể phóng to.
    - Từ khóa phim (`keywords`) tương tác được.
    - Danh sách máy chủ (Server 1, Server 2) & tập phim để chọn xem.
  - **Trang Library**:
    - Danh sách phim yêu thích đã lưu (Watchlist).
    - Lịch sử xem phim (Continue Watching).
- **Cột Phải (Right Panel - 300px)**:
  - **Picked for you**: 4 phim gợi ý cá nhân hóa kèm nút thêm nhanh vào danh sách.
  - **Your top genres**: Cloud thể loại tương tác (Action, Sci-Fi, Drama...).
  - **Now Playing Mini-Player**: Widget phim đang chọn, thanh thời gian, nút Play/Pause, tua tập trước/sau.

---

## 5. Phased Roadmap

### Giai đoạn 1: Khởi tạo Cargo & Cấu hình Slint
- `Cargo.toml`: Thêm `slint`, `slint-build`, `tokio`, `reqwest`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `image`, `directories`, `async-trait`.
- `build.rs`: Biên dịch toàn bộ cây giao diện Slint.
- Cấu trúc thư mục `ui/` và `src/`.

### Giai đoạn 2: Slint Design System & Toàn bộ Components
- `ui/themes/theme.slint`: Bảng màu Cinematic Dark & Liquid Glass tokens.
- `ui/components/`:
  - `sidebar.slint`, `topbar.slint`, `hero-banner.slint`, `movie-card.slint`, `continue-card.slint`.
  - `right-panel.slint`, `peoples-list.slint`, `gallery-view.slint`, `episode-selector.slint`.
  - `loading.slint`, `empty-view.slint`.

### Giai đoạn 3: Slint Pages Assembly
- Xây dựng 4 màn hình chính: `home.slint`, `discover.slint`, `movie-detail.slint`, `library.slint`.
- Gắn kết logic chuyển trang và trạng thái hiển thị trong `app-window.slint`.

### Giai đoạn 4: Triển khai Engine Đa Nguồn API (Rust)
- Định nghĩa đầy đủ models Serde cho cả 3 nguồn (`phimapi`, `nguonc`, `vsmov`).
- Cài đặt trait `MovieProvider` cho:
  - `PhimApiEngine` (hỗ trợ toàn bộ 17+ endpoint KKPhim).
  - `NguonCEngine` (hỗ trợ 8 endpoint Nguồn C).
  - `VsmovEngine` (hỗ trợ 8 endpoint Vsmov).
- Cài đặt `MovieAggregatorService` quản lý nguồn đang chọn và tự động fallback.

### Giai đoạn 5: Async State Bridge & Tích hợp vào Slint UI
- Cầu nối dữ liệu Slint ↔ Tokio Async Tasks.
- Kết nối dữ liệu thời gian thực:
  - Nạp Home feed (Hero, Trending, Phim mới).
  - Nạp Discover với đầy đủ bộ lọc Thể loại, Quốc gia, Năm.
  - Nạp Movie Detail với diễn viên (`peoples`), bộ sưu tập ảnh (`images`), từ khóa (`keywords`), và danh sách tập phim.
  - Nạp gợi ý ở Right Panel và Mini-Player.

### Giai đoạn 6: Image Cache & Watchlist Cục bộ
- Cache poster & backdrop vào ổ đĩa (`~/.cache/nolelamphim-desktop/images`).
- Lưu trữ Watchlist và lịch sử xem phim vào `watchlist.json`.

### Giai đoạn 7: Kiểm thử, Tối ưu & Đóng gói Cross-Platform
- Kiểm thử phản hồi nhanh khi mất mạng / server API quá tải.
- Tối ưu bộ nhớ khi tải hàng trăm poster.
- Kiểm tra build release trên Windows, Linux và macOS.
