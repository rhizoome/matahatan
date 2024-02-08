use matahatan_lib::run_simulation;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run_simulation(true);
    Ok(())
}
