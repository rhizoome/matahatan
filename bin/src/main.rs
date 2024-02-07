use matahatan_lib::show_maze;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    show_maze()?;
    Ok(())
}
