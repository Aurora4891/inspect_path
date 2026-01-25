//! Cross-platform inspection of filesystem paths.
//!
//! This crate provides utilities for determining the type and status of
//! filesystem paths on both Windows and Unix-like systems. It can distinguish
//! between fixed, removable, and remote paths, and provides additional
//! information for network-backed paths.
//!
//! The primary entry point is [`inspect_path`], which returns a [`PathInfo`] struct
//! describing the given path.
//!
//! # Platform support
//!
//! * **Windows**
//!   * Local drives (fixed, removable, CD-ROM, RAM disk)
//!   * Network shares (UNC paths and mapped drives)
//!
//! * **Unix**
//!   * Basic filesystem detection (expanding in future releases)
//!
//! # Examples
//!
//! ```rust
//! use std::path::Path;
//! use inspect_path::inspect_path;
//!
//! # #[cfg(target_os = "windows")]
//! # {
//! let info = inspect_path(Path::new(r"C:\")).unwrap();
//!
//! if info.is_fixed() {
//!     println!("Fixed path detected");
//! }
//! # }
//!
//! # #[cfg(target_os = "unix")]
//! # {
//! let mut info = inspect_path(Path::new("/home/")).unwrap();
//!
//! if info.is_status_unknown() {
//!     info.check_status();
//!     if info.is_status_mounted() {
//!         println!("Path Mounted!")
//!     }
//! }
//! # }
//! ```
//!
//! # Notes
//!
//! Some operations (such as determining network mount status) may perform
//! blocking I/O depending on the platform and filesystem.
use std::path::PathBuf;
use thiserror::Error;
pub mod platform;
pub use platform::inspect_path;

#[derive(Debug, Error)]
pub enum InspectPathError {
    #[error("Failed to get path type")]
    PathTypeError,
    #[error("Invalid path '{0}'")]
    InvalidPath(String),
    #[error("General Error '{0}'")]
    General(String),
}

/// The connection status of a path.
#[derive(Debug, PartialEq)]
pub enum PathStatus {
    Mounted,
    Disconnected,
    Unknown,
    Other(String),
}

/// The underlying remote filesystem type, if applicable.
///
/// This value is meaningful only when the path is classified as remote.
#[derive(Debug, PartialEq)]
pub enum RemoteType {
    WindowsShare,
    NFS,
    SMB,
    AFS,
    Other(String),
    Unknown,
}

/// The general category of a filesystem path.
#[derive(Debug, PartialEq)]
pub enum PathType {
    Unknown,
    Removable,
    Fixed,
    Remote,
    CDRom,
    RamDisk,
}

/// Information about a filesystem path, including its type and mount status.
///
/// `PathInfo` represents both local and remote paths and provides methods
/// to inspect their characteristics in a platform-independent way.
#[derive(Debug, PartialEq)]
pub struct PathInfo {
    path: PathBuf,
    kind: PathType,
    remote_kind: Option<RemoteType>,
    status: PathStatus,
}

impl PathInfo {
    pub fn is_removable(&self) -> bool {
        matches!(self.kind, PathType::Removable)
    }
    pub fn is_fixed(&self) -> bool {
        matches!(self.kind, PathType::Fixed)
    }
    pub fn is_remote(&self) -> bool {
        matches!(self.kind, PathType::Remote)
    }
    pub fn is_cdrom(&self) -> bool {
        matches!(self.kind, PathType::CDRom)
    }
    pub fn is_ramdisk(&self) -> bool {
        matches!(self.kind, PathType::RamDisk)
    }
    pub fn is_status_mounted(&self) -> bool {
        matches!(self.status, PathStatus::Mounted)
    }
    pub fn is_status_disconnected(&self) -> bool {
        matches!(self.status, PathStatus::Disconnected)
    }
    pub fn is_status_unknown(&self) -> bool {
        matches!(self.status, PathStatus::Unknown)
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn kind(&self) -> &PathType {
        &self.kind
    }
    pub fn status(&self) -> &PathStatus {
        &self.status
    }
    pub fn remote_type(&self) -> Option<&RemoteType> {
        self.remote_kind.as_ref()
    }

    /// Updates the mount status of the path.
    ///
    /// This function attempts to access the underlying filesystem to determine
    /// whether the path is currently mounted or disconnected. On network paths,
    /// this may perform a blocking I/O operation.
    ///
    /// The status is updated based on the result of probing the path and is
    /// stored in the `status` field.
    pub fn check_status(&mut self) {
        self.status = platform::check_status(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    //use crate::platform::inspect_path;

    #[cfg(target_os = "windows")]
    #[test]
    fn fixed_path_type() {
        let path_type = PathInfo {
            path: Path::new(r"C:\").to_path_buf(),
            kind: PathType::Fixed,
            remote_kind: None,
            status: PathStatus::Unknown,
        };

        let path = Path::new(r"C:\");
        let answer = inspect_path(path).unwrap();

        assert_eq!(path_type, answer);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn fixed_path_type() {
        let path_type = PathInfo {
            path: Path::new(r"/etc/").to_path_buf(),
            kind: PathType::Fixed,
            remote_kind: None,
            status: PathStatus::Unknown,
        };

        let path = Path::new(r"/etc/");
        let answer = inspect_path(path).unwrap();

        assert_eq!(path_type, answer);
    }
}
