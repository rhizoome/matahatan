use super::{maze_from_seed_and_kind, MazeSpec, SharedState, SimulationState};
use egui::{vec2, Color32, Pos2, Rect, RichText, Rounding, Shape, Stroke, Ui, Vec2};
use maze_generator::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MatahatanAppState {}

impl Default for MatahatanAppState {
    fn default() -> Self {
        Self {}
    }
}

struct MazeInfo {
    border: Rect,
    square: Vec2,
    x: f32,
    y: f32,
}

pub struct MatahatanApp {
    maze: Maze,
    maze_spec: MazeSpec,
    shared_state: Arc<Mutex<SharedState>>,
    app_state: MatahatanAppState,
}

enum FormatType {
    BigInt,
    MidFloat,
    Float,
}

impl MatahatanApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared_state: Arc<Mutex<SharedState>>) -> Self {
        let app;
        let maze_spec;
        {
            let mut state = shared_state.lock().unwrap();
            state.ctx = Some(cc.egui_ctx.clone());
            maze_spec = state.maze_spec.clone();
        }
        let spec = maze_spec.clone();
        let maze = maze_from_seed_and_kind(spec.seed, spec.kind);
        let app_state;
        if let Some(storage) = cc.storage {
            app_state = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            app_state = MatahatanAppState::default();
        }
        app = MatahatanApp {
            maze,
            maze_spec,
            shared_state,
            app_state,
        };
        app
    }

    fn update_maze(&mut self) {
        let maze_spec;
        {
            let state = self.shared_state.lock().unwrap();
            maze_spec = state.maze_spec.clone();
        }
        if self.maze_spec != maze_spec {
            self.maze = maze_from_seed_and_kind(maze_spec.seed, maze_spec.kind);
        }
    }
}

impl eframe::App for MatahatanApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.app_state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_maze();
        let simulation;
        {
            let state = self.shared_state.lock().unwrap();
            simulation = state.simulation.clone();
        }
        egui::SidePanel::right("debug view").show(ctx, |ui| {
            debug_view(ui, &simulation);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let maze_info = maze_info(ui, &self.maze);
            draw_maze(ui, &self.maze, &maze_info);
            draw_car(ui, &simulation, &maze_info);
        });
    }
}

fn debug_view(ui: &mut Ui, state: &SimulationState) {
    debug_view_row(ui, "Frame", state.frame as f64, FormatType::BigInt);
    debug_view_row(
        ui,
        "Position x",
        state.position.x as f64,
        FormatType::MidFloat,
    );
    debug_view_row(
        ui,
        "Position y",
        state.position.y as f64,
        FormatType::MidFloat,
    );
    debug_view_row(ui, "Velocity", state.velocity as f64, FormatType::Float);
    debug_view_row(
        ui,
        "Velocity (true)",
        state.velocity_v.length() as f64,
        FormatType::Float,
    );
    let angle_deg = state.angle as f64 * (180.0 / std::f64::consts::PI);
    debug_view_row(ui, "Angle (deg)", angle_deg, FormatType::MidFloat);
    debug_view_row(ui, "Angle (rad)", state.angle as f64, FormatType::Float);
    debug_view_row(
        ui,
        "Steering (input)",
        state.steering as f64,
        FormatType::Float,
    );
    debug_view_row(
        ui,
        "Acceleration (input)",
        state.acceleration as f64,
        FormatType::Float,
    );
}

fn debug_view_row(ui: &mut Ui, title: &str, value: f64, format_type: FormatType) {
    ui.label(format!("{title}:"));
    let display = match format_type {
        FormatType::BigInt => format!("{:010}", value),
        FormatType::MidFloat => format!("{:+08.3}", value),
        FormatType::Float => format!("{:+.5}", value),
    };
    ui.label(RichText::new(display).strong());
}

fn maze_info(ui: &mut Ui, maze: &Maze) -> MazeInfo {
    let size = ui.available_size();
    let top_left = Pos2 { x: 20.0, y: 15.0 };
    let bottom_right = Pos2 {
        x: size.x,
        y: size.y,
    };
    let border = Rect::from_two_pos(top_left, bottom_right);
    let y = maze.size.0 as f32;
    let x = maze.size.1 as f32;
    let square = vec2(border.width() / x as f32, border.height() / y as f32);
    MazeInfo {
        border,
        square,
        x,
        y,
    }
}

fn rotate(v: egui::Vec2, angle_rad: f32) -> egui::Vec2 {
    let cos_theta = angle_rad.cos();
    let sin_theta = angle_rad.sin();

    egui::Vec2::new(
        v.x * cos_theta - v.y * sin_theta,
        v.x * sin_theta + v.y * cos_theta,
    )
}

fn draw_car(ui: &mut Ui, simulation: &SimulationState, maze_info: &MazeInfo) {
    let stroke = Stroke::new(1.0, Color32::WHITE);
    let pos = &simulation.position;
    let square = maze_info.square;
    let v1 = vec2(0.6 * square.x, 0.0);
    let v2 = vec2(0.0, 0.15 * square.y);
    let v3 = vec2(0.0, -0.15 * square.y);
    let mut vec = vec![v1, v2, v3];
    for i in 0..vec.len() {
        vec[i] = rotate(vec[i], simulation.angle);
    }
    let mut shape = Shape::convex_polygon(
        vec![vec[0].to_pos2(), vec[1].to_pos2(), vec[2].to_pos2()],
        Color32::LIGHT_YELLOW,
        stroke,
    );
    let x = pos.x / maze_info.x;
    let y = pos.y / maze_info.y;
    let center = maze_info.border.lerp_inside(vec2(x, y)).to_vec2();
    shape.translate(center);
    ui.painter().add(shape);
}

fn draw_maze(ui: &mut Ui, maze: &Maze, maze_info: &MazeInfo) {
    let stroke = Stroke::new(1.0, Color32::WHITE);
    let shape = Shape::rect_stroke(maze_info.border, Rounding::ZERO, stroke);
    ui.painter().add(shape);
    let gx = (0.5 + maze.goal.x as f32) / maze_info.x;
    let gy = (0.5 + maze.goal.y as f32) / maze_info.y;
    let center = maze_info.border.lerp_inside(vec2(gx, gy));
    let start = Rect::from_center_size(center, maze_info.square);
    let shape = Shape::rect_filled(start, Rounding::ZERO, Color32::DARK_GREEN);
    ui.painter().add(shape);
    for ix in 0..maze_info.x as i32 {
        for iy in 0..maze_info.y as i32 {
            if let Some(field) = maze.get_field(&(ix, iy).into()) {
                let x = (ix as f32 + 0.5) / maze_info.x;
                let y = (iy as f32 + 0.5) / maze_info.y;
                let center = maze_info.border.lerp_inside(vec2(x, y));
                let square = Rect::from_center_size(center, maze_info.square);
                draw_field(ui, stroke, &square, &field);
            }
        }
    }
}

fn draw_field(ui: &mut Ui, stroke: Stroke, rect: &Rect, field: &Field) {
    if !field.has_passage(&Direction::West) {
        draw_line(ui, stroke, rect.left_top(), rect.left_bottom());
    }
    if !field.has_passage(&Direction::North) {
        draw_line(ui, stroke, rect.left_top(), rect.right_top());
    }
}

fn draw_line(ui: &mut Ui, stroke: Stroke, a: Pos2, b: Pos2) {
    let points = vec![a, b];
    let shape = Shape::line(points, stroke);
    ui.painter().add(shape);
}
