use crate::car::{Car, Direction};
use crate::config::{
    BORDER, BORDER_WIDTH, CAR_BLACK, CAR_RED, CELL_SIZE, FREE, OBSTACLE, UNKNOWN, WIDTH,
};
use crate::grid::{Cell, Grid};

pub(crate) fn draw(buffer: &mut [u32], grid: &Grid, car: Car) {
    for (row, cells) in grid.cells.iter().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            draw_cell(buffer, col, row, *cell);
        }
    }
    draw_car_cell(buffer, car);
}

pub(crate) fn draw_cell(buffer: &mut [u32], col: usize, row: usize, cell: Cell) {
    let fill = match cell {
        Cell::Free => FREE,
        Cell::Obstacle => OBSTACLE,
        Cell::Unknown => UNKNOWN,
    };

    let x0 = col * CELL_SIZE;
    let y0 = row * CELL_SIZE;

    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            let on_border = !(BORDER_WIDTH..CELL_SIZE - BORDER_WIDTH).contains(&dx)
                || !(BORDER_WIDTH..CELL_SIZE - BORDER_WIDTH).contains(&dy);

            let color = if on_border { BORDER } else { fill };
            buffer[(y0 + dy) * WIDTH + (x0 + dx)] = color;
        }
    }
}

pub(crate) fn draw_car_cell(buffer: &mut [u32], car: Car) {
    let x0 = car.col * CELL_SIZE;
    let y0 = car.row * CELL_SIZE;

    for dy in 0..CELL_SIZE {
        for dx in 0..CELL_SIZE {
            buffer[(y0 + dy) * WIDTH + (x0 + dx)] = CAR_RED;
        }
    }

    match car.direction {
        Direction::Up => {
            for row in 0..(CELL_SIZE / 2) {
                let y = y0 + row;
                let width = (row * 2 + 1).min(CELL_SIZE - 2);
                let x_start = x0 + CELL_SIZE / 2 - width / 2;
                for x in x_start..(x_start + width) {
                    buffer[y * WIDTH + x] = CAR_BLACK;
                }
            }
        }
        Direction::Right => {
            for col in 0..(CELL_SIZE / 2) {
                let x = x0 + CELL_SIZE - 1 - col;
                let height = (col * 2 + 1).min(CELL_SIZE - 2);
                let y_start = y0 + CELL_SIZE / 2 - height / 2;
                for y in y_start..(y_start + height) {
                    buffer[y * WIDTH + x] = CAR_BLACK;
                }
            }
        }
        Direction::Down => {
            for row in 0..(CELL_SIZE / 2) {
                let y = y0 + CELL_SIZE - 1 - row;
                let width = (row * 2 + 1).min(CELL_SIZE - 2);
                let x_start = x0 + CELL_SIZE / 2 - width / 2;
                for x in x_start..(x_start + width) {
                    buffer[y * WIDTH + x] = CAR_BLACK;
                }
            }
        }
        Direction::Left => {
            for col in 0..(CELL_SIZE / 2) {
                let x = x0 + col;
                let height = (col * 2 + 1).min(CELL_SIZE - 2);
                let y_start = y0 + CELL_SIZE / 2 - height / 2;
                for y in y_start..(y_start + height) {
                    buffer[y * WIDTH + x] = CAR_BLACK;
                }
            }
        }
    }
}
