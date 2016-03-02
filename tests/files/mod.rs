extern crate flate2;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read, Write};

use self::flate2::FlateReadExt;

#[allow(dead_code)]
pub fn read(path: &str) -> Vec<u8> {
    let path = Path::new("tests").join("input").join(&format!("{}.gz", path));
    let size = path.metadata().expect("Couldn't determine filesize").len();
    let mut out = Vec::with_capacity(size as usize);
    let mut input = BufReader::new(File::open(path).expect("Couldn't open file"))
                        .gz_decode()
                        .expect("Couldn't decode GZ stream");
    input.read_to_end(&mut out).expect("Couldn't read data");
    out
}

#[allow(dead_code)]
pub fn write<P: AsRef<Path>>(p: P, b: &[u8]) {
    let mut out = File::create(p).expect("Couldn't create file");
    out.write_all(b).expect("Couldn't write data");
}
