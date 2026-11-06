mod app;
mod car;
mod config;
mod grid;
mod platform;
mod render;

fn main() {
    if let Err(error) = platform::run() {
        eprintln!("Failed to start application: {error}");
        std::process::exit(1);
    }
}
