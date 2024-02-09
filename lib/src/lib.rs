mod app;
pub use app::MatahatanApp;
use std::sync::{Arc, Mutex};

use egui::{vec2, Vec2};
use gamepads::Gamepads;
use maze_generator::ellers_algorithm::EllersGenerator;
use maze_generator::growing_tree::GrowingTreeGenerator;
use maze_generator::prelude::*;
use maze_generator::prims_algorithm::PrimsGenerator;
use maze_generator::recursive_backtracking::RbGenerator;
use ncollide2d::bounding_volume::HasBoundingVolume;
use ncollide2d::math::{Isometry, Point, Vector};
use ncollide2d::pipeline::object::{CollisionGroups, GeometricQueryType};
use ncollide2d::query::{contact, PointQuery, Ray};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use rand::Rng;
use rand::RngCore;
use std::{thread, time};

const MAZE_X: i32 = 25;
const MAZE_Y: i32 = 25;
const PI: f32 = std::f32::consts::PI;

const STEERING_SCALER: f32 = 0.4;
const ACCELERATION_SCALER: f32 = 0.01;

#[derive(Clone)]
pub struct Config {
    pub gui: bool,
    pub stick: bool,
    pub framerate: f32,
}

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
pub struct SimulationConfig {
    framerate: f32,
    steering_scaler: f32,
    acceleration_scaler: f32,
    zero: Vec2,
    size: Vec2,
    human: bool,
}

impl SimulationConfig {
    pub fn new(config: &Config, size: Vec2) -> Self {
        SimulationConfig {
            framerate: config.framerate,
            steering_scaler: STEERING_SCALER,
            acceleration_scaler: ACCELERATION_SCALER,
            zero: vec2(0.0, 0.0),
            size,
            human: config.stick,
        }
    }
}

#[derive(Clone)]
pub struct SimulationState {
    frame: i64,
    position: Vec2,
    velocity: f32,
    velocity_v: Vec2,
    angle_v: Vec2,
    angle: f32,        // radian
    steering: f32,     // input
    acceleration: f32, // input
}

pub struct LocalState {
    gamepads: Option<Gamepads>,
    maze: Maze,
    shared_state: Arc<Mutex<SharedState>>,
    world: CollisionWorld<f32, ()>,
    cuboid: Cuboid<f32>,
    cuboid2: Cuboid<f32>,
    wall: Cuboid<f32>,
    active: CollisionGroups,
    passive: CollisionGroups,
    query_type: GeometricQueryType<f32>,
}

impl LocalState {
    pub fn new(config: &Config, maze: Maze, shared_state: Arc<Mutex<SharedState>>) -> Self {
        let gamepads;
        if config.stick {
            gamepads = Some(Gamepads::new());
        } else {
            gamepads = None;
        }
        let mut active = CollisionGroups::new();
        active.set_membership(&[1]);
        let mut passive = CollisionGroups::new();
        passive.set_membership(&[2]);
        passive.set_whitelist(&[1]);
        let cub_dim = 0.15;
        LocalState {
            gamepads,
            maze,
            shared_state,
            world: CollisionWorld::new(0.02),
            cuboid: Cuboid::new(Vector::new(cub_dim, cub_dim)),
            cuboid2: Cuboid::new(Vector::new(cub_dim, cub_dim)),
            wall: Cuboid::new(Vector::new(0.51, 0.1)),
            active,
            passive,
            query_type: GeometricQueryType::Contacts(0.0, 0.0),
        }
    }
}

pub struct SharedState {
    ctx: Option<egui::Context>,
    maze_spec: MazeSpec,
    simulation: SimulationState,
    config: SimulationConfig,
}

impl SharedState {
    pub fn new(config: &Config, maze_spec: MazeSpec, size: Vec2) -> Self {
        SharedState {
            ctx: None,
            maze_spec,
            simulation: SimulationState {
                frame: 0,
                position: vec2(0.5, 0.5),
                velocity: 0.0,
                velocity_v: vec2(0.0, 0.0),
                angle_v: vec2(0.0, 0.0),
                angle: 0.0,
                steering: 0.0,
                acceleration: 0.0,
            },
            config: SimulationConfig::new(config, size),
        }
    }
}

pub fn run_simulation(config: &Config) {
    let maze_spec = MazeSpec::random();
    let maze_spec2 = maze_spec.clone();
    let maze = maze_from_seed_and_kind(maze_spec2.seed, maze_spec2.kind);
    let size = vec2(maze.size.1 as f32, maze.size.0 as f32);
    let shared_state = Arc::new(Mutex::new(SharedState::new(
        config,
        maze_spec.clone(),
        size,
    )));
    let mut local_state = LocalState::new(config, maze, shared_state.clone());
    add_maze(&mut local_state);
    local_state.world.update();
    let handle = thread::spawn(move || {
        simulation_loop(&mut local_state);
    });
    if config.gui {
        show_maze(shared_state).unwrap();
    }
    handle.join().unwrap();
}

fn simulation_loop(local_state: &mut LocalState) {
    let mut running = true;
    let shared_state = local_state.shared_state.clone();
    let sleep_time;
    {
        let state = shared_state.lock().unwrap();
        sleep_time = time::Duration::from_secs_f32(1.0 / state.config.framerate);
    }
    while running {
        {
            let mut state = shared_state.lock().unwrap();
            let config = state.config.clone();
            input_step(local_state, &mut state);
            simulation_step(local_state, config, &mut state.simulation);
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

fn input_step(local_state: &mut LocalState, state: &mut SharedState) {
    if let Some(gamepads) = &mut local_state.gamepads {
        gamepads.poll();

        for gamepad in gamepads.all() {
            let ls = gamepad.left_stick();
            let rs = gamepad.right_stick();
            state.simulation.steering = (ls.0 + rs.0).min(1.0).max(-1.0);
            state.simulation.acceleration = (ls.1 + rs.1).min(1.0).max(-1.0);
        }
    }
}

fn simulation_step(
    local_state: &LocalState,
    config: SimulationConfig,
    state: &mut SimulationState,
) {
    let max_velocity = 0.2;
    state.frame += 1;
    if config.human {
        if state.steering.abs() < 0.2 {
            state.steering = 0.0;
        }
        state.steering = state.steering.powi(7);
        state.acceleration = state.acceleration.powi(3);
    }
    if state.acceleration.signum() < 0.0 {
        state.velocity += state.acceleration * config.acceleration_scaler * 5.0;
    } else {
        state.velocity += state.acceleration * config.acceleration_scaler;
    }
    state.velocity = state.velocity.max(0.0);
    state.velocity = state.velocity.min(max_velocity);
    let vel_scale = (state.velocity.abs() * 10.0).max(1.0);
    state.angle += state.steering * config.steering_scaler / vel_scale;
    state.angle_v = Vec2::angled(state.angle);
    state.velocity_v = state.angle_v * state.velocity;
    let mut vel = state.velocity_v;
    let pos = state.position;
    let velocity_v = Vector::new(state.velocity_v.x, state.velocity_v.y);
    let trans_vec = Vector::new(pos.x, pos.y);
    let trans_matrix = Isometry::new(trans_vec, 0.0);
    let aabb = local_state.cuboid.bounding_volume(&trans_matrix);
    let interferences = local_state
        .world
        .interferences_with_aabb(&aabb, &local_state.active);
    let mut found = false;
    for interference in interferences {
        if let Some(shape) = interference.1.shape().as_shape::<Cuboid<f32>>() {
            let origin = Point::new(pos.x, pos.y);
            let closest_point = shape
                .project_point(&interference.1.position(), &origin, true)
                .point;
            let direction = closest_point - origin;
            if direction.angle(&velocity_v) < PI / 2.0 {
                let dir = vec2(direction.x, direction.y);
                let norm = dir.rot90();
                vel = vel.dot(norm) / norm.dot(norm) * norm;
                found = true;
            }
        }
    }
    state.velocity_v = vel;
    if found {
        state.velocity -= state.velocity * 0.3 + 0.001;
        state.velocity = state.velocity.max(0.0);
    }
    state.position += vel;
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

fn add_maze(state: &mut LocalState) {
    let x = state.maze.size.1;
    let y = state.maze.size.0;
    for ix in 0..x {
        for iy in 0..y {
            if let Some(field) = state.maze.get_field(&(ix, iy).into()) {
                add_field(state, &field);
            }
        }
    }
    // Outer wall
}

fn add_field(state: &mut LocalState, field: &Field) {
    if !field.has_passage(&Direction::West) {
        add_wall(state, field, true);
    }
    if !field.has_passage(&Direction::North) {
        add_wall(state, field, false);
    }
}

fn add_wall(state: &mut LocalState, field: &Field, vertical: bool) {
    let wall_shape = ShapeHandle::new(state.wall);
    let angle;
    let mut x = field.coordinates.x as f32;
    let mut y = field.coordinates.y as f32;
    if vertical {
        angle = std::f32::consts::PI / 2.0;
        y += 0.5;
    } else {
        angle = 0.0;
        x += 0.5;
    }
    let wall_position = Isometry::new(Vector::new(x, y), angle);
    state.world.add(
        wall_position,
        wall_shape,
        state.passive,
        state.query_type,
        (),
    );
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
