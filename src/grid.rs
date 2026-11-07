use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::{CLEAR_RADIUS, GRID_SIZE};

/// Terrain of one grid cell. The serialized names are part of the map file format
/// (`schemas/map.schema.json`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Cell {
    Free,
    Obstacle,
    Unknown,
}

pub(crate) type Map = [[Cell; crate::config::GRID_SIZE]; crate::config::GRID_SIZE];

pub(crate) struct Grid {
    pub(crate) cells: Map,
}

/// Footprint ranges (inclusive) in cells: (count, rows, cols).
type Feature = (usize, (usize, usize), (usize, usize));

const PATIO: Feature = (1, (3, 5), (4, 6));
const TABLES: Feature = (2, (2, 2), (2, 3));
const ROCKS: (usize, usize) = (5, 8);
const ROCK: Feature = (1, (1, 1), (1, 2));
const MAX_PLACEMENT_ATTEMPTS: usize = 50;

impl Grid {
    /// A blank map where every cell is still unknown.
    pub(crate) fn unknown() -> Self {
        Self {
            cells: [[Cell::Unknown; GRID_SIZE]; GRID_SIZE],
        }
    }

    pub(crate) fn backyard(car_row: usize, car_col: usize) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as u64);
        Self::backyard_with_seed(car_row, car_col, seed)
    }

    pub(crate) fn backyard_with_seed(car_row: usize, car_col: usize, seed: u64) -> Self {
        let mut rng = Rng(seed);
        let mut grid = Self::unknown();
        // r² + r gives a rounder discrete circle than r².
        let near_car = |row: usize, col: usize| {
            let (dr, dc) = (row.abs_diff(car_row), col.abs_diff(car_col));
            dr * dr + dc * dc <= CLEAR_RADIUS * CLEAR_RADIUS + CLEAR_RADIUS
        };

        grid.scatter(&mut rng, PATIO, &near_car);
        grid.scatter(&mut rng, TABLES, &near_car);
        let rock_count = rng.range(ROCKS.0, ROCKS.1);
        grid.scatter(&mut rng, (rock_count, ROCK.1, ROCK.2), &near_car);

        for (row, cells) in grid.cells.iter_mut().enumerate() {
            for (col, cell) in cells.iter_mut().enumerate() {
                if near_car(row, col) {
                    *cell = Cell::Free;
                }
            }
        }

        grid
    }

    fn scatter(
        &mut self,
        rng: &mut Rng,
        feature: Feature,
        keep_clear: impl Fn(usize, usize) -> bool,
    ) {
        let (count, rows, cols) = feature;
        for _ in 0..count {
            for _ in 0..MAX_PLACEMENT_ATTEMPTS {
                let (height, width) = (rng.range(rows.0, rows.1), rng.range(cols.0, cols.1));
                let top = rng.range(0, GRID_SIZE - height);
                let left = rng.range(0, GRID_SIZE - width);

                // Keep a one-cell gap so separate objects stay distinguishable.
                let touches_obstacle = (top.saturating_sub(1)..(top + height + 1).min(GRID_SIZE))
                    .any(|row| {
                        (left.saturating_sub(1)..(left + width + 1).min(GRID_SIZE))
                            .any(|col| self.cells[row][col] == Cell::Obstacle)
                    });
                let blocks_car = (top..top + height)
                    .any(|row| (left..left + width).any(|col| keep_clear(row, col)));
                if touches_obstacle || blocks_car {
                    continue;
                }

                for row in &mut self.cells[top..top + height] {
                    row[left..left + width].fill(Cell::Obstacle);
                }
                break;
            }
        }
    }
}

/// SplitMix64: tiny, dependency-free and good enough for map generation.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn range(&mut self, min: usize, max_inclusive: usize) -> usize {
        min + (self.next_u64() % (max_inclusive - min + 1) as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, Grid};
    use crate::config::{CLEAR_RADIUS, GRID_SIZE};

    #[test]
    fn backyard_is_mostly_unknown_with_obstacles_and_a_clear_car_zone() {
        let (car_row, car_col) = (GRID_SIZE - 1, GRID_SIZE / 2);
        for seed in 0..100 {
            let grid = Grid::backyard_with_seed(car_row, car_col, seed);
            let count = |kind: Cell| grid.cells.iter().flatten().filter(|&&c| c == kind).count();

            assert!(count(Cell::Unknown) > GRID_SIZE * GRID_SIZE / 2);
            assert!(count(Cell::Obstacle) > 0);
            for row in car_row - CLEAR_RADIUS..=car_row {
                assert_eq!(grid.cells[row][car_col], Cell::Free);
            }
            assert_eq!(grid.cells[car_row][car_col - CLEAR_RADIUS], Cell::Free);
            assert_eq!(grid.cells[car_row][car_col + CLEAR_RADIUS], Cell::Free);
        }
    }

    #[test]
    fn same_seed_gives_same_backyard() {
        let a = Grid::backyard_with_seed(5, 5, 42);
        let b = Grid::backyard_with_seed(5, 5, 42);
        assert_eq!(a.cells, b.cells);
    }

    #[test]
    fn unknown_grid_has_only_unknown_cells() {
        let cells = Grid::unknown().cells;
        assert!(cells.iter().flatten().all(|&cell| cell == Cell::Unknown));
    }
}


