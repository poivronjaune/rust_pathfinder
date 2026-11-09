use crate::car::{Car, Direction};
use crate::config::{
    BORDER, BORDER_WIDTH, CAR_BLACK, CAR_RED, CELL_SIZE, EDIT_BORDER, FREE, OBSTACLE, UNKNOWN,
};
use crate::grid::{Cell, Grid};

/// Colors explained by the legend, in display order.
pub(crate) const LEGEND: [(&str, u32); 4] = [
    ("Free", FREE),
    ("Obstacle", OBSTACLE),
    ("Unknown", UNKNOWN),
    ("Vehicle", CAR_RED),
];

/// Draws the whole map into `buffer`, which holds `grid.rows * CELL_SIZE` lines of
/// `grid.cols * CELL_SIZE` pixels. In edit mode the grid lines use the edit color.
pub(crate) fn draw(buffer: &mut [u32], grid: &Grid, car: Car, editing: bool) {
    let width = grid.cols * CELL_SIZE;
    for (row, cells) in grid.cells.iter().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            draw_cell(buffer, width, col, row, *cell, editing);
        }
    }
    draw_car_cell(buffer, width, car);
}

/// Draws one cell into a buffer whose lines are `width` pixels long.
pub(crate) fn draw_cell(
    buffer: &mut [u32],
    width: usize,
    col: usize,
    row: usize,
    cell: Cell,
    editing: bool,
) {
    let fill = cell_color(cell);
    let border = if editing { EDIT_BORDER } else { BORDER };

    let x0 = col * CELL_SIZE;
    let y0 = row * CELL_SIZE;

    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            let on_border = !(BORDER_WIDTH..CELL_SIZE - BORDER_WIDTH).contains(&dx)
                || !(BORDER_WIDTH..CELL_SIZE - BORDER_WIDTH).contains(&dy);

            let color = if on_border { border } else { fill };
            buffer[(y0 + dy) * width + (x0 + dx)] = color;
        }
    }
}

fn cell_color(cell: Cell) -> u32 {
    match cell {
        Cell::Free => FREE,
        Cell::Obstacle => OBSTACLE,
        Cell::Unknown => UNKNOWN,
    }
}

/// Draws the car over its cell in a buffer whose lines are `width` pixels long.
pub(crate) fn draw_car_cell(buffer: &mut [u32], width: usize, car: Car) {
    let x0 = car.col * CELL_SIZE;
    let y0 = car.row * CELL_SIZE;

    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            buffer[(y0 + dy) * width + (x0 + dx)] = CAR_RED;
        }
    }

    match car.direction {
        Direction::Up => {
            for row in 0..(CELL_SIZE / 2) {
                let y = y0 + row;
                let triangle = (row * 2 + 1).min(CELL_SIZE - 2);
                let x_start = x0 + CELL_SIZE / 2 - triangle / 2;
                for x in x_start..(x_start + triangle) {
                    buffer[y * width + x] = CAR_BLACK;
                }
            }
        }
        Direction::Right => {
            for col in 0..(CELL_SIZE / 2) {
                let x = x0 + CELL_SIZE - 1 - col;
                let triangle = (col * 2 + 1).min(CELL_SIZE - 2);
                let y_start = y0 + CELL_SIZE / 2 - triangle / 2;
                for y in y_start..(y_start + triangle) {
                    buffer[y * width + x] = CAR_BLACK;
                }
            }
        }
        Direction::Down => {
            for row in 0..(CELL_SIZE / 2) {
                let y = y0 + CELL_SIZE - 1 - row;
                let triangle = (row * 2 + 1).min(CELL_SIZE - 2);
                let x_start = x0 + CELL_SIZE / 2 - triangle / 2;
                for x in x_start..(x_start + triangle) {
                    buffer[y * width + x] = CAR_BLACK;
                }
            }
        }
        Direction::Left => {
            for col in 0..(CELL_SIZE / 2) {
                let x = x0 + col;
                let triangle = (col * 2 + 1).min(CELL_SIZE - 2);
                let y_start = y0 + CELL_SIZE / 2 - triangle / 2;
                for y in y_start..(y_start + triangle) {
                    buffer[y * width + x] = CAR_BLACK;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LEGEND, cell_color};
    use crate::grid::Cell;

    #[test]
    fn legend_explains_every_terrain_color() {
        for cell in [Cell::Free, Cell::Obstacle, Cell::Unknown] {
            assert!(LEGEND.iter().any(|&(_, color)| color == cell_color(cell)));
        }
    }
}
