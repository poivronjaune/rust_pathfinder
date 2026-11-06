#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cell {
    Free,
    Obstacle,
    Unknown,
}

pub(crate) type Map = [[Cell; crate::config::GRID_SIZE]; crate::config::GRID_SIZE];

pub(crate) struct Grid {
    pub(crate) cells: Map,
}

impl Grid {
    pub(crate) fn new() -> Self {
        let cells = std::array::from_fn(|row| {
            std::array::from_fn(|col| {
                let is_top_left_zone =
                    row < crate::config::GRID_SIZE / 3 && col < crate::config::GRID_SIZE / 3;
                let is_obstacle = (row + col) % 6 == 0
                    || (row > crate::config::GRID_SIZE / 2 && col % 5 == 0 && col > 2)
                    || (row > crate::config::GRID_SIZE / 3
                        && row < crate::config::GRID_SIZE * 2 / 3
                        && col == crate::config::GRID_SIZE / 2);
                let is_unknown = (row + col) % 9 == 0 && !is_obstacle && !is_top_left_zone;

                if is_obstacle {
                    Cell::Obstacle
                } else if is_unknown {
                    Cell::Unknown
                } else {
                    Cell::Free
                }
            })
        });

        Self { cells }
    }
}
