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

const DEFAULT_FRAMERATE: f32 = 25.0;
const STEERING_SCALER: f32 = 1.0;
const ACCELERATION_SCALER: f32 = 1.0;

#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
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

#[derive(Clone)]
pub struct SimuationConfig {
    framerate: f32,
    steering_scaler: f32,
    acceleration_scaler: f32,
}

impl SimuationConfig {
    pub fn new(framerate: f32) -> Self {
        SimuationConfig {
            framerate: framerate,
            steering_scaler: STEERING_SCALER / framerate,
            acceleration_scaler: ACCELERATION_SCALER / framerate,
        }
    }
}

#[derive(Clone)]
pub struct SimVec2 {
    x: f32,
    y: f32,
}

#[derive(Clone)]
pub struct SimuationState {
    frame: i64,
    position: SimVec2,
    velocity: f32,
    angle: f32,        // radian
    steering: f32,     // input
    acceleration: f32, // input
}

pub struct SharedState {
    ctx: Option<egui::Context>,
    maze_spec: MazeSpec,
    simulation: SimuationState,
    config: SimuationConfig,
}

impl SharedState {
    pub fn new(framerate: f32) -> Self {
        SharedState {
            ctx: None,
            maze_spec: MazeSpec::random(),
            simulation: SimuationState {
                frame: 0,
                position: SimVec2 { x: 0.0, y: 0.0 },
                velocity: 0.0,
                angle: 0.0,
                steering: 0.0,
                acceleration: 0.0,
            },
            config: SimuationConfig::new(framerate),
        }
    }
}

pub fn run_simulation(gui: bool) {
    let shared_state = Arc::new(Mutex::new(SharedState::new(DEFAULT_FRAMERATE)));
    let shared_state_clone = Arc::clone(&shared_state);
    let handle = thread::spawn(move || {
        simulation_loop(shared_state_clone);
    });
    if gui {
        show_maze(shared_state).unwrap();
    }
    handle.join().unwrap();
}

fn simulation_loop(shared_state: Arc<Mutex<SharedState>>) {
    let mut running = true;
    let sleep_time;
    {
        let mut state = shared_state.lock().unwrap();
        sleep_time = time::Duration::from_secs_f32(1.0 / state.config.framerate);
    }
    while running {
        {
            let mut state = shared_state.lock().unwrap();
            let config = state.config.clone();
            simulation_step(config, &mut state.simulation);
            if let Some(ctx) = &state.ctx {
                ctx.input(|s| {
                    if s.viewport().close_requested() {
                        running = false;
                    }
                });
                ctx.request_repaint();
            }
        }
        thread::sleep(sleep_time);
    }
}

fn simulation_step(config: SimuationConfig, state: &mut SimuationState) {
    state.frame += 1;
}

fn show_maze(shared_state: Arc<Mutex<SharedState>>) -> eframe::Result<()> {
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
        "Matahatan",
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
