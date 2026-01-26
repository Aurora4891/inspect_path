use crate::{InspectPathError, PathInfo, PathStatus, PathType, RemoteType};
use std::{io::ErrorKind, path::Path};
use windows::{Win32::Storage::FileSystem::GetDriveTypeW, core::{PWSTR, PCWSTR}};
use windows::Win32::NetworkManagement::WNet::{
    WNetAddConnection2W, NETRESOURCEW, RESOURCETYPE_DISK,
};
use windows::Win32::Foundation::NO_ERROR;
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
    let drive = path.to_string_lossy().chars().take(2).collect::<String>();

    let wide: Vec<u16> = drive.encode_utf16().chain(Some(0)).collect();

    let result = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) };

    let kind = match result {
        0 => return Err(InspectPathError::PathTypeError), // DRIVE_UNKNOWN
        1 => return Err(InspectPathError::InvalidPath(path.display().to_string())), // DRIVE_NO_ROOT_DIR
        2 => PathType::Removable, // DRIVE_REMOVABLE
        3 => PathType::Fixed,     // DRIVE_FIXED
        4 => PathType::Remote,    // DRIVE_REMOTE
        5 => PathType::CDRom,     // DRIVE_CDROM
        6 => PathType::RamDisk,   // DRIVE_RAMDISK
        e => return Err(InspectPathError::General(e.to_string())),
    };

    Ok(PathInfo {
        path: path.to_path_buf(),
        kind,
        remote_kind: if result == 4 {
            Some(RemoteType::Unknown)
        } else {
            None
        },
        status: PathStatus::Unknown,
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

fn to_pwstr(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0); // null terminator
    v
}

pub fn connect_drive(local: &str, remote: &str) -> Result<(), InspectPathError> {
let mut local = to_pwstr(local); // "Z:"
let mut remote = to_pwstr(remote); // r"\\server\share"

let mut nr = NETRESOURCEW {
    dwType: RESOURCETYPE_DISK,
    lpLocalName: PWSTR::from_raw(local.as_mut_ptr()),
    lpRemoteName: PWSTR::from_raw(remote.as_mut_ptr()),
    lpProvider: PWSTR::null(),
    ..Default::default()
    };

    let result = unsafe {
        WNetAddConnection2W(
            &mut nr,
            PCWSTR::null(), // password
            PCWSTR::null(), // username
            windows::Win32::NetworkManagement::WNet::NET_CONNECT_FLAGS(0),
        )
    };

    if result == NO_ERROR {
        Ok(())
    } else {
        Err(InspectPathError::General("Win32_Error".into()))
    }
}
