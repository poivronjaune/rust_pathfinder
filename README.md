# Rust Pathfinder

[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platforms: Linux and Windows](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-lightgrey.svg)](#supported-platforms)

A small cross-platform Rust desktop prototype for exploring a grid-based map and vehicle movement. It opens a 20 × 20 grid with sample terrain and a vehicle that you can move and turn using the arrow keys. You can edit the map with the mouse, start a blank map, and save or load maps as `.map` files. This is an interactive prototype—not yet a route planner.

## Features

- Native desktop window on Linux and Windows
- Resizable 20 × 20 grid with free, obstacle, and unknown terrain, and a legend that explains the colors
- Keyboard-controlled vehicle: Up/Down to move, Left/Right to turn
- [Map editing](#editing-the-map) with the mouse: left click marks obstacles, right click marks unknown terrain
- Status display showing the vehicle's position and heading
- File menu to start a blank map, save and load [map files](#map-files), and exit
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

Movement is bounded by the grid edges. The current prototype does not prevent movement into obstacle cells.

## Editing the map

Choose **Map → Edit** to turn edit mode on, and choose it again to turn it off. While edit mode is on, the menu item shows a check mark, the grid lines turn orange, and the status bar starts with **Editing**.

| Mouse | Action |
| --- | --- |
| Left click | Mark the cell as an obstacle |
| Right click | Mark the cell as unknown |

Hold a mouse button and move the pointer to paint several cells at once. Clicks do nothing outside edit mode. You can still drive the vehicle while editing; if you edit the cell under the vehicle, the change shows once the vehicle moves away.

Edits are not saved automatically: use **File → Save** to keep them. Escape and **File → Exit** close the application without asking, even in edit mode.

The legend between the map and the status bar shows what each color means: free, obstacle, and unknown terrain, and the vehicle.

## Map files

| File menu command | Action |
| --- | --- |
| New | Replace the map with a blank one where every cell is unknown |
| Save | Save the current map to a `.map` file |
| Load | Replace the current map with one saved earlier |

Map files use the `.map` extension and contain JSON. They store only the terrain: the vehicle keeps its position and heading when you start, save, or load a map. If a file can't be read or isn't a valid 20 × 20 map, an error message is shown and the current map is left unchanged.

The format is defined by a [JSON Schema](https://json-schema.org/) (draft 2020-12) in [`schemas/map.schema.json`](schemas/map.schema.json). Saved files reference it through their `$schema` property, so editors that open `.map` files as JSON can validate them (in VS Code, add `"files.associations": { "*.map": "json" }` to your settings). Each grid row is written on its own line.

| Property | Description |
| --- | --- |
| `$schema` | Optional link to the schema, used by editors for validation |
| `formatVersion` | Version of the file format, currently `1` |
| `rows`, `columns` | Size of the grid |
| `cells` | Terrain addressed as `cells[row][column]`, with row 0 at the top: `"free"`, `"obstacle"`, or `"unknown"` |

## Build and test

```sh
cargo build
cargo test
```

To create an optimized build, run `cargo build --release`. The executable is written under `target/release/` (`rust_pathfinder` on Linux, `rust_pathfinder.exe` on Windows).

## Libraries and dependencies

The project uses the following direct dependencies, declared in `Cargo.toml` (the user interface libraries are platform-specific):

- **[gtk4](https://crates.io/crates/gtk4) `0.10`** — Linux desktop UI and drawing, using GTK 4 (and its Cairo integration).
- **[windows-sys](https://crates.io/crates/windows-sys) `0.61`** — Windows API bindings for the native window, menus, file dialogs, status bar, and GDI rendering.
- **[serde](https://crates.io/crates/serde) `1.0`** and **[serde_json](https://crates.io/crates/serde_json) `1.0`** — reading and writing map files.
- **Rust standard library** — shared application logic and platform-independent functionality.

Cargo resolves and downloads Rust crate dependencies automatically. Linux also needs the GTK 4 system development libraries installed separately.

## License

This project is distributed under the MIT License. See [LICENSE](LICENSE).
```sh
git stash push --include-untracked -m "work in progress"
```
