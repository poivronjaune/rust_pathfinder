use serde::{Deserialize, Serialize};

/// Heading of the car. The serialized names are part of the map file format
/// (`schemas/map.schema.json`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Car {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) direction: Direction,
}

impl Car {
    /// Default start of a map with the given size: the bottom-center cell, facing up.
    pub(crate) fn start_for(rows: usize, cols: usize) -> Self {
        Self {
            row: rows.saturating_sub(1),
            col: cols / 2,
            direction: Direction::Up,
        }
    }

    pub(crate) fn turn_left(&mut self) {
        self.direction = match self.direction {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        };
    }

    pub(crate) fn turn_right(&mut self) {
        self.direction = match self.direction {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        };
    }

    /// Cell one step forward (or backward) along the heading, or `None` past the edge of a map
    /// with `rows` rows and `cols` columns.
    pub(crate) fn next_position(
        &self,
        forward: bool,
        rows: usize,
        cols: usize,
    ) -> Option<(usize, usize)> {
        let (row_step, col_step) = match self.direction {
            Direction::Up => (-1, 0),
            Direction::Right => (0, 1),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
        };
        let sign = if forward { 1 } else { -1 };
        let row = self.row.checked_add_signed(row_step * sign)?;
        let col = self.col.checked_add_signed(col_step * sign)?;
        (row < rows && col < cols).then_some((row, col))
    }
}

#[cfg(test)]
mod tests {
    use super::{Car, Direction};

    const ROWS: usize = 20;
    const COLS: usize = 12;

    #[test]
    fn default_start_is_bottom_center_facing_up() {
        let car = Car::start_for(ROWS, COLS);

        assert_eq!(car.row, ROWS - 1);
        assert_eq!(car.col, COLS / 2);
        assert_eq!(car.direction, Direction::Up);
    }

    #[test]
    fn forward_and_backward_follow_heading() {
        let mut car = Car::start_for(ROWS, COLS);
        car.direction = Direction::Right;

        assert_eq!(car.next_position(true, ROWS, COLS), Some((ROWS - 1, COLS / 2 + 1)));
        assert_eq!(car.next_position(false, ROWS, COLS), Some((ROWS - 1, COLS / 2 - 1)));
        car.direction = Direction::Up;
        assert_eq!(car.next_position(true, ROWS, COLS), Some((ROWS - 2, COLS / 2)));
    }

    #[test]
    fn turns_rotate_in_place() {
        let mut car = Car::start_for(ROWS, COLS);
        let position = (car.row, car.col);

        car.turn_left();
        assert_eq!(car.direction, Direction::Left);
        car.turn_right();
        assert_eq!(car.direction, Direction::Up);
        assert_eq!((car.row, car.col), position);
    }

    #[test]
    fn movement_stays_within_grid_bounds() {
        let mut car = Car::start_for(ROWS, COLS);
        assert_eq!(car.next_position(false, ROWS, COLS), None);

        car.col = 0;
        car.direction = Direction::Left;
        assert_eq!(car.next_position(true, ROWS, COLS), None);

        car.col = COLS - 1;
        car.direction = Direction::Right;
        assert_eq!(car.next_position(true, ROWS, COLS), None);

        car.row = 0;
        car.direction = Direction::Up;
        assert_eq!(car.next_position(true, ROWS, COLS), None);
    }
}
