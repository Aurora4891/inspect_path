use std::path::Path;
use crate::PathStatus;
use nix::sys::statfs::statfs;

pub fn path_type(path: &Path) -> Result<PathType, NetPathError> {
    let stats = statfs(path)
        .map_err(|e| NetPathError::General(e.to_string()))?;
    Ok(PathType::Unknown)
}

pub fn update_status(path: &Path) -> PathStatus {
    match std::fs::metadata(path) {
        Ok(_) => PathStatus::Mounted,
        Err(_) => PathStatus::Unknown,
    }
}
