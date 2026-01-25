use std::{error::Error, path::Path};
use inspect_path::inspect_path;
fn main() -> Result<(), Box<dyn Error>>{
    #[cfg(target_family = "unix")]
    let mut _p1 = inspect_path(Path::new("/"))?;
    #[cfg(target_family = "unix")]
    let _p2 = inspect_path(Path::new("/run/"))?;
    #[cfg(target_os = "windows")]
    let p1 = inspect_path(Path::new("C:"));
    #[cfg(target_os = "windows")]
    let p2 = inspect_path(Path::new("C:"));

//    _p1.check_status();
    println!("{_p1:#?} {:#?}", _p2.is_ramdisk());
    Ok(())
}