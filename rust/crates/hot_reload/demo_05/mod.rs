use crate::{AspectViewport, DemoHud};
use nannou::prelude::*;
use std::f32::consts::{PI, TAU};

// Receives keyboard and mouse events for this demo.
// The leading underscores mean these inputs are intentionally unused here.
pub fn window_event(_app: &App, _state: &mut State, _event: &WindowEvent) {}

// Returns the small text block shown in the demo-specific status overlay.
pub fn render_hud(_state: &State) -> DemoHud {
    DemoHud {
        demo_hud_text: "Pi Radius\nInput: None".to_string(),
    }
}

const TRACE_STEPS: usize = 720;
const MIN_COMPLEXITY: f32 = 2.0;
const MAX_COMPLEXITY: f32 = 13.0;
const COMPLEXITY_SECONDS: f32 = 34.0;
const TRACE_SECONDS: f32 = 33.0;
const PATTERN_SCALE: f32 = 0.76;
const CIRCLE_WEIGHT: f32 = 0.55;
const CIRCLE_EVERY_STEPS: usize = 9;

pub struct State;

impl State {
    pub fn new() -> Self {
        Self
    }
}

pub fn view(app: &App, _state: &State, draw: &Draw, viewport: AspectViewport) {
    draw.background().color(BLACK);

    let content = viewport.content_rect;
    let max_radius = content.w().min(content.h()) * PATTERN_SCALE * 0.5;
    let complexity = evolving_complexity(app.time);
    let visible_steps = visible_trace_steps(app.time);

    draw_orbit_circles(draw, max_radius, complexity, visible_steps);
}



fn draw_orbit_circles(draw: &Draw, max_radius: f32, complexity: f32, visible_steps: usize) {
    for step in (0..visible_steps).step_by(CIRCLE_EVERY_STEPS) {
        let angle = pi_angle(step);
        let center_radius = max_radius * (0.5 + 0.42 * radius_wave(angle, complexity));
        let circle_radius = max_radius * (0.045 + 0.115 * radius_wave(angle + PI * 0.5, complexity));
        let center = pt2(angle.cos() * center_radius, angle.sin() * center_radius);
        let alpha = 0.05 + 0.13 * step as f32 / TRACE_STEPS as f32;

        draw.ellipse()
            .xy(center)
            .radius(circle_radius)
            .no_fill()
            .stroke(rgba(1.0, 1.0, 1.0, alpha))
            .stroke_weight(CIRCLE_WEIGHT);

        draw.ellipse()
            .xy(-center)
            .radius(circle_radius)
            .no_fill()
            .stroke(rgba(1.0, 1.0, 1.0, alpha))
            .stroke_weight(CIRCLE_WEIGHT);
    }
}

fn pi_angle(step: usize) -> f32 {
    let progress = step as f32 / (TRACE_STEPS - 1) as f32;

    progress * TAU * PI
}

fn radius_wave(angle: f32, complexity: f32) -> f32 {
    let primary = (angle * complexity).cos().abs();
    let pi_modulation = (angle * PI * 0.5).sin().abs();

    (0.72 * primary + 0.28 * pi_modulation).clamp(0.0, 1.0)
}

fn evolving_complexity(time: f32) -> f32 {
    let cycle = (time / COMPLEXITY_SECONDS).sin() * 0.5 + 0.5;

    MIN_COMPLEXITY + (MAX_COMPLEXITY - MIN_COMPLEXITY) * cycle
}

fn visible_trace_steps(time: f32) -> usize {
    let cycle = (time / TRACE_SECONDS).fract();
    let eased = cycle * cycle * (3.0 - 2.0 * cycle);

    ((TRACE_STEPS as f32 * eased).round() as usize).clamp(2, TRACE_STEPS)
}
