use crate::{AspectViewport, DemoHud};
use nannou::prelude::*;

// These constants keep the drawing choices easy to find and tweak.
const BACKGROUND_GRAY: f32 = 0.12;
const CUBE_SIZE: f32 = 170.0;
const TITLE_TEXT: &str = "Text & Square";
const TITLE_FONT_SIZE: u32 = 36;
const TITLE_GAP: f32 = 54.0;

// Demo state stores values that need to survive from frame to frame.
// This first demo is static, so it does not need any fields yet.
pub struct State;

impl State {
    // Creates a fresh copy of the demo when the app starts or reloads.
    pub fn new() -> Self {
        Self
    }
}

// Receives keyboard and mouse events for this demo.
// The leading underscores mean these inputs are intentionally unused here.
pub fn window_event(_app: &App, _state: &mut State, _event: &WindowEvent) {}

// Returns the small text block shown in the demo-specific status overlay.
pub fn render_hud(_state: &State) -> DemoHud {
    DemoHud {
        demo_hud_text: "Template\nInput: None".to_string(),
    }
}

// Draws one frame of the demo.
// nannou calls this every time the window needs to be redrawn.
pub fn view(_app: &App, _state: &State, draw: &Draw, viewport: AspectViewport) {
    // Fill only the fixed-aspect content area, leaving any letterbox bars black.
    draw.rect()
        .xy(viewport.content_rect.xy())
        .wh(viewport.content_rect.wh())
        .color(rgb(BACKGROUND_GRAY, BACKGROUND_GRAY, BACKGROUND_GRAY));

    // Draw the white square at the center of the nannou coordinate system.
    draw.rect()
        .xy(pt2(0.0, 0.0))
        .w_h(CUBE_SIZE, CUBE_SIZE)
        .color(WHITE);

    // Place the label above the square by offsetting from half the square size.
    draw.text(TITLE_TEXT)
        .xy(pt2(0.0, CUBE_SIZE * 0.5 + TITLE_GAP))
        .font_size(TITLE_FONT_SIZE)
        .color(WHITE);
}
