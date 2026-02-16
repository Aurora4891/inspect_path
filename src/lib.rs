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
use std::{
    num::ParseIntError,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub mod platform;

/// Always available APIs
pub use platform::{check_status, inspect_path};

/// Windows-only APIs
#[cfg(any(windows, docsrs))]
pub use platform::{mount_path, mount_path_as_user, try_mount_if_needed};

// Unix-only APIs
//#[cfg(unix)]
//#[cfg_attr(docsrs, doc(cfg(unix)))]

#[derive(Debug, Error)]
pub enum InspectPathError {
    #[error("Parse Int Error")]
    ParseInt(#[from] ParseIntError),
    #[error("Parse General Error")]
    ParseGen,
    #[error("I/O Error")]
    Io(#[from] std::io::Error),
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
    WebDAV,
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
    #[cfg(any(target_family = "unix", docsrs))]
    /// unix only
    Virtual(String),
}

/// Information about a filesystem path, including its type and mount status.
///
/// `PathInfo` represents both local and remote paths and provides methods
/// to inspect their characteristics in a platform-independent way.
#[derive(Debug, PartialEq)]
pub struct PathInfo {
    path: PathBuf,
    #[cfg(target_family = "unix")]
    resolved_path: Option<PathBuf>,
    #[cfg(target_family = "unix")]
    is_symlink: bool,
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
    #[cfg(target_family = "unix")]
    pub fn is_virtual(&self) -> bool {
        matches!(self.kind, PathType::Virtual(_))
    }
    #[cfg(target_family = "unix")]
    pub fn is_symlink(&self) -> bool {
        self.is_symlink
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
    #[cfg(target_family = "unix")]
    pub fn resolved_path(&self) -> &Option<PathBuf> {
        &self.resolved_path
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

    pub fn check_status(&mut self) {
        self.status = platform::check_status(&self.path);
    }
}

/// Inspects a filesystem path and immediately checks its mount status.
///
/// This is a convenience wrapper around [`inspect_path`] that also calls
/// [`PathInfo::check_status`] before returning. It is useful when you want
/// both the path classification and its current availability in one step.
///
/// On network-backed paths (such as SMB or WebDAV shares), checking status
/// may perform blocking I/O and can be slower than calling [`inspect_path`]
/// alone.
///
/// # Errors
///
/// Returns an error if the path type cannot be determined or if the platform
/// inspection call fails.
///
/// # Examples
///
/// ```rust
/// # #[cfg(target_os = "windows")]
/// # {
/// use std::path::Path;
/// use inspect_path::inspect_path_and_status;
///
/// let info = inspect_path_and_status(Path::new(r"C:\")).unwrap();
///
/// if info.is_status_mounted() {
///     println!("Path is available");
/// }
/// # }
/// # #[cfg(target_family = "unix")]
/// # {
/// use std::path::Path;
/// use inspect_path::inspect_path_and_status;
///
/// let info = inspect_path_and_status(Path::new(r"/home/")).unwrap();
///
/// if info.is_status_mounted() {
///     println!("Path is available");
/// }
/// # }
/// ```
///
/// # Platform behavior
///
/// - **Windows:** Uses Win32 APIs and filesystem probing
/// - **Unix:** Uses `statfs` and filesystem metadata probing
pub fn inspect_path_and_status(path: &Path) -> Result<PathInfo, InspectPathError> {
    let mut inspect = inspect_path(path)?;
    inspect.check_status();
    Ok(inspect)
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
            resolved_path: Some(PathBuf::from("/etc")),
            is_symlink: false,
            kind: PathType::Fixed,
            remote_kind: None,
            status: PathStatus::Unknown,
        };

        let path = Path::new(r"/etc/");
        let answer = inspect_path(path).unwrap();

        assert_eq!(path_type, answer);
    }
}
