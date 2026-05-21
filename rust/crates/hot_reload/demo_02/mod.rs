use crate::{AspectViewport, DemoHud};
use nannou::prelude::*;

// Trail tuning lives together so the memory drawing can be adjusted quickly.
const MOUSE_TRAIL_LENGTH: usize = 30;
const PLAYBACK_SECONDS: f32 = 1.2;
const PLAYBACK_FIDELITY: f32 = 1.0;
const TRAIL_TAIL_WEIGHT: f32 = 2.0;
const TRAIL_HEAD_WEIGHT: f32 = 28.0;
const TRAIL_BLUR_WEIGHT: f32 = 18.0;
const TRAIL_BLUR_ALPHA: f32 = 0.16;
const SPHERE_RADIUS_SCALE: f32 = 0.58;

pub struct State {
    // Recent cursor points drive the live trail.
    cursor_positions: Vec<Point2>,
    is_recording: bool,
    // Dragged points become a replaying memory after the mouse is released.
    recording_positions: Vec<Point2>,
    playback_trails: Vec<PlaybackTrail>,
}

impl State {
    pub fn new() -> Self {
        Self {
            cursor_positions: Vec::with_capacity(MOUSE_TRAIL_LENGTH),
            is_recording: false,
            recording_positions: Vec::with_capacity(MOUSE_TRAIL_LENGTH),
            playback_trails: Vec::new(),
        }
    }
}

struct PlaybackTrail {
    points: Vec<Point2>,
    started_at: f32,
}

// Handles mouse input and keeps all recorded points inside the visible viewport.
pub fn window_event(app: &App, state: &mut State, event: &WindowEvent) {
    let viewport = AspectViewport::current(app);

    match event {
        WindowEvent::MouseMoved(position) if viewport.contains(*position) => {
            let position = viewport.clamp(*position);

            record_cursor_position(state, position);

            if state.is_recording {
                record_mouse_position(state, position);
            }
        }
        WindowEvent::MousePressed(MouseButton::Left)
            if !state.is_recording && viewport.contains(app.mouse.position()) =>
        {
            state.is_recording = true;
            state.recording_positions.clear();
            record_mouse_position(state, viewport.clamp(app.mouse.position()));
        }
        WindowEvent::MouseReleased(MouseButton::Left) if state.is_recording => {
            state.is_recording = false;

            if state.recording_positions.len() > 1 {
                state.playback_trails.push(PlaybackTrail {
                    points: std::mem::take(&mut state.recording_positions),
                    started_at: app.time,
                });
                state.recording_positions = Vec::with_capacity(MOUSE_TRAIL_LENGTH);
            } else {
                state.recording_positions.clear();
            }
        }
        WindowEvent::MousePressed(MouseButton::Right)
            if viewport.contains(app.mouse.position()) =>
        {
            state.playback_trails.clear();
            state.recording_positions.clear();

            if state.is_recording {
                record_mouse_position(state, viewport.clamp(app.mouse.position()));
            }
        }
        _ => {}
    }
}

pub fn render_hud(_state: &State) -> DemoHud {
    DemoHud {
        demo_hud_text:
            "Demo_02: Memories\nInput:\n• Left-mouse = Create Memory\n• Right-mouse = Clear Memory"
                .to_string(),
    }
}

// Draws the background, two silhouettes, and the live/replayed memory trails.
pub fn view(app: &App, state: &State, draw: &Draw, viewport: AspectViewport) {
    let content = viewport.content_rect;
    let radius = content.w().min(content.h()) * 0.2;
    let center_offset = vec2(content.w() * 0.06, content.h() * 0.08);

    draw.rect()
        .xy(content.xy())
        .wh(content.wh())
        .color(rgb(0.96, 0.96, 0.95));
    draw_black_sphere_silhouette(draw, pt2(0.0, 0.0), radius);
    draw_black_sphere_silhouette(draw, pt2(center_offset.x, center_offset.y), radius);
    draw_mouse_trails(draw, state, app.time);
}

fn record_mouse_position(state: &mut State, position: Point2) {
    state.recording_positions.push(position);
    trim_positions(&mut state.recording_positions);
}

fn record_cursor_position(state: &mut State, position: Point2) {
    state.cursor_positions.push(position);
    trim_positions(&mut state.cursor_positions);
}

fn trim_positions(positions: &mut Vec<Point2>) {
    if positions.len() > MOUSE_TRAIL_LENGTH {
        // Drop oldest points first so the trail stays a fixed, recent length.
        positions.drain(0..positions.len() - MOUSE_TRAIL_LENGTH);
    }
}

// The silhouette is flat black with a soft offset shadow for depth.
fn draw_black_sphere_silhouette(draw: &Draw, position: Point2, radius: f32) {
    draw.ellipse()
        .xy(position + vec2(radius * 0.08, -radius * 0.1))
        .radius(radius * 1.05)
        .color(rgba(0.0, 0.0, 0.0, 0.18));

    draw.ellipse().xy(position).radius(radius).color(BLACK);
}

fn draw_mouse_trails(draw: &Draw, state: &State, time: f32) {
    for trail in &state.playback_trails {
        // rem_euclid loops each saved trail smoothly from 0.0 back to 1.0.
        let progress = ((time - trail.started_at) / PLAYBACK_SECONDS).rem_euclid(1.0);
        let positions = playback_positions(&trail.points, progress, PLAYBACK_FIDELITY);
        draw_trail(draw, &positions, 0.9);
    }

    draw_trail(draw, &state.cursor_positions, 0.45);
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

    // Layered ellipses create a small glossy bead without using textures.
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

    // Convert normalized playback progress into a point between two samples.
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

    // Average each interior point with its neighbors to soften jitter.
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
