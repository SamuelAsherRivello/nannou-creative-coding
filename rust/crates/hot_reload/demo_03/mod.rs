use crate::{AspectViewport, DemoHud};
use nannou::prelude::*;
use std::f32::consts::TAU;

const COLOR_RESOLUTION_X: usize = 125;
const COLOR_RESOLUTION_Y: usize = 80;
const BASE_ARROW_COLUMNS: usize = 17;
const BASE_ARROW_ROWS: usize = 10;
const ARROW_DENSITY_MULTIPLIER: usize = 5;
const ARROW_BASE_LENGTH: f32 = 18.0;
const ARROW_LENGTH_RANGE: f32 = 34.0;
const ARROW_RELATIVE_POWER_SCALE: f32 = 0.1;
const ARROW_WEIGHT: f32 = 1.25;
const ARROW_HEAD_LENGTH: f32 = 7.0;
const ARROW_HEAD_SPREAD: f32 = 0.5;
const TRAIL_SPACING_UNITS: f32 = 32.0;
const TRAIL_STEPS: usize = 7;
const TRAIL_STEP_DISTANCE: f32 = 7.0;
const TRAIL_SPEED: f32 = 1.15;
const TRAIL_HEAD_ADVECTION_STEPS: usize = 42;
const TRAIL_MIN_LIFETIME_SECONDS: f32 = 0.01;
const TRAIL_MAX_LIFETIME_SECONDS: f32 = 0.22;
const TRAIL_SEGMENT_SECONDS: f32 = 0.026;

pub struct State {
    show_arrows: bool,
    trail_density: DensityLevel,
    color_resolution: DensityLevel,
    trail_speed: DensityLevel,
}

impl State {
    pub fn new() -> Self {
        Self {
            show_arrows: true,
            trail_density: DensityLevel::Low,
            color_resolution: DensityLevel::Medium,
            trail_speed: DensityLevel::Low,
        }
    }
}

pub fn window_event(_app: &App, state: &mut State, event: &WindowEvent) {
    match event {
        WindowEvent::KeyPressed(Key::A) => state.show_arrows = !state.show_arrows,
        WindowEvent::KeyPressed(Key::T) => state.trail_density = state.trail_density.next(),
        WindowEvent::KeyPressed(Key::C) => state.color_resolution = state.color_resolution.next(),
        WindowEvent::KeyPressed(Key::S) => state.trail_speed = state.trail_speed.next(),
        _ => {}
    }
}

pub fn render_hud(state: &State) -> DemoHud {
    DemoHud {
        demo_hud_text: format!(
            "Demo_03: Climate\nInput: A = Arrows ({})\nInput: T = Trails ({})\nInput: C = Colors ({})\nInput: S = Speed ({})",
            if state.show_arrows { "On" } else { "Off" },
            state.trail_density.label(),
            state.color_resolution.label(),
            state.trail_speed.label()
        ),
    }
}

pub fn view(app: &App, state: &State, draw: &Draw, viewport: AspectViewport) {
    let content = viewport.content_rect;

    draw_speed_field(draw, content, state.color_resolution);
    draw_white_trails(
        draw,
        content,
        app.time,
        state.trail_density,
        state.trail_speed,
    );

    if state.show_arrows {
        draw_arrow_grid(draw, content);
    }
}

fn draw_speed_field(draw: &Draw, content: Rect, resolution: DensityLevel) {
    let (resolution_x, resolution_y) = color_resolution(resolution);
    let cell_width = content.w() / resolution_x as f32;
    let cell_height = content.h() / resolution_y as f32;

    for row in 0..resolution_y {
        for column in 0..resolution_x {
            let u = (column as f32 + 0.5) / resolution_x as f32;
            let v = (row as f32 + 0.5) / resolution_y as f32;
            let position = point_from_uv(content, u, v);
            let sample = sample_vector_field(u, v);

            draw.rect()
                .xy(position)
                .w_h(cell_width + 1.0, cell_height + 1.0)
                .color(speed_color(sample.magnitude));
        }
    }
}

fn draw_arrow_grid(draw: &Draw, content: Rect) {
    let arrow_columns = arrow_columns();
    let arrow_rows = arrow_rows();

    for row in 0..arrow_rows {
        for column in 0..arrow_columns {
            let u = column as f32 / (arrow_columns - 1) as f32;
            let v = row as f32 / (arrow_rows - 1) as f32;
            let position = point_from_uv(content, u, v);
            let sample = sample_vector_field(u, v);
            let arrow_power = sample.magnitude * ARROW_RELATIVE_POWER_SCALE;
            let length = ARROW_BASE_LENGTH + ARROW_LENGTH_RANGE * arrow_power;

            draw_arrow(draw, position, sample.direction, length, arrow_power);
        }
    }
}

fn draw_arrow(draw: &Draw, center: Point2, direction: Vec2, length: f32, magnitude: f32) {
    let shaft_start = center - direction * (length * 0.5);
    let shaft_end = center + direction * (length * 0.5);
    let head_left = rotate_vector(-direction, ARROW_HEAD_SPREAD);
    let head_right = rotate_vector(-direction, -ARROW_HEAD_SPREAD);
    let alpha = 0.3 + magnitude * 0.45;
    let color = rgba(0.02, 0.08, 0.1, alpha);

    draw.line()
        .start(shaft_start)
        .end(shaft_end)
        .weight(ARROW_WEIGHT + magnitude * 1.1)
        .color(color);

    draw.line()
        .start(shaft_end)
        .end(shaft_end + head_left * ARROW_HEAD_LENGTH)
        .weight(ARROW_WEIGHT + magnitude * 1.1)
        .color(color);

    draw.line()
        .start(shaft_end)
        .end(shaft_end + head_right * ARROW_HEAD_LENGTH)
        .weight(ARROW_WEIGHT + magnitude * 1.1)
        .color(color);
}

fn draw_white_trails(
    draw: &Draw,
    content: Rect,
    time: f32,
    density: DensityLevel,
    speed: DensityLevel,
) {
    let columns = trail_columns_for(content, density);
    let rows = trail_rows_for(content, density);

    for row in 0..rows {
        for column in 0..columns {
            let index = row * columns + column;
            let trail_count = rows * columns;
            let seed = index as f32 / trail_count as f32;
            let jitter = trail_jitter(seed);
            let base_u = ((column as f32 + 0.5 + jitter.x) / columns as f32).rem_euclid(1.0);
            let base_v = ((row as f32 + 0.5 + jitter.y) / rows as f32).rem_euclid(1.0);

            draw_white_trail(draw, content, base_u, base_v, seed, time, speed);
        }
    }
}

fn draw_white_trail(
    draw: &Draw,
    content: Rect,
    base_u: f32,
    base_v: f32,
    seed: f32,
    time: f32,
    speed: DensityLevel,
) {
    let (mut u, mut v) = advected_trail_head(base_u, base_v, seed, time, speed);

    for step in 0..TRAIL_STEPS {
        let sample = sample_vector_field(u, v);
        let lifetime = trail_lifetime_seconds(sample.magnitude);
        let age = step as f32 * TRAIL_SEGMENT_SECONDS;
        let alpha =
            ((1.0 - age / lifetime).clamp(0.0, 1.0)).powf(1.8) * (0.28 + sample.magnitude * 0.62);
        let current = point_from_uv(content, u, v);
        let step_uv = trail_step_uv(sample, content);
        let next_u = (u - step_uv.x).rem_euclid(1.0);
        let next_v = (v - step_uv.y).rem_euclid(1.0);
        let next = point_from_uv(content, next_u, next_v);

        if !crosses_wrap(u, v, next_u, next_v) {
            draw.line()
                .start(current)
                .end(next)
                .weight(0.5 + sample.magnitude * 1.45)
                .color(rgba(1.0, 1.0, 1.0, alpha));
        }

        u = next_u;
        v = next_v;
    }
}

fn arrow_columns() -> usize {
    scaled_grid_axis(BASE_ARROW_COLUMNS, ARROW_DENSITY_MULTIPLIER)
}

fn arrow_rows() -> usize {
    scaled_grid_axis(BASE_ARROW_ROWS, ARROW_DENSITY_MULTIPLIER)
}

fn scaled_grid_axis(base: usize, density_multiplier: usize) -> usize {
    let axis_multiplier = (density_multiplier as f32).sqrt();

    ((base - 1) as f32 * axis_multiplier).round() as usize + 1
}

fn trail_columns_for(content: Rect, density: DensityLevel) -> usize {
    let spacing = trail_spacing_units(density);

    (content.w().max(spacing) / spacing).round().max(1.0) as usize
}

fn trail_rows_for(content: Rect, density: DensityLevel) -> usize {
    let spacing = trail_spacing_units(density);

    (content.h().max(spacing) / spacing).round().max(1.0) as usize
}

#[derive(Clone, Copy)]
enum DensityLevel {
    Low,
    Medium,
    High,
}

impl DensityLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        }
    }
}

fn trail_spacing_units(density: DensityLevel) -> f32 {
    match density {
        DensityLevel::Low => TRAIL_SPACING_UNITS,
        DensityLevel::Medium => TRAIL_SPACING_UNITS * 0.75,
        DensityLevel::High => TRAIL_SPACING_UNITS * 0.5,
    }
}

fn color_resolution(resolution: DensityLevel) -> (usize, usize) {
    match resolution {
        DensityLevel::Low => (COLOR_RESOLUTION_X / 4, COLOR_RESOLUTION_Y / 4),
        DensityLevel::Medium => (COLOR_RESOLUTION_X / 2, COLOR_RESOLUTION_Y / 2),
        DensityLevel::High => (COLOR_RESOLUTION_X, COLOR_RESOLUTION_Y),
    }
}

fn trail_speed_multiplier(speed: DensityLevel) -> f32 {
    match speed {
        DensityLevel::Low => 0.1,
        DensityLevel::Medium => 0.16,
        DensityLevel::High => 0.235,
    }
}

fn advected_trail_head(
    mut u: f32,
    mut v: f32,
    seed: f32,
    time: f32,
    speed: DensityLevel,
) -> (f32, f32) {
    let speed_multiplier = trail_speed_multiplier(speed);
    let travel = (time * TRAIL_SPEED * speed_multiplier + seed * 9.0).rem_euclid(1.0);
    let advection_steps =
        (TRAIL_HEAD_ADVECTION_STEPS as f32 * (0.35 + travel * 0.65)).round() as usize;

    for step in 0..advection_steps {
        let sample = sample_vector_field(u, v);
        let phase = step as f32 * 0.37 + seed * TAU;
        let liquid_wobble = rotate_vector(sample.direction, 0.24 * phase.sin()) * 0.0025;
        let velocity = sample.direction * (0.006 + sample.magnitude * 0.018) * speed_multiplier
            + liquid_wobble;

        u = (u + velocity.x).rem_euclid(1.0);
        v = (v + velocity.y).rem_euclid(1.0);
    }

    (u, v)
}

fn trail_step_uv(sample: FieldSample, content: Rect) -> Vec2 {
    let pixel_step = TRAIL_STEP_DISTANCE * (0.45 + sample.magnitude * 0.95);

    vec2(
        sample.direction.x * pixel_step / content.w(),
        sample.direction.y * pixel_step / content.h(),
    )
}

fn trail_jitter(seed: f32) -> Vec2 {
    vec2(
        0.28 * (seed * TAU * 37.0).sin(),
        0.28 * (seed * TAU * 53.0).cos(),
    )
}

fn crosses_wrap(u: f32, v: f32, next_u: f32, next_v: f32) -> bool {
    (u - next_u).abs() > 0.25 || (v - next_v).abs() > 0.25
}

fn trail_lifetime_seconds(magnitude: f32) -> f32 {
    lerp(
        TRAIL_MIN_LIFETIME_SECONDS,
        TRAIL_MAX_LIFETIME_SECONDS,
        magnitude.clamp(0.0, 1.0),
    )
}

#[derive(Clone, Copy)]
struct FieldSample {
    direction: Vec2,
    magnitude: f32,
}

fn sample_vector_field(u: f32, v: f32) -> FieldSample {
    let jet = gaussian(v, 0.58 + 0.16 * (u * TAU * 1.15).sin(), 0.13) * gaussian(u, 0.48, 0.34);
    let south_band = gaussian(v, 0.24 + 0.08 * (u * TAU * 1.8).cos(), 0.12) * 0.58;
    let quiet_pocket = gaussian(u, 0.62, 0.12) * gaussian(v, 0.33, 0.11) * 0.62;
    let curl = vec2(
        0.38 * (v * TAU * 2.0 + u * 1.7).sin(),
        0.34 * (u * TAU * 1.6 - v * 2.2).cos(),
    );
    let flow = vec2(0.88 + jet * 1.15 + south_band * 0.55, 0.1 + curl.y) + curl;
    let direction = flow.normalize_or_zero();
    let magnitude = (0.18 + jet * 0.82 + south_band * 0.38 - quiet_pocket).clamp(0.0, 1.0);

    FieldSample {
        direction,
        magnitude,
    }
}

fn speed_color(magnitude: f32) -> Rgba<f32> {
    let stops = [
        (0.0, (0.08, 0.28, 0.72)),
        (0.22, (0.0, 0.55, 0.7)),
        (0.43, (0.12, 0.62, 0.26)),
        (0.62, (0.9, 0.68, 0.24)),
        (0.78, (0.93, 0.38, 0.22)),
        (0.92, (0.78, 0.2, 0.52)),
        (1.0, (0.55, 0.12, 0.44)),
    ];

    for window in stops.windows(2) {
        let (start_t, start_color) = window[0];
        let (end_t, end_color) = window[1];

        if magnitude <= end_t {
            let local_t = ((magnitude - start_t) / (end_t - start_t)).clamp(0.0, 1.0);
            return rgba(
                lerp(start_color.0, end_color.0, local_t),
                lerp(start_color.1, end_color.1, local_t),
                lerp(start_color.2, end_color.2, local_t),
                1.0,
            );
        }
    }

    rgba(0.55, 0.12, 0.44, 1.0)
}

fn point_from_uv(content: Rect, u: f32, v: f32) -> Point2 {
    pt2(
        content.left() + content.w() * u,
        content.bottom() + content.h() * v,
    )
}

fn gaussian(value: f32, center: f32, width: f32) -> f32 {
    let distance = value - center;
    (-distance * distance / (2.0 * width * width)).exp()
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + (end - start) * amount
}

fn rotate_vector(vector: Vec2, angle: f32) -> Vec2 {
    vec2(
        vector.x * angle.cos() - vector.y * angle.sin(),
        vector.x * angle.sin() + vector.y * angle.cos(),
    )
}
