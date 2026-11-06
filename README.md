# Rust Pathfinder

[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platforms: Linux and Windows](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows-lightgrey.svg)](#supported-platforms)

A small cross-platform Rust desktop prototype for exploring a grid-based map and vehicle movement. It opens a 20 × 20 grid with sample terrain and a vehicle that you can move and turn using the arrow keys. This is an interactive prototype—not yet a route planner; map editing and save/load actions are placeholders.

## Features

- Native desktop window on Linux and Windows
- Resizable 20 × 20 grid with free, obstacle, and unknown terrain
- Keyboard-controlled vehicle: Up/Down to move, Left/Right to turn
- Status display showing the vehicle's position and heading
- Native menus with placeholder actions

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

Movement is bounded by the grid edges. The current prototype does not prevent movement into obstacle cells.

## Build and test

```sh
cargo build
cargo test
```

To create an optimized build, run `cargo build --release`. The executable is written under `target/release/` (`my_app` on Linux, `my_app.exe` on Windows).

## Libraries and dependencies

The project uses the following direct dependencies, declared by platform in `Cargo.toml`:

- **[gtk4](https://crates.io/crates/gtk4) `0.10`** — Linux desktop UI and drawing, using GTK 4 (and its Cairo integration).
- **[windows-sys](https://crates.io/crates/windows-sys) `0.61`** — Windows API bindings for the native window, menus, status bar, and GDI rendering.
- **Rust standard library** — shared application logic and platform-independent functionality.

Cargo resolves and downloads Rust crate dependencies automatically. Linux also needs the GTK 4 system development libraries installed separately.

## License

This project is distributed under the MIT License. See [LICENSE](LICENSE).
