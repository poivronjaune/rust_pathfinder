//! The simulator: a grid map of terrain and a car driven on it, one cell at a time.
//!
//! The simulator knows nothing about windows, drawing, or input devices. A driver steers the car
//! by sending [`Command`]s to [`Simulation::drive`], which applies the driving rules: the car stays
//! on the map, and obstacles block it. The desktop interface (`crate::ui`) is one driver, turning
//! the arrow keys into commands; other modules, such as a self-driving one, can send the same
//! commands.
//!
//! Code outside this module uses only the items re-exported here, so the simulator can change
//! inside, or move to its own crate, without breaking the code that uses it.

mod car;
mod config;
mod grid;
mod map_file;
mod simulation;

pub(crate) use car::{Car, Direction};
pub(crate) use config::{DEFAULT_GRID_SIZE, MAX_GRID_SIZE, MIN_GRID_SIZE};
pub(crate) use grid::{Cell, Grid};
pub(crate) use map_file::{DEFAULT_FILE_NAME, FILE_EXTENSION};
pub(crate) use simulation::{Command, Outcome, Simulation};
