use crate::PathStatus;
use std::path::Path;

#[cfg(docsrs)]
mod documents;
#[cfg(docsrs)]
pub use documents::mount_path;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::{inspect_path, mount_path};
        pub fn check_status(path: &Path) -> PathStatus {
            windows::check_status(path)
        }
    } else if #[cfg(target_family = "unix")] {
        mod unix;
        pub use unix::{inspect_path};

        /// Probes a path to determine its current mount/connection status.
        ///
        /// This function attempts to access filesystem metadata for the given path
        /// and classifies its availability based on the result.
        ///
        /// It is primarily used to detect whether a remote or removable filesystem
        /// is currently reachable.
        ///
        /// # Returns
        ///
        /// - [`PathStatus::Mounted`] — The path responded to metadata access
        /// - [`PathStatus::Disconnected`] — The path appears unavailable (typically
        ///   network or device not connected) *(Windows only — see below)*
        /// - [`PathStatus::Unknown`] — Status could not be determined reliably
        ///
        /// # Behavior
        ///
        /// This function performs a real filesystem probe using `std::fs::metadata`.
        /// On remote filesystems this may involve network I/O and can block for a
        /// noticeable amount of time if the target is unreachable.
        ///
        /// # Platform differences
        ///
        /// ## Windows
        ///
        /// Error kinds are mapped to status:
        ///
        /// - `NotFound`, `TimedOut`, `NetworkDown`, `NotConnected` → Disconnected
        /// - `PermissionDenied` → Mounted (exists but access restricted)
        /// - Other errors → Unknown
        ///
        /// ## Unix
        ///
        /// Currently uses a simpler probe:
        ///
        /// - Success → Mounted
        /// - Any error → Unknown
        ///
        /// (Future versions may distinguish disconnected network mounts more precisely.)
        ///
        /// # Examples
        ///
        /// ```rust,no_run
        /// use std::path::Path;
        /// use inspect_path::inspect_path;
        ///
        /// let mut info = inspect_path(Path::new("/")).unwrap();
        /// info.check_status();
        ///
        /// if info.is_status_mounted() {
        ///     println!("Path is reachable");
        /// }
        /// ```
        ///
        /// # Notes
        ///
        /// This is a heuristic check. Some filesystems may report as available even
        /// if later operations fail, and some virtual filesystems may always appear
        /// mounted.
        pub fn check_status(path: &Path) -> PathStatus {
            unix::check_status(path)
        }
    } else {
        compile_error!("unsupported platform");
    }
}
