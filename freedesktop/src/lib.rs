//! # freedesktop
//! 
//! Rust implementations of the freedesktop.org specifications for Linux desktop integration.
//! 
//! This crate provides a unified interface to freedesktop standards through optional features.
//! 
//! ## Features
//! 
//! - **`core`** (default) - XDG base directories and desktop environment detection
//! - **`apps`** (default) - Desktop Entry parsing and application execution  
//! - **`cli`** - Command-line utilities (enables `apps`)
//! 
//! ## Quick Start
//! 
//! ### XDG Base Directories
//! 
//! ```rust
//! # #[cfg(feature = "core")]
//! # {
//! use freedesktop::base_directories;
//! 
//! for dir in base_directories() {
//!     println!("XDG data directory: {}", dir.display());
//! }
//! # }
//! ```
//! 
//! ### Desktop Applications
//! 
//! ```rust
//! # #[cfg(feature = "apps")]
//! # {
//! use freedesktop::ApplicationEntry;
//! 
//! // List all applications
//! for app in ApplicationEntry::all() {
//!     if app.should_show() {
//!         println!("{}: {}", app.id().unwrap_or_default(), app.name().unwrap_or_default());
//!     }
//! }
//! 
//! // Parse a desktop file (would normally execute an app)
//! # let desktop_content = "[Desktop Entry]\nType=Application\nName=Test App\nExec=test-app\n";
//! # std::fs::write("/tmp/test.desktop", desktop_content).unwrap();
//! let app = ApplicationEntry::try_from_path("/tmp/test.desktop").unwrap();
//! // app.execute().unwrap(); // Would launch the application
//! # std::fs::remove_file("/tmp/test.desktop").ok();
//! # }
//! ```
//! 
//! ## Feature Usage
//! 
//! ```toml
//! # Default - includes core and apps
//! freedesktop = "0.1.0"
//! 
//! # Only XDG base directories
//! freedesktop = { version = "0.1.0", default-features = false, features = ["core"] }
//! 
//! # Only desktop applications (automatically includes core)
//! freedesktop = { version = "0.1.0", default-features = false, features = ["apps"] }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export core functionality
#[cfg(feature = "core")]
#[cfg_attr(docsrs, doc(cfg(feature = "core")))]
pub use freedesktop_core::*;

// Re-export apps functionality
#[cfg(feature = "apps")]
#[cfg_attr(docsrs, doc(cfg(feature = "apps")))]
pub use freedesktop_apps::*;