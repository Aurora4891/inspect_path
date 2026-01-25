use std::path::Path;
use crate::{PathInfo, PathType, PathStatus, RemoteType, InspectPathError};
use nix::sys::statfs::statfs;

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
pub const FS_EXT4: i64      = 61267;        // 0xEF53
/// B-Tree File System (Btrfs)
pub const FS_BTRFS: i64     = 2435016766;   // 0x9123683E
/// tmpfs (RAM-backed filesystem)
pub const FS_TMPFS: i64 = 16914836;
/// CD-Rom / DVD-Rom
pub const FS_ROM: i64 = 38496;

// Linux filesystem magic numbers (base 10)

/// XFS
pub const FS_XFS: i64       = 1481003842;   // 0x58465342
/// New Technology File System (NTFS)
pub const FS_NTFS: i64      = 1397118030;   // 0x5346544E
/// FAT / FAT32 / MSDOS
pub const FS_FAT: i64       = 16390;        // 0x4006 (FAT / FAT32 / MSDOS)
/// Extended FAT
pub const FS_EXFAT: i64     = 538032816;    // 0x2011BAB0


pub fn inspect_path(path: &Path) -> Result<PathInfo, InspectPathError> {
    let statfs = statfs(path)
        .map_err(|e| InspectPathError::General(e.to_string()))?;
    
    let (kind, remote_kind) = match statfs.filesystem_type().0 {
        FS_EXT4 |
        FS_XFS |
        FS_BTRFS |
        FS_FAT |
        FS_EXFAT => (PathType::Fixed, None),
        FS_NTFS => (PathType::Fixed, None), // document: "Linux cannot infer backing device"
        FS_NFS => (PathType::Remote, Some(RemoteType::NFS)),
        FS_CIFS |
        FS_SMB => (PathType::Remote, Some(RemoteType::SMB)),
        FS_AFS => (PathType::Remote, Some(RemoteType::AFS)),
        FS_FUSE => (PathType::Remote, Some(RemoteType::Unknown)),
        FS_TMPFS => (PathType::RamDisk, None),
        FS_ROM => (PathType::CDRom, None),
        _ => (PathType::Unknown, None)
    };
    Ok(
        PathInfo {
            path: path.to_path_buf(),
            kind,
            remote_kind,
            status: PathStatus::Unknown
        }
    )
}

pub fn check_status(path: &Path) -> PathStatus {
    match std::fs::metadata(path) {
        Ok(_) => PathStatus::Mounted,
        Err(_) => PathStatus::Unknown,
    }
}
