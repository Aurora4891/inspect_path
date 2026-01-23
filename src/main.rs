use std::path::Path;

fn main() {
    let p1 = netpath::inspect(Path::new("/"))
        .unwrap_or_default();
    let p2 = netpath::inspect(Path::new("/home"));
    let p3 = netpath::inspect(Path::new("S:"));
    let p4 = netpath::path_type(Path::new("C:\\"));
    if let Err(e) = netpath::path_type(Path::new("CTioewurouewjfndslfsdkfjksdfh")) {
        println!("{e}");
    }

    if p1.is_fixed() {
        println!("{p1:#?}");
    }

    println!("{p2:#?} {p3:#?} {p4:#?}");
}