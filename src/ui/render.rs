use crate::simulator::{Car, Cell, Direction, Grid};
use crate::ui::config::{
    ACTION_MARK, BORDER, BORDER_WIDTH, BOUNDARY, CAR_BLACK, CAR_RED, CELL_SIZE, EDIT_BORDER, FREE,
    OBSTACLE, TRAVEL_MARK, UNKNOWN,
};

/// Colors explained by the legend, in display order.
pub(crate) const LEGEND: [(&str, u32); 7] = [
    ("Free", FREE),
    ("Obstacle", OBSTACLE),
    ("Unknown", UNKNOWN),
    ("Boundary", BOUNDARY),
    ("Vehicle", CAR_RED),
    ("Action complete", ACTION_MARK),
    ("Traveled path", TRAVEL_MARK),
];

/// Draws the whole map into `buffer`, which holds `grid.rows * CELL_SIZE` lines of
/// `grid.cols * CELL_SIZE` pixels. In edit mode the grid lines use the edit color.
pub(crate) fn draw_with_navigation(
    buffer: &mut [u32],
    grid: &Grid,
    car: Car,
    editing: bool,
    action_cells: &[(usize, usize)],
    traveled_cells: &[(usize, usize)],
) {
    let width = grid.cols * CELL_SIZE;
    for (row, cells) in grid.cells.iter().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            draw_cell(buffer, width, col, row, *cell, editing);
        }
    }
    for &(row, col) in traveled_cells {
        draw_cell_outline(buffer, width, col, row, TRAVEL_MARK);
    }
    for &(row, col) in action_cells {
        draw_action_mark(buffer, width, col, row);
    }
    draw_car_cell(buffer, width, car);
}

fn draw_cell_outline(buffer: &mut [u32], width: usize, col: usize, row: usize, color: u32) {
    let x = col * CELL_SIZE;
    let y = row * CELL_SIZE;
    let inset = 2;
    for offset in inset..CELL_SIZE - inset {
        for thickness in 0..2 {
            buffer[(y + inset + thickness) * width + x + offset] = color;
            buffer[(y + CELL_SIZE - inset - 1 - thickness) * width + x + offset] = color;
            buffer[(y + offset) * width + x + inset + thickness] = color;
            buffer[(y + offset) * width + x + CELL_SIZE - inset - 1 - thickness] = color;
        }
    }
}

fn draw_action_mark(buffer: &mut [u32], width: usize, col: usize, row: usize) {
    let center = (
        col * CELL_SIZE + CELL_SIZE / 2,
        row * CELL_SIZE + CELL_SIZE / 2,
    );
    for dy in 0..4 {
        for dx in 0..4 {
            buffer[(center.1 + dy - 2) * width + center.0 + dx - 2] = ACTION_MARK;
        }
    }
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
        Cell::Boundary => BOUNDARY,
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
    use crate::simulator::Cell;

    #[test]
    fn legend_explains_every_terrain_color() {
        for cell in [Cell::Free, Cell::Obstacle, Cell::Unknown, Cell::Boundary] {
            assert!(LEGEND.iter().any(|&(_, color)| color == cell_color(cell)));
        }
    }
}
