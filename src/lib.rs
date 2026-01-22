use std::path::{Component, Path};
use thiserror::Error;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::GetDriveTypeW;

#[derive(Debug, Error)]
pub enum NetPathError {
    #[error("Failed to get path type")]
    PathTypeError,
    #[error("Invalid path '{0}'")]
    InvalidPath(String),
}

#[derive(Debug, PartialEq)]
pub enum RemoteStatus {
    Mounted,
    Disconnected,
    Unknown
}

#[derive(Debug, PartialEq)]
pub enum PathType {
    Unknown,
    Removable,
    Fixed,
    Remote(RemoteStatus),
    CDRom,
    RamDisk
}

impl PathType {
    pub fn is_removable(&self) -> bool {
        matches!(self, PathType::Removable)
    }
    pub fn is_fixed(&self) -> bool {
        matches!(self, PathType::Fixed)
    }
    pub fn is_remote(&self) -> bool {
        matches!(self, PathType::Remote(_))
    }
    pub fn is_cdrom(&self) -> bool {
        matches!(self, PathType::CDRom)
    }
    pub fn is_ramdisk(&self) -> bool {
        matches!(self, PathType::RamDisk)
    }
}

#[cfg(target_os = "windows")]
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
            4 => Ok(PathType::Remote(RemoteStatus::Unknown)),
            5 => Ok(PathType::CDRom),
            6 => Ok(PathType::RamDisk),
            _ => Err(NetPathError::PathTypeError)
    }
}

#[cfg(target_os = "windows")]
fn windows_root(path: &Path) -> Option<String> {
    match path.components().next() {
        Some(Component::Prefix(prefix)) => Some(prefix.as_os_str().to_string_lossy().to_string()),
        _ => None
    }
}