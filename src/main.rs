mod simulator;
mod ui;

fn main() {
    if let Err(error) = ui::run() {
        eprintln!("Failed to start application: {error}");
        std::process::exit(1);
    }
}
