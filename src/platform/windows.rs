use windows::{core::PCWSTR, Win32::Storage::FileSystem::GetDriveTypeW};
use std::{path::Path, io::ErrorKind};
use crate::{PathInfo, PathType, PathStatus, RemoteType, InspectPathError};
/// Inspects a filesystem path and returns detailed information about it.
///
/// This function determines the general type of the path (fixed, removable,
/// remote, etc.) and returns a [`PathInfo`] structure containing the results.
///
/// On some platforms, this function may perform system calls to query the
/// underlying filesystem.
///
/// # Errors
///
/// Returns an error if the path is invalid or its type cannot be determined.
pub fn inspect_path(path: &Path) -> Result<PathInfo, InspectPathError> {
    let drive = path
        .to_string_lossy()
        .chars()
        .take(2)
        .collect::<String>();

    let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

    let result = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr()))};

    let kind = match result {
            0 => return Err(InspectPathError::PathTypeError), // DRIVE_UNKNOWN
            1 => return Err(InspectPathError::InvalidPath(path.display().to_string())), // DRIVE_NO_ROOT_DIR
            2 => PathType::Removable, // DRIVE_REMOVABLE
            3 => PathType::Fixed, // DRIVE_FIXED
            4 => PathType::Remote, // DRIVE_REMOTE
            5 => PathType::CDRom, // DRIVE_CDROM
            6 => PathType::RamDisk, // DRIVE_RAMDISK
            e => return Err(InspectPathError::General(e.to_string()))
    };

    Ok(PathInfo {
        path: path.to_path_buf(),
        kind,
        remote_kind: if result == 4 { Some(RemoteType::Unknown) } else { None },
        status: PathStatus::Unknown
    })
}
pub fn check_status(path: &Path) -> PathStatus {
    match std::fs::metadata(path) {
        Ok(_) => PathStatus::Mounted,
        Err(e) => match e.kind() {
            ErrorKind::NotFound
            | ErrorKind::TimedOut
            | ErrorKind::NetworkDown
            | ErrorKind::NotConnected => PathStatus::Disconnected,

            ErrorKind::PermissionDenied => PathStatus::Mounted,

            _ => PathStatus::Unknown,
        },
    }
}