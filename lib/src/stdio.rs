use super::SimulationState;
use maze_generator::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub acceleration: f32,
    pub steering: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub collision: bool,
}

#[derive(Serialize, Deserialize)]
struct MazeJson {
    x: i32,
    y: i32,
    passages: Vec<Vec<Vec<String>>>,
}

pub fn get_input() -> Option<Input> {
    let mut input_string = String::new();
    match io::stdin().read_line(&mut input_string) {
        Ok(_) => match serde_json::from_str::<Input>(&input_string) {
            Ok(input) => Some(input),
            Err(error) => {
                eprintln!("json error: {error}");
                None
            }
        },
        Err(_) => None,
    }
}

pub fn write_output(state: &SimulationState) {
    let output = Output {
        x: state.position.x,
        y: state.position.y,
        angle: state.angle,
        collision: state.collision,
    };
    match serde_json::to_string(&output) {
        Ok(out_str) => println!("{}", out_str),
        Err(_) => (),
    }
}

pub fn write_maze(maze: &Maze) {
    let x = maze.size.1;
    let y = maze.size.0;
    let empty: Vec<String> = Vec::new();
    let mut passages: Vec<Vec<Vec<String>>> = vec![vec![empty; y as usize]; x as usize];
    for ix in 0..x {
        for iy in 0..y {
            if let Some(field) = maze.get_field(&(ix, iy).into()) {
                let out = &mut passages[ix as usize][iy as usize];
                if field.has_passage(&Direction::North) {
                    out.push("n".to_string())
                }
                if field.has_passage(&Direction::West) {
                    out.push("w".to_string())
                }
                if field.has_passage(&Direction::South) {
                    out.push("s".to_string())
                }
                if field.has_passage(&Direction::East) {
                    out.push("e".to_string())
                }
            }
        }
    }
    let maze_json = MazeJson { x, y, passages };
    match serde_json::to_string(&maze_json) {
        Ok(out_str) => println!("{}", out_str),
        Err(_) => (),
    }
}
