#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_family = "unix")]
mod unix;

#[cfg(target_os = "windows")]
pub use windows::inspect_path;
#[cfg(target_family = "unix")]
pub use unix::inspect_path;

use std::path::Path;
use crate::PathStatus;

#[cfg(target_os = "windows")]
pub fn update_status(path: &Path) -> PathStatus {
    windows::update_status(path)
}

#[cfg(target_family = "unix")]
pub fn update_status(path: &Path) -> PathStatus {
    unix::update_status(path)
}