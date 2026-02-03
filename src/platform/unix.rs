use crate::{InspectPathError, PathInfo, PathStatus, PathType, RemoteType, get_resolved_path};
use std::{
    fs::{self, read_to_string},
    path::{Path, PathBuf},
};

// path to mountinfo
const MOUNTINFO_PATH: &str = "/proc/self/mountinfo";
// remote fs types
const REMOTE_FS_TYPES: &[&[&str]] = &[
    NFS,
    SMB,
    SSH,
    CLUSTER,
    PROTOCOL,
    OTHER,
];
const NFS: &[&str] = &[
    // NFS
    "nfs",
    "nfs4",
];
const SMB: &[&str] = &[
    // SMB / CIFS
    "cifs",
    "smbfs",
    "smb3",
];
const SSH: &[&str] = &[
    // SSH
    "sshfs",
    "fuse.sshfs",
];
const CLUSTER: &[&str] = &[
    // Cluster / distributed
    "ceph",
    "fuse.ceph",
    "glusterfs",
    "fuse.glusterfs",
];
const PROTOCOL: &[&str] = &[
    // Network / protocol FS
    "9p",
    "afp",
    "davfs",
    "fuse.davfs",
];
const OTHER: &[&str] = &[
    // Older / less common but still seen
    "ncpfs",
    "coda",
    "ocfs2",
    "gfs",
    "gfs2",
];
// local fs types
const LOCAL_BLOCK_FS_TYPES: &[&str] = &[
    // Linux native
    "ext2",
    "ext3",
    "ext4",
    "xfs",
    "btrfs",
    "f2fs",
    "jfs",
    "reiserfs",
    "reiser4",
    "bcachefs",

    // FAT family
    "vfat",
    "msdos",
    "exfat",

    // NTFS
    "ntfs",
    "ntfs3",

    // ZFS (out of tree but common)
    "zfs",
];
const CDROM_FS_TYPES: &[&str] = &[
    // Optical / legacy media
    "iso9660",
    "udf",
];

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
    let (resolved_path, is_symlink) = get_resolved_path(path);

    let miv = mountinfo_into_vec(&mountinfo_to_string()?)?;
let candidates: Vec<&MountInfo> = miv
    .iter()
    .filter(|m| 
        if let Some(rp) = &resolved_path {
            rp.starts_with(&m.mount_point)
        } else {
            path.starts_with(&m.mount_point)
        })
    .collect();

let best = candidates
    .into_iter()
    .max_by_key(|m| m.mount_point.components().count())
    .ok_or(InspectPathError::ParseGen)?;

    let kind = get_kind(best)?;
    let remote_kind = if kind != PathType::Remote {
        None
    } else {
        get_remote_kind(best)?
    };

    Ok(PathInfo {
        path: path.to_path_buf(),
        resolved_path,
        is_symlink,
        kind,
        remote_kind,
        status: PathStatus::Unknown
    })
}

fn get_kind(best: &MountInfo) -> Result<PathType, InspectPathError> {
    let removable_path = format!("/sys/dev/block/{}:0/removable", best.device_number.major);
    let removable: u8 = fs::read_to_string(Path::new(&removable_path))
    .unwrap_or_else(|_| "0".to_string())
    .parse().map_err(|e| InspectPathError::ParseInt(e))?;
    let fs_type = best.fs_type.as_str();

    if best.device_number.major == 0 {
            Ok(PathType::Virtual(fs_type.into()))
        } else if removable == 1 {
            Ok(PathType::Removable)
        } else if CDROM_FS_TYPES.contains(&fs_type) {
            Ok(PathType::CDRom)
        } else if REMOTE_FS_TYPES.iter().any(|fst| fst.contains(&fs_type)) {
            Ok(PathType::Remote)
        } else if fs_type.starts_with("fuse") {
            Ok(PathType::Unknown)
        } else if LOCAL_BLOCK_FS_TYPES.contains(&fs_type) {
            Ok(PathType::Fixed)
        } else {
            Ok(PathType::Unknown)
        }
    }

fn get_remote_kind(best: &MountInfo) -> Result<Option<RemoteType>, InspectPathError> {
    let fs_type = best.fs_type.as_str();

    if NFS.contains(&fs_type) {
        Ok(Some(RemoteType::NFS))
    } else if SMB.contains(&fs_type) {
        Ok(Some(RemoteType::SMB))
    } else if SSH.contains(&fs_type) {
        Ok(Some(RemoteType::Other("SSH".into())))
    } else if CLUSTER.contains(&fs_type) {
        Ok(Some(RemoteType::Other("Cluster / distributed".into())))
    } else if PROTOCOL.contains(&fs_type) {
        Ok(Some(RemoteType::Other("Network / protocol FS".into())))
    } else if OTHER.contains(&fs_type) {
        Ok(Some(RemoteType::Other("Other".into())))
    } else {
        Ok(Some(RemoteType::Unknown))
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

#[cfg(test)]
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
