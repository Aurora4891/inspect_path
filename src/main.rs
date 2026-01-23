use std::path::Path;

fn main() {
    let p1 = netpath::path_type(Path::new("/"))
        .unwrap_or_default();
    let p2 = netpath::path_type(Path::new("/home"));
    let p3 = netpath::path_type(Path::new("\\\\blaa\\blaa\\"));
    let p4 = netpath::path_type(Path::new("C:\\"));
    if let Err(e) = netpath::path_type(Path::new("CTioewurouewjfndslfsdkfjksdfh")) {
        println!("{e}");
    }

    if p1.is_fixed() {
        println!("it's fixed!")
    }

    println!("{p2:?} {p3:?} {p4:?}");
}