use crate::car::Car;
#[cfg(target_os = "windows")]
use crate::config::CELL_SIZE;
use crate::config::{HEIGHT, WIDTH};
use crate::grid::Grid;
use crate::render;

#[derive(Clone, Copy)]
pub(crate) enum VehicleKey {
    Forward,
    Reverse,
    TurnLeft,
    TurnRight,
}

pub(crate) struct AppState {
    pub(crate) buffer: Vec<u32>,
    grid: Grid,
    car: Car,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let grid = Grid::backyard(car.row, car.col);
        let car = Car::default();
        let mut buffer = vec![0; WIDTH * HEIGHT];
        render::draw(&mut buffer, &grid, car);
        Self { buffer, grid, car }
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
        );
        render::draw_car_cell(&mut self.buffer, self.car);

        (previous, current)
    }

    pub(crate) fn status_text(&self) -> String {
        let direction = match self.car.direction {
            crate::car::Direction::Up => "Up",
            crate::car::Direction::Right => "Right",
            crate::car::Direction::Down => "Down",
            crate::car::Direction::Left => "Left",
        };
        format!(
            "Ready  |  Row: {}  Column: {}  Heading: {direction}",
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
}

#[cfg(test)]
mod tests {
    use super::{AppState, VehicleKey};
    use crate::config::{CELL_SIZE, WIDTH};

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
