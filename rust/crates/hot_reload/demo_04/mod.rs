use crate::{AspectViewport, DemoHud};
use nannou::prelude::*;

// Receives keyboard and mouse events for this demo.
// The leading underscores mean these inputs are intentionally unused here.
pub fn window_event(_app: &App, _state: &mut State, _event: &WindowEvent) {}

// Returns the small text block shown in the demo-specific status overlay.
pub fn render_hud(_state: &State) -> DemoHud {
    DemoHud {
        demo_hud_text: "Squares\nInput: None".to_string(),
    }
}

const GRID_COLUMNS: usize = 10;
const GRID_ROWS: usize = 10;
const GRID_SCALE: f32 = 0.72;
const COLOR_1: Rgb<u8> = BLACK;
const COLOR_2: Rgb<u8> = DARKGREY;

pub struct State {
    pub rotation_delta: f32,
    pub filled_square_index: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            rotation_delta: random_range(0.02, 0.2),
            filled_square_index: random_range(0, GRID_COLUMNS * GRID_ROWS),
        }
    }
}

pub fn view(_app: &App, _state: &State, draw: &Draw, viewport: AspectViewport) {
    draw.background().color(COLOR_1);

    let content = viewport.content_rect;
    let grid_size = content.w().min(content.h()) * GRID_SCALE;
    let square_size = grid_size / GRID_COLUMNS as f32;
    let grid_left = -grid_size * 0.5;
    let grid_top = grid_size * 0.5;

    let mut rotation: f32 = 0.0;

    for column in 0..GRID_COLUMNS {
        for row in 0..GRID_ROWS {
            let square_index = row * GRID_COLUMNS + column;

            let x = grid_left + square_size * (column as f32 + 0.5);
            let y = grid_top - square_size * (row as f32 + 0.5);

            if square_index == _state.filled_square_index {
                draw.rect()
                    .color(COLOR_2)
                    .w(square_size)
                    .h(square_size)
                    .x_y(x, y)
                    .rotate(rotation);
            }

            draw.rect()
                .no_fill()
                .stroke(COLOR_2)
                .stroke_weight(1.0)
                .w(square_size)
                .h(square_size)
                .x_y(x, y)
                .rotate(rotation);

            rotation += _state.rotation_delta;
        }
    }
}
