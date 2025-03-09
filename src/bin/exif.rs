extern crate exif;

use std::env;

fn dump_exif(path: &String) {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).unwrap();
    for f in exif.fields() {
        println!("{} {} {}",
        f.tag, f.ifd_num, f.display_value().with_unit(&exif));
    }
}

fn main() {
   let args: Vec<String> = env::args().collect();
   dump_exif(&args[1]);
}
