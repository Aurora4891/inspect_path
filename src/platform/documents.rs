use crate::{InspectPathError, inspect_path, inspect_path_and_status};

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
/// **Windows only.** This function is not available on Unix platforms yet.
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
pub fn mount_path(local: &str, remote: &str) -> Result<(), InspectPathError> {}
