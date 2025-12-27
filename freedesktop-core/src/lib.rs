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

pub fn xdg_cache_home() -> PathBuf {
    let Ok(dir) = std::env::var("XDG_CACHE_HOME") else {
        let home = std::env::var("HOME").expect("CRITICAL: $HOME variable not set or available");
        return PathBuf::from(home).join(".cache");
    };

    PathBuf::from(dir)
}

pub fn xdg_config_home() -> PathBuf {
    let Ok(dir) = std::env::var("XDG_CONFIG_HOME") else {
        let home = std::env::var("HOME").expect("CRITICAL: $HOME variable not set or available");
        return PathBuf::from(home).join(".config");
    };

    PathBuf::from(dir)
}

pub fn xdg_data_home() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .expect("$HOME environment variable not set. This is a critical failure");
            PathBuf::from(home).join(".local/share")
        })
}

pub fn xdg_state_home() -> PathBuf {
    let Ok(dir) = std::env::var("XDG_STATE_HOME") else {
        let home = std::env::var("HOME").expect("CRITICAL: $HOME variable not set or available");
        return PathBuf::from(home).join(".local/state");
    };

    PathBuf::from(dir)
}

pub fn xdg_data_dirs() -> Vec<PathBuf> {
    let raw = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    raw.split(':').map(PathBuf::from).collect()
}
