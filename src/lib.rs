use std::path::{Component, Path, PathBuf};
use thiserror::Error;
#[cfg(target_os = "windows")]
use windows::{core::PCWSTR, Win32::Storage::FileSystem::GetDriveTypeW};
#[cfg(target_family = "unix")]
use nix::sys::statfs::statfs;

#[derive(Debug, Error)]
pub enum NetPathError {
    #[error("Failed to get path type")]
    PathTypeError,
    #[error("Invalid path '{0}'")]
    InvalidPath(String),
    #[error("General Error '{0}'")]
    General(String)
}

#[derive(Debug, PartialEq)]
pub enum PathStatus {
    Mounted,
    Disconnected,
    Unknown,
    Other(String)
}

#[derive(Debug, PartialEq)]
pub enum RemoteType {
    WindowsShare,
    NFS,
    SMB,
    AFP,
    Other(String),
    Unknown,
    NonRemote // not needed if changed to option
}

#[derive(Debug, PartialEq)]
pub enum PathType {
    Unknown,
    Removable,
    Fixed,
    Remote,
    CDRom,
    RamDisk
}

impl Default for PathInfo {
    fn default() -> Self {
        PathInfo { 
            path: None,
            kind: PathType::Unknown,
            remote_kind: RemoteType::Unknown,
            status: PathStatus::Unknown
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PathInfo {
    pub(crate) path: Option<PathBuf>,
    pub(crate) kind: PathType,
    pub(crate) remote_kind: RemoteType, // maybe change this to an option.
    pub(crate) status: PathStatus,
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
    pub fn update_status(&mut self) {
        if let Some(p) = &self.path {
            match std::fs::metadata(p) {
                Ok(_) => self.status = PathStatus::Mounted,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::TimedOut => self.status = PathStatus::Disconnected,
                        std::io::ErrorKind::NotFound => self.status = PathStatus::Disconnected,
                        std::io::ErrorKind::NetworkDown => self.status = PathStatus::Disconnected,
                        std::io::ErrorKind::NotConnected => self.status = PathStatus::Disconnected,
                        std::io::ErrorKind::PermissionDenied => self.status = PathStatus::Mounted,
                        _ => self.status = PathStatus::Other(e.to_string())
                    }
                }
            }
        }
    }
    pub fn get_remote_type(&self) -> RemoteType {
        //temp
        RemoteType::Unknown
    }
}

//mod windows_rs {
//    use super::*;
    // move to 'windows.rs' later

    #[cfg(target_os = "windows")]
    pub fn inspect(path: &Path) -> Result<PathInfo, NetPathError> {
        let drive = path
            .to_string_lossy()
            .chars()
            .take(2)
            .collect::<String>();

        let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

        let result = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr()))};

        let kind = match result {
                0 => return Err(NetPathError::PathTypeError), // DRIVE_UNKNOWN
                1 => return Err(NetPathError::InvalidPath(path.display().to_string())), // DRIVE_NO_ROOT_DIR
                2 => PathType::Removable, // DRIVE_REMOVABLE
                3 => PathType::Fixed, // DRIVE_FIXED
                4 => PathType::Remote, // DRIVE_REMOTE
                5 => PathType::CDRom, // DRIVE_CDROM
                6 => PathType::RamDisk, // DRIVE_RAMDISK
                e => return Err(NetPathError::General(e.to_string()))
        };

        Ok(PathInfo {
            path: Some(path.to_path_buf()),
            kind,
            remote_kind: if result == 4 { RemoteType::Unknown } else { RemoteType::NonRemote },
            status: PathStatus::Unknown
        })
    }
    /// verify the drive type of the path it receives.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # #[cfg(target_os = "windows")]
    /// # {
    /// use std::path::Path;
    /// use netpath::{PathInfo, RemoteType, PathType, PathStatus, inspect};
    /// 
    /// let path_type = PathInfo {
    ///     path: Some(Path::new("\\\\server\\share\\").to_path_buf()),
    ///     kind: PathType::Remote,
    ///     remote_kind: RemoteType::Unknown,
    ///     status: PathStatus::Unknown
    /// };
    ///
    /// let path = Path::new("\\\\server\\share\\");
    /// let answer = inspect(path).unwrap();
    ///
    /// assert_eq!(path_type, answer);
    /// # }
    /// ```
    pub fn path_type(path: &Path) -> Result<PathType, NetPathError> {
        //let drive = windows_root(&path).ok_or(NetPathError::InvalidPath(path.display().to_string()))?;
        let drive = path
            .to_string_lossy()
            .chars()
            .take(2)
            .collect::<String>();

        let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

        let path_type = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr()))};

        match path_type {
                0 => Ok(PathType::Unknown),
                1 => Err(NetPathError::InvalidPath(path.display().to_string())),
                2 => Ok(PathType::Removable),
                3 => Ok(PathType::Fixed),
                4 => Ok(PathType::Remote),
                5 => Ok(PathType::CDRom),
                6 => Ok(PathType::RamDisk),
                _ => Err(NetPathError::PathTypeError)
        }
    }

    /*
    #[cfg(target_os = "windows")]
    pub fn path_type_with_status(path: &Path) -> Result<PathType, NetPathError> {
        //let drive = windows_root(&path).ok_or(NetPathError::InvalidPath(path.display().to_string()))?;
        let drive = path
            .to_string_lossy()
            .chars()
            .take(2)
            .collect::<String>();

        let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

        let path_type = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr()))};

        match path_type {
                0 => Ok(PathType::Unknown),
                1 => Err(NetPathError::InvalidPath(path.display().to_string())),
                2 => Ok(PathType::Removable),
                3 => Ok(PathType::Fixed),
                4 => Ok(PathType::Remote(remote_status(path))),
                5 => Ok(PathType::CDRom),
                6 => Ok(PathType::RamDisk),
                _ => Err(NetPathError::PathTypeError)
        }
    }


    #[cfg(target_os = "windows")]
    pub fn remote_status(path: &Path) -> RemoteStatus {
        match std::fs::metadata(path) {
            Ok(_) => RemoteStatus::Mounted,
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut => RemoteStatus::Disconnected,
                    std::io::ErrorKind::NotFound => RemoteStatus::Disconnected,
                    std::io::ErrorKind::NetworkDown => RemoteStatus::Disconnected,
                    std::io::ErrorKind::NotConnected => RemoteStatus::Disconnected,
                    std::io::ErrorKind::PermissionDenied => RemoteStatus::Mounted,
                    _ => RemoteStatus::Other(e.to_string())
                }
            }
        }
    }
    */

    #[cfg(target_os = "windows")]
    fn _windows_root(path: &Path) -> Option<String> {
        match path.components().next() {
            Some(Component::Prefix(prefix)) => Some(prefix.as_os_str().to_string_lossy().to_string()),
            _ => None
        }
    }
//}

//mod unix {
    // move to 'unix.rs later'
    //use super::*;
    #[cfg(target_family = "unix")]
    pub fn path_type(path: &Path) -> Result<PathType, NetPathError> {
        let stats = statfs(path)
            .map_err(|e| NetPathError::General(e.to_string()))?;
        Ok(PathType::Unknown)
    }
//}

#[cfg(test)]
mod tests {
    use super::*;
    //use crate::windows_rs::path_type;
    #[test]
    fn remote_path_type() {
        let path_type = PathInfo {
            path: Some(Path::new("\\\\server\\share\\").to_path_buf()),
            kind: PathType::Remote,
            remote_kind: RemoteType::Unknown,
            status: PathStatus::Unknown
        };

        let path = Path::new("\\\\server\\share\\");
        let answer = inspect(path).unwrap();

        assert_eq!(path_type, answer);
    }
}