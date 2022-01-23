use std::fs;
use std::io::ErrorKind;

pub fn try_create_dir(dir: &str) {
    match fs::create_dir(dir) {
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => {},
            _ => panic!("{}", e)
        },
        _ => {}
    }
}
