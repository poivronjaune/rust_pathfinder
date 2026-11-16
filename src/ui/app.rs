use std::path::Path;

use crate::navigator::{
    Event as NavigatorEvent, ExecutionState, Navigator, Position, Terrain as DiscoveredTerrain,
};
use crate::simulator::{
    Car, Cell, Command, Direction, Grid, MAX_GRID_SIZE, MIN_GRID_SIZE, Outcome, Simulation,
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
    navigator: Navigator,
    nav_origin: Car,
    session_start: Car,
    editing: bool,
    nav_stale: bool,
    edit_brush: Option<Cell>,
    discovered_view: bool,
    /// Whether an obstacle stopped the last move.
    blocked: Option<Outcome>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let simulation = Simulation::new();
        let start = simulation.car();
        let mut state = Self {
            buffer: Vec::new(),
            simulation,
            navigator: Navigator::new(true),
            nav_origin: start,
            session_start: start,
            editing: false,
            nav_stale: false,
            edit_brush: None,
            discovered_view: false,
            blocked: None,
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
        if self.editing
            || matches!(
                self.navigator.state(),
                ExecutionState::Running | ExecutionState::Inspecting
            )
        {
            return None;
        }
        let previous = self.simulation.car();
        let outcome = self.simulation.drive(command);
        self.blocked = matches!(
            outcome,
            Outcome::Blocked | Outcome::BlockedByBoundary | Outcome::AtEdge
        )
        .then_some(outcome);
        let car = self.simulation.car();

        self.redraw();

        Some(((previous.row, previous.col), (car.row, car.col)))
    }

    /// Replaces the map with one of the same size where every cell is unknown. The car stays
    /// where it is.
    pub(crate) fn new_map(&mut self) {
        self.simulation.new_map();
        let car = self.simulation.car();
        self.simulation.set_cell(car.row, car.col, Cell::Free);
        self.reset_navigator_map(car);
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
        let start = self.simulation.start();
        self.reset_navigator_map(start);
        self.show_replaced_map();
        Ok(())
    }

    /// Resizes the map and moves the car to its new start. Cells keep their place from the
    /// top-left corner and new cells are unknown. `settings` must come from `MapSettings`'s
    /// constructors, which keep the start on the map.
    pub(crate) fn apply_settings(&mut self, settings: MapSettings) {
        self.simulation
            .resize(settings.rows, settings.cols, settings.start);
        self.reset_navigator_map(settings.start);
        self.show_replaced_map();
    }

    fn reset_navigator_map(&mut self, anchor: Car) {
        self.nav_origin = anchor;
        self.session_start = anchor;
        self.navigator.reset_map();
        self.navigator
            .synchronize(Position { row: 0, col: 0 }, anchor.direction);
        self.nav_stale = false;
    }

    pub(crate) fn navigator_state(&self) -> ExecutionState {
        self.navigator.state()
    }

    pub(crate) fn pending_inspection(&self) -> Option<Position> {
        self.navigator.pending_inspection()
    }

    pub(crate) fn navigator_start(&mut self) -> Result<(), String> {
        if self.editing {
            return Err("Turn off Map → Edit before starting navigation.".to_owned());
        }
        if !matches!(
            self.navigator.state(),
            ExecutionState::Idle | ExecutionState::Paused | ExecutionState::Inspecting
        ) {
            return Err("Reset the Navigator session before starting again.".to_owned());
        }
        if self.nav_stale {
            return Err(
                "The simulator map changed. Choose Navigator → Reset Map before continuing."
                    .to_owned(),
            );
        }
        let car = self.simulation.car();
        if self.relative_position(car) != self.navigator.position()
            || car.direction != self.navigator.heading()
        {
            self.reset_navigator_map(car);
        }
        self.session_start = car;
        self.navigator.start();
        Ok(())
    }

    pub(crate) fn navigator_pause(&mut self) {
        self.navigator.pause();
    }

    pub(crate) fn navigator_step(&mut self) -> NavigatorEvent {
        if self.editing || self.nav_stale {
            return NavigatorEvent::UnableToProgress;
        }
        let event = self.navigator.step();
        match event {
            NavigatorEvent::Drive(command) => {
                let outcome = self.simulation.drive(command);
                let car = self.simulation.car();
                self.navigator.record_drive(
                    command,
                    outcome,
                    self.relative_position(car),
                    car.direction,
                );
                self.blocked = matches!(
                    outcome,
                    Outcome::Blocked | Outcome::BlockedByBoundary | Outcome::AtEdge
                )
                .then_some(outcome);
                self.redraw();
                if matches!(
                    outcome,
                    Outcome::Blocked | Outcome::BlockedByBoundary | Outcome::AtEdge
                ) {
                    NavigatorEvent::UnableToProgress
                } else {
                    event
                }
            }
            NavigatorEvent::RequestInspection(position) => {
                let Some((row, col)) = self.absolute_position(position) else {
                    self.navigator
                        .classify(position, DiscoveredTerrain::Boundary);
                    self.redraw();
                    return NavigatorEvent::WaitForInspection;
                };
                if row >= self.simulation.grid().rows || col >= self.simulation.grid().cols {
                    self.navigator
                        .classify(position, DiscoveredTerrain::Boundary);
                    self.redraw();
                    NavigatorEvent::WaitForInspection
                } else {
                    event
                }
            }
            _ => event,
        }
    }

    pub(crate) fn resolve_inspection(&mut self, terrain: DiscoveredTerrain) -> bool {
        let Some(position) = self.navigator.pending_inspection() else {
            return false;
        };
        let Some((row, col)) = self.absolute_position(position) else {
            let classified = self
                .navigator
                .classify(position, DiscoveredTerrain::Boundary);
            self.redraw();
            return classified;
        };
        if row >= self.simulation.grid().rows || col >= self.simulation.grid().cols {
            let classified = self
                .navigator
                .classify(position, DiscoveredTerrain::Boundary);
            self.redraw();
            return classified;
        }
        let cell = match terrain {
            DiscoveredTerrain::Free => Cell::Free,
            DiscoveredTerrain::Obstacle => Cell::Obstacle,
            DiscoveredTerrain::Boundary => Cell::Boundary,
            DiscoveredTerrain::Unknown => return false,
        };
        let Some(actual) = self.simulation.classify_unknown(row, col, cell) else {
            return false;
        };
        let discovered = match actual {
            Cell::Free => DiscoveredTerrain::Free,
            Cell::Obstacle => DiscoveredTerrain::Obstacle,
            Cell::Boundary => DiscoveredTerrain::Boundary,
            Cell::Unknown => unreachable!("classification replaces unknown terrain"),
        };
        let classified = self.navigator.classify(position, discovered);
        if classified {
            self.redraw();
        }
        classified
    }

    pub(crate) fn reset_navigator_session(&mut self) {
        self.simulation.set_car(self.session_start);
        self.nav_origin = self.session_start;
        self.navigator
            .reset_session(Position { row: 0, col: 0 }, self.session_start.direction);
        self.blocked = None;
        self.redraw();
    }

    pub(crate) fn reset_navigator_map_from_ui(&mut self) {
        let start = self.session_start;
        self.simulation.set_car(start);
        self.nav_origin = start;
        self.navigator.reset_map();
        self.navigator
            .synchronize(Position { row: 0, col: 0 }, start.direction);
        self.nav_stale = false;
        self.blocked = None;
        self.redraw();
    }

    pub(crate) fn navigator_progress(&self) -> (usize, usize) {
        let total = self.navigator.actions().len();
        let completed = self
            .navigator
            .actions()
            .values()
            .filter(|done| **done)
            .count();
        (completed, total)
    }

    #[cfg(test)]
    pub(crate) fn navigator_terrain(
        &self,
    ) -> &std::collections::HashMap<Position, DiscoveredTerrain> {
        self.navigator.terrain()
    }

    #[cfg(test)]
    pub(crate) fn navigator_travel(&self) -> &[Position] {
        self.navigator.travel()
    }

    #[cfg(test)]
    pub(crate) fn navigator_is_stale(&self) -> bool {
        self.nav_stale
    }

    fn relative_position(&self, car: Car) -> Position {
        Position {
            row: car.row as i32 - self.nav_origin.row as i32,
            col: car.col as i32 - self.nav_origin.col as i32,
        }
    }

    fn absolute_position(&self, position: Position) -> Option<(usize, usize)> {
        Some((
            usize::try_from(self.nav_origin.row as i32 + position.row).ok()?,
            usize::try_from(self.nav_origin.col as i32 + position.col).ok()?,
        ))
    }

    /// Redraws the whole map after it was replaced or resized. The last move no longer counts as
    /// blocked.
    fn show_replaced_map(&mut self) {
        self.blocked = None;
        self.redraw();
    }

    /// Redraws the whole map, resizing the buffer to the size of the map.
    fn redraw(&mut self) {
        let pixels = self.pixel_width() * self.pixel_height();
        let mut buffer = vec![0; pixels];
        self.draw_into(&mut buffer);
        self.buffer = buffer;
    }

    fn draw_into(&self, buffer: &mut [u32]) {
        let discovered_grid = self.discovered_view.then(|| {
            let source = self.simulation.grid();
            let mut grid = Grid::unknown(source.rows, source.cols);
            for (position, terrain) in self.navigator.terrain() {
                if let Some((row, col)) = self.absolute_position(*position) {
                    if let Some(cell) = grid.cells.get_mut(row).and_then(|row| row.get_mut(col)) {
                        *cell = match terrain {
                            DiscoveredTerrain::Unknown => Cell::Unknown,
                            DiscoveredTerrain::Free => Cell::Free,
                            DiscoveredTerrain::Obstacle => Cell::Obstacle,
                            DiscoveredTerrain::Boundary => Cell::Boundary,
                        };
                    }
                }
            }
            grid
        });
        let grid = discovered_grid
            .as_ref()
            .unwrap_or_else(|| self.simulation.grid());
        let car = if self.discovered_view {
            let position = self
                .absolute_position(self.navigator.position())
                .unwrap_or((self.nav_origin.row, self.nav_origin.col));
            Car {
                row: position.0,
                col: position.1,
                direction: self.navigator.heading(),
            }
        } else {
            self.simulation.car()
        };
        let actions = self
            .navigator
            .actions()
            .iter()
            .filter(|(_, completed)| **completed)
            .filter_map(|(position, _)| self.absolute_position(*position))
            .filter(|(row, col)| *row < grid.rows && *col < grid.cols)
            .collect::<Vec<_>>();
        let traveled = self
            .navigator
            .travel()
            .iter()
            .filter_map(|position| self.absolute_position(*position))
            .filter(|(row, col)| *row < grid.rows && *col < grid.cols)
            .collect::<Vec<_>>();
        render::draw_with_navigation(buffer, grid, car, self.editing, &actions, &traveled);
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing
    }

    /// Turns edit mode on or off. The grid lines change color so the mode shows on the map.
    pub(crate) fn set_editing(&mut self, editing: bool) {
        if editing
            && !matches!(
                self.navigator.state(),
                ExecutionState::Idle | ExecutionState::Paused
            )
        {
            return;
        }
        self.editing = editing;
        if editing {
            self.discovered_view = false;
        }
        if !editing {
            self.edit_brush = None;
        }
        self.blocked = None;
        self.redraw();
    }

    /// Applies a mouse button to a cell while the map is in edit mode: the left button marks an
    /// obstacle and the right button marks unknown terrain. Returns whether the cell changed.
    pub(crate) fn edit_cell(&mut self, row: usize, col: usize, button: MouseButton) -> bool {
        let cell = self.edit_brush.unwrap_or(match button {
            MouseButton::Left => Cell::Obstacle,
            MouseButton::Right => Cell::Unknown,
        });
        let car = self.simulation.car();
        if !self.editing
            || (row, col) == (car.row, car.col)
            || !self.simulation.set_cell(row, col, cell)
        {
            return false;
        }
        self.nav_stale = true;
        self.redraw();
        true
    }

    pub(crate) fn set_edit_brush(&mut self, cell: Option<Cell>) {
        if self.editing {
            self.edit_brush = cell;
        }
    }

    pub(crate) fn toggle_discovered_view(&mut self) {
        self.discovered_view = !self.discovered_view;
        if self.editing {
            self.editing = false;
        }
        self.redraw();
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn is_discovered_view(&self) -> bool {
        self.discovered_view
    }

    pub(crate) fn status_text(&self) -> String {
        let mode = if self.nav_stale {
            "Map edited — Reset Map before navigation".to_owned()
        } else if self.editing {
            let brush = match self.edit_brush {
                Some(Cell::Free) => "Free",
                Some(Cell::Boundary) => "Boundary",
                _ => "left click: obstacle, right click: unknown",
            };
            format!("Editing ({brush}; the car is parked)")
        } else if self.navigator.state() != ExecutionState::Idle {
            let (completed, total) = self.navigator_progress();
            format!(
                "Navigator {:?} — Actions: {completed}/{total} (provisional)",
                self.navigator.state()
            )
        } else if self.blocked == Some(Outcome::Blocked) {
            "Blocked by an obstacle".to_owned()
        } else if self.blocked == Some(Outcome::AtEdge) {
            "Blocked at the map boundary".to_owned()
        } else if self.blocked == Some(Outcome::BlockedByBoundary) {
            "Blocked by a boundary cell".to_owned()
        } else {
            "Ready".to_owned()
        };
        let car = self.simulation.car();
        let heading = HEADINGS
            .iter()
            .find(|&&(_, direction)| direction == car.direction)
            .map_or("", |&(name, _)| name);
        let view = if self.discovered_view {
            "Discovered map | "
        } else {
            ""
        };
        format!(
            "{view}{mode}  |  Row: {}  Column: {}  Heading: {heading}",
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
    use crate::navigator::{Event as NavigatorEvent, ExecutionState, Position, Terrain};
    use crate::simulator::{Car, Cell, Command, Direction, MAX_GRID_SIZE};
    use crate::ui::config::{BORDER, CELL_SIZE, EDIT_BORDER};
    /// A state whose map is blank, so tests do not depend on the random sample map.
    fn blank_state() -> AppState {
        let mut state = AppState::new();
        state.new_map();
        state
    }

    fn expected_buffer(state: &AppState) -> Vec<u32> {
        let mut expected = vec![0; state.pixel_width() * state.pixel_height()];
        state.draw_into(&mut expected);
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
        assert!(cells.iter().enumerate().all(|(row, cells)| {
            cells.iter().enumerate().all(|(col, cell)| {
                (row, col) == (car.row, car.col) && *cell == Cell::Free
                    || (row, col) != (car.row, car.col) && *cell == Cell::Unknown
            })
        }));
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
        assert!(
            state
                .status_text()
                .contains("Row: 2  Column: 3  Heading: Right")
        );
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

        assert!(!state.edit_cell(row, col, MouseButton::Left));
        assert_eq!(state.simulation.grid().cells[row][col], Cell::Free);
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
        assert_eq!(
            cell_at(left + width - 0.1, top + height - 0.1),
            Some((3, 7))
        );
        assert_eq!(cell_at(left - 0.1, top), None);
        assert_eq!(cell_at(left + width, top), None);
        assert_eq!(cell_at(left, top + height), None);
    }

    #[test]
    fn navigator_inspection_updates_only_the_requested_simulator_cell() {
        let mut state = blank_state();
        state.navigator_start().unwrap();
        assert_eq!(
            state.navigator_step(),
            NavigatorEvent::RequestInspection(Position { row: -1, col: 0 })
        );
        assert!(state.resolve_inspection(Terrain::Free));
        assert_eq!(state.simulation.grid().cells[18][10], Cell::Free);
        assert_eq!(
            state.navigator_terrain().get(&Position { row: -1, col: 0 }),
            Some(&Terrain::Free)
        );
        assert_eq!(state.navigator_progress(), (1, 2));
        assert_eq!(state.navigator_state(), ExecutionState::Paused);
        assert!(state.navigator_travel().len() == 1);
    }

    #[test]
    fn manual_controls_stay_disabled_while_inspection_is_pending() {
        let mut state = blank_state();
        state.navigator_start().unwrap();
        state.navigator_step();
        let car = state.simulation.car();
        assert_eq!(state.navigator_state(), ExecutionState::Inspecting);
        assert_eq!(state.drive(Command::Forward), None);
        assert_eq!(state.simulation.car(), car);

        state.navigator_pause();
        assert_eq!(state.navigator_state(), ExecutionState::Paused);
        state.navigator_start().unwrap();
        assert_eq!(state.navigator_step(), NavigatorEvent::WaitForInspection);
    }

    #[test]
    fn simulator_edge_is_learned_as_boundary_without_an_inspection_dialog() {
        let mut state = AppState::new();
        state.apply_settings(MapSettings::from_values(2, 2, 1, 1, 0).unwrap());
        state.navigator_start().unwrap();
        assert_eq!(state.navigator_step(), NavigatorEvent::WaitForInspection);
        assert_eq!(
            state.navigator_terrain().get(&Position { row: -1, col: 0 }),
            Some(&Terrain::Boundary)
        );
        assert_eq!(state.pending_inspection(), None);
    }

    #[test]
    fn manual_edge_collision_is_reported_in_the_status() {
        let mut state = AppState::new();
        state.apply_settings(MapSettings::from_values(2, 2, 1, 1, 0).unwrap());
        state.drive(Command::Forward);
        assert!(
            state
                .status_text()
                .starts_with("Blocked at the map boundary")
        );
    }

    #[test]
    fn editing_invalidates_navigation_until_map_reset() {
        let mut state = blank_state();
        state.set_editing(true);
        assert!(state.edit_cell(0, 0, MouseButton::Left));
        assert!(state.navigator_is_stale());
        state.set_editing(false);
        assert!(state.navigator_start().unwrap_err().contains("Reset Map"));

        state.reset_navigator_map_from_ui();
        assert!(!state.navigator_is_stale());
        assert!(state.navigator_start().is_ok());
    }

    #[test]
    fn discovered_view_does_not_render_hidden_simulator_terrain() {
        let mut state = AppState::new();
        let hidden = state
            .simulation
            .grid()
            .cells
            .iter()
            .enumerate()
            .flat_map(|(row, cells)| {
                cells
                    .iter()
                    .enumerate()
                    .map(move |(col, cell)| (row, col, cell))
            })
            .find(|(row, col, cell)| {
                **cell == Cell::Obstacle
                    && (*row, *col) != (state.simulation.car().row, state.simulation.car().col)
            })
            .map(|(row, col, _)| (row, col))
            .unwrap();
        state.toggle_discovered_view();
        let index = (hidden.0 * CELL_SIZE + 4) * state.pixel_width() + hidden.1 * CELL_SIZE + 4;
        assert_eq!(state.buffer[index], crate::ui::config::UNKNOWN);
    }

    #[test]
    fn navigator_completes_an_open_grid_without_traveling_into_unknown_cells() {
        let mut state = AppState::new();
        state.apply_settings(MapSettings::from_values(2, 2, 1, 1, 1).unwrap());
        state.new_map();
        state.navigator_start().unwrap();

        let mut completed = false;
        for _ in 0..200 {
            match state.navigator_step() {
                NavigatorEvent::RequestInspection(_) => {
                    assert!(state.resolve_inspection(Terrain::Free));
                }
                NavigatorEvent::MissionCompleted => {
                    completed = true;
                    break;
                }
                NavigatorEvent::UnableToProgress => {
                    panic!("the open grid should be navigable")
                }
                NavigatorEvent::WaitForInspection | NavigatorEvent::Drive(_) => {}
            }
            if state.navigator_state() != ExecutionState::Running {
                state.navigator_start().unwrap();
            }
        }

        assert!(
            completed,
            "the mission should finish after exploring the open grid"
        );
        assert_eq!(state.navigator_progress(), (4, 4));
        assert!(state.navigator_travel().len() >= 4);
    }

    fn assert_cells_changed_only(
        state: &AppState,
        before: &[u32],
        changed_cells: &[(usize, usize)],
    ) {
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
