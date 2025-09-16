pub mod info;
use std::path::PathBuf;

/// The base directories all other searches are
/// based on. Data comes from XDG_DATA_DIRS
pub fn base_directories() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Ok(var_str) = std::env::var("XDG_DATA_DIRS") {
        for p in var_str.split(":") {
            let pb = PathBuf::from(p);

            if pb.exists() {
                dirs.push(pb);
            }
        }
    }

    if let Ok(var_str) = std::env::var("XDG_DATA_HOME") {
        let pb = PathBuf::from(var_str);

        if pb.exists() {
            dirs.push(pb);
        }
    }

    dirs
}
