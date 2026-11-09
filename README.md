# Rust Pathfinder

[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platforms: Linux and Windows](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-lightgrey.svg)](#supported-platforms)

A small cross-platform Rust desktop prototype for exploring a grid-based map and vehicle movement. It opens a grid with sample terrain (20 × 20 by default, from 2 to 100 rows and columns) and a vehicle that you can move and turn using the arrow keys; obstacles block it. You can edit the map with the mouse, change its size and where the vehicle starts, start a blank map, save or load maps as `.map` files, and reopen recent files from the File menu. This is an interactive prototype—not yet a route planner.

## Features

- Native desktop window on Linux and Windows
- Grid of 2 to 100 rows and columns (20 × 20 by default) with free, obstacle, and unknown terrain, scaled to fit the window, and a legend that explains the colors
- Keyboard-controlled vehicle: Up/Down to move, Left/Right to turn; obstacles block it
- [Map editing](#editing-the-map) with the mouse: left click marks obstacles, right click marks unknown terrain
- [Grid size and vehicle start](#grid-size-and-vehicle-start) chosen in a dialog: number of rows and columns, start cell, and heading
- Status display showing the vehicle's position and heading, and when an obstacle blocks it
- File menu to start a blank map, save and load [map files](#map-files), reopen [recent files](#recent-files), and exit
- Native menus

## Requirements

- Rust toolchain with Cargo (the project uses Rust edition 2024)
- Linux: GTK 4 development files and `pkg-config`
- Windows: Windows desktop development environment and Rust's MSVC toolchain

## Download

Clone the repository and enter its directory:

```sh
git clone https://github.com/poivronjaune/rust_pathfinder.git
cd rust_pathfinder
```

Alternatively, use **Code → Download ZIP** on GitHub and extract the archive.

## Install and run

Cargo downloads the Rust crate dependencies and builds the application on the first run.

### Linux (Debian/Ubuntu)

Install GTK 4 development dependencies:

```sh
sudo apt update
sudo apt install build-essential pkg-config libgtk-4-dev
```

Then build and launch:

```sh
cargo run
```

On other Linux distributions, install the equivalent GTK 4 development package and `pkg-config` using that distribution's package manager.

### Windows

Install Rust using [rustup](https://rustup.rs/) and make sure the MSVC C++ build tools are available. From PowerShell or Command Prompt, in the project directory, run:

```powershell
cargo run
```

## Controls

| Key | Action |
| --- | --- |
| Up arrow | Move forward one cell |
| Down arrow | Move backward one cell |
| Left arrow | Turn counterclockwise |
| Right arrow | Turn clockwise |
| Escape | Close the application |

You can also close the application with **File → Exit**.

Movement is bounded by the grid edges, and the vehicle can't drive into an obstacle: the move is ignored and the status bar shows **Blocked by an obstacle** until the next move or turn. Unknown cells don't block the vehicle, and it can always turn in place. The arrow keys do nothing while the map is in edit mode.

## Editing the map

Choose **Map → Edit** to turn edit mode on, and choose it again to turn it off. While edit mode is on, the menu item shows a check mark, the grid lines turn orange, and the status bar starts with **Editing**.

| Mouse | Action |
| --- | --- |
| Left click | Mark the cell as an obstacle |
| Right click | Mark the cell as unknown |

Hold a mouse button and move the pointer to paint several cells at once. Clicks do nothing outside edit mode. The vehicle is parked while you edit: the arrow keys do nothing until you turn edit mode off. If you edit the cell under the vehicle, the change shows once the vehicle moves away.

Edits are not saved automatically: use **File → Save** to keep them. Escape and **File → Exit** close the application without asking, even in edit mode.

The legend between the map and the status bar shows what each color means: free, obstacle, and unknown terrain, and the vehicle.

## Grid size and vehicle start

Choose **Map → Grid Size...** to change the size of the map and where the vehicle starts. The dialog opens with the current values:

| Field | Description |
| --- | --- |
| Rows, Columns | Size of the map, from 2 to 100 |
| Row, Column | Start cell of the vehicle, counted from 1 at the top-left corner, like the status bar |
| Heading | Direction the vehicle faces at the start: Up, Right, Down, or Left |

Choose **OK** (or press Enter) to apply the settings, or **Cancel** (or press Escape) to keep the map as it is. If a value is missing or out of range, a message explains what to correct and the dialog stays open.

When you apply the settings, the map is resized from its top-left corner: cells that still fit keep their terrain, new cells are unknown, and cells beyond the new edges are dropped. The vehicle moves to its start cell and faces the chosen heading; if the start cell was an obstacle, it becomes free. The window title shows the size of the map, for example **30 x 40 Grid**, and the map is scaled to fit the window. Save the map to keep its size and vehicle start.

## Map files

| File menu command | Action |
| --- | --- |
| New | Replace the map with a blank one of the same size where every cell is unknown |
| Save | Save the current map, its size, and the vehicle start to a `.map` file |
| Load | Replace the current map with one saved earlier and put the vehicle at its start |
| 1 to 9 | Load one of the [recent files](#recent-files) |
| Clear Recent Files | Empty the list of recent files |

Map files use the `.map` extension and contain JSON. They store the terrain, the size of the map, and where the vehicle starts—the start chosen with **Map → Grid Size...**, not the vehicle's current position. Loading a map moves the vehicle to its start; **New** leaves the vehicle where it is. If a file can't be read or isn't a valid map, an error message is shown and the current map is left unchanged.

The format is defined by a [JSON Schema](https://json-schema.org/) (draft 2020-12) in [`schemas/map.schema.json`](schemas/map.schema.json). Saved files reference it through their `$schema` property, so editors that open `.map` files as JSON can validate them (in VS Code, add `"files.associations": { "*.map": "json" }` to your settings). Each grid row is written on its own line.

| Property | Description |
| --- | --- |
| `$schema` | Optional link to the schema, used by editors for validation |
| `formatVersion` | Version of the file format, currently `2`. Version `1` files, which have no `start`, still load |
| `rows`, `columns` | Size of the grid, from 2 to 100 |
| `start` | Optional vehicle start: `row` and `column` counted from 0 at the top-left corner, and `heading` (`"up"`, `"right"`, `"down"`, or `"left"`). Without it, the vehicle starts in the bottom-center cell facing up |
| `cells` | Terrain addressed as `cells[row][column]`, with row 0 at the top: `"free"`, `"obstacle"`, or `"unknown"` |

For example, a 2 × 3 map whose vehicle starts in the bottom-right cell facing right:

```json
{
  "$schema": "https://raw.githubusercontent.com/poivronjaune/rust_pathfinder/main/schemas/map.schema.json",
  "formatVersion": 2,
  "rows": 2,
  "columns": 3,
  "start": {
    "row": 1,
    "column": 2,
    "heading": "right"
  },
  "cells": [
    ["free","obstacle","unknown"],
    ["free","free","free"]
  ]
}
```

## Recent files

Like Microsoft Word and Excel, the File menu lists the map files you saved or loaded most recently, newest first, numbered from 1. Choose a file, or press its number while the File menu is open, to load it. Up to 9 files are listed, and long paths are shortened in the middle. A file that no longer exists is removed from the list when you choose it, and **File → Clear Recent Files** empties the list.

The list is kept between sessions in a `recent_files.txt` file, in `%APPDATA%\rust_pathfinder` on Windows and in `$XDG_CONFIG_HOME/rust_pathfinder` (by default `~/.config/rust_pathfinder`) on Linux.

## Project structure

The source code is split into modules that depend on each other in one direction only:

| Module | Contents |
| --- | --- |
| [`src/simulator/`](src/simulator) | The simulated world: the grid and its terrain, the vehicle, the driving rules (the vehicle stays on the map and obstacles block it), map generation, and map files. It has no user interface code. |
| [`src/ui/`](src/ui) | The desktop interface: the Linux (GTK 4) and Windows (Win32) windows, menus, and dialogs, the drawing of the map, the legend, the status bar, map editing, and the recent files. |
| [`src/main.rs`](src/main.rs) | Starts the desktop interface. |

The interface uses the simulator, but the simulator never uses the interface. Other code uses only the items that [`src/simulator/mod.rs`](src/simulator/mod.rs) re-exports. A driver steers the vehicle by sending commands (`Forward`, `Reverse`, `TurnLeft`, `TurnRight`) to `Simulation::drive`, which reports what each command did: the vehicle moved, turned, was blocked by an obstacle, or reached the edge of the map. The arrow keys are one such driver, and future modules, such as a self-driving one, can drive the vehicle the same way.

## Build and test

```sh
cargo build
cargo test
```

To create an optimized build, run `cargo build --release`. The executable is written under `target/release/` (`rust_pathfinder` on Linux, `rust_pathfinder.exe` on Windows).

## Libraries and dependencies

The project uses the following direct dependencies, declared in `Cargo.toml` (the user interface libraries are platform-specific):

- **[gtk4](https://crates.io/crates/gtk4) `0.10`** — Linux desktop UI and drawing, using GTK 4 (and its Cairo integration).
- **[windows-sys](https://crates.io/crates/windows-sys) `0.61`** — Windows API bindings for the native window, menus, Grid Size and file dialogs, status bar, and GDI rendering.
- **[serde](https://crates.io/crates/serde) `1.0`** and **[serde_json](https://crates.io/crates/serde_json) `1.0`** — reading and writing map files.
- **Rust standard library** — shared application logic and platform-independent functionality.

Cargo resolves and downloads Rust crate dependencies automatically. Linux also needs the GTK 4 system development libraries installed separately.

## License

This project is distributed under the MIT License. See [LICENSE](LICENSE).
```sh
git stash push --include-untracked -m "work in progress"
```
