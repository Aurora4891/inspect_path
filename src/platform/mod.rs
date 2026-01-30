use crate::PathStatus;
use std::path::Path;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::{inspect_path, inspect_path_and_status, connect_drive};
        pub fn check_status(path: &Path) -> PathStatus {
            windows::check_status(path)
        }
    } else if #[cfg(target_family = "unix")] {
        mod unix;
        pub use unix::{inspect_path, inspect_path_new};
        pub fn check_status(path: &Path) -> PathStatus {
            unix::check_status(path)
        }
    } else {
        compile_error!("unsupported platform");
    }
}
