use console_error_panic_hook::set_once;
use hot_reload::Model;
use nannou::prelude::*;
use wasm_bindgen::prelude::*;

const WEBGPU_COMMON_MAX_TEXTURE_SIZE: f64 = 2048.0;

// Keep web sizing in one helper so tests can guard common WebGPU limits.
fn web_window_size() -> (u32, u32) {
    browser_viewport_size()
        .map(|(viewport_width, viewport_height, device_pixel_ratio)| {
            largest_safe_web_window_size(viewport_width, viewport_height, device_pixel_ratio)
        })
        .unwrap_or((hot_reload::WINDOW_WIDTH, hot_reload::WINDOW_HEIGHT))
}

fn largest_safe_web_window_size(
    viewport_width: f64,
    viewport_height: f64,
    device_pixel_ratio: f64,
) -> (u32, u32) {
    let aspect = hot_reload::ASPECT_RATIO_WIDTH as f64 / hot_reload::ASPECT_RATIO_HEIGHT as f64;
    let viewport_width = viewport_width.max(1.0);
    let viewport_height = viewport_height.max(1.0);
    let pixel_ratio = device_pixel_ratio.max(1.0);
    let viewport_aspect = viewport_width / viewport_height;

    let fitted_width = if viewport_aspect > aspect {
        viewport_height * aspect
    } else {
        viewport_width
    };

    let max_logical_width = WEBGPU_COMMON_MAX_TEXTURE_SIZE / pixel_ratio;
    let max_logical_height = WEBGPU_COMMON_MAX_TEXTURE_SIZE / pixel_ratio;
    let width = fitted_width
        .min(max_logical_width)
        .min(max_logical_height * aspect)
        .floor()
        .max(1.0) as u32;
    let height = ((width as f64) / aspect).floor().max(1.0) as u32;

    (width, height)
}

#[cfg(target_arch = "wasm32")]
fn browser_viewport_size() -> Option<(f64, f64, f64)> {
    let window = web_sys::window()?;
    Some((
        window.inner_width().ok()?.as_f64()?,
        window.inner_height().ok()?.as_f64()?,
        window.device_pixel_ratio(),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn browser_viewport_size() -> Option<(f64, f64, f64)> {
    None
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
    const WEBGPU_COMMON_MAX_TEXTURE_SIZE: u32 = super::WEBGPU_COMMON_MAX_TEXTURE_SIZE as u32;

    #[test]
    fn web_window_size_stays_below_common_webgpu_texture_limit_at_2x_dpi() {
        let (width, height) = web_window_size();

        assert!(width * WEBGPU_SAFE_DEVICE_PIXEL_RATIO <= WEBGPU_COMMON_MAX_TEXTURE_SIZE);
        assert!(height * WEBGPU_SAFE_DEVICE_PIXEL_RATIO <= WEBGPU_COMMON_MAX_TEXTURE_SIZE);
    }

    #[test]
    fn web_window_size_expands_to_viewport_until_texture_limit() {
        let (width, height) = largest_safe_web_window_size(1920.0, 1080.0, 1.5);

        assert!(width > hot_reload::WINDOW_WIDTH);
        assert_eq!((width, height), (1365, 853));
        assert!((width as f64 * 1.5) <= WEBGPU_COMMON_MAX_TEXTURE_SIZE as f64);
        assert!((height as f64 * 1.5) <= WEBGPU_COMMON_MAX_TEXTURE_SIZE as f64);
    }

    #[test]
    fn runweb_fullscreen_shortcut_claims_plain_f_before_canvas_handlers() {
        let runweb_script = include_str!("../../../scripts/other/RunWeb.ps1");

        assert!(runweb_script.contains("event.stopImmediatePropagation();"));
        assert!(runweb_script.contains(
            "window.addEventListener(\"keydown\", handleFullscreenShortcut, { capture: true });"
        ));
    }

    #[test]
    fn runweb_canvas_display_size_refits_to_current_viewport() {
        let runweb_script = include_str!("../../../scripts/other/RunWeb.ps1");

        assert!(runweb_script.contains("const viewportWidth = window.innerWidth;"));
        assert!(runweb_script.contains("const viewportHeight = window.innerHeight;"));
        assert!(
            runweb_script.contains("window.addEventListener(\"fullscreenchange\", focusCanvas);")
        );
    }
}
