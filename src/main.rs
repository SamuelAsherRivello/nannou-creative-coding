use nannou::prelude::*;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct Model;

fn main() {
    nannou::app(model).run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("nannou-creative-coding")
        .view(view)
        .build()
        .expect("failed to build nannou window");

    center_window(app, window_id);

    Model
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
    let x = monitor_position.x + (monitor_size.width as i32 - WINDOW_WIDTH as i32) / 2;
    let y = monitor_position.y + (monitor_size.height as i32 - WINDOW_HEIGHT as i32) / 2;

    window.set_outer_position_pixels(x, y);
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(rgb(0.96, 0.96, 0.95));
    draw.ellipse().x_y(0.0, 0.0).radius(120.0).color(BLACK);

    draw.to_frame(app, &frame).expect("failed to draw frame");
}
