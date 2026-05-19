use nannou::prelude::*;
use std::time::Instant;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;
const FPS_DISPLAY_UPDATE_SECONDS: f32 = 2.0;
const MOUSE_TRAIL_LENGTH: usize = 30;
const PLAYBACK_SECONDS: f32 = 1.2;
const PLAYBACK_FIDELITY: f32 = 1.0;
const TRAIL_TAIL_WEIGHT: f32 = 2.0;
const TRAIL_HEAD_WEIGHT: f32 = 28.0;
const TRAIL_BLUR_WEIGHT: f32 = 18.0;
const TRAIL_BLUR_ALPHA: f32 = 0.16;
const SPHERE_RADIUS_SCALE: f32 = 0.58;
const STATUS_MARGIN: f32 = 18.0;
const STATUS_WIDTH: f32 = 330.0;
const STATUS_HEIGHT: f32 = 126.0;
const STATUS_TEXT_SIZE: u32 = 15;

pub struct Model {
    pub last_window_state_save: Instant,
    cursor_positions: Vec<Point2>,
    displayed_fps: f32,
    fps_accumulator: f32,
    fps_sample_count: u32,
    last_fps_display_update: f32,
    is_recording: bool,
    recording_positions: Vec<Point2>,
    playback_trails: Vec<PlaybackTrail>,
    pub was_updated: bool,
}

impl Model {
    pub fn new() -> Self {
        Self {
            last_window_state_save: Instant::now(),
            cursor_positions: Vec::with_capacity(MOUSE_TRAIL_LENGTH),
            displayed_fps: 120.0,
            fps_accumulator: 0.0,
            fps_sample_count: 0,
            last_fps_display_update: 0.0,
            is_recording: false,
            recording_positions: Vec::with_capacity(MOUSE_TRAIL_LENGTH),
            playback_trails: Vec::new(),
            was_updated: false,
        }
    }
}

struct PlaybackTrail {
    points: Vec<Point2>,
    started_at: f32,
}

#[no_mangle]
pub fn window_event(app: &App, model: &mut Model, event: &WindowEvent) {
    match event {
        WindowEvent::MouseMoved(position) => {
            record_cursor_position(model, *position);

            if model.is_recording {
                record_mouse_position(model, *position);
            }
        }
        WindowEvent::MousePressed(MouseButton::Left) if !model.is_recording => {
            model.is_recording = true;
            model.recording_positions.clear();
            record_mouse_position(model, app.mouse.position());
        }
        WindowEvent::MouseReleased(MouseButton::Left) if model.is_recording => {
            model.is_recording = false;

            if model.recording_positions.len() > 1 {
                model.playback_trails.push(PlaybackTrail {
                    points: std::mem::take(&mut model.recording_positions),
                    started_at: app.time,
                });
                model.recording_positions = Vec::with_capacity(MOUSE_TRAIL_LENGTH);
            } else {
                model.recording_positions.clear();
            }
        }
        WindowEvent::MousePressed(MouseButton::Right) => {
            model.playback_trails.clear();
            model.recording_positions.clear();

            if model.is_recording {
                record_mouse_position(model, app.mouse.position());
            }
        }
        _ => {}
    }
}

fn record_mouse_position(model: &mut Model, position: Point2) {
    model.recording_positions.push(position);
    trim_positions(&mut model.recording_positions);
}

fn record_cursor_position(model: &mut Model, position: Point2) {
    model.cursor_positions.push(position);
    trim_positions(&mut model.cursor_positions);
}

fn trim_positions(positions: &mut Vec<Point2>) {
    if positions.len() > MOUSE_TRAIL_LENGTH {
        positions.drain(0..positions.len() - MOUSE_TRAIL_LENGTH);
    }
}

#[no_mangle]
pub fn update(app: &App, model: &mut Model, _update: Update) {
    update_displayed_fps(app, model);
    model.was_updated = false;
}

fn update_displayed_fps(app: &App, model: &mut Model) {
    model.fps_accumulator += app.fps();
    model.fps_sample_count += 1;

    let elapsed = app.time - model.last_fps_display_update;
    if elapsed < FPS_DISPLAY_UPDATE_SECONDS || model.fps_sample_count == 0 {
        return;
    }

    let average = model.fps_accumulator / model.fps_sample_count as f32;
    model.displayed_fps = round_to_half(average);
    model.fps_accumulator = 0.0;
    model.fps_sample_count = 0;
    model.last_fps_display_update = app.time;
}

fn round_to_half(value: f32) -> f32 {
    (value * 2.0).round() / 2.0
}

#[no_mangle]
pub fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb(0.96, 0.96, 0.95));
    draw_black_sphere_silhouette(&draw, pt2(0.0, 0.0), 120.0);
    draw_black_sphere_silhouette(&draw, pt2(50.0, 150.1), 120.0);

    draw_mouse_trails(&draw, _model, app.time);
    draw_status_overlay(app, _model, &draw);

    draw.to_frame(app, &frame).expect("failed to draw frame");
}

fn draw_status_overlay(app: &App, model: &Model, draw: &Draw) {
    let window = app.window_rect();
    let center = pt2(
        window.left() + STATUS_MARGIN + STATUS_WIDTH * 0.5,
        window.top() - STATUS_MARGIN - STATUS_HEIGHT * 0.5,
    );
    let text = format!(
        "FPS {:.1}\nMemories: {:03}\n* Hold Left Mouse = Start Memory\n* Release Left Mouse = End Memory\n* Right Mouse = Clear Memories",
        model.displayed_fps,
        model.playback_trails.len()
    );

    draw.rect()
        .xy(center)
        .w_h(STATUS_WIDTH, STATUS_HEIGHT)
        .color(rgba(0.0, 0.0, 0.0, 0.86));

    draw.text(&text)
        .xy(center)
        .w_h(STATUS_WIDTH - 12.0, STATUS_HEIGHT - 8.0)
        .left_justify()
        .align_text_top()
        .font_size(STATUS_TEXT_SIZE)
        .color(WHITE);
}

fn draw_black_sphere_silhouette(draw: &Draw, position: Point2, radius: f32) {
    draw.ellipse()
        .xy(position + vec2(radius * 0.08, -radius * 0.1))
        .radius(radius * 1.05)
        .color(rgba(0.0, 0.0, 0.0, 0.18));

    draw.ellipse().xy(position).radius(radius).color(BLACK);
}

fn draw_mouse_trails(draw: &Draw, model: &Model, time: f32) {
    for trail in &model.playback_trails {
        let progress = ((time - trail.started_at) / PLAYBACK_SECONDS).rem_euclid(1.0);
        let positions = playback_positions(&trail.points, progress, PLAYBACK_FIDELITY);
        draw_trail(draw, &positions, 0.9);
    }

    draw_trail(draw, &model.cursor_positions, 0.45);
}

fn draw_trail(draw: &Draw, positions: &[Point2], max_alpha: f32) {
    let positions = smooth_positions(positions);
    let segment_count = positions.len().saturating_sub(1);

    for (index, position) in positions.iter().enumerate() {
        let weight = trail_weight(index, segment_count);
        let alpha = trail_alpha(index, segment_count) * max_alpha;
        let radius = weight * SPHERE_RADIUS_SCALE;
        draw_sphere_bead(draw, *position, radius, alpha);
    }
}

fn draw_sphere_bead(draw: &Draw, position: Point2, radius: f32, alpha: f32) {
    let radius = radius.max(1.5);
    let shadow_offset = vec2(radius * 0.22, -radius * 0.28);
    let highlight_offset = vec2(-radius * 0.34, radius * 0.36);

    draw.ellipse()
        .xy(position + shadow_offset)
        .radius(radius * 1.35)
        .color(rgba(0.0, 0.0, 0.0, alpha * TRAIL_BLUR_ALPHA * 0.42));

    draw.ellipse()
        .xy(position)
        .radius(radius + TRAIL_BLUR_WEIGHT * 0.23)
        .color(rgba(0.05, 0.18, 0.22, alpha * TRAIL_BLUR_ALPHA));

    draw.ellipse()
        .xy(position)
        .radius(radius)
        .color(rgba(0.02, 0.08, 0.1, alpha));

    draw.ellipse()
        .xy(position + highlight_offset * 0.18)
        .w_h(radius * 1.28, radius * 1.12)
        .color(rgba(0.1, 0.32, 0.36, alpha * 0.68));

    draw.ellipse()
        .xy(position + highlight_offset)
        .w_h(radius * 0.42, radius * 0.32)
        .color(rgba(0.82, 0.96, 0.93, alpha * 0.78));
}

fn playback_positions(positions: &[Point2], progress: f32, fidelity: f32) -> Vec<Point2> {
    if positions.len() <= 1 {
        return positions.to_vec();
    }

    let fidelity = fidelity.clamp(0.05, 1.0);
    let stride = (1.0 / fidelity).round().max(1.0) as usize;
    let segment_progress = progress * (positions.len() - 1) as f32;
    let segment_index = segment_progress.floor() as usize;
    let segment_t = segment_progress.fract();
    let last_whole_index = segment_index.min(positions.len() - 1);
    let mut playback = Vec::with_capacity(MOUSE_TRAIL_LENGTH);

    for index in (0..=last_whole_index).step_by(stride) {
        playback.push(positions[index]);
    }

    if playback.last().copied() != Some(positions[last_whole_index]) {
        playback.push(positions[last_whole_index]);
    }

    if last_whole_index < positions.len() - 1 {
        let start = positions[last_whole_index];
        let end = positions[last_whole_index + 1];
        playback.push(pt2(
            start.x + (end.x - start.x) * segment_t,
            start.y + (end.y - start.y) * segment_t,
        ));
    }

    playback
}

fn smooth_positions(positions: &[Point2]) -> Vec<Point2> {
    if positions.len() < 3 {
        return positions.to_vec();
    }

    positions
        .iter()
        .enumerate()
        .map(|(index, position)| {
            if index == 0 || index == positions.len() - 1 {
                *position
            } else {
                let previous = positions[index - 1];
                let next = positions[index + 1];
                pt2(
                    previous.x * 0.2 + position.x * 0.6 + next.x * 0.2,
                    previous.y * 0.2 + position.y * 0.6 + next.y * 0.2,
                )
            }
        })
        .collect()
}

fn trail_weight(index: usize, segment_count: usize) -> f32 {
    if segment_count == 0 {
        return TRAIL_HEAD_WEIGHT;
    }

    let t = index as f32 / segment_count as f32;
    TRAIL_TAIL_WEIGHT + (TRAIL_HEAD_WEIGHT - TRAIL_TAIL_WEIGHT) * t.powf(1.65)
}

fn trail_alpha(index: usize, segment_count: usize) -> f32 {
    if segment_count == 0 {
        return 1.0;
    }

    let t = index as f32 / segment_count as f32;
    0.18 + 0.72 * t
}
