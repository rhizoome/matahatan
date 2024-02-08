use super::{maze_from_seed_and_kind, SimState};
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke, Ui, Vec2};
use maze_generator::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MatahatanApp {
    #[serde(skip)]
    maze: Option<Maze>,
    #[serde(skip)]
    shared_state: Option<Arc<Mutex<SimState>>>,
}

impl Default for MatahatanApp {
    fn default() -> Self {
        Self {
            maze: None,
            shared_state: None,
        }
    }
}

impl MatahatanApp {
    pub fn new(cc: &eframe::CreationContext<'_>, shared_state: Arc<Mutex<SimState>>) -> Self {
        let mut app;
        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        } else {
            app = MatahatanApp::default();
        }
        {
            let mut state = shared_state.lock().unwrap();
            state.ctx = Some(cc.egui_ctx.clone());
            let spec = state.maze_spec.clone();
            app.maze = Some(maze_from_seed_and_kind(spec.seed, spec.kind));
        }
        app.shared_state = Some(shared_state);
        app
    }
}

impl eframe::App for MatahatanApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(maze) = &self.maze {
                draw_maze(ui, &maze);
            }
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
