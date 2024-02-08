use clap::{Arg, Command};
use matahatan_lib::run_simulation;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("Matahatan")
        .version("0.1")
        .author("Adfinis AG")
        .about("Virtual Maze Solving Challenge")
        .subcommand(
            Command::new("simulate")
                .about("Test/train your maze-solver")
                .arg(
                    Arg::new("lua")
                        .short('l')
                        .long("lua")
                        .value_name("FILE")
                        .help("Run the lua-script FILE (enables lua-mode)")
                        .num_args(1),
                )
                .arg(
                    Arg::new("fps")
                        .short('f')
                        .long("fps")
                        .value_name("FPS")
                        .help("FPS of the simulation not the GUI (0 as fast as possible)")
                        .num_args(1),
                )
                .arg(
                    Arg::new("--no-gui")
                        .short('x')
                        .long("no-gui")
                        .help("Do not run GUI (unattended training) sets FPS to 0")
                        .num_args(0),
                )
                .arg(
                    Arg::new("stdio")
                        .short('o')
                        .long("stdio")
                        .help("Run the simulation in stdio-mode")
                        .num_args(0),
                ),
        )
        .subcommand(
            Command::new("server")
                .about("Run the maze-solver server (the actual challenges will be missing)"),
        );
    let matches = command.clone().get_matches();

    match matches.subcommand() {
        Some(("simulate", _m)) => run_simulation(true),
        Some(("server", _m)) => (),
        _ => command.print_long_help()?,
    }
    Ok(())
}
