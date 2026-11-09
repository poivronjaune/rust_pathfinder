use std::path::Path;

use crate::simulator::{
    Car, Cell, Command, Direction, MAX_GRID_SIZE, MIN_GRID_SIZE, Outcome, Simulation,
};
use crate::ui::config::CELL_SIZE;
use crate::ui::render;

/// Mouse buttons that edit the map: the left button marks obstacles and the right button marks
/// unknown terrain.
#[derive(Clone, Copy)]
pub(crate) enum MouseButton {
    Left,
    Right,
}

/// Car headings with their display names, in the order the Grid Size dialog lists them.
pub(crate) const HEADINGS: [(&str, Direction); 4] = [
    ("Up", Direction::Up),
    ("Right", Direction::Right),
    ("Down", Direction::Down),
    ("Left", Direction::Left),
];

/// Size of the map and where the car starts on it, as chosen with Map → Grid Size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MapSettings {
    pub(crate) rows: usize,
    pub(crate) cols: usize,
    pub(crate) start: Car,
}

impl MapSettings {
    /// Checks the values entered in the Grid Size dialog. The start row and column count from 1,
    /// like the status bar, and `heading` is an index into `HEADINGS`.
    pub(crate) fn from_values(
        rows: i64,
        cols: i64,
        start_row: i64,
        start_col: i64,
        heading: usize,
    ) -> Result<Self, String> {
        let sizes = MIN_GRID_SIZE as i64..=MAX_GRID_SIZE as i64;
        if !sizes.contains(&rows) || !sizes.contains(&cols) {
            return Err(format!(
                "The grid must have between {MIN_GRID_SIZE} and {MAX_GRID_SIZE} rows and columns."
            ));
        }
        if !(1..=rows).contains(&start_row) {
            return Err(format!("The start row must be between 1 and {rows}."));
        }
        if !(1..=cols).contains(&start_col) {
            return Err(format!("The start column must be between 1 and {cols}."));
        }
        let (_, direction) = HEADINGS
            .get(heading)
            .copied()
            .ok_or_else(|| "Choose the heading of the car.".to_owned())?;
        Ok(Self {
            rows: rows as usize,
            cols: cols as usize,
            start: Car {
                row: (start_row - 1) as usize,
                col: (start_col - 1) as usize,
                direction,
            },
        })
    }

    /// Like `from_values`, for the text typed in the dialog.
    pub(crate) fn parse(
        rows: &str,
        cols: &str,
        start_row: &str,
        start_col: &str,
        heading: usize,
    ) -> Result<Self, String> {
        let number = |text: &str, name: &str| {
            text.trim()
                .parse::<i64>()
                .map_err(|_| format!("{name} must be a whole number."))
        };
        Self::from_values(
            number(rows, "Rows")?,
            number(cols, "Columns")?,
            number(start_row, "The start row")?,
            number(start_col, "The start column")?,
            heading,
        )
    }

    /// Index of the start heading in `HEADINGS`.
    pub(crate) fn heading_index(&self) -> usize {
        HEADINGS
            .iter()
            .position(|&(_, direction)| direction == self.start.direction)
            .unwrap_or(0)
    }
}

/// What the window shows: the simulation, drawn into a pixel buffer, and whether the map is being
/// edited.
pub(crate) struct AppState {
    /// The map drawn as `pixel_height()` lines of `pixel_width()` pixels.
    pub(crate) buffer: Vec<u32>,
    simulation: Simulation,
    editing: bool,
    /// Whether an obstacle stopped the last move.
    blocked: bool,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let mut state = Self {
            buffer: Vec::new(),
            simulation: Simulation::new(),
            editing: false,
            blocked: false,
        };
        state.redraw();
        state
    }

    #[cfg(test)]
    pub(crate) fn rows(&self) -> usize {
        self.simulation.grid().rows
    }

    #[cfg(test)]
    pub(crate) fn cols(&self) -> usize {
        self.simulation.grid().cols
    }

    pub(crate) fn pixel_width(&self) -> usize {
        self.simulation.grid().cols * CELL_SIZE
    }

    pub(crate) fn pixel_height(&self) -> usize {
        self.simulation.grid().rows * CELL_SIZE
    }

    /// Window title, which shows the size of the map.
    pub(crate) fn title(&self) -> String {
        let grid = self.simulation.grid();
        format!("{} x {} Grid", grid.rows, grid.cols)
    }

    pub(crate) fn settings(&self) -> MapSettings {
        let grid = self.simulation.grid();
        MapSettings {
            rows: grid.rows,
            cols: grid.cols,
            start: self.simulation.start(),
        }
    }

    /// Moves or turns the car, unless the map is in edit mode. Returns the cells the car left and
    /// entered, or `None` when the command was ignored.
    pub(crate) fn drive(&mut self, command: Command) -> Option<((usize, usize), (usize, usize))> {
        if self.editing {
            return None;
        }
        let previous = self.simulation.car();
        self.blocked = self.simulation.drive(command) == Outcome::Blocked;
        let car = self.simulation.car();

        let width = self.pixel_width();
        render::draw_cell(
            &mut self.buffer,
            width,
            previous.col,
            previous.row,
            self.simulation.grid().cells[previous.row][previous.col],
            self.editing,
        );
        render::draw_car_cell(&mut self.buffer, width, car);

        Some(((previous.row, previous.col), (car.row, car.col)))
    }

    /// Replaces the map with one of the same size where every cell is unknown. The car stays
    /// where it is.
    pub(crate) fn new_map(&mut self) {
        self.simulation.new_map();
        self.show_replaced_map();
    }

    /// Saves the map with its size and car start.
    pub(crate) fn save_map(&self, path: &Path) -> Result<(), String> {
        self.simulation.save_map(path)
    }

    /// Replaces the map with the one saved at `path` and puts the car at the map's start (the
    /// bottom-center cell, facing up, for files without one). The current map is kept if the file
    /// cannot be loaded.
    pub(crate) fn load_map(&mut self, path: &Path) -> Result<(), String> {
        self.simulation.load_map(path)?;
        self.show_replaced_map();
        Ok(())
    }

    /// Resizes the map and moves the car to its new start. Cells keep their place from the
    /// top-left corner and new cells are unknown. `settings` must come from `MapSettings`'s
    /// constructors, which keep the start on the map.
    pub(crate) fn apply_settings(&mut self, settings: MapSettings) {
        self.simulation
            .resize(settings.rows, settings.cols, settings.start);
        self.show_replaced_map();
    }

    /// Redraws the whole map after it was replaced or resized. The last move no longer counts as
    /// blocked.
    fn show_replaced_map(&mut self) {
        self.blocked = false;
        self.redraw();
    }

    /// Redraws the whole map, resizing the buffer to the size of the map.
    fn redraw(&mut self) {
        let pixels = self.pixel_width() * self.pixel_height();
        self.buffer.clear();
        self.buffer.resize(pixels, 0);
        render::draw(
            &mut self.buffer,
            self.simulation.grid(),
            self.simulation.car(),
            self.editing,
        );
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing
    }

    /// Turns edit mode on or off. The grid lines change color so the mode shows on the map.
    pub(crate) fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
        self.blocked = false;
        self.redraw();
    }

    /// Applies a mouse button to a cell while the map is in edit mode: the left button marks an
    /// obstacle and the right button marks unknown terrain. Returns whether the cell changed.
    pub(crate) fn edit_cell(&mut self, row: usize, col: usize, button: MouseButton) -> bool {
        let cell = match button {
            MouseButton::Left => Cell::Obstacle,
            MouseButton::Right => Cell::Unknown,
        };
        if !self.editing || !self.simulation.set_cell(row, col, cell) {
            return false;
        }
        // The car is drawn over its own cell, so that cell looks the same until the car moves.
        let car = self.simulation.car();
        if (row, col) != (car.row, car.col) {
            let width = self.pixel_width();
            render::draw_cell(&mut self.buffer, width, col, row, cell, self.editing);
        }
        true
    }

    pub(crate) fn status_text(&self) -> String {
        let mode = if self.editing {
            "Editing (left click: obstacle, right click: unknown; the car is parked)"
        } else if self.blocked {
            "Blocked by an obstacle"
        } else {
            "Ready"
        };
        let car = self.simulation.car();
        let heading = HEADINGS
            .iter()
            .find(|&&(_, direction)| direction == car.direction)
            .map_or("", |&(name, _)| name);
        format!(
            "{mode}  |  Row: {}  Column: {}  Heading: {heading}",
            car.row + 1,
            car.col + 1
        )
    }

    /// Client-area rectangle `(left, top, right, bottom)` covered by a cell when the map is drawn
    /// as large as fits, and centered, in an area of `width` by `height` pixels.
    #[cfg(target_os = "windows")]
    pub(crate) fn cell_bounds(
        &self,
        row: usize,
        col: usize,
        width: i32,
        height: i32,
    ) -> (i32, i32, i32, i32) {
        let (map_width, map_height) = (self.pixel_width() as f64, self.pixel_height() as f64);
        let scale = (width as f64 / map_width).min(height as f64 / map_height);
        let grid_width = (map_width * scale).round() as i32;
        let grid_height = (map_height * scale).round() as i32;
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
        &self,
        x: f64,
        y: f64,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Option<(usize, usize)> {
        let grid = self.simulation.grid();
        let (rows, cols) = (grid.rows as f64, grid.cols as f64);
        let col = ((x - left) / width * cols).floor();
        let row = ((y - top) / height * rows).floor();
        let inside = (0.0..rows).contains(&row) && (0.0..cols).contains(&col);
        inside.then_some((row as usize, col as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, HEADINGS, MapSettings, MouseButton};
    use crate::simulator::{Car, Cell, Command, Direction, MAX_GRID_SIZE};
    use crate::ui::config::{BORDER, CELL_SIZE, EDIT_BORDER};
    use crate::ui::render;

    /// A state whose map is blank, so tests do not depend on the random sample map.
    fn blank_state() -> AppState {
        let mut state = AppState::new();
        state.new_map();
        state
    }

    fn expected_buffer(state: &AppState) -> Vec<u32> {
        let mut expected = vec![0; state.pixel_width() * state.pixel_height()];
        render::draw(
            &mut expected,
            state.simulation.grid(),
            state.simulation.car(),
            state.editing,
        );
        expected
    }

    /// Row and column of the car.
    fn car_cell(state: &AppState) -> (usize, usize) {
        let car = state.simulation.car();
        (car.row, car.col)
    }

    #[test]
    fn movement_updates_only_the_previous_and_current_cells() {
        let mut state = blank_state();
        let previous_buffer = state.buffer.clone();
        let changed = state.drive(Command::Forward).unwrap();

        assert_eq!(changed.0, (19, 10));
        assert_eq!(changed.1, (18, 10));
        assert_cells_changed_only(&state, &previous_buffer, &[changed.0, changed.1]);
    }

    #[test]
    fn turning_updates_only_the_current_cell() {
        let mut state = blank_state();
        let previous_buffer = state.buffer.clone();
        let changed = state.drive(Command::TurnRight).unwrap();

        assert_eq!(changed.0, changed.1);
        assert_cells_changed_only(&state, &previous_buffer, &[changed.1]);
    }

    #[test]
    fn obstacles_block_the_car_both_ways() {
        let mut state = blank_state();
        state.apply_settings(MapSettings::from_values(20, 20, 11, 11, 0).unwrap());
        state.simulation.set_cell(9, 10, Cell::Obstacle);
        state.simulation.set_cell(11, 10, Cell::Obstacle);
        state.redraw();
        let before = state.buffer.clone();

        for command in [Command::Forward, Command::Reverse] {
            assert_eq!(state.drive(command), Some(((10, 10), (10, 10))));
            assert_eq!(car_cell(&state), (10, 10));
            assert!(state.status_text().starts_with("Blocked"));
            assert!(state.buffer == before);
        }

        state.drive(Command::TurnRight);
        assert!(state.status_text().starts_with("Ready"));
        assert_eq!(state.drive(Command::Forward), Some(((10, 10), (10, 11))));
    }

    #[test]
    fn unknown_and_free_cells_do_not_block_the_car() {
        let mut state = blank_state();
        state.simulation.set_cell(18, 10, Cell::Free);
        state.drive(Command::Forward);
        state.drive(Command::Forward);

        assert_eq!(car_cell(&state), (17, 10));
        assert!(state.status_text().starts_with("Ready"));
    }

    #[test]
    fn car_keys_are_ignored_in_edit_mode() {
        let mut state = blank_state();
        state.set_editing(true);
        let (car, before) = (state.simulation.car(), state.buffer.clone());

        for command in [
            Command::Forward,
            Command::Reverse,
            Command::TurnLeft,
            Command::TurnRight,
        ] {
            assert_eq!(state.drive(command), None);
        }
        assert_eq!(state.simulation.car(), car);
        assert!(state.buffer == before);

        state.set_editing(false);
        assert!(state.drive(Command::TurnLeft).is_some());
    }

    #[test]
    fn new_map_marks_every_cell_unknown_and_keeps_the_size_and_the_car() {
        let mut state = AppState::new();
        state.apply_settings(MapSettings::from_values(6, 9, 6, 5, 0).unwrap());
        state.drive(Command::TurnRight);
        state.drive(Command::Forward);
        let car = state.simulation.car();

        state.new_map();

        assert_eq!((state.rows(), state.cols()), (6, 9));
        let cells = &state.simulation.grid().cells;
        assert!(cells.iter().flatten().all(|&cell| cell == Cell::Unknown));
        assert_eq!(state.simulation.car(), car);
        assert!(state.buffer == expected_buffer(&state));
    }

    #[test]
    fn saved_map_loads_back_with_its_size_and_car_start() {
        let file_name = format!("rust_pathfinder_test_map_{}.map", std::process::id());
        let path = std::env::temp_dir().join(file_name);
        let mut state = AppState::new();
        let settings = MapSettings::from_values(7, 12, 3, 4, 3).unwrap();
        state.apply_settings(settings);
        state.set_editing(true);
        state.edit_cell(0, 11, MouseButton::Left);
        state.set_editing(false);
        let (grid, buffer) = (state.simulation.grid().clone(), state.buffer.clone());
        state.drive(Command::Forward);

        let saved = state.save_map(&path);
        state.apply_settings(MapSettings::from_values(20, 20, 20, 11, 0).unwrap());
        let loaded = state.load_map(&path);
        let _ = std::fs::remove_file(&path);

        saved.unwrap();
        loaded.unwrap();
        assert_eq!(state.simulation.grid(), &grid);
        assert_eq!(state.settings(), settings);
        assert_eq!(state.simulation.car(), settings.start);
        assert!(state.buffer == buffer);
    }

    #[test]
    fn failed_load_keeps_the_current_map() {
        let path = std::env::temp_dir()
            .join("rust_pathfinder_missing_folder")
            .join("missing.map");
        let mut state = AppState::new();
        let (simulation, buffer) = (state.simulation.clone(), state.buffer.clone());

        assert!(state.load_map(&path).is_err());
        assert_eq!(state.simulation, simulation);
        assert!(state.buffer == buffer);
    }

    #[test]
    fn grid_size_settings_resize_the_map_and_move_the_car_to_its_start() {
        let mut state = blank_state();
        state.simulation.set_cell(0, 0, Cell::Obstacle);
        state.simulation.set_cell(1, 2, Cell::Obstacle);
        let settings = MapSettings::from_values(5, 8, 2, 3, 1).unwrap();

        state.apply_settings(settings);

        assert_eq!((state.rows(), state.cols()), (5, 8));
        assert_eq!(state.title(), "5 x 8 Grid");
        assert_eq!(state.buffer.len(), 8 * CELL_SIZE * 5 * CELL_SIZE);
        assert_eq!(state.settings(), settings);
        let start = Car {
            row: 1,
            col: 2,
            direction: Direction::Right,
        };
        assert_eq!(state.simulation.car(), start);
        assert_eq!(state.simulation.grid().cells[0][0], Cell::Obstacle);
        // The start cell is cleared so the car does not start inside an obstacle.
        assert_eq!(state.simulation.grid().cells[1][2], Cell::Free);
        assert!(state.buffer == expected_buffer(&state));
        assert!(state.status_text().contains("Row: 2  Column: 3  Heading: Right"));
    }

    #[test]
    fn grid_size_settings_are_checked() {
        let error = |result: Result<MapSettings, String>| result.unwrap_err();

        assert!(error(MapSettings::from_values(1, 5, 1, 1, 0)).contains("between"));
        let too_many = MAX_GRID_SIZE as i64 + 1;
        assert!(error(MapSettings::from_values(5, too_many, 1, 1, 0)).contains("between"));
        assert!(error(MapSettings::from_values(5, 5, 6, 1, 0)).contains("start row"));
        assert!(error(MapSettings::from_values(5, 5, 0, 1, 0)).contains("start row"));
        assert!(error(MapSettings::from_values(5, 5, 1, 6, 0)).contains("start column"));
        assert!(error(MapSettings::from_values(5, 5, 1, 1, HEADINGS.len())).contains("heading"));
        assert!(error(MapSettings::parse("ten", "5", "1", "1", 0)).contains("Rows"));
        assert!(error(MapSettings::parse("5", "5", "", "1", 0)).contains("start row"));

        let parsed = MapSettings::parse(" 12 ", "30", "12", "1", 2).unwrap();
        assert_eq!((parsed.rows, parsed.cols), (12, 30));
        assert_eq!((parsed.start.row, parsed.start.col), (11, 0));
        assert_eq!(parsed.start.direction, Direction::Down);
        assert_eq!(parsed.heading_index(), 2);
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
        let mut state = blank_state();
        assert!(!state.edit_cell(0, 0, MouseButton::Left));
        assert_eq!(state.simulation.grid().cells[0][0], Cell::Unknown);

        state.set_editing(true);
        let before = state.buffer.clone();
        assert!(state.edit_cell(0, 0, MouseButton::Left));
        assert_eq!(state.simulation.grid().cells[0][0], Cell::Obstacle);
        assert_cells_changed_only(&state, &before, &[(0, 0)]);
        assert!(!state.edit_cell(0, 0, MouseButton::Left));

        assert!(state.edit_cell(0, 0, MouseButton::Right));
        assert_eq!(state.simulation.grid().cells[0][0], Cell::Unknown);
        assert!(state.buffer == before);
        assert!(!state.edit_cell(state.rows(), 0, MouseButton::Left));
        assert!(!state.edit_cell(0, state.cols(), MouseButton::Left));
    }

    #[test]
    fn editing_the_car_cell_keeps_the_car_on_top() {
        let mut state = AppState::new();
        state.set_editing(true);
        let (row, col) = car_cell(&state);
        let before = state.buffer.clone();

        assert!(state.edit_cell(row, col, MouseButton::Left));
        assert_eq!(state.simulation.grid().cells[row][col], Cell::Obstacle);
        assert!(state.buffer == before);
    }

    #[test]
    fn cell_at_finds_the_cell_drawn_under_a_point() {
        let mut state = AppState::new();
        state.apply_settings(MapSettings::from_values(4, 8, 1, 1, 0).unwrap());
        let (left, top, width, height) = (10.0, 20.0, 400.0, 200.0);
        let cell = width / 8.0;
        let cell_at = |x: f64, y: f64| state.cell_at(x, y, left, top, width, height);

        assert_eq!(cell_at(left, top), Some((0, 0)));
        assert_eq!(cell_at(left + 1.5 * cell, top + 0.5 * cell), Some((0, 1)));
        assert_eq!(cell_at(left + 2.5 * cell, top + 3.5 * cell), Some((3, 2)));
        assert_eq!(cell_at(left + width - 0.1, top + height - 0.1), Some((3, 7)));
        assert_eq!(cell_at(left - 0.1, top), None);
        assert_eq!(cell_at(left + width, top), None);
        assert_eq!(cell_at(left, top + height), None);
    }

    fn assert_cells_changed_only(state: &AppState, before: &[u32], changed_cells: &[(usize, usize)]) {
        let after = &state.buffer;
        let width = state.pixel_width();
        let mut any_expected_pixel_changed = false;
        for row in 0..state.rows() {
            for col in 0..state.cols() {
                let expected_cell = changed_cells.contains(&(row, col));
                for y in row * CELL_SIZE..(row + 1) * CELL_SIZE {
                    for x in col * CELL_SIZE..(col + 1) * CELL_SIZE {
                        let index = y * width + x;
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
