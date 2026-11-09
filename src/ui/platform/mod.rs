#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub(crate) fn run() -> Result<(), String> {
    linux::run()
}

#[cfg(target_os = "windows")]
pub(crate) fn run() -> Result<(), String> {
    windows::run()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub(crate) fn run() -> Result<(), String> {
    Err("This application currently supports Windows and Linux.".to_owned())
}
