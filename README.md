# inspect_path

`inspect_path` is a small cross-platform Rust crate for inspecting filesystem paths.
It can determine whether a path refers to a fixed, removable, or remote filesystem,
and can optionally probe whether that path is currently mounted or disconnected.

The crate works on both **Windows** and **Unix-like** systems and hides
platform-specific details behind a simple, consistent API.

---

## Features

- Detects filesystem **path type**
  - Fixed disks
  - Removable media
  - Network / remote mounts
  - CD-ROM and RAM disks
- Identifies **remote filesystem types** when possible
  - Windows shares (SMB / CIFS)
  - NFS
  - AFS
- Allows probing a path to determine **mount status**
- Platform-specific implementations with a shared public API

---

## Platform Support

### Windows
- Fixed drives
- Removable drives
- CD-ROM drives
- RAM disks
- Network shares (UNC paths and mapped drives)

### Unix / Linux
- Local filesystems (ext4, xfs, btrfs, etc.)
- Network filesystems (NFS, SMB/CIFS, AFS)
- tmpfs and optical media
- Uses filesystem magic numbers via `statfs`

> Some filesystem details cannot be inferred on all platforms
> (for example, Linux cannot always distinguish NTFS backing devices).

---

## Usage

### Inspecting a path

```rust
use std::path::Path;
use inspect_path::inspect_path;

let info = inspect_path(Path::new(r"C:\")).unwrap();

if info.is_fixed() {
    println!("This is a fixed filesystem");
}
