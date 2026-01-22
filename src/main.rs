use std::path::Path;

fn main() {
    let p1 = netpath::path_type(Path::new("c:")).unwrap();
    let p2 = netpath::path_type(Path::new("D:\\blaa\\blaa")).unwrap();
    let p3 = netpath::path_type(Path::new("\\\\")).unwrap();
    let p4 = netpath::path_type(Path::new("C:\\")).unwrap();

    println!("{p1} {p2} {p3} {p4}");
}