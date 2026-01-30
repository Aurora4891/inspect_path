use crate::{InspectPathError, PathInfo, PathStatus, PathType, RemoteType};
use nix::sys::statfs::statfs;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";
// Filesystem magic numbers from statfs (base-10)
/// Network File System (NFS)
pub const FS_NFS: i64 = 26985;
/// SMB (legacy smbfs)
pub const FS_SMB: i64 = 20859;
/// CIFS (modern SMB, Windows shares)
pub const FS_CIFS: i64 = -187242602;
/// Andrew File System (AFS)
pub const FS_AFS: i64 = 1397113167;
/// FUSE-based filesystems (e.g., SSHFS)
pub const FS_FUSE: i64 = 1702057286;

// ---- Common local filesystems (useful for filtering) ----
/// ext2 / ext3 / ext4
pub const FS_EXT4: i64 = 61267; // 0xEF53
/// B-Tree File System (Btrfs)
pub const FS_BTRFS: i64 = 2435016766; // 0x9123683E
/// tmpfs (RAM-backed filesystem)
pub const FS_TMPFS: i64 = 16914836;
/// CD-Rom / DVD-Rom
pub const FS_ROM: i64 = 38496;

// Linux filesystem magic numbers (base 10)

/// XFS
pub const FS_XFS: i64 = 1481003842; // 0x58465342
/// New Technology File System (NTFS)
pub const FS_NTFS: i64 = 1397118030; // 0x5346544E
/// FAT / FAT32 / MSDOS
pub const FS_FAT: i64 = 16390; // 0x4006 (FAT / FAT32 / MSDOS)
/// Extended FAT
pub const FS_EXFAT: i64 = 538032816; // 0x2011BAB0

fn inspect_path_new(path: &Path) -> Result<(), InspectPathError> {
    let miv = mountinfo_into_vec(&mountinfo_to_string()?)?;
    let mut candidates: Vec<&MountInfo> = Vec::new();
    for line in &miv {
        if path.starts_with(&line.mount_point) {
            candidates.push(line);
        }
    }
    candidates.sort_by_key(|m| m.mount_point.components().count());
    let best = candidates.last().ok_or(InspectPathError::ParseGen)?;
    dbg!(&best);
    Ok(())
}

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
    let statfs = statfs(path).map_err(|e| InspectPathError::General(e.to_string()))?;

    let (kind, remote_kind) = match statfs.filesystem_type().0 {
        FS_EXT4 | FS_XFS | FS_BTRFS | FS_FAT | FS_EXFAT => (PathType::Fixed, None),
        FS_NTFS => (PathType::Fixed, None), // document: "Linux cannot infer backing device"
        FS_NFS => (PathType::Remote, Some(RemoteType::NFS)),
        FS_CIFS | FS_SMB => (PathType::Remote, Some(RemoteType::SMB)),
        FS_AFS => (PathType::Remote, Some(RemoteType::AFS)),
        FS_FUSE => (PathType::Remote, Some(RemoteType::Unknown)),
        FS_TMPFS => (PathType::RamDisk, None),
        FS_ROM => (PathType::CDRom, None),
        _ => (PathType::Unknown, None),
    };
    Ok(PathInfo {
        path: path.to_path_buf(),
        kind,
        remote_kind,
        status: PathStatus::Unknown,
    })
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
        Err(_) => PathStatus::Unknown,
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DeviceNumber {
    major: u32,
    minor: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MountInfo {
    mount_id: u32,
    parent_id: u32,
    device_number: DeviceNumber,
    fs_root: PathBuf,
    mount_point: PathBuf,
    fs_type: String,
    block_device: PathBuf,
    mount_options: String,
}
fn mountinfo_to_string() -> Result<String, InspectPathError> {
    let mountinfo_file = read_to_string(Path::new(MOUNTINFO_PATH))?;
    Ok(mountinfo_file)
}

fn mountinfo_into_vec(s: &str) -> Result<Vec<MountInfo>, InspectPathError> {
    let mut out: Vec<MountInfo> = Vec::new();

    for line in s.lines() {
        let (pre, post) = line.split_once(" - ").ok_or(InspectPathError::ParseGen)?;

        let mut vfs = pre.split_whitespace();

        let mount_id: u32 = vfs.next().ok_or(InspectPathError::ParseGen)?.parse()?;
        let parent_id: u32 = vfs.next().ok_or(InspectPathError::ParseGen)?.parse()?;

        let (major, minor) = vfs
            .next()
            .ok_or(InspectPathError::ParseGen)?
            .split_once(":")
            .ok_or(InspectPathError::ParseGen)?;

        let device_number: DeviceNumber = DeviceNumber {
            major: major.parse()?,
            minor: minor.parse()?,
        };

        let fs_root: PathBuf = vfs.next().ok_or(InspectPathError::ParseGen)?.into();
        let mount_point: PathBuf = vfs.next().ok_or(InspectPathError::ParseGen)?.into();
        // rest of vfs not parsed

        let mut fs = post.split_whitespace();

        let fs_type: String = fs.next().ok_or(InspectPathError::ParseGen)?.into();
        let block_device: PathBuf = fs.next().ok_or(InspectPathError::ParseGen)?.into();
        let mount_options: String = fs.next().ok_or(InspectPathError::ParseGen)?.into();

        let value = MountInfo {
            mount_id,
            parent_id,
            device_number,
            fs_root,
            mount_point,
            fs_type,
            block_device,
            mount_options,
        };
        out.push(value);
    }
    Ok(out)
}

mod tests {
    use super::*;

    #[test]
    fn mountinfo_to_vec_virtual() {
        let line =
            "40 28 0:20 / /dev/mqueue rw,nosuid,nodev,noexec,relatime shared:15 - mqueue mqueue rw";
        let right = mountinfo_into_vec(line).unwrap();

        let device_number = DeviceNumber {
            major: 0,
            minor: 20,
        };
        let left = vec![MountInfo {
            mount_id: 40,
            parent_id: 28,
            device_number,
            fs_root: PathBuf::from("/"),
            mount_point: PathBuf::from("/dev/mqueue"),
            fs_type: String::from("mqueue"),
            block_device: PathBuf::from("mqueue"),
            mount_options: String::from("rw"),
        }];

        assert_eq!(left, right);
    }
}
