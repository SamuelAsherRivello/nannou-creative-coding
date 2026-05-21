use console_error_panic_hook::set_once;
use hot_reload::Model;
use nannou::prelude::*;
use wasm_bindgen::prelude::*;

// Keep web sizing in one helper so tests can guard common WebGPU limits.
fn web_window_size() -> (u32, u32) {
    (hot_reload::WINDOW_WIDTH, hot_reload::WINDOW_HEIGHT)
}

#[wasm_bindgen(start)]
pub fn start() {
    // Convert Rust panics into browser console errors for easier debugging.
    set_once();
    let (width, height) = web_window_size();
    nannou::app::Builder::new(model)
        .size(width, height)
        .simple_window(hot_reload::view)
        .event(event)
        .update(hot_reload::update)
        .backends(wgpu::Backends::BROWSER_WEBGPU.union(wgpu::Backends::GL))
        .run();
}

fn model(app: &App) -> Model {
    eprintln!("[runweb] building model");
    app.set_fullscreen_on_shortcut(false);
    eprintln!("[runweb] window build succeeded");

    Model::new()
}

fn event(app: &App, model: &mut Model, event: Event) {
    // The shared hot_reload crate only needs simplified window events.
    if let Event::WindowEvent {
        simple: Some(event),
        ..
    } = event
    {
        hot_reload::window_event(app, model, &event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBGPU_SAFE_DEVICE_PIXEL_RATIO: u32 = 2;
    const WEBGPU_COMMON_MAX_TEXTURE_SIZE: u32 = 2048;

    #[test]
    fn web_window_size_stays_below_common_webgpu_texture_limit_at_2x_dpi() {
        let (width, height) = web_window_size();

        assert!(width * WEBGPU_SAFE_DEVICE_PIXEL_RATIO <= WEBGPU_COMMON_MAX_TEXTURE_SIZE);
        assert!(height * WEBGPU_SAFE_DEVICE_PIXEL_RATIO <= WEBGPU_COMMON_MAX_TEXTURE_SIZE);
    }

    #[test]
    fn runweb_fullscreen_shortcut_claims_plain_f_before_canvas_handlers() {
        let runweb_script = include_str!("../../../scripts/other/RunWeb.ps1");

        assert!(runweb_script.contains("event.stopImmediatePropagation();"));
        assert!(runweb_script.contains(
            "window.addEventListener(\"keydown\", handleFullscreenShortcut, { capture: true });"
        ));
    }
}
