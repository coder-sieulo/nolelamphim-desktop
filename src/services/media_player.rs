use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use crate::models::converter::format_cinema_duration;
use crate::AppWindow;
use slint::SharedString;

#[cfg(target_os = "linux")]
pub mod x11_embed {
    use std::ffi::c_void;
    use std::os::raw::{c_int, c_uint, c_ulong};

    type Display = c_void;
    type Window = c_ulong;

    #[link(name = "X11")]
    extern "C" {
        fn XOpenDisplay(display_name: *const std::os::raw::c_char) -> *mut Display;
        fn XCloseDisplay(display: *mut Display) -> c_int;
        fn XCreateSimpleWindow(
            display: *mut Display,
            parent: Window,
            x: c_int,
            y: c_int,
            width: c_uint,
            height: c_uint,
            border_width: c_uint,
            border: c_ulong,
            background: c_ulong,
        ) -> Window;
        fn XMapWindow(display: *mut Display, w: Window) -> c_int;
        fn XUnmapWindow(display: *mut Display, w: Window) -> c_int;
        fn XMoveResizeWindow(
            display: *mut Display,
            w: Window,
            x: c_int,
            y: c_int,
            width: c_uint,
            height: c_uint,
        ) -> c_int;
        fn XDestroyWindow(display: *mut Display, w: Window) -> c_int;
        fn XFlush(display: *mut Display) -> c_int;
    }

    pub struct X11ChildWindow {
        display: *mut Display,
        pub child_xid: u64,
    }

    unsafe impl Send for X11ChildWindow {}
    unsafe impl Sync for X11ChildWindow {}

    impl X11ChildWindow {
        pub fn create(parent_xid: u64, x: i32, y: i32, width: u32, height: u32) -> Option<Self> {
            unsafe {
                let display = XOpenDisplay(std::ptr::null());
                if display.is_null() {
                    return None;
                }
                let child = XCreateSimpleWindow(
                    display,
                    parent_xid as Window,
                    x as c_int,
                    y as c_int,
                    width as c_uint,
                    height as c_uint,
                    0,
                    0,
                    0,
                );
                if child == 0 {
                    XCloseDisplay(display);
                    return None;
                }
                XMapWindow(display, child);
                XFlush(display);
                Some(Self {
                    display,
                    child_xid: child as u64,
                })
            }
        }

        pub fn resize(&self, x: i32, y: i32, width: u32, height: u32) {
            unsafe {
                if !self.display.is_null() && self.child_xid != 0 {
                    XMoveResizeWindow(
                        self.display,
                        self.child_xid as Window,
                        x as c_int,
                        y as c_int,
                        width as c_uint,
                        height as c_uint,
                    );
                    XFlush(self.display);
                }
            }
        }

        pub fn hide(&self) {
            unsafe {
                if !self.display.is_null() && self.child_xid != 0 {
                    XUnmapWindow(self.display, self.child_xid as Window);
                    XFlush(self.display);
                }
            }
        }

        pub fn show(&self) {
            unsafe {
                if !self.display.is_null() && self.child_xid != 0 {
                    XMapWindow(self.display, self.child_xid as Window);
                    XFlush(self.display);
                }
            }
        }
    }

    impl Drop for X11ChildWindow {
        fn drop(&mut self) {
            unsafe {
                if !self.display.is_null() {
                    if self.child_xid != 0 {
                        XDestroyWindow(self.display, self.child_xid as Window);
                        XFlush(self.display);
                        self.child_xid = 0;
                    }
                    XCloseDisplay(self.display);
                    self.display = std::ptr::null_mut();
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MediaPlayerService {
    socket_path: PathBuf,
    child: Arc<Mutex<Option<Child>>>,
    active_stream: Arc<Mutex<Option<String>>>,
    active_title: Arc<Mutex<Option<(String, String)>>>,
    parent_window_id: Arc<Mutex<Option<u64>>>,
    viewport_rect: Arc<Mutex<(i32, i32, u32, u32)>>,
    #[cfg(target_os = "linux")]
    embedded_child_window: Arc<Mutex<Option<Arc<x11_embed::X11ChildWindow>>>>,
}

impl Default for MediaPlayerService {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaPlayerService {
    pub fn new() -> Self {
        let socket_path = std::env::temp_dir().join("nolelamphim_mpv.sock");
        Self {
            socket_path,
            child: Arc::new(Mutex::new(None)),
            active_stream: Arc::new(Mutex::new(None)),
            active_title: Arc::new(Mutex::new(None)),
            parent_window_id: Arc::new(Mutex::new(None)),
            viewport_rect: Arc::new(Mutex::new((248, 148, 836, 480))),
            #[cfg(target_os = "linux")]
            embedded_child_window: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the host application window ID for embedding
    pub async fn set_parent_window_id(&self, xid: u64) {
        let mut p = self.parent_window_id.lock().await;
        *p = Some(xid);
        tracing::info!("MediaPlayerService registered parent window ID: {}", xid);
    }

    /// Checks if MPV player instance is currently running and responsive
    pub async fn is_running(&self) -> bool {
        if !self.socket_path.exists() {
            return false;
        }

        match UnixStream::connect(&self.socket_path).await {
            Ok(mut stream) => {
                let check_cmd = b"{\"command\": [\"get_property\", \"idle-active\"]}\n";
                if stream.write_all(check_cmd).await.is_err() {
                    return false;
                }
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.is_ok()
            }
            Err(_) => false,
        }
    }

    /// Plays a video stream URL embedded directly inside the host app window.
    /// If MPV is already running, seamlessly loads the new stream via IPC.
    pub async fn play(
        &self,
        stream_url: &str,
        movie_title: &str,
        episode_name: &str,
        win_weak: slint::Weak<AppWindow>,
    ) -> Result<(), String> {
        let trimmed_url = stream_url.trim();
        if trimmed_url.is_empty() {
            return Err("URL phát phim rỗng".to_string());
        }

        tracing::info!(
            "MediaPlayerService::play (embedded) -> title='{}', episode='{}', url='{}'",
            movie_title,
            episode_name,
            trimmed_url
        );

        {
            let mut s = self.active_stream.lock().await;
            *s = Some(trimmed_url.to_string());
            let mut t = self.active_title.lock().await;
            *t = Some((movie_title.to_string(), episode_name.to_string()));
        }

        if self.is_running().await {
            tracing::info!("MPV already running, loading new stream via IPC loadfile...");
            let load_res = self
                .send_command(serde_json::json!(["loadfile", trimmed_url]))
                .await;
            if load_res.is_ok() {
                let full_title = format!("Nô Lệ Làm Phim - {} [{}]", movie_title, episode_name);
                let _ = self
                    .send_command(serde_json::json!(["set_property", "title", full_title]))
                    .await;
                let _ = self
                    .send_command(serde_json::json!(["set_property", "pause", false]))
                    .await;
                return Ok(());
            }
        }

        // Clean up stale socket file if player was killed or crashed
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let full_title = format!("Nô Lệ Làm Phim - {} [{}]", movie_title, episode_name);

        // Check if mpv is available on the system
        let has_mpv = std::path::Path::new("/usr/bin/mpv").exists() || which_binary("mpv");

        if has_mpv {
            let mut cmd = Command::new("mpv");
            cmd.arg(trimmed_url);
            cmd.arg(format!("--title={}", full_title));
            cmd.arg("--x11-name=nolelamphim-desktop");
            cmd.arg(format!("--input-ipc-server={}", self.socket_path.display()));
            cmd.arg("--keep-open=yes");
            cmd.arg("--hwdec=auto-safe");
            cmd.arg("--osd-level=1");
            cmd.arg("--msg-level=all=warn");
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());

            // Prepare embedded window on Linux X11
            #[cfg(target_os = "linux")]
            {
                let mut parent_id = *self.parent_window_id.lock().await;
                if parent_id.is_none() {
                    // Fallback to searching window ID by title with xdotool
                    if let Ok(out) = std::process::Command::new("xdotool")
                        .args(["search", "--name", "Nolelamphim"])
                        .output()
                    {
                        if let Ok(text) = String::from_utf8(out.stdout) {
                            if let Some(line) = text.lines().next() {
                                if let Ok(id) = line.trim().parse::<u64>() {
                                    parent_id = Some(id);
                                    let mut p = self.parent_window_id.lock().await;
                                    *p = Some(id);
                                }
                            }
                        }
                    }
                }

                if let Some(parent_xid) = parent_id {
                    let (x, y, w, h) = *self.viewport_rect.lock().await;

                    let mut win_guard = self.embedded_child_window.lock().await;
                    if win_guard.is_none() {
                        if let Some(child_win) = x11_embed::X11ChildWindow::create(parent_xid, x, y, w, h) {
                            let wid = child_win.child_xid;
                            tracing::info!("Created embedded X11 child window ID {} at ({}, {}, {}, {}) inside parent {}", wid, x, y, w, h, parent_xid);
                            cmd.arg(format!("--wid={}", wid));
                            *win_guard = Some(Arc::new(child_win));
                        }
                    } else if let Some(ref child_win) = *win_guard {
                        child_win.resize(x, y, w, h);
                        child_win.show();
                        cmd.arg(format!("--wid={}", child_win.child_xid));
                    }
                } else {
                    // Fallback if no parent window found
                    cmd.arg("--geometry=75%x75%+50%+50%");
                    cmd.arg("--autofit=1280x720");
                }
            }

            match cmd.spawn() {
                Ok(child) => {
                    {
                        let mut c = self.child.lock().await;
                        *c = Some(child);
                    }

                    // Wait up to 2.5 seconds for IPC socket to become ready
                    let mut socket_ready = false;
                    for _ in 0..50 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        if self.socket_path.exists() {
                            socket_ready = true;
                            break;
                        }
                    }

                    if socket_ready {
                        tracing::info!("MPV IPC socket ready at {}. Spawning monitor loop...", self.socket_path.display());
                        let sock_path = self.socket_path.clone();
                        let service_clone = self.clone();
                        tokio::spawn(async move {
                            monitor_ipc_loop(sock_path, win_weak, service_clone).await;
                        });
                    } else {
                        tracing::warn!("MPV process spawned but socket was not created within timeout.");
                    }

                    Ok(())
                }
                Err(e) => {
                    let err_msg = format!("Lỗi khởi động trình phát MPV: {}", e);
                    tracing::error!("{}", err_msg);
                    Err(err_msg)
                }
            }
        } else if std::path::Path::new("/usr/bin/vlc").exists() || which_binary("vlc") {
            tracing::info!("Falling back to VLC player for stream: {}", trimmed_url);
            let mut cmd = Command::new("vlc");
            cmd.arg(trimmed_url);
            cmd.arg(format!("--meta-title={}", full_title));
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
            let _ = cmd.spawn();
            Ok(())
        } else if std::path::Path::new("/usr/bin/ffplay").exists() || which_binary("ffplay") {
            tracing::info!("Falling back to ffplay for stream: {}", trimmed_url);
            let mut cmd = Command::new("ffplay");
            cmd.arg("-window_title");
            cmd.arg(&full_title);
            cmd.arg(trimmed_url);
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
            let _ = cmd.spawn();
            Ok(())
        } else {
            let err = "Không tìm thấy media player (mpv, vlc, ffplay) trên hệ thống!".to_string();
            tracing::error!("{}", err);
            Err(err)
        }
    }

    /// Dynamically updates the exact pixel geometry (x, y, w, h) of the embedded video subwindow
    pub async fn update_viewport_rect(&self, x: i32, y: i32, w: u32, h: u32) {
        {
            let mut rect = self.viewport_rect.lock().await;
            *rect = (x, y, w, h);
        }
        tracing::debug!("MediaPlayerService viewport rect updated to ({}, {}, {}, {})", x, y, w, h);

        #[cfg(target_os = "linux")]
        {
            let guard = self.embedded_child_window.lock().await;
            if let Some(ref child_win) = *guard {
                child_win.resize(x, y, w, h);
                child_win.show();
            }
        }
    }

    /// Resizes and repositions the embedded video viewport inside the app
    pub async fn update_viewport(&self, is_theater: bool) {
        #[cfg(target_os = "linux")]
        {
            let (x, y, w, h) = if is_theater {
                (12, 72, 1376, 740)
            } else {
                (248, 148, 836, 480)
            };
            self.update_viewport_rect(x, y, w, h).await;
        }
    }

    /// Hides embedded video when navigating away from player
    pub async fn hide_embedded(&self) {
        #[cfg(target_os = "linux")]
        {
            let guard = self.embedded_child_window.lock().await;
            if let Some(ref child_win) = *guard {
                child_win.hide();
            }
        }
        let _ = self.send_command(serde_json::json!(["set_property", "pause", true])).await;
    }

    /// Shows embedded video when navigating to player
    pub async fn show_embedded(&self) {
        #[cfg(target_os = "linux")]
        {
            let (x, y, w, h) = *self.viewport_rect.lock().await;
            let guard = self.embedded_child_window.lock().await;
            if let Some(ref child_win) = *guard {
                child_win.resize(x, y, w, h);
                child_win.show();
            }
        }
    }

    /// Toggles play / pause state
    pub async fn toggle_pause(&self) -> Result<(), String> {
        let _ = self.send_command(serde_json::json!(["cycle", "pause"])).await;
        Ok(())
    }

    /// Seeks to absolute percentage (0..100)
    pub async fn seek_percent(&self, percent: i32) -> Result<(), String> {
        let p = percent.clamp(0, 100);
        let _ = self
            .send_command(serde_json::json!(["seek", p, "absolute-percent"]))
            .await;
        Ok(())
    }

    /// Skips forward or backward by seconds (e.g. +10.0 or -10.0)
    pub async fn skip_seconds(&self, seconds: f64) -> Result<(), String> {
        let _ = self
            .send_command(serde_json::json!(["seek", seconds, "relative"]))
            .await;
        Ok(())
    }

    /// Sets playback volume (0..100)
    pub async fn set_volume(&self, volume: i32) -> Result<(), String> {
        let v = volume.clamp(0, 100);
        let _ = self
            .send_command(serde_json::json!(["set_property", "volume", v]))
            .await;
        Ok(())
    }

    /// Sets playback speed multiplier (e.g. 1.0, 1.25, 1.5, 2.0)
    pub async fn set_speed(&self, speed: f64) -> Result<(), String> {
        let _ = self
            .send_command(serde_json::json!(["set_property", "speed", speed]))
            .await;
        Ok(())
    }

    /// Toggles fullscreen in the player
    pub async fn toggle_fullscreen(&self) -> Result<(), String> {
        let _ = self.send_command(serde_json::json!(["cycle", "fullscreen"])).await;
        Ok(())
    }

    /// Stops playback and terminates the player process
    pub async fn stop(&self) -> Result<(), String> {
        let _ = self.send_command(serde_json::json!(["quit"])).await;
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
        #[cfg(target_os = "linux")]
        {
            let mut win_guard = self.embedded_child_window.lock().await;
            *win_guard = None;
        }
        Ok(())
    }

    /// Sends a raw command line over Unix Domain Socket to MPV
    async fn send_command(&self, cmd: serde_json::Value) -> Result<serde_json::Value, String> {
        if !self.socket_path.exists() {
            return Err("MPV socket không tồn tại".to_string());
        }

        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("Không thể kết nối MPV IPC: {}", e))?;

        let payload = format!("{}\n", serde_json::json!({ "command": cmd }));
        stream
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| format!("Lỗi ghi IPC MPV: {}", e))?;
        stream.flush().await.map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("Lỗi đọc phản hồi MPV: {}", e))?;

        serde_json::from_str(&line).map_err(|e| format!("JSON không hợp lệ từ MPV: {}", e))
    }
}

/// Helper to check if binary is found in PATH
fn which_binary(name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let p = std::path::Path::new(dir).join(name);
            if p.is_file() {
                return true;
            }
        }
    }
    false
}

/// Dedicated background task that observes MPV properties and dispatches real-time updates to Slint UI
async fn monitor_ipc_loop(
    socket_path: PathBuf,
    win_weak: slint::Weak<AppWindow>,
    _service: MediaPlayerService,
) {
    let stream = match UnixStream::connect(&socket_path).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Không thể kết nối socket MPV IPC để lắng nghe: {}", e);
            return;
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Register property observers
    let observe_cmds = [
        serde_json::json!({"command": ["observe_property", 1, "time-pos"]}),
        serde_json::json!({"command": ["observe_property", 2, "duration"]}),
        serde_json::json!({"command": ["observe_property", 3, "pause"]}),
        serde_json::json!({"command": ["observe_property", 4, "volume"]}),
        serde_json::json!({"command": ["observe_property", 5, "speed"]}),
    ];

    for cmd in observe_cmds {
        let msg = format!("{}\n", cmd);
        if let Err(e) = writer.write_all(msg.as_bytes()).await {
            tracing::warn!("Lỗi gửi lệnh observe_property: {}", e);
            return;
        }
    }
    let _ = writer.flush().await;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                tracing::info!("Kết nối MPV IPC đã ngắt (người dùng đóng rạp phim hoặc hết phim)");
                break;
            }
            Ok(_) => {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(event) = val.get("event").and_then(|e| e.as_str()) {
                        if event == "property-change" {
                            let name = val.get("name").and_then(|n| n.as_str()).unwrap_or("");
                            let data = val.get("data");

                            let win_weak_clone = win_weak.clone();
                            match name {
                                "time-pos" => {
                                    if let Some(sec_f) = data.and_then(|d| d.as_f64()) {
                                        let sec = sec_f as i32;
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(w) = win_weak_clone.upgrade() {
                                                w.set_player_progress_seconds(sec);
                                                let dur = w.get_player_duration_seconds().max(1);
                                                let pct = ((sec as f32 / dur as f32) * 100.0) as i32;
                                                w.set_player_progress_percent(pct.clamp(0, 100));
                                                w.set_player_time_display(SharedString::from(format_cinema_duration(sec as i64, dur as i64)));
                                            }
                                        });
                                    }
                                }
                                "duration" => {
                                    if let Some(dur_f) = data.and_then(|d| d.as_f64()) {
                                        let dur = dur_f as i32;
                                        if dur > 0 {
                                            let _ = slint::invoke_from_event_loop(move || {
                                                if let Some(w) = win_weak_clone.upgrade() {
                                                    w.set_player_duration_seconds(dur);
                                                    let sec = w.get_player_progress_seconds();
                                                    let pct = ((sec as f32 / dur as f32) * 100.0) as i32;
                                                    w.set_player_progress_percent(pct.clamp(0, 100));
                                                    w.set_player_time_display(SharedString::from(format_cinema_duration(sec as i64, dur as i64)));
                                                }
                                            });
                                        }
                                    }
                                }
                                "pause" => {
                                    if let Some(paused) = data.and_then(|d| d.as_bool()) {
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(w) = win_weak_clone.upgrade() {
                                                w.set_is_playing(!paused);
                                            }
                                        });
                                    }
                                }
                                "volume" => {
                                    if let Some(vol_f) = data.and_then(|d| d.as_f64()) {
                                        let vol = vol_f as i32;
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(w) = win_weak_clone.upgrade() {
                                                w.set_player_volume(vol);
                                            }
                                        });
                                    }
                                }
                                "speed" => {
                                    if let Some(spd_f) = data.and_then(|d| d.as_f64()) {
                                        let spd_str = format!("{:.1}x", spd_f);
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(w) = win_weak_clone.upgrade() {
                                                w.set_player_speed(SharedString::from(spd_str));
                                            }
                                        });
                                    }
                                }
                                _ => {}
                            }
                        } else if event == "end-file" {
                            tracing::info!("Tập phim đã phát xong, tự động chuyển tập tiếp theo...");
                            let win_weak_clone = win_weak.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(w) = win_weak_clone.upgrade() {
                                    w.invoke_next_episode();
                                }
                            });
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Lỗi đọc IPC từ MPV: {}", e);
                break;
            }
        }
    }

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(w) = win_weak.upgrade() {
            w.set_is_playing(false);
        }
    });
}
