use hot_reload_sketch::*;
use nannou::prelude::*;
use nannou::window::Fullscreen;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const TARGET_FPS: f64 = 120.0;

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

    hot_functions_from_file!("rust/crates/hot-reload/lib.rs");

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

    let saved_window = StoredWindow::load();
    let window_id = app
        .new_window()
        .size(hot_reload::WINDOW_WIDTH, hot_reload::WINDOW_HEIGHT)
        .title("nannou-creative-coding")
        .visible(false)
        .event(window_event)
        .view(view)
        .build()
        .expect("failed to build nannou window");

    restore_window(app, window_id, saved_window.as_ref());
    show_window(app, window_id);

    Model::new()
}

fn update(app: &App, model: &mut Model, update: Update) {
    if model.last_window_state_save.elapsed().as_millis() >= 500 {
        persist_focused_window(app);
        model.last_window_state_save = std::time::Instant::now();
    }

    model.was_updated = hot_reload_sketch::was_updated();
    hot_reload_sketch::update(app, model, update);
}

fn window_event(app: &App, _model: &mut Model, event: WindowEvent) {
    hot_reload_sketch::window_event(app, _model, &event);

    match event {
        KeyPressed(Key::F) => toggle_fullscreen(app),
        KeyPressed(Key::Escape) | Moved(_) | Resized(_) => persist_focused_window(app),
        _ => {}
    }
}

fn toggle_fullscreen(app: &App) {
    let window = app.main_window();

    if window.fullscreen().is_some() {
        window.set_fullscreen_with(None);
    } else {
        let monitor = window.current_monitor().or_else(|| app.primary_monitor());
        window.set_fullscreen_with(Some(Fullscreen::Borderless(monitor)));
    }

    StoredWindow::from_window(&window).save();
}

fn persist_focused_window(app: &App) {
    let window = app.main_window();
    StoredWindow::from_window(&window).save();
}

fn restore_window(app: &App, window_id: WindowId, saved_window: Option<&StoredWindow>) {
    let Some(window) = app.window(window_id) else {
        return;
    };

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct StoredWindow {
    fullscreen: bool,
    position: Option<StoredPosition>,
    size: Option<StoredSize>,
    monitor: Option<StoredMonitor>,
}

impl StoredWindow {
    fn from_window(window: &nannou::window::Window) -> Self {
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
        }
    }

    fn load() -> Option<Self> {
        let content = fs::read_to_string(storage_path()).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save(&self) {
        let path = storage_path();

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
