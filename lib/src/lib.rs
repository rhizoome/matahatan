mod app;
pub use app::MatahatanApp;
use std::sync::{Arc, Mutex};

use maze_generator::ellers_algorithm::EllersGenerator;
use maze_generator::growing_tree::GrowingTreeGenerator;
use maze_generator::prelude::*;
use maze_generator::prims_algorithm::PrimsGenerator;
use maze_generator::recursive_backtracking::RbGenerator;
use rand::Rng;
use rand::RngCore;
use std::{thread, time};

const MAZE_X: i32 = 25;
const MAZE_Y: i32 = 25;

#[derive(Clone)]
pub enum MazeKind {
    Ellers,
    Backtracking,
    GrowingTree,
    Prims,
}

impl MazeKind {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => MazeKind::Ellers,
            1 => MazeKind::Backtracking,
            2 => MazeKind::GrowingTree,
            3 => MazeKind::Prims,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct MazeSpec {
    seed: [u8; 32],
    kind: MazeKind,
}

impl MazeSpec {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        MazeSpec {
            seed: seed,
            kind: MazeKind::random(),
        }
    }
}

pub struct SimState {
    ctx: Option<egui::Context>,
    maze_spec: MazeSpec,
}

impl SimState {
    pub fn new() -> Self {
        SimState {
            ctx: None,
            maze_spec: MazeSpec::random(),
        }
    }
}

pub fn run_simulation(gui: bool) {
    let shared_state = Arc::new(Mutex::new(SimState::new()));
    let shared_state_clone = Arc::clone(&shared_state);
    let handle = thread::spawn(move || {
        simulation_loop(shared_state_clone);
    });
    if gui {
        show_maze(shared_state).unwrap();
    }
    handle.join().unwrap();
}

fn simulation_loop(shared_state: Arc<Mutex<SimState>>) {
    let mut close = false;
    let sleep_time = time::Duration::from_millis(1000);
    while !close {
        {
            let mut state = shared_state.lock().unwrap();
            state.maze_spec = MazeSpec::random();
            if let Some(ctx) = &state.ctx {
                ctx.input(|s| {
                    if s.viewport().close_requested() {
                        close = true;
                    }
                });
                ctx.request_repaint();
            }
        }
        thread::sleep(sleep_time); // TODO will be driven by rlua or stdin
    }
}

fn show_maze(shared_state: Arc<Mutex<SimState>>) -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "MatahatanApp",
        native_options,
        Box::new(|cc| Box::new(app::MatahatanApp::new(cc, shared_state))),
    )
}

pub fn maze_from_seed_and_kind(seed: [u8; 32], kind: MazeKind) -> Maze {
    match kind {
        MazeKind::Backtracking => {
            let mut generator = RbGenerator::new(Some(seed));
            generator.generate(MAZE_X, MAZE_Y).unwrap()
        }
        MazeKind::Ellers => {
            let mut generator = EllersGenerator::new(Some(seed));
            generator.generate(MAZE_X, MAZE_Y).unwrap()
        }
        MazeKind::GrowingTree => {
            let mut generator = GrowingTreeGenerator::new(Some(seed));
            generator.generate(MAZE_X, MAZE_Y).unwrap()
        }
        MazeKind::Prims => {
            let mut generator = PrimsGenerator::new(Some(seed));
            generator.generate(MAZE_X, MAZE_Y).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
