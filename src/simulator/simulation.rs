use std::path::Path;

use crate::simulator::map_file::{self, MapData};
use crate::simulator::{Car, Cell, DEFAULT_GRID_SIZE, Grid};

/// What a driver tells the car to do: move one cell forward or backward along its heading, or turn
/// in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Forward,
    Reverse,
    TurnLeft,
    TurnRight,
}

/// What a command did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The car moved one cell.
    Moved,
    /// The car turned in place.
    Turned,
    /// An obstacle is in the way, so the car stayed where it was.
    Blocked,
    /// The edge of the map is in the way, so the car stayed where it was.
    AtEdge,
}

/// A map, the car driven on it, and where the car starts. The map and the car change only through
/// these methods, which keep the driving rules: the car stays on the map, and obstacles block it
/// while unknown terrain does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Simulation {
    grid: Grid,
    car: Car,
    /// Where the car starts on this map. It is saved with the map.
    start: Car,
}

impl Simulation {
    /// A sample backyard of the default size, with the car at its default start: the bottom-center
    /// cell, facing up.
    pub(crate) fn new() -> Self {
        let start = Car::start_for(DEFAULT_GRID_SIZE, DEFAULT_GRID_SIZE);
        let grid = Grid::backyard(DEFAULT_GRID_SIZE, DEFAULT_GRID_SIZE, start.row, start.col);
        Self {
            grid,
            car: start,
            start,
        }
    }

    pub(crate) fn grid(&self) -> &Grid {
        &self.grid
    }

    pub(crate) fn car(&self) -> Car {
        self.car
    }

    /// Where the car starts on this map.
    pub(crate) fn start(&self) -> Car {
        self.start
    }

    /// Moves or turns the car.
    pub(crate) fn drive(&mut self, command: Command) -> Outcome {
        match command {
            Command::Forward | Command::Reverse => {
                let forward = matches!(command, Command::Forward);
                let target = self
                    .car
                    .next_position(forward, self.grid.rows, self.grid.cols);
                match target {
                    None => Outcome::AtEdge,
                    Some((row, col)) if self.grid.cells[row][col] == Cell::Obstacle => {
                        Outcome::Blocked
                    }
                    Some((row, col)) => {
                        (self.car.row, self.car.col) = (row, col);
                        Outcome::Moved
                    }
                }
            }
            Command::TurnLeft => {
                self.car.turn_left();
                Outcome::Turned
            }
            Command::TurnRight => {
                self.car.turn_right();
                Outcome::Turned
            }
        }
    }

    /// Replaces the map with one of the same size where every cell is unknown. The car stays where
    /// it is.
    pub(crate) fn new_map(&mut self) {
        self.grid = Grid::unknown(self.grid.rows, self.grid.cols);
    }

    /// Saves the map with its size and car start.
    pub(crate) fn save_map(&self, path: &Path) -> Result<(), String> {
        map_file::save(&self.grid, self.start, path)
    }

    /// Replaces the map with the one saved at `path` and puts the car at the map's start (the
    /// bottom-center cell, facing up, for files without one). The current map is kept if the file
    /// cannot be loaded.
    pub(crate) fn load_map(&mut self, path: &Path) -> Result<(), String> {
        let MapData { grid, start } = map_file::load(path)?;
        self.start = start.unwrap_or_else(|| Car::start_for(grid.rows, grid.cols));
        self.car = self.start;
        self.grid = grid;
        Ok(())
    }

    /// Resizes the map and moves the car to `start`, which must be on the resized map. Cells keep
    /// their place from the top-left corner and new cells are unknown.
    pub(crate) fn resize(&mut self, rows: usize, cols: usize, start: Car) {
        let mut grid = self.grid.resized(rows, cols);
        // The car cannot start inside an obstacle.
        let start_cell = &mut grid.cells[start.row][start.col];
        if *start_cell == Cell::Obstacle {
            *start_cell = Cell::Free;
        }
        self.start = start;
        self.car = start;
        self.grid = grid;
    }

    /// Sets the terrain of a cell. Returns whether the cell changed, so cells outside the map and
    /// cells that already have this terrain return `false`.
    pub(crate) fn set_cell(&mut self, row: usize, col: usize, cell: Cell) -> bool {
        let terrain = &mut self.grid.cells;
        let Some(current) = terrain.get_mut(row).and_then(|cells| cells.get_mut(col)) else {
            return false;
        };
        if *current == cell {
            return false;
        }
        *current = cell;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Outcome, Simulation};
    use crate::simulator::{Car, Cell, DEFAULT_GRID_SIZE, Direction, Grid};

    /// A simulation on a blank 5 x 5 map with the car in the center, facing up, so tests do not
    /// depend on the random sample map.
    fn blank_simulation() -> Simulation {
        let car = Car {
            row: 2,
            col: 2,
            direction: Direction::Up,
        };
        Simulation {
            grid: Grid::unknown(5, 5),
            car,
            start: car,
        }
    }

    fn position(simulation: &Simulation) -> (usize, usize) {
        (simulation.car.row, simulation.car.col)
    }

    #[test]
    fn new_simulation_starts_in_the_bottom_center_facing_up() {
        let simulation = Simulation::new();
        let start = Car::start_for(DEFAULT_GRID_SIZE, DEFAULT_GRID_SIZE);

        assert_eq!(simulation.grid().rows, DEFAULT_GRID_SIZE);
        assert_eq!(simulation.grid().cols, DEFAULT_GRID_SIZE);
        assert_eq!(simulation.car(), start);
        assert_eq!(simulation.start(), start);
        assert_eq!(simulation.grid().cells[start.row][start.col], Cell::Free);
    }

    #[test]
    fn commands_move_and_turn_the_car() {
        let mut simulation = blank_simulation();

        assert_eq!(simulation.drive(Command::Forward), Outcome::Moved);
        assert_eq!(position(&simulation), (1, 2));
        assert_eq!(simulation.drive(Command::Reverse), Outcome::Moved);
        assert_eq!(position(&simulation), (2, 2));
        assert_eq!(simulation.drive(Command::TurnRight), Outcome::Turned);
        assert_eq!(simulation.car().direction, Direction::Right);
        assert_eq!(simulation.drive(Command::TurnLeft), Outcome::Turned);
        assert_eq!(simulation.car().direction, Direction::Up);
        assert_eq!(position(&simulation), (2, 2));
    }

    #[test]
    fn obstacles_block_the_car_both_ways() {
        let mut simulation = blank_simulation();
        simulation.set_cell(1, 2, Cell::Obstacle);
        simulation.set_cell(3, 2, Cell::Obstacle);
        let car = simulation.car();

        assert_eq!(simulation.drive(Command::Forward), Outcome::Blocked);
        assert_eq!(simulation.drive(Command::Reverse), Outcome::Blocked);
        assert_eq!(simulation.car(), car);
    }

    #[test]
    fn unknown_and_free_cells_do_not_block_the_car() {
        let mut simulation = blank_simulation();
        simulation.set_cell(1, 2, Cell::Free);

        assert_eq!(simulation.drive(Command::Forward), Outcome::Moved);
        assert_eq!(simulation.drive(Command::Forward), Outcome::Moved);
        assert_eq!(position(&simulation), (0, 2));
    }

    #[test]
    fn the_edge_of_the_map_stops_the_car() {
        let mut simulation = blank_simulation();
        simulation.drive(Command::Forward);
        simulation.drive(Command::Forward);

        assert_eq!(simulation.drive(Command::Forward), Outcome::AtEdge);
        assert_eq!(position(&simulation), (0, 2));
    }

    #[test]
    fn new_map_marks_every_cell_unknown_and_keeps_the_car() {
        let mut simulation = blank_simulation();
        simulation.set_cell(0, 0, Cell::Obstacle);
        simulation.drive(Command::Forward);
        let car = simulation.car();

        simulation.new_map();

        assert_eq!(simulation.grid(), &Grid::unknown(5, 5));
        assert_eq!(simulation.car(), car);
    }

    #[test]
    fn resizing_keeps_the_terrain_and_frees_the_start_cell() {
        let mut simulation = blank_simulation();
        simulation.set_cell(0, 0, Cell::Obstacle);
        simulation.set_cell(1, 2, Cell::Obstacle);
        let start = Car {
            row: 1,
            col: 2,
            direction: Direction::Left,
        };

        simulation.resize(3, 4, start);

        let grid = simulation.grid();
        assert_eq!((grid.rows, grid.cols), (3, 4));
        assert_eq!(grid.cells[0][0], Cell::Obstacle);
        assert_eq!(grid.cells[1][2], Cell::Free);
        assert_eq!(simulation.car(), start);
        assert_eq!(simulation.start(), start);
    }

    #[test]
    fn set_cell_reports_changes_inside_the_map() {
        let mut simulation = blank_simulation();

        assert!(simulation.set_cell(0, 0, Cell::Obstacle));
        assert!(!simulation.set_cell(0, 0, Cell::Obstacle));
        assert_eq!(simulation.grid().cells[0][0], Cell::Obstacle);
        assert!(!simulation.set_cell(5, 0, Cell::Obstacle));
        assert!(!simulation.set_cell(0, 5, Cell::Obstacle));
    }

    #[test]
    fn saved_map_loads_back_with_the_car_at_its_start() {
        let file_name = format!("rust_pathfinder_simulation_{}.map", std::process::id());
        let path = std::env::temp_dir().join(file_name);
        let mut simulation = blank_simulation();
        simulation.set_cell(4, 4, Cell::Obstacle);
        let saved = simulation.clone();
        simulation.drive(Command::TurnRight);

        let written = simulation.save_map(&path);
        let mut loaded = Simulation::new();
        let read = loaded.load_map(&path);
        let _ = std::fs::remove_file(&path);

        written.unwrap();
        read.unwrap();
        assert_eq!(loaded, saved);
    }
}
