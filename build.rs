use std::fs::{self, File};
use std::path::Path;

fn main() {
    let path = Path::new(".git").join("refs").join("heads").join("master");
    if !path.exists() {
        let _ = fs::create_dir_all(path.parent().unwrap());
        let _ = File::create(&path);
    }
}
