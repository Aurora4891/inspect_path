use inspect_path::inspect_path;
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(target_family = "unix")]
    {
        let mut root = inspect_path(Path::new("/"))?;
        let run = inspect_path(Path::new("/run"))?;

        root.check_status();

        println!("Root path: {root:#?}");
        println!("Is /run a ramdisk? {}", run.is_ramdisk());
    }

    #[cfg(target_os = "windows")]
    {
        let c_drive = inspect_path(Path::new(r"C:\"))?;
        let windows = inspect_path(Path::new(r"C:\Windows"))?;

        println!("C drive: {c_drive:#?}");
        println!("Windows directory: {windows:#?}");
    }

    Ok(())
}
