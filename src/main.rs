use std::path::Path;
use inspect_path::inspect_path;
fn main() {
    #[cfg(target_family = "unix")]
    let mut _p1 = inspect_path(Path::new("/")).unwrap();
    #[cfg(target_family = "unix")]
    let _p2 = inspect_path(Path::new("/run/user/1000/gvfs/smb-share:server=serverpi4.local,share=public"));
    #[cfg(target_os = "windows")]
    let p1 = inspect_path(Path::new("C:"));
    #[cfg(target_os = "windows")]
    let p2 = inspect_path(Path::new("C:"));

    _p1.update_status();
    println!("{_p1:#?} {_p2:#?}");
}