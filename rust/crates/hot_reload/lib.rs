use instant::Instant;
use nannou::prelude::*;

// Each demo module owns one hot-reloadable sketch and its local state.
mod demo_01;
mod demo_02;
mod demo_03;

// The desktop and web runners share this fixed design size and aspect ratio.
pub const WINDOW_WIDTH: u32 = 1024;
pub const WINDOW_HEIGHT: u32 = 640;
pub const ASPECT_RATIO_WIDTH: f32 = 16.0;
pub const ASPECT_RATIO_HEIGHT: f32 = 10.0;
const FPS_DISPLAY_UPDATE_SECONDS: f32 = 2.0;
const STATUS_MARGIN: f32 = 18.0;
const HUD_WIDTH: f32 = 264.0;
const STATUS_GAP: f32 = 8.0;
const STATUS_TEXT_SIZE: u32 = 15;
const STATUS_LINE_HEIGHT: f32 = 20.0;
const STATUS_VERTICAL_PADDING: f32 = 14.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoHud {
    pub demo_hud_text: String,
}

// The viewport maps the resizable window onto the fixed-aspect drawing area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectViewport {
    pub frame_rect: Rect,
    pub content_rect: Rect,
}

impl AspectViewport {
    pub fn current(app: &App) -> Self {
        Self::fit(app.window_rect())
    }

    // Fits the target aspect ratio inside any window while preserving center.
    pub fn fit(frame_rect: Rect) -> Self {
        let target_aspect = ASPECT_RATIO_WIDTH / ASPECT_RATIO_HEIGHT;
        let frame_width = frame_rect.w().max(0.0);
        let frame_height = frame_rect.h().max(0.0);
        let frame_aspect = if frame_height > 0.0 {
            frame_width / frame_height
        } else {
            target_aspect
        };

        let (content_width, content_height) = if frame_aspect > target_aspect {
            (frame_height * target_aspect, frame_height)
        } else {
            (frame_width, frame_width / target_aspect)
        };

        let content_rect = Rect::from_x_y_w_h(
            frame_rect.x(),
            frame_rect.y(),
            content_width,
            content_height,
        );

        Self {
            frame_rect,
            content_rect,
        }
    }

    pub fn contains(self, position: Point2) -> bool {
        self.content_rect.contains(position)
    }

    pub fn clamp(self, position: Point2) -> Point2 {
        pt2(
            position
                .x
                .clamp(self.content_rect.left(), self.content_rect.right()),
            position
                .y
                .clamp(self.content_rect.bottom(), self.content_rect.top()),
        )
    }
}

pub struct Model {
    pub last_window_state_save: Instant,
    // FPS is averaged for readability instead of changing every frame.
    displayed_fps: f32,
    fps_accumulator: f32,
    fps_sample_count: u32,
    last_fps_display_update: f32,
    demo_index: usize,
    demo_state: demo_state,
    pub was_updated: bool,
}

impl Model {
    pub fn new() -> Self {
        Self::new_with_demo_index(0)
    }

    pub fn new_with_demo_index(demo_index: usize) -> Self {
        let demo_index = normalize_demo_index(demo_index);

        Self {
            last_window_state_save: Instant::now(),
            displayed_fps: 120.0,
            fps_accumulator: 0.0,
            fps_sample_count: 0,
            last_fps_display_update: 0.0,
            demo_index,
            demo_state: demo_state::new(demo_index),
            was_updated: false,
        }
    }

    pub fn current_demo_index(&self) -> usize {
        self.demo_index
    }
}

#[allow(non_camel_case_types)]
struct demo_state {
    // Dynamic dispatch lets the runner switch demos through one common API.
    inner: Box<dyn demo_runtime>,
}

impl demo_state {
    fn new(index: usize) -> Self {
        match index {
            0 => Self {
                inner: Box::new(demo_01::State::new()),
            },
            1 => Self {
                inner: Box::new(demo_02::State::new()),
            },
            2 => Self {
                inner: Box::new(demo_03::State::new()),
            },
            _ => Self {
                inner: Box::new(demo_01::State::new()),
            },
        }
    }

    fn render_hud(&self) -> DemoHud {
        self.inner.render_hud()
    }

    fn window_event(&mut self, app: &App, event: &WindowEvent) {
        self.inner.window_event(app, event);
    }

    fn view(&self, app: &App, draw: &Draw, viewport: AspectViewport) {
        self.inner.view(app, draw, viewport);
    }
}

#[allow(non_camel_case_types)]
trait demo_runtime {
    // Every demo exposes the same small lifecycle surface to the hot runner.
    fn render_hud(&self) -> DemoHud;
    fn window_event(&mut self, app: &App, event: &WindowEvent);
    fn view(&self, app: &App, draw: &Draw, viewport: AspectViewport);
}

impl demo_runtime for demo_01::State {
    fn render_hud(&self) -> DemoHud {
        demo_01::render_hud(self)
    }

    fn window_event(&mut self, app: &App, event: &WindowEvent) {
        demo_01::window_event(app, self, event);
    }

    fn view(&self, app: &App, draw: &Draw, viewport: AspectViewport) {
        demo_01::view(app, self, draw, viewport);
    }
}

impl demo_runtime for demo_02::State {
    fn render_hud(&self) -> DemoHud {
        demo_02::render_hud(self)
    }

    fn window_event(&mut self, app: &App, event: &WindowEvent) {
        demo_02::window_event(app, self, event);
    }

    fn view(&self, app: &App, draw: &Draw, viewport: AspectViewport) {
        demo_02::view(app, self, draw, viewport);
    }
}

impl demo_runtime for demo_03::State {
    fn render_hud(&self) -> DemoHud {
        demo_03::render_hud(self)
    }

    fn window_event(&mut self, app: &App, event: &WindowEvent) {
        demo_03::window_event(app, self, event);
    }

    fn view(&self, app: &App, draw: &Draw, viewport: AspectViewport) {
        demo_03::view(app, self, draw, viewport);
    }
}

#[no_mangle]
pub fn window_event(app: &App, model: &mut Model, event: &WindowEvent) {
    match event {
        WindowEvent::KeyPressed(Key::F) => toggle_fullscreen(app),
        // Demo switches recreate state so each sketch starts cleanly.
        WindowEvent::KeyPressed(Key::Left) => select_previous_demo(model),
        WindowEvent::KeyPressed(Key::Right) => select_next_demo(model),
        _ => model.demo_state.window_event(app, event),
    }
}

fn toggle_fullscreen(app: &App) {
    let window = app.main_window();
    window.set_fullscreen(window.fullscreen().is_none());
}

fn select_previous_demo(model: &mut Model) {
    model.demo_index = (model.demo_index + demo_count() - 1) % demo_count();
    model.demo_state = demo_state::new(model.demo_index);
}

fn select_next_demo(model: &mut Model) {
    model.demo_index = (model.demo_index + 1) % demo_count();
    model.demo_state = demo_state::new(model.demo_index);
}

fn demo_count() -> usize {
    3
}

#[no_mangle]
pub fn update(app: &App, model: &mut Model, _update: Update) {
    if model.was_updated {
        // Hot reload swaps code; rebuilding demo state keeps old layouts out.
        reload_current_demo(model);
    }

    update_displayed_fps(app, model);
    model.was_updated = false;
}

fn reload_current_demo(model: &mut Model) {
    model.demo_index = normalize_demo_index(model.demo_index);
    model.demo_state = demo_state::new(model.demo_index);
}

fn normalize_demo_index(index: usize) -> usize {
    index % demo_count()
}

fn update_displayed_fps(app: &App, model: &mut Model) {
    model.fps_accumulator += app.fps();
    model.fps_sample_count += 1;

    let elapsed = app.time - model.last_fps_display_update;
    if elapsed < FPS_DISPLAY_UPDATE_SECONDS || model.fps_sample_count == 0 {
        return;
    }

    model.displayed_fps = model.fps_accumulator / model.fps_sample_count as f32;
    model.fps_accumulator = 0.0;
    model.fps_sample_count = 0;
    model.last_fps_display_update = app.time;
}

fn format_fps(value: f32) -> String {
    format!("{:06.2}", value.clamp(0.0, 999.99))
}

#[no_mangle]
pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let viewport = AspectViewport::current(app);

    // The shared black background shows through any letterbox or pillarbox area.
    draw.background().color(BLACK);
    model.demo_state.view(app, &draw, viewport);
    draw_status_overlay(model, &draw, viewport);

    if let Err(error) = draw.to_frame(app, &frame) {
        eprintln!("[runweb] failed to draw frame: {:?}", error);
    }
}

fn draw_status_overlay(model: &Model, draw: &Draw, viewport: AspectViewport) {
    let content = viewport.content_rect;
    let app_text = format!(
        "FPS {}\nKeys: ←, →, F, Esc",
        format_fps(model.displayed_fps)
    );
    let demo_hud = model.demo_state.render_hud();
    let demo_text = demo_hud.demo_hud_text;
    let app_height = status_height_for_text(&app_text);
    let demo_height = status_height_for_text(&demo_text);
    // HUD boxes are pinned inside the content area, independent of window size.
    let app_center = pt2(
        content.left() + STATUS_MARGIN + HUD_WIDTH * 0.5,
        content.top() - STATUS_MARGIN - app_height * 0.5,
    );
    let demo_center = pt2(
        app_center.x,
        app_center.y - app_height * 0.5 - STATUS_GAP - demo_height * 0.5,
    );

    draw_status_box(draw, app_center, app_height, &app_text);
    draw_status_box(draw, demo_center, demo_height, &demo_text);
}

fn status_height_for_text(text: &str) -> f32 {
    let line_count = text.lines().count().max(1) as f32;
    line_count * STATUS_LINE_HEIGHT + STATUS_VERTICAL_PADDING
}

fn draw_status_box(draw: &Draw, center: Point2, height: f32, text: &str) {
    draw.rect()
        .xy(center)
        .w_h(HUD_WIDTH, height)
        .color(rgba(0.0, 0.0, 0.0, 0.86));

    draw.text(&text)
        .xy(center)
        .w_h(HUD_WIDTH - 12.0, height - 8.0)
        .left_justify()
        .align_text_top()
        .font_size(STATUS_TEXT_SIZE)
        .color(WHITE);
}

#[cfg(test)]
mod tests {
    use super::{
        demo_01, demo_02, demo_03, format_fps, reload_current_demo, select_next_demo,
        select_previous_demo, AspectViewport, Model, HUD_WIDTH,
    };
    use nannou::prelude::Rect;

    #[test]
    fn formats_fps_with_two_decimal_places() {
        assert_eq!(format_fps(120.0), "120.00");
        assert_eq!(format_fps(9.4), "009.40");
    }

    #[test]
    fn caps_fps_display_length() {
        assert_eq!(format_fps(1_234.56), "999.99");
        assert!(format_fps(1_234.56).len() <= "999.99".len());
    }

    #[test]
    fn hud_width_is_reduced_to_eighty_percent() {
        assert_eq!(HUD_WIDTH, 264.0);
    }

    #[test]
    fn view_does_not_log_every_frame() {
        let source = include_str!("lib.rs");
        let noisy_frame_log = concat!("[runweb] ", "view callback");

        assert!(!source.contains(noisy_frame_log));
    }

    #[test]
    fn aspect_viewport_letterboxes_taller_frames() {
        let viewport = AspectViewport::fit(Rect::from_w_h(800.0, 600.0));

        assert_eq!(viewport.content_rect.w(), 800.0);
        assert_eq!(viewport.content_rect.h(), 500.0);
        assert_eq!(viewport.content_rect.left(), -400.0);
        assert_eq!(viewport.content_rect.right(), 400.0);
        assert_eq!(viewport.content_rect.top(), 250.0);
        assert_eq!(viewport.content_rect.bottom(), -250.0);
    }

    #[test]
    fn aspect_viewport_pillarboxes_wider_frames() {
        let viewport = AspectViewport::fit(Rect::from_w_h(1600.0, 800.0));

        assert_eq!(viewport.content_rect.w(), 1280.0);
        assert_eq!(viewport.content_rect.h(), 800.0);
        assert_eq!(viewport.content_rect.left(), -640.0);
        assert_eq!(viewport.content_rect.right(), 640.0);
        assert_eq!(viewport.content_rect.top(), 400.0);
        assert_eq!(viewport.content_rect.bottom(), -400.0);
    }

    #[test]
    fn aspect_viewport_uses_exact_ratio_frames() {
        let viewport = AspectViewport::fit(Rect::from_w_h(1600.0, 1000.0));

        assert_eq!(viewport.content_rect.w(), 1600.0);
        assert_eq!(viewport.content_rect.h(), 1000.0);
        assert_eq!(viewport.content_rect.left(), -800.0);
        assert_eq!(viewport.content_rect.right(), 800.0);
        assert_eq!(viewport.content_rect.top(), 500.0);
        assert_eq!(viewport.content_rect.bottom(), -500.0);
    }

    #[test]
    fn demo_selection_wraps() {
        let mut model = Model::new();
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_01: Template\nInput: None"
        );

        select_next_demo(&mut model);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
        );

        select_next_demo(&mut model);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_03: Climate\nInput: A = Arrows (On)\nInput: T = Trails (Low)\nInput: C = Colors (Medium)\nInput: S = Speed (Low)"
        );

        select_next_demo(&mut model);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_01: Template\nInput: None"
        );

        select_previous_demo(&mut model);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_03: Climate\nInput: A = Arrows (On)\nInput: T = Trails (Low)\nInput: C = Colors (Medium)\nInput: S = Speed (Low)"
        );

        select_previous_demo(&mut model);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
        );
    }

    #[test]
    fn reload_keeps_current_demo() {
        let mut model = Model::new();
        select_next_demo(&mut model);

        reload_current_demo(&mut model);

        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
        );
    }

    #[test]
    fn model_can_start_on_saved_demo_index() {
        let model = Model::new_with_demo_index(1);

        assert_eq!(model.current_demo_index(), 1);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
        );
    }

    #[test]
    fn saved_demo_index_wraps_to_available_demo() {
        let model = Model::new_with_demo_index(3);

        assert_eq!(model.current_demo_index(), 0);
        assert_eq!(
            model.demo_state.render_hud().demo_hud_text,
            "Demo_01: Template\nInput: None"
        );
    }

    #[test]
    fn demo_01_declares_its_hud() {
        let state = demo_01::State::new();
        let hud = demo_01::render_hud(&state);

        assert_eq!(hud.demo_hud_text, "Demo_01: Template\nInput: None");
    }

    #[test]
    fn demo_02_declares_its_hud() {
        let state = demo_02::State::new();
        let hud = demo_02::render_hud(&state);

        assert_eq!(
            hud.demo_hud_text,
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
        );
    }

    #[test]
    fn demo_03_declares_its_hud() {
        let state = demo_03::State::new();
        let hud = demo_03::render_hud(&state);

        assert_eq!(
            hud.demo_hud_text,
            "Demo_03: Climate\nInput: A = Arrows (On)\nInput: T = Trails (Low)\nInput: C = Colors (Medium)\nInput: S = Speed (Low)"
        );
    }
}
