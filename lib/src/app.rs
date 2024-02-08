use super::{maze_from_seed_and_kind, MazeSpec, SimState};
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke, Ui, Vec2};
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
pub struct MatahatanApp {
    maze: Maze,
    maze_spec: MazeSpec,
    shared_state: Arc<Mutex<SimState>>,
    app_state: MatahatanAppState,
}

impl MatahatanApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared_state: Arc<Mutex<SimState>>) -> Self {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            draw_maze(ui, &self.maze);
        });
    }
}

fn draw_maze(ui: &mut Ui, maze: &Maze) {
    let size = ui.available_size();
    let top_left = Pos2 { x: 20.0, y: 15.0 };
    let bottom_right = Pos2 {
        x: size.x,
        y: size.y,
    };
    let border = Rect::from_two_pos(top_left, bottom_right);
    let border_size = border.size();
    let stroke = Stroke::new(1.0, Color32::WHITE);
    let shape = Shape::rect_stroke(border, Rounding::ZERO, stroke);
    ui.painter().add(shape);
    let maze_size_y = maze.size.0;
    let maze_size_x = maze.size.1;
    let square_size = Vec2::new(
        border.width() / maze_size_x as f32,
        border.height() / maze_size_y as f32,
    );
    for ix in 0..maze_size_x {
        for iy in 0..maze_size_y {
            if let Some(field) = maze.get_field(&(ix, iy).into()) {
                let x = border_size.x / maze_size_x as f32 * ix as f32
                    + square_size.x / 2.0
                    + top_left.x;
                let y = border_size.y / maze_size_y as f32 * iy as f32
                    + square_size.y / 2.0
                    + top_left.y;
                let center = Pos2::new(x, y);
                let square = Rect::from_center_size(center, square_size);
                draw_field(ui, stroke, &square, &field);
            }
        }
    }
}

fn draw_field(ui: &mut Ui, stroke: Stroke, rect: &Rect, field: &Field) {
    if !field.has_passage(&Direction::West) {
        draw_line(ui, stroke, rect.left_top(), rect.left_bottom());
    }
    if !field.has_passage(&Direction::East) {
        draw_line(ui, stroke, rect.right_top(), rect.right_bottom());
    }
    if !field.has_passage(&Direction::North) {
        draw_line(ui, stroke, rect.left_top(), rect.right_top());
    }
    if !field.has_passage(&Direction::South) {
        draw_line(ui, stroke, rect.left_bottom(), rect.right_bottom());
    }
}

fn draw_line(ui: &mut Ui, stroke: Stroke, a: Pos2, b: Pos2) {
    let points = vec![a, b];
    let shape = Shape::line(points, stroke);
    ui.painter().add(shape);
}
