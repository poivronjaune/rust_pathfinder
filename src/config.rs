pub(crate) const GRID_SIZE: usize = 20;
pub(crate) const CELL_SIZE: usize = 20;
pub(crate) const BORDER_WIDTH: usize = 1;
/// Radius (in cells) of the free area generated around the car.
pub(crate) const CLEAR_RADIUS: usize = 4;

pub(crate) const WIDTH: usize = GRID_SIZE * CELL_SIZE;
pub(crate) const HEIGHT: usize = GRID_SIZE * CELL_SIZE;

pub(crate) const FREE: u32 = 0x00DFF0E7;
pub(crate) const OBSTACLE: u32 = 0x00B0A494;
pub(crate) const UNKNOWN: u32 = 0x00F0E3D3;
pub(crate) const BORDER: u32 = 0x000000FF;
pub(crate) const CAR_RED: u32 = 0x00FF0000;
pub(crate) const CAR_BLACK: u32 = 0x00000000;
