# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace implementing freedesktop standards. The project is structured as a multi-crate workspace with three main components:

- **freedesktop-core**: Core functionality for XDG base directory discovery and desktop environment detection
- **freedesktop-apps**: Application entry parsing and discovery (desktop files)
- **freedesktop-cli**: Command-line interface for listing applications

## Architecture

The crates follow a layered dependency structure:
- `freedesktop-cli` depends on `freedesktop-apps`
- `freedesktop-apps` depends on `freedesktop-core`
- `freedesktop-core` has minimal external dependencies (only `dirs`)

### Key Components

**freedesktop-core** (`freedesktop-core/src/`):
- `lib.rs`: XDG base directory discovery using `XDG_DATA_DIRS` and `XDG_DATA_HOME`
- `info.rs`: Desktop environment detection via `XDG_CURRENT_DESKTOP`

**freedesktop-apps** (`freedesktop-apps/src/`):
- `lib.rs`: Desktop entry parsing and application discovery
- `ApplicationEntry` struct: Represents parsed .desktop files with group-based structure
- Uses regex for parsing desktop entry group headers

**freedesktop-cli** (`freedesktop-cli/src/`):
- `main.rs`: Simple CLI that lists all discovered application names

## Development Commands

Build the workspace:
```bash
cargo build
```

Build specific crate:
```bash
cargo build -p freedesktop-core
cargo build -p freedesktop-apps
cargo build -p freedesktop-cli
```

Run the CLI:
```bash
cargo run -p freedesktop-cli
```

Run tests:
```bash
cargo test
```

Check code:
```bash
cargo check
cargo clippy
```

## Key Patterns

- Desktop files are parsed using a group-based structure (e.g., "[Desktop Entry]")
- XDG environment variables are used for directory discovery
- Error handling uses `expect()` for file operations that should not fail
- Path building uses `PathBuf::join()` for cross-platform compatibility
- Directory existence is checked before adding to search paths

## Project Memories

- We're building an implementation of the freedesktop spec in rust