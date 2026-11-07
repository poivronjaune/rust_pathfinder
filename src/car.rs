use crate::config::GRID_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Default for Car {
    fn default() -> Self {
        Self {
            row: GRID_SIZE - 1,
            col: GRID_SIZE / 2,
            direction: Direction::Up,
        }
    }
}

impl Car {
    pub(crate) fn move_forward(&mut self) {
        self.move_in(self.direction, true);
    }

    pub(crate) fn move_backward(&mut self) {
        self.move_in(self.direction, false);
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

    fn move_in(&mut self, direction: Direction, forward: bool) {
        match (direction, forward) {
            (Direction::Up, true) | (Direction::Down, false) if self.row > 0 => self.row -= 1,
            (Direction::Right, true) | (Direction::Left, false) if self.col < GRID_SIZE - 1 => {
                self.col += 1
            }
            (Direction::Down, true) | (Direction::Up, false) if self.row < GRID_SIZE - 1 => {
                self.row += 1
            }
            (Direction::Left, true) | (Direction::Right, false) if self.col > 0 => self.col -= 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Car, Direction};
    use crate::config::GRID_SIZE;

    #[test]
    fn default_car_starts_at_bottom_center_facing_up() {
        let car = Car::default();

        assert_eq!(car.row, GRID_SIZE - 1);
        assert_eq!(car.col, GRID_SIZE / 2);
        assert_eq!(car.direction, Direction::Up);
    }

    #[test]
    fn forward_and_backward_follow_heading() {
        let mut car = Car::default();
        car.direction = Direction::Right;

        car.move_forward();
        assert_eq!((car.row, car.col), (GRID_SIZE - 1, GRID_SIZE / 2 + 1));

        car.move_backward();
        assert_eq!((car.row, car.col), (GRID_SIZE - 1, GRID_SIZE / 2));
        assert_eq!(car.direction, Direction::Right);
    }

    #[test]
    fn turns_rotate_in_place() {
        let mut car = Car::default();
        let position = (car.row, car.col);

        car.turn_left();
        assert_eq!(car.direction, Direction::Left);
        car.turn_right();
        assert_eq!(car.direction, Direction::Up);
        assert_eq!((car.row, car.col), position);
    }

    #[test]
    fn movement_stays_within_grid_bounds() {
        let mut car = Car::default();

        car.move_backward();
        assert_eq!(car.row, GRID_SIZE - 1);

        car.col = 0;
        car.direction = Direction::Left;
        car.move_forward();
        assert_eq!(car.col, 0);
    }
}
