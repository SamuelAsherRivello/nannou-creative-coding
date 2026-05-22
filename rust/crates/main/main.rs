use hot_reload_sketch::*;
use nannou::prelude::*;
use nannou::window::{Fullscreen, SurfaceConfigurationBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

// The runner targets high-FPS creative sketches and throttles manually.
const TARGET_FPS: f64 = 120.0;
const TARGET_FRAME_INTERVAL: Duration = Duration::from_nanos(8_333_333);
const TARGET_PRESENT_MODE: nannou::wgpu::PresentMode = nannou::wgpu::PresentMode::AutoNoVsync;

// hot-lib-reloader rebuilds the sketch crate and swaps it while the app runs.
#[hot_lib_reloader::hot_module(
    dylib = "hot_reload",
    lib_dir = if cfg!(debug_assertions) {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../target/debug")
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../target/release")
    },
    file_watch_debounce = 50
)]
mod hot_reload_sketch {
    pub use hot_reload::*;
    pub use nannou::prelude::*;

    hot_functions_from_file!("rust/crates/hot_reload/lib.rs");

    #[lib_updated]
    pub fn was_updated() -> bool {}
}

fn main() {
    nannou::app(model)
        .loop_mode(LoopMode::rate_fps(TARGET_FPS))
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    app.set_fullscreen_on_shortcut(false);

    // Restore the last window placement before showing the window.
    let saved_window = StoredWindow::load();
    let saved_demo_index = saved_window
        .as_ref()
        .map(|saved_window| saved_window.demo_index)
        .unwrap_or_default();
    let saved_hud_visible = saved_window
        .as_ref()
        .map(|saved_window| saved_window.hud_visible)
        .unwrap_or(true);
    let window_id = app
        .new_window()
        .size(hot_reload::WINDOW_WIDTH, hot_reload::WINDOW_HEIGHT)
        .title("nannou-creative-coding")
        .surface_conf_builder(surface_configuration_builder())
        .visible(false)
        .event(window_event)
        .view(view)
        .build()
        .expect("failed to build nannou window");

    restore_window(app, window_id, saved_window.as_ref());
    show_window(app, window_id);

    Model::new_with_settings(saved_demo_index, saved_hud_visible)
}

fn update(app: &App, model: &mut Model, update: Update) {
    throttle_to_target_fps(update.since_last);

    // Save window state periodically so move/resize changes survive restarts.
    if model.last_window_state_save.elapsed().as_millis() >= 500 {
        persist_focused_window(app, model);
        model.last_window_state_save = std::time::Instant::now();
    }

    model.was_updated = hot_reload_sketch::was_updated();
    hot_reload_sketch::update(app, model, update);
}

fn throttle_to_target_fps(elapsed_since_last_update: Duration) {
    if let Some(remaining) = target_frame_sleep_duration(elapsed_since_last_update) {
        std::thread::sleep(remaining);
    }
}

fn target_frame_sleep_duration(elapsed_since_last_update: Duration) -> Option<Duration> {
    TARGET_FRAME_INTERVAL.checked_sub(elapsed_since_last_update)
}

fn window_event(app: &App, model: &mut Model, event: WindowEvent) {
    hot_reload_sketch::window_event(app, model, &event);

    match event {
        KeyPressed(Key::F)
        | KeyPressed(Key::H)
        | KeyPressed(Key::Left)
        | KeyPressed(Key::Right)
        | KeyPressed(Key::Escape)
        | Moved(_)
        | Resized(_) => persist_focused_window(app, model),
        _ => {}
    }
}

fn persist_focused_window(app: &App, model: &Model) {
    let window = app.main_window();
    StoredWindow::from_window(&window, model.current_demo_index(), model.hud_visible()).save();
}

fn restore_window(app: &App, window_id: WindowId, saved_window: Option<&StoredWindow>) {
    let Some(window) = app.window(window_id) else {
        return;
    };

    // Fullscreen restore prefers the original monitor, then falls back safely.
    if let Some(saved_window) = saved_window {
        if saved_window.fullscreen {
            let monitor = saved_window
                .monitor
                .as_ref()
                .and_then(|saved_monitor| find_monitor(app, saved_monitor))
                .or_else(|| app.primary_monitor());

            window.set_fullscreen_with(Some(Fullscreen::Borderless(monitor)));
            return;
        }

        if let Some(size) = saved_window.size {
            window.set_inner_size_pixels(size.width, size.height);
        }

        if let Some(position) = saved_window.position {
            window.set_outer_position_pixels(position.x, position.y);
            return;
        }
    }

    center_window(app, window_id);
}

fn show_window(app: &App, window_id: WindowId) {
    if let Some(window) = app.window(window_id) {
        window.set_visible(true);
        window.winit_window().focus_window();
        window.winit_window().request_user_attention(Some(
            nannou::winit::window::UserAttentionType::Informational,
        ));
    }
}

fn center_window(app: &App, window_id: WindowId) {
    let Some(window) = app.window(window_id) else {
        return;
    };

    let Some(monitor) = app.primary_monitor() else {
        return;
    };

    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let x = monitor_position.x + (monitor_size.width as i32 - hot_reload::WINDOW_WIDTH as i32) / 2;
    let y =
        monitor_position.y + (monitor_size.height as i32 - hot_reload::WINDOW_HEIGHT as i32) / 2;

    window.set_outer_position_pixels(x, y);
}

fn find_monitor(
    app: &App,
    saved_monitor: &StoredMonitor,
) -> Option<nannou::winit::monitor::MonitorHandle> {
    // Match monitor geometry as well as name because names can be duplicated.
    app.available_monitors().into_iter().find(|monitor| {
        let position = monitor.position();
        let size = monitor.size();

        monitor.name() == saved_monitor.name
            && position.x == saved_monitor.x
            && position.y == saved_monitor.y
            && size.width == saved_monitor.width
            && size.height == saved_monitor.height
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredWindow {
    fullscreen: bool,
    position: Option<StoredPosition>,
    size: Option<StoredSize>,
    monitor: Option<StoredMonitor>,
    #[serde(default)]
    demo_index: usize,
    #[serde(default = "default_hud_visible")]
    hud_visible: bool,
}

impl Default for StoredWindow {
    fn default() -> Self {
        Self {
            fullscreen: false,
            position: None,
            size: None,
            monitor: None,
            demo_index: 0,
            hud_visible: default_hud_visible(),
        }
    }
}

impl StoredWindow {
    // Capture enough window metadata to restore the user's workspace next run.
    fn from_window(window: &nannou::window::Window, demo_index: usize, hud_visible: bool) -> Self {
        let position = window
            .outer_position_pixels()
            .ok()
            .map(|(x, y)| StoredPosition { x, y });
        let (width, height) = window.inner_size_pixels();

        Self {
            fullscreen: window.fullscreen().is_some(),
            position,
            size: Some(StoredSize { width, height }),
            monitor: window.current_monitor().map(StoredMonitor::from_monitor),
            demo_index,
            hud_visible,
        }
    }

    fn load() -> Option<Self> {
        let content = fs::read_to_string(storage_path()).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save(&self) {
        let path = storage_path();

        // The target directory may not exist on a fresh checkout.
        if let Some(parent) = path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("failed to create window-state directory: {error}");
                return;
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(error) = fs::write(path, content) {
                    eprintln!("failed to save window state: {error}");
                }
            }
            Err(error) => eprintln!("failed to serialize window state: {error}"),
        }
    }
}

fn default_hud_visible() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct StoredPosition {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct StoredSize {
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredMonitor {
    name: Option<String>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl StoredMonitor {
    fn from_monitor(monitor: nannou::winit::monitor::MonitorHandle) -> Self {
        let position = monitor.position();
        let size = monitor.size();

        Self {
            name: monitor.name(),
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }
}

fn storage_path() -> PathBuf {
    target_path().join("window-state.json")
}

fn target_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("target")
}

fn surface_configuration_builder() -> SurfaceConfigurationBuilder {
    SurfaceConfigurationBuilder::new().present_mode(TARGET_PRESENT_MODE)
}

#[cfg(test)]
mod tests {
    use super::{
        surface_configuration_builder, target_frame_sleep_duration, StoredWindow, TARGET_FPS,
        TARGET_FRAME_INTERVAL, TARGET_PRESENT_MODE,
    };
    use std::time::Duration;

    #[test]
    fn targets_120_fps_without_vsync_throttling() {
        assert_eq!(TARGET_FPS, 120.0);
        assert_eq!(TARGET_FRAME_INTERVAL, Duration::from_nanos(8_333_333));
        assert_eq!(
            surface_configuration_builder().present_mode,
            Some(TARGET_PRESENT_MODE)
        );
    }

    #[test]
    fn frame_throttle_sleeps_until_120_fps_interval() {
        assert_eq!(
            target_frame_sleep_duration(Duration::from_millis(5)),
            Some(Duration::from_nanos(3_333_333))
        );
        assert_eq!(target_frame_sleep_duration(Duration::from_millis(9)), None);
    }

    #[test]
    fn window_state_serializes_demo_index() {
        let state = StoredWindow {
            demo_index: 1,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&state).expect("state should serialize");
        let restored: StoredWindow =
            serde_json::from_str(&serialized).expect("state should deserialize");

        assert_eq!(restored.demo_index, 1);
    }

    #[test]
    fn window_state_serializes_hud_visibility() {
        let state = StoredWindow {
            hud_visible: false,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&state).expect("state should serialize");
        let restored: StoredWindow =
            serde_json::from_str(&serialized).expect("state should deserialize");

        assert!(!restored.hud_visible);
    }

    #[test]
    fn window_state_defaults_missing_demo_index_to_first_demo() {
        let restored: StoredWindow =
            serde_json::from_str(r#"{"fullscreen":false}"#).expect("old state should deserialize");

        assert_eq!(restored.demo_index, 0);
    }

    #[test]
    fn window_state_defaults_missing_hud_visibility_to_visible() {
        let restored: StoredWindow =
            serde_json::from_str(r#"{"fullscreen":false}"#).expect("old state should deserialize");

        assert!(restored.hud_visible);
    }
}
