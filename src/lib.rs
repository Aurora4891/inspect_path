use std::{fmt::Display, path::Path};
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
pub enum PathType {
    Unknown,
    Removable,
    Fixed,
    Remote,
    CDRom,
    RamDisk
}
impl Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PathType {
    pub fn is_removable(&self) -> bool {
        if self == &PathType::Removable {
            return true;
        }
        false
    }
    pub fn is_fixed(&self) -> bool {
        if self == &PathType::Fixed {
            return true;
        }
        false
    }
    pub fn is_remote(&self) -> bool {
        if self == &PathType::Remote {
            return true;
        }
        false
    }
    pub fn is_cdrom(&self) -> bool {
        if self == &PathType::CDRom {
            return true;
        }
        false
    }
    pub fn is_ramdisk(&self) -> bool {
        if self == &PathType::RamDisk {
            return true;
        }
        false
    }
}

pub fn path_type(path: &Path) -> Result<PathType, NetPathError> {
    let drive = path
        .to_string_lossy()
        .chars()
        .take(2)
        .collect::<String>();

    let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

    let path_type = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr()))};

    Ok( match path_type {
            0 => PathType::Unknown,
            1 => return Err(NetPathError::InvalidPath(path.display().to_string())),
            2 => PathType::Removable,
            3 => PathType::Fixed,
            4 => PathType::Remote,
            5 => PathType::CDRom,
            6 => PathType::RamDisk,
            _ => return Err(NetPathError::PathTypeError)
        })
}