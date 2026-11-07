use std::path::Path;

use crate::car::Car;
#[cfg(target_os = "windows")]
use crate::config::CELL_SIZE;
use crate::config::{GRID_SIZE, HEIGHT, WIDTH};
use crate::grid::{Cell, Grid};
use crate::map_file;
use crate::render;

#[derive(Clone, Copy)]
pub(crate) enum VehicleKey {
    Forward,
    Reverse,
    TurnLeft,
    TurnRight,
}

/// Mouse buttons that edit the map: the left button marks obstacles and the right button marks
/// unknown terrain.
#[derive(Clone, Copy)]
pub(crate) enum MouseButton {
    Left,
    Right,
}

pub(crate) struct AppState {
    pub(crate) buffer: Vec<u32>,
    grid: Grid,
    car: Car,
    editing: bool,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let car = Car::default();
        let grid = Grid::backyard(car.row, car.col);
        let mut buffer = vec![0; WIDTH * HEIGHT];
        render::draw(&mut buffer, &grid, car, false);
        Self {
            buffer,
            grid,
            car,
            editing: false,
        }
    }

    pub(crate) fn handle_vehicle_key(
        &mut self,
        key: VehicleKey,
    ) -> ((usize, usize), (usize, usize)) {
        let previous = (self.car.row, self.car.col);
        match key {
            VehicleKey::Forward => self.car.move_forward(),
            VehicleKey::Reverse => self.car.move_backward(),
            VehicleKey::TurnLeft => self.car.turn_left(),
            VehicleKey::TurnRight => self.car.turn_right(),
        }
        let current = (self.car.row, self.car.col);

        render::draw_cell(
            &mut self.buffer,
            previous.1,
            previous.0,
            self.grid.cells[previous.0][previous.1],
            self.editing,
        );
        render::draw_car_cell(&mut self.buffer, self.car);

        (previous, current)
    }

    /// Replaces the map with one where every cell is unknown. The car stays where it is.
    pub(crate) fn new_map(&mut self) {
        self.set_grid(Grid::unknown());
    }

    pub(crate) fn save_map(&self, path: &Path) -> Result<(), String> {
        map_file::save(&self.grid, path)
    }

    /// Replaces the map with the one saved at `path`, keeping the current map if the file cannot be
    /// loaded. The car stays where it is.
    pub(crate) fn load_map(&mut self, path: &Path) -> Result<(), String> {
        let grid = map_file::load(path)?;
        self.set_grid(grid);
        Ok(())
    }

    fn set_grid(&mut self, grid: Grid) {
        self.grid = grid;
        render::draw(&mut self.buffer, &self.grid, self.car, self.editing);
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing
    }

    /// Turns edit mode on or off. The grid lines change color so the mode shows on the map.
    pub(crate) fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
        render::draw(&mut self.buffer, &self.grid, self.car, editing);
    }

    /// Applies a mouse button to a cell while the map is in edit mode: the left button marks an
    /// obstacle and the right button marks unknown terrain. Returns whether the cell changed.
    pub(crate) fn edit_cell(&mut self, row: usize, col: usize, button: MouseButton) -> bool {
        let cell = match button {
            MouseButton::Left => Cell::Obstacle,
            MouseButton::Right => Cell::Unknown,
        };
        let terrain = &mut self.grid.cells;
        let Some(current) = terrain.get_mut(row).and_then(|cells| cells.get_mut(col)) else {
            return false;
        };
        if !self.editing || *current == cell {
            return false;
        }
        *current = cell;
        // The car is drawn over its own cell, so that cell looks the same until the car moves.
        if (row, col) != (self.car.row, self.car.col) {
            render::draw_cell(&mut self.buffer, col, row, cell, self.editing);
        }
        true
    }

    pub(crate) fn status_text(&self) -> String {
        let mode = if self.editing {
            "Editing (left click: obstacle, right click: unknown)"
        } else {
            "Ready"
        };
        let direction = match self.car.direction {
            crate::car::Direction::Up => "Up",
            crate::car::Direction::Right => "Right",
            crate::car::Direction::Down => "Down",
            crate::car::Direction::Left => "Left",
        };
        format!(
            "{mode}  |  Row: {}  Column: {}  Heading: {direction}",
            self.car.row + 1,
            self.car.col + 1
        )
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn cell_bounds(
        row: usize,
        col: usize,
        width: i32,
        height: i32,
    ) -> (i32, i32, i32, i32) {
        let scale = (width as f64 / crate::config::WIDTH as f64)
            .min(height as f64 / crate::config::HEIGHT as f64);
        let grid_width = (crate::config::WIDTH as f64 * scale).round() as i32;
        let grid_height = (crate::config::HEIGHT as f64 * scale).round() as i32;
        let origin_x = (width - grid_width) / 2;
        let origin_y = (height - grid_height) / 2;
        let left = origin_x + ((col * CELL_SIZE) as f64 * scale).floor() as i32;
        let top = origin_y + ((row * CELL_SIZE) as f64 * scale).floor() as i32;
        let right = origin_x + (((col + 1) * CELL_SIZE) as f64 * scale).ceil() as i32;
        let bottom = origin_y + (((row + 1) * CELL_SIZE) as f64 * scale).ceil() as i32;
        (left, top, right, bottom)
    }

    /// Returns the `(row, column)` of the cell under the point `(x, y)` when the whole grid is
    /// drawn in the rectangle that starts at `(left, top)` and measures `width` by `height`, or
    /// `None` when the point is outside the grid.
    pub(crate) fn cell_at(
        x: f64,
        y: f64,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Option<(usize, usize)> {
        let col = ((x - left) / width * GRID_SIZE as f64).floor();
        let row = ((y - top) / height * GRID_SIZE as f64).floor();
        let cells = 0.0..GRID_SIZE as f64;
        (cells.contains(&row) && cells.contains(&col)).then_some((row as usize, col as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, MouseButton, VehicleKey};
    use crate::config::{BORDER, CELL_SIZE, EDIT_BORDER, GRID_SIZE, HEIGHT, WIDTH};
    use crate::grid::Cell;
    use crate::render;

    #[test]
    fn movement_updates_only_the_previous_and_current_cells() {
        let mut state = AppState::new();
        let previous_buffer = state.buffer.clone();
        let changed = state.handle_vehicle_key(VehicleKey::Forward);

        assert_eq!(changed.0, (19, 10));
        assert_eq!(changed.1, (18, 10));
        assert_cells_changed_only(&previous_buffer, &state.buffer, &[changed.0, changed.1]);
    }

    #[test]
    fn turning_updates_only_the_current_cell() {
        let mut state = AppState::new();
        let previous_buffer = state.buffer.clone();
        let changed = state.handle_vehicle_key(VehicleKey::TurnRight);

        assert_eq!(changed.0, changed.1);
        assert_cells_changed_only(&previous_buffer, &state.buffer, &[changed.1]);
    }

    #[test]
    fn new_map_marks_every_cell_unknown_and_keeps_the_car() {
        let mut state = AppState::new();
        state.handle_vehicle_key(VehicleKey::Forward);
        state.handle_vehicle_key(VehicleKey::TurnRight);
        let car = state.car;

        state.new_map();

        let cells = state.grid.cells;
        assert!(cells.iter().flatten().all(|&cell| cell == Cell::Unknown));
        assert_eq!(state.car, car);
        let mut expected = vec![0; WIDTH * HEIGHT];
        render::draw(&mut expected, &state.grid, car, false);
        assert!(state.buffer == expected);
    }

    #[test]
    fn saved_map_loads_back_and_keeps_the_car() {
        let file_name = format!("rust_pathfinder_test_map_{}.map", std::process::id());
        let path = std::env::temp_dir().join(file_name);
        let mut state = AppState::new();
        state.handle_vehicle_key(VehicleKey::Forward);
        let (car, cells, buffer) = (state.car, state.grid.cells, state.buffer.clone());

        let saved = state.save_map(&path);
        state.new_map();
        let loaded = state.load_map(&path);
        let _ = std::fs::remove_file(&path);

        saved.unwrap();
        loaded.unwrap();
        assert_eq!(state.car, car);
        assert_eq!(state.grid.cells, cells);
        assert!(state.buffer == buffer);
    }

    #[test]
    fn failed_load_keeps_the_current_map() {
        let path = std::env::temp_dir()
            .join("rust_pathfinder_missing_folder")
            .join("missing.map");
        let mut state = AppState::new();
        let (cells, buffer) = (state.grid.cells, state.buffer.clone());

        assert!(state.load_map(&path).is_err());
        assert_eq!(state.grid.cells, cells);
        assert!(state.buffer == buffer);
    }

    #[test]
    fn edit_mode_recolors_the_grid_lines_and_updates_the_status() {
        let mut state = AppState::new();
        assert!(!state.is_editing());
        assert!(state.status_text().starts_with("Ready"));
        assert_eq!(state.buffer[0], BORDER);

        state.set_editing(true);
        assert!(state.is_editing());
        assert!(state.status_text().starts_with("Editing"));
        assert_eq!(state.buffer[0], EDIT_BORDER);

        state.set_editing(false);
        assert_eq!(state.buffer[0], BORDER);
    }

    #[test]
    fn mouse_buttons_edit_cells_only_in_edit_mode() {
        let mut state = AppState::new();
        state.new_map();
        assert!(!state.edit_cell(0, 0, MouseButton::Left));
        assert_eq!(state.grid.cells[0][0], Cell::Unknown);

        state.set_editing(true);
        let before = state.buffer.clone();
        assert!(state.edit_cell(0, 0, MouseButton::Left));
        assert_eq!(state.grid.cells[0][0], Cell::Obstacle);
        assert_cells_changed_only(&before, &state.buffer, &[(0, 0)]);
        assert!(!state.edit_cell(0, 0, MouseButton::Left));

        assert!(state.edit_cell(0, 0, MouseButton::Right));
        assert_eq!(state.grid.cells[0][0], Cell::Unknown);
        assert!(state.buffer == before);
        assert!(!state.edit_cell(GRID_SIZE, 0, MouseButton::Left));
    }

    #[test]
    fn editing_the_car_cell_keeps_the_car_on_top() {
        let mut state = AppState::new();
        state.set_editing(true);
        let (row, col) = (state.car.row, state.car.col);
        let before = state.buffer.clone();

        assert!(state.edit_cell(row, col, MouseButton::Left));
        assert_eq!(state.grid.cells[row][col], Cell::Obstacle);
        assert!(state.buffer == before);
    }

    #[test]
    fn cell_at_finds_the_cell_drawn_under_a_point() {
        let (left, top, size) = (10.0, 20.0, 400.0);
        let cell = size / GRID_SIZE as f64;
        let cell_at = |x: f64, y: f64| AppState::cell_at(x, y, left, top, size, size);
        let last = GRID_SIZE - 1;

        assert_eq!(cell_at(left, top), Some((0, 0)));
        assert_eq!(cell_at(left + 1.5 * cell, top + 0.5 * cell), Some((0, 1)));
        assert_eq!(cell_at(left + 2.5 * cell, top + 3.5 * cell), Some((3, 2)));
        assert_eq!(cell_at(left + size - 0.1, top + size - 0.1), Some((last, last)));
        assert_eq!(cell_at(left - 0.1, top), None);
        assert_eq!(cell_at(left + size, top), None);
        assert_eq!(cell_at(left, top + size), None);
    }

    fn assert_cells_changed_only(before: &[u32], after: &[u32], changed_cells: &[(usize, usize)]) {
        let mut any_expected_pixel_changed = false;
        for row in 0..crate::config::GRID_SIZE {
            for col in 0..crate::config::GRID_SIZE {
                let expected_cell = changed_cells.contains(&(row, col));
                for y in row * CELL_SIZE..(row + 1) * CELL_SIZE {
                    for x in col * CELL_SIZE..(col + 1) * CELL_SIZE {
                        let index = y * WIDTH + x;
                        if expected_cell {
                            any_expected_pixel_changed |= before[index] != after[index];
                        } else {
                            assert_eq!(before[index], after[index]);
                        }
                    }
                }
            }
        }
        assert!(any_expected_pixel_changed);
    }
}

