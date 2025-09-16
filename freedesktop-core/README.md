# freedesktop-core

Core freedesktop utilities for XDG base directories and desktop environment detection.

## Features

- **XDG Base Directory discovery** - Find standard data, config, and cache directories
- **Desktop environment detection** - Identify the current desktop environment
- **Cross-platform** - Works on Linux, BSD, and other Unix-like systems

## Usage

```rust
use freedesktop_core::{base_directories, current_desktop};

// Get XDG data directories
for dir in base_directories() {
    let apps_dir = dir.join("applications");
    if apps_dir.exists() {
        println!("Applications directory: {}", apps_dir.display());
    }
}

// Detect desktop environment
if let Some(desktop) = current_desktop() {
    println!("Running on: {}", desktop);
}
```

## XDG Specification

This crate implements the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

- Uses `XDG_DATA_DIRS` environment variable (defaults to `/usr/local/share:/usr/share`)
- Uses `XDG_DATA_HOME` environment variable (defaults to `~/.local/share`)
- Respects `XDG_CURRENT_DESKTOP` for desktop environment detection