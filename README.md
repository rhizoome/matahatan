# Matahatan

Virtual Maze Solving Challenge

## State

* stick-mode working
* stdio-mode working

## stdio

- input: `{"steering":1,"acceleration":1}`
- output: `{"x":0.5092106,"y":0.5038942,"angle":0.4,"collision":false}`

### maze

- n: north passage exists
- o: east passage exists
- s: source passage exists
- w: west passage exists

coordiates of output (x, y: f32) match index of the pssages (floor(x), floor(y)).

## Command-line

Not implemented options are removed. (--lua)

```
Test/train your maze-solver

Usage: matahatan simulate [OPTIONS]

Options:
  -f, --fps <FPS>              FPS of the simulation not the GUI (0 as fast as
                               possible) [default: 25]
  -x, --no-gui                 Do not run GUI (unattended training) sets FPS to 0
  -o, --stdio                  Run the simulation in stdio-mode (disables FPS)
  -s, --stick                  Run the simulation with stick (gamepad/joystick)
  -m, --maze-seed <maze-seed>  Maze seed (any string) [default: ]
  -k, --maze-kind <maze-kind>  Maze kind ('ellers', 'backtracking',
                               'growing_tree', 'prims') [default: backtracking]
  -h, --help                   Print help
```
