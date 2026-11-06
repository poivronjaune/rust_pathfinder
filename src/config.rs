pub(crate) const GRID_SIZE: usize = 20;
pub(crate) const CELL_SIZE: usize = 20;
pub(crate) const BORDER_WIDTH: usize = 1;

pub(crate) const WIDTH: usize = GRID_SIZE * CELL_SIZE;
pub(crate) const HEIGHT: usize = GRID_SIZE * CELL_SIZE;

pub(crate) const FREE: u32 = 0x0038A169;
pub(crate) const OBSTACLE: u32 = 0x006B4F2A;
pub(crate) const UNKNOWN: u32 = 0x004A4A4A;
pub(crate) const BORDER: u32 = 0x000000FF;
pub(crate) const CAR_RED: u32 = 0x00FF0000;
pub(crate) const CAR_BLACK: u32 = 0x00000000;
