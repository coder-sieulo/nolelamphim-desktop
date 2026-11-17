# Nolelamphim-desktop

**A cross-platform native desktop movie discovery application built with Rust and Slint.**

Nolelamphim-desktop là ứng dụng desktop giúp người dùng khám phá, tìm kiếm và quản lý phim với giao diện hiện đại, mang phong cách cinematic dark UI kết hợp Liquid Glass-inspired design.

Dự án hướng đến ba nền tảng Windows, Linux và macOS, sử dụng một codebase Rust thống nhất.

---

## 1. Project Overview

### Mục tiêu

- Xây dựng ứng dụng desktop native đa nền tảng.
- Giao diện hiện đại, mượt mà và tối ưu trải nghiệm người dùng.
- Khám phá phim thông qua poster, thông tin và nội dung giới thiệu.
- Tìm kiếm phim và xem thông tin chi tiết.
- Kết nối với Web API hiện có.
- Có khả năng mở rộng thêm các tính năng trong tương lai.

### Phạm vi dự án

**Trong phạm vi:**
- Desktop UI/UX.
- Điều hướng giữa các trang.
- Hiển thị danh sách phim.
- Tìm kiếm và lọc phim.
- Movie detail.
- Kết nối API hiện có.
- Loading, error và empty states.
- Build ứng dụng cho Windows, Linux và macOS.

**Ngoài phạm vi:**
- Không xây dựng backend mới.
- Không xây dựng lại hệ thống API.
- Không thay đổi database hiện có.
- Không triển khai hệ thống quản lý phim phía server.
- Không tự ý tạo endpoint API mới.

> Backend và Web API đã được triển khai sẵn. Dự án này chỉ chịu trách nhiệm phát triển ứng dụng desktop và tích hợp với API hiện có.

---

## 2. Technology Stack

| Thành phần | Công nghệ | Mục đích |
|---|---|---|
| Programming Language | Rust | Core application |
| UI Framework | Slint | Native desktop UI |
| Build System | Cargo | Dependency và build |
| HTTP Client | Reqwest | Gọi Web API |
| Async Runtime | Tokio | Xử lý asynchronous |
| Serialization | Serde | JSON serialization |
| Logging | Tracing | Logging và debugging |
| Local Storage | SQLite (optional) | Cache và local preferences |
| Version Control | Git | Quản lý source code |

### Vì sao chọn Rust + Slint?

- Rust có hiệu năng cao, quản lý bộ nhớ an toàn.
- Có thể sử dụng một codebase cho nhiều hệ điều hành.
- Slint hỗ trợ xây dựng giao diện desktop bằng declarative UI.
- Không cần đóng gói ứng dụng dưới dạng website chạy trong WebView.
- Phù hợp với ứng dụng desktop cần hiệu năng và khả năng mở rộng.

**Lưu ý:** Slint tạo giao diện desktop đa nền tảng với hệ thống render riêng; không có nghĩa mọi widget đều tự động sử dụng native control mặc định của từng hệ điều hành.

---

## 3. Target Platforms

| Platform | Architecture | Target |
|---|---|---|
| Windows | x86_64 | `x86_64-pc-windows-msvc` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |

Development platform ban đầu: **Linux x86_64** (`x86_64-unknown-linux-gnu`).

---

## 4. Architecture

### Application Architecture

```text
┌───────────────────────────────────────────────┐
│             Nolelamphim-desktop               │
│                                               │
│  ┌─────────────────────────────────────────┐  │
│  │               Slint UI                  │  │
│  │                                         │  │
│  │  Home | Discover | Search | Detail      │  │
│  │                                         │  │
│  └───────────────────┬─────────────────────┘  │
│                      │                        │
│  ┌───────────────────▼─────────────────────┐  │
│  │            Application State            │  │
│  │                                         │  │
│  │  Navigation | Events | UI State         │  │
│  └───────────────────┬─────────────────────┘  │
│                      │                        │
│  ┌───────────────────▼─────────────────────┐  │
│  │               API Client                │  │
│  │                                         │  │
│  │  HTTP | Request | Response | Error      │  │
│  └───────────────────┬─────────────────────┘  │
│                      │                        │
└──────────────────────┼────────────────────────┘
                       │
                       ▼
             Existing Web API Server
                       │
                       ▼
               Existing Backend
```

### Architecture Principles

1. UI không gọi HTTP trực tiếp.
2. Mọi API request được xử lý thông qua API Client.
3. Không hardcode endpoint khi chưa có tài liệu API.
4. Không block UI thread khi thực hiện network request.
5. Dữ liệu API cần được deserialize thành Rust models.
6. UI chỉ nhận dữ liệu đã được xử lý.
7. Tách biệt UI components, application state và services.

---

## 5. Project Structure

```text
Nolelamphim-desktop/
│
├── Cargo.toml
├── Cargo.lock
├── build.rs
├── README.md
├── .gitignore
├── .env.example
│
├── ui/
│   ├── app-window.slint
│   │
│   ├── components/
│   │   ├── sidebar.slint
│   │   ├── header.slint
│   │   ├── movie-card.slint
│   │   ├── movie-carousel.slint
│   │   ├── search-bar.slint
│   │   └── loading.slint
│   │
│   ├── pages/
│   │   ├── home.slint
│   │   ├── discover.slint
│   │   ├── search.slint
│   │   ├── movie-detail.slint
│   │   └── library.slint
│   │
│   └── themes/
│       └── theme.slint
│
├── src/
│   ├── main.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── navigation.rs
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── endpoints.rs
│   │
│   ├── services/
│   │   ├── mod.rs
│   │   └── movie_service.rs
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   └── movie.rs
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs
│   │
│   └── platform/
│       ├── mod.rs
│       └── system.rs
│
├── assets/
│   ├── icons/
│   ├── images/
│   └── fonts/
│
└── tests/
    └── api_tests.rs
```

Cấu trúc trên là định hướng ban đầu. Có thể điều chỉnh trong quá trình phát triển, nhưng cần giữ nguyên nguyên tắc phân tách trách nhiệm.

---

## 6. UI/UX Design System

### Design Direction

Phong cách thiết kế:

- Cinematic Dark UI.
- Liquid Glass-inspired surfaces.
- Modern desktop experience.
- Smooth transitions.
- Large movie posters.
- Minimalist navigation.
- Orange accent color.

### Color Palette

| Token | Color | Usage |
|---|---|---|
| Background | `#0B0B0D` | Main background |
| Surface | `#151518` | Cards và panels |
| Surface Secondary | `#1D1D21` | Secondary panels |
| Glass | `rgba(255,255,255,0.06)` | Glass surfaces |
| Primary | `#FF8A24` | Primary accent |
| Primary Hover | `#FFA34D` | Hover state |
| Text Primary | `#FFFFFF` | Main text |
| Text Secondary | `#A1A1AA` | Secondary text |
| Border | `rgba(255,255,255,0.10)` | Borders |

### UI Principles

- Ưu tiên bố cục rõ ràng, dễ sử dụng.
- Không lạm dụng hiệu ứng blur.
- Sử dụng animation có chủ đích.
- Poster phim phải giữ đúng tỷ lệ.
- Nội dung cần có khoảng trắng hợp lý.
- Các trạng thái loading/error phải được thiết kế đầy đủ.
- Tối ưu giao diện cho desktop, không chỉ phóng to giao diện web.

### Main Pages

| Page | Description |
|---|---|
| Home | Phim nổi bật và nội dung đề xuất |
| Discover | Khám phá phim theo danh mục |
| Search | Tìm kiếm phim |
| Movie Detail | Thông tin chi tiết phim |
| Library | Danh sách phim cá nhân nếu API hỗ trợ |

---

## 7. API Integration

### Existing API

Ứng dụng sẽ kết nối đến server API đã có.

Base URL được cấu hình thông qua environment/configuration.

Ví dụ `.env.example`:

```env
API_BASE_URL=http://localhost:3000
REQUEST_TIMEOUT=30
```

Đây chỉ là giá trị mẫu. Hãy thay bằng địa chỉ server thực tế.

### API Integration Rules

1. Sử dụng đúng endpoint từ tài liệu API hiện có.
2. Sử dụng đúng HTTP method.
3. Không tự ý tạo endpoint.
4. Không giả định cấu trúc JSON response.
5. Deserialize response bằng Serde.
6. Xử lý timeout và network errors.
7. Không lưu API secret hoặc credential trong source code.
8. Không gửi request đồng bộ trên UI thread.

### API Client Responsibilities

API Client chịu trách nhiệm:

- Khởi tạo HTTP client.
- Quản lý Base URL.
- Gửi HTTP request.
- Xử lý response.
- Deserialize JSON.
- Chuẩn hóa lỗi API.
- Quản lý request timeout.

### Data Flow

```text
User Interaction
       │
       ▼
Slint UI Event
       │
       ▼
Application State
       │
       ▼
Movie Service
       │
       ▼
API Client
       │
       ▼
Existing API Server
       │
       ▼
JSON Response
       │
       ▼
Rust Models
       │
       ▼
Application State
       │
       ▼
Update UI
```

**Important:** Chỉ triển khai API integration sau khi có endpoint và response schema thực tế.

---

## 8. Application State

Các trạng thái cơ bản:

| State | Description |
|---|---|
| Idle | Chưa có request |
| Loading | Đang tải dữ liệu |
| Success | Tải dữ liệu thành công |
| Error | Request thất bại |
| Empty | Không có dữ liệu |
| Offline | Không kết nối được server |

UI cần xử lý đầy đủ các trạng thái trên, tránh để ứng dụng bị trống hoặc treo khi API gặp lỗi.

---

## 9. Development Environment

### Required Software

| Software | Purpose |
|---|---|
| Rustup | Rust toolchain |
| Cargo | Build system |
| GCC / Clang / build-essential | Linux C/C++ build tools |
| System GUI libraries (fontconfig, xkbcommon, etc.) | Slint Linux rendering dependencies |
| VS Code / RustRover | IDE |
| Git | Version control |

### Linux Setup (Ubuntu / Debian)

**Step 1: Install system build dependencies for Slint**

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libfontconfig1-dev libxkbcommon-dev libxkbcommon-x11-dev
```

**Step 2: Install Rust & target**

Tải Rust tại:

https://rustup.rs/

Cài đặt target Linux:

```bash
rustup target add x86_64-unknown-linux-gnu
```

**Step 3: Verify environment**

```bash
rustup show
cargo --version
rustc --version
```

---

## 10. Getting Started

### Clone repository

```bash
git clone <YOUR_GIT_REPOSITORY>
cd Nolelamphim-desktop
```

### Kiểm tra cú pháp (Check)

```bash
cargo check
```

### Lệnh chạy ứng dụng (Run)

- **Chạy trực tiếp chế độ phát triển (Development mode)**:
  ```bash
  cargo run
  ```

- **Chạy chế độ tối ưu (Release mode)**:
  ```bash
  cargo run --release
  ```

- **Chạy với log chi tiết (Debug logging)**:
  ```bash
  RUST_LOG=debug cargo run
  ```

### Lệnh đóng gói và biên dịch (Build)

- **Biên dịch bản Debug (nhanh)**:
  ```bash
  cargo build
  ```
  File thực thi sinh ra tại:
  ```bash
  ./target/debug/nolelamphim-desktop
  ```

- **Biên dịch bản Release (tối ưu hóa hiệu năng cao nhất)**:
  ```bash
  cargo build --release
  ```
  File thực thi sinh ra tại:
  ```bash
  ./target/release/nolelamphim-desktop
  ```

- **Chạy trực tiếp file nhị phân đã build**:
  ```bash
  ./target/release/nolelamphim-desktop
  ```

---

## 11. Cross-platform Build

Sau khi hoàn thiện môi trường build cho từng nền tảng:

### Linux

```bash
rustup target add x86_64-unknown-linux-gnu

cargo build --release \
  --target x86_64-unknown-linux-gnu
```

### macOS Intel

```bash
rustup target add x86_64-apple-darwin

cargo build --release \
  --target x86_64-apple-darwin
```

### macOS Apple Silicon

```bash
rustup target add aarch64-apple-darwin

cargo build --release \
  --target aarch64-apple-darwin
```

**Lưu ý:** Cross-compilation cần toolchain và thư viện hệ thống phù hợp. Đặc biệt, việc build macOS thường nên thực hiện trên máy macOS có Xcode Command Line Tools để đảm bảo quá trình build và đóng gói ổn định.

---

## 12. Development Roadmap

### Phase 1 — Foundation

- [ ] Khởi tạo Rust project.
- [ ] Cấu hình Slint.
- [ ] Thiết lập cấu trúc source code.
- [ ] Cấu hình logging.
- [ ] Chạy được cửa sổ desktop đầu tiên.

### Phase 2 — Design System

- [ ] Xây dựng theme.
- [ ] Thiết lập màu sắc.
- [ ] Thiết lập typography.
- [ ] Xây dựng reusable components.
- [ ] Thiết lập navigation layout.

### Phase 3 — Static UI

- [ ] Home page.
- [ ] Discover page.
- [ ] Search page.
- [ ] Movie Detail page.
- [ ] Library page.
- [ ] Loading/Error/Empty states.

### Phase 4 — API Integration

- [ ] Xác định API endpoints thực tế.
- [ ] Xây dựng API Client.
- [ ] Xây dựng Rust data models.
- [ ] Tích hợp danh sách phim.
- [ ] Tích hợp tìm kiếm.
- [ ] Tích hợp Movie Detail.
- [ ] Kiểm thử xử lý lỗi mạng.

### Phase 5 — Polish

- [ ] Tối ưu animation.
- [ ] Tối ưu hiệu năng.
- [ ] Kiểm thử điều hướng.
- [ ] Kiểm thử nhiều kích thước cửa sổ.
- [ ] Kiểm thử trạng thái mất kết nối.

### Phase 6 — Cross-platform

- [ ] Build Windows.
- [ ] Build Linux.
- [ ] Build macOS.
- [ ] Kiểm tra UI trên từng nền tảng.
- [ ] Đóng gói ứng dụng.
- [ ] Chuẩn bị release.

---

## 13. Git Workflow

### Branch Strategy

```text
main
│
├── develop
│
├── feature/ui-layout
├── feature/movie-card
├── feature/home-page
├── feature/search
├── feature/api-client
│
└── fix/bug-description
```

### Commit Convention

Sử dụng Conventional Commits:

```text
feat: add movie card component
feat: implement home page
fix: resolve movie poster rendering
refactor: improve api client
style: update dark theme
docs: update readme
test: add api client tests
chore: update dependencies
```

---

## 14. Performance & Security

### Performance

- Không block UI thread.
- Sử dụng asynchronous HTTP requests.
- Giới hạn số lượng poster tải đồng thời.
- Cân nhắc caching hình ảnh.
- Tránh render lại UI không cần thiết.
- Tối ưu kích thước ảnh theo mục đích hiển thị.
- Theo dõi memory usage.

### Security

- Không hardcode API credentials.
- Không commit `.env`.
- Kiểm tra dữ liệu API trước khi hiển thị.
- Thiết lập timeout cho network requests.
- Xử lý lỗi an toàn.
- Không log thông tin nhạy cảm.

---

## 15. MVP Definition

MVP được xem là hoàn thành khi:

- [ ] Ứng dụng chạy ổn định trên Linux (`x86_64-unknown-linux-gnu`).
- [ ] Có giao diện Home hoàn chỉnh.
- [ ] Có trang Discover.
- [ ] Có chức năng Search.
- [ ] Có trang Movie Detail.
- [ ] Kết nối được cả 3 hệ thống API (PhimAPI, Nguồn C, Vsmov).
- [ ] Hiển thị poster và thông tin phim.
- [ ] Có Loading, Error và Empty states.
- [ ] Không làm treo UI khi API phản hồi chậm.
- [ ] Có thể build Release.

Sau MVP, tiếp tục kiểm thử và hoàn thiện Windows/macOS.

---

## 16. Official Documentation

- Rust: https://www.rust-lang.org/
- Rust Book: https://doc.rust-lang.org/book/
- Cargo: https://doc.rust-lang.org/cargo/
- Slint: https://slint.dev/
- Slint Documentation: https://docs.slint.dev/
- Tokio: https://tokio.rs/
- Reqwest: https://docs.rs/reqwest/
- Serde: https://serde.rs/
- Tracing: https://docs.rs/tracing/

---

## 17. Project Direction

Nolelamphim-desktop được định hướng phát triển theo các nguyên tắc:

1. Một codebase Rust thống nhất.
2. Một hệ thống UI dùng chung cho ba nền tảng.
3. Native desktop application, không phải web wrapper.
4. Backend và API được tái sử dụng từ hệ thống hiện có.
5. Ưu tiên trải nghiệm UI/UX trước khi mở rộng chức năng.
6. Kiến trúc module hóa, dễ bảo trì và mở rộng.

**Nolelamphim-desktop — Discover movies. Enjoy the experience.**
