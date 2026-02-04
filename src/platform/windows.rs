use crate::{InspectPathError, PathInfo, PathStatus, PathType, RemoteType};
use std::{ffi::c_void, io::ErrorKind, path::Path};
use windows::Win32::Foundation::NO_ERROR;
use windows::Win32::NetworkManagement::WNet::{
    NETRESOURCEW, RESOURCETYPE_DISK, WNetAddConnection2W, WNetGetUniversalNameW,
};
use windows::{
    Win32::{
        Foundation::ERROR_MORE_DATA,
        NetworkManagement::WNet::{UNC_INFO_LEVEL, UNIVERSAL_NAME_INFOW},
        Storage::FileSystem::GetDriveTypeW,
    },
    core::{PCWSTR, PWSTR},
};
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
    let wide = path_to_wide(path);
    let base_path = get_universal_name(&wide);

    let result = match &base_path {
        Some(real_path) => {
            let wide = return_first_two(Path::new(&real_path));
            unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) }
        }
        None => unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) },
    };

    let kind = match &result {
        0 => return Err(InspectPathError::PathTypeError), // DRIVE_UNKNOWN
        1 => return Err(InspectPathError::InvalidPath(path.display().to_string())), // DRIVE_NO_ROOT_DIR
        2 => PathType::Removable, // DRIVE_REMOVABLE
        3 => PathType::Fixed,     // DRIVE_FIXED
        4 => PathType::Remote,    // DRIVE_REMOTE
        5 => PathType::CDRom,     // DRIVE_CDROM
        6 => PathType::RamDisk,   // DRIVE_RAMDISK
        e => return Err(InspectPathError::General(e.to_string())),
    };

    let remote_kind = if matches!(kind, PathType::Remote) {
        get_remote_type(&base_path)
    } else {
        None
    };

    Ok(PathInfo {
        path: path.to_path_buf(),
        kind,
        remote_kind,
        status: PathStatus::Unknown,
    })
}

fn get_remote_type(base_path: &Option<String>) -> Option<RemoteType> {
    match base_path {
        None => None,
        Some(bp) => {
            match (
                bp.contains(r"\\"),
                bp.contains('@'),
                bp.contains("DavWWWRoot"),
            ) {
                (true, false, false) | (true, true, false) => Some(RemoteType::SMB),
                (true, false, true) | (true, true, true) => Some(RemoteType::WebDAV),
                (false, _, _) => Some(RemoteType::Unknown),
            }
        }
    }
}

fn return_first_two(path: &Path) -> Vec<u16> {
    let drive = path.to_string_lossy().chars().take(2).collect::<String>();
    drive.encode_utf16().chain(Some(0)).collect()
}

fn path_to_wide(path: &Path) -> Vec<u16> {
    let drive = path.to_string_lossy().chars().collect::<String>();
    drive.encode_utf16().chain(Some(0)).collect()
}

fn get_universal_name(wide: &[u16]) -> Option<String> {
    let mut size: u32 = 0;
    let dwinfolevel = UNC_INFO_LEVEL(1);
    let mut buffer: Vec<u8> = Vec::new();

    let result = unsafe {
        WNetGetUniversalNameW(
            PCWSTR(wide.as_ptr()),
            dwinfolevel,
            buffer.as_mut_ptr() as *mut c_void,
            &mut size,
        )
    };

    if result != ERROR_MORE_DATA {
        return None;
    }

    let mut buffer: Vec<u8> = vec![0u8; size as usize];

    let result = unsafe {
        WNetGetUniversalNameW(
            PCWSTR(wide.as_ptr()),
            dwinfolevel,
            buffer.as_mut_ptr() as *mut c_void,
            &mut size,
        )
    };

    if result != NO_ERROR {
        return None;
    }

    let un = buffer.as_ptr() as *const UNIVERSAL_NAME_INFOW;

    unsafe {
        match (*un).lpUniversalName.to_string() {
            Ok(uni_name) => Some(uni_name),
            Err(_) => None,
        }
    }
}

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

/// Connects (maps) a network share to a local drive letter on Windows.
///
/// This function wraps the Win32 `WNetAddConnection2W` API to create a mapped
/// network drive. The mapping is created for disk resources only.
///
/// # Parameters
///
/// * `local` — Local drive name such as `"Z:"`
/// * `remote` — Remote UNC path such as `"\\\\server\\share"`
///
/// Both parameters must be valid Windows path strings.
///
/// # Errors
///
/// Returns an error if the Win32 API call fails. The error variant will
/// contain the raw Win32 error code as text.
///
/// # Examples
///
/// ```rust,no_run
/// use inspect_path::mount_path;
///
/// mount_path("Z:", r"\\server\share").unwrap();
/// ```
///
/// # Platform
///
/// **Windows only.** This function is not available on Unix platforms.
///
/// # Notes
///
/// - This call may prompt for credentials depending on system configuration.
/// - Existing mappings using the same drive letter may cause failure.
/// - The connection is created using default credentials unless otherwise configured.
/// - This function performs a system-level change.
///
/// # See also
///
/// - [`inspect_path`] — inspect mapped drives after connecting
/// - [`inspect_path_and_status`] — inspect and verify availability
pub fn mount_path(local: &str, remote: &str) -> Result<(), InspectPathError> {
    let mut local = to_pwstr(local); // "Z:"
    let mut remote = to_pwstr(remote); // r"\\server\share"

    let nr = NETRESOURCEW {
        dwType: RESOURCETYPE_DISK,
        lpLocalName: PWSTR::from_raw(local.as_mut_ptr()),
        lpRemoteName: PWSTR::from_raw(remote.as_mut_ptr()),
        lpProvider: PWSTR::null(),
        ..Default::default()
    };

    let result = unsafe {
        WNetAddConnection2W(
            &nr,
            PCWSTR::null(), // password
            PCWSTR::null(), // username
            windows::Win32::NetworkManagement::WNet::NET_CONNECT_FLAGS(0),
        )
    };

    if result == NO_ERROR {
        Ok(())
    } else {
        Err(InspectPathError::General(format!(
            "Win32 error: {}",
            result.0
        )))
    }
}

pub fn try_mount_if_needed(path: &Path, remote: &Path) -> Result<(), InspectPathError> {
    if let Err(e) = inspect_path(path) {
        match e {
            InspectPathError::InvalidPath(_) => {
                mount_path(
                    path.to_string_lossy()
                        .chars()
                        .take(2)
                        .collect::<String>()
                        .as_str(),
                    remote
                        .to_str()
                        .ok_or(InspectPathError::General("Conversion Error".into()))?,
                )?;
            }
            e => return Err(e),
        }
    };

    Ok(())
}
