//! The desktop user interface on Linux (GTK 4) and Windows (Win32): the window, menus, dialogs,
//! legend, and status bar. It draws the simulator's map and car and drives the car with the arrow
//! keys.

mod app;
mod config;
mod platform;
mod recent;
mod render;

pub(crate) use platform::run;
