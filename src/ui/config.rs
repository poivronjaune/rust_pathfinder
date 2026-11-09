pub(crate) const APP_NAME: &str = "Rust Pathfinder";
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const APP_DESCRIPTION: &str =
    "A grid-based map and vehicle movement prototype.\nUse the arrow keys to move and turn the car (obstacles block it), Esc to quit.";
pub(crate) const APP_AUTHOR: &str = "poivronjaune";
pub(crate) const APP_COPYRIGHT: &str = "Copyright (c) 2026 poivronjaune - MIT License";
pub(crate) const APP_WEBSITE: &str = "https://github.com/poivronjaune/rust_pathfinder";

/// Width and height of a cell in the drawn map, in pixels, before it is scaled to the window.
pub(crate) const CELL_SIZE: usize = 20;
pub(crate) const BORDER_WIDTH: usize = 1;

pub(crate) const FREE: u32 = 0x00DFF0E7;
pub(crate) const OBSTACLE: u32 = 0x00B0A494;
pub(crate) const UNKNOWN: u32 = 0x00F0E3D3;
pub(crate) const BORDER: u32 = 0x000000FF;
/// Grid line color while the map is in edit mode, so the mode is visible on the map itself.
pub(crate) const EDIT_BORDER: u32 = 0x00FF8C00;
pub(crate) const CAR_RED: u32 = 0x00FF0000;
pub(crate) const CAR_BLACK: u32 = 0x00000000;
