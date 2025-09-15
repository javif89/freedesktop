use std::path::{Path, PathBuf};

mod parser;
use parser::{DesktopEntry, ValueType};

// Re-export the ParseError from parser
pub use parser::ParseError;

pub fn application_entry_paths() -> Vec<PathBuf> {
    freedesktop_core::base_directories()
        .iter()
        .map(|path| path.join("applications"))
        .filter(|path| path.exists())
        .collect()
}

#[derive(Debug)]
pub struct ApplicationEntry {
    inner: DesktopEntry,
}

impl Default for ApplicationEntry {
    fn default() -> Self {
        Self {
            inner: DesktopEntry::default(),
        }
    }
}

impl ApplicationEntry {
    /// Get the application name
    pub fn name(&self) -> Option<String> {
        self.get_string("Name")
    }

    /// Get the desktop file ID according to the freedesktop specification
    /// 
    /// The desktop file ID is computed by making the file path relative to the
    /// XDG_DATA_DIRS component, removing "applications/" prefix, and converting
    /// '/' to '-'. For example: /usr/share/applications/foo/bar.desktop → foo-bar.desktop
    pub fn id(&self) -> Option<String> {
        let file_path = &self.inner.path;
        
        // Check if this file is within any applications directory
        if let Some(apps_pos) = file_path.to_string_lossy().find("/applications/") {
            let after_apps = &file_path.to_string_lossy()[apps_pos + "/applications/".len()..];
            if let Some(desktop_entry_path) = after_apps.strip_suffix(".desktop") {
                // Convert path separators to dashes for subdirectories
                return Some(desktop_entry_path.replace('/', "-"));
            }
        }
        
        // Fallback: just use filename without extension
        file_path.file_stem()
            .map(|name| name.to_string_lossy().to_string())
    }

    /// Get the executable command
    pub fn exec(&self) -> Option<String> {
        self.get_string("Exec")
    }

    /// Get the icon name or path
    pub fn icon(&self) -> Option<String> {
        self.get_string("Icon")
    }

    /// Get a string value from the Desktop Entry group
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.inner
            .get_desktop_entry_group()
            .and_then(|group| group.get_field(key))
            .and_then(|value| match value {
                ValueType::String(s) | ValueType::LocaleString(s) | ValueType::IconString(s) => {
                    Some(s.clone())
                }
                _ => None,
            })
    }

    /// Get a localized string value from the Desktop Entry group
    pub fn get_localized_string(&self, key: &str, locale: Option<&str>) -> Option<String> {
        self.inner
            .get_desktop_entry_group()
            .and_then(|group| group.get_localized_field(key, locale))
            .and_then(|value| match value {
                ValueType::String(s) | ValueType::LocaleString(s) | ValueType::IconString(s) => {
                    Some(s.clone())
                }
                _ => None,
            })
    }

    /// Get a boolean value from the Desktop Entry group
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.inner
            .get_desktop_entry_group()
            .and_then(|group| group.get_field(key))
            .and_then(|value| match value {
                ValueType::Boolean(b) => Some(*b),
                _ => None,
            })
    }

    /// Get a numeric value from the Desktop Entry group
    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.inner
            .get_desktop_entry_group()
            .and_then(|group| group.get_field(key))
            .and_then(|value| match value {
                ValueType::Numeric(n) => Some(*n),
                _ => None,
            })
    }

    /// Get a vector of strings from the Desktop Entry group
    pub fn get_vec(&self, key: &str) -> Option<Vec<String>> {
        self.inner
            .get_desktop_entry_group()
            .and_then(|group| group.get_field(key))
            .and_then(|value| match value {
                ValueType::StringList(list) | ValueType::LocaleStringList(list) => {
                    Some(list.clone())
                }
                _ => None,
            })
    }

    /// Get the file path of this desktop entry
    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    /// Get the entry type (Application, Link, Directory)
    pub fn entry_type(&self) -> Option<String> {
        self.get_string("Type")
    }

    /// Get generic name (e.g., "Web Browser")
    pub fn generic_name(&self) -> Option<String> {
        self.get_string("GenericName")
    }

    /// Get comment/description
    pub fn comment(&self) -> Option<String> {
        self.get_string("Comment")
    }

    pub fn should_show(&self) -> bool {
        !self.is_hidden() && !self.no_display()
    }

    /// Check if entry should be hidden
    pub fn is_hidden(&self) -> bool {
        self.get_bool("Hidden").unwrap_or(false)
    }

    /// Check if entry should not be displayed in menus
    pub fn no_display(&self) -> bool {
        self.get_bool("NoDisplay").unwrap_or(false)
    }

    /// Get supported MIME types
    pub fn mime_types(&self) -> Option<Vec<String>> {
        self.get_vec("MimeType")
    }

    /// Get categories
    pub fn categories(&self) -> Option<Vec<String>> {
        self.get_vec("Categories")
    }

    /// Get keywords for searching
    pub fn keywords(&self) -> Option<Vec<String>> {
        self.get_vec("Keywords")
    }

    /// Check if application runs in terminal
    pub fn terminal(&self) -> bool {
        self.get_bool("Terminal").unwrap_or(false)
    }

    /// Get working directory
    pub fn path_dir(&self) -> Option<String> {
        self.get_string("Path")
    }
}

impl ApplicationEntry {
    /// Get all application entries from standard directories
    pub fn all() -> Vec<ApplicationEntry> {
        let mut entries: Vec<ApplicationEntry> = Vec::new();
        for p in application_entry_paths() {
            if let Ok(dir_entries) = std::fs::read_dir(p) {
                for entry in dir_entries.filter_map(|e| e.ok()) {
                    if entry.path().extension().is_some_and(|ext| ext == "desktop") {
                        if let Ok(app_entry) = ApplicationEntry::try_from_path(entry.path()) {
                            entries.push(app_entry);
                        }
                    }
                }
            }
        }
        entries
    }

    /// Create an ApplicationEntry from a path, panicking on error (for compatibility)
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self::try_from_path(path).unwrap_or_else(|_| {
            // Return empty entry if parsing fails to maintain compatibility
            ApplicationEntry::default()
        })
    }

    /// Try to create an ApplicationEntry from a path, returning Result
    pub fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let desktop_entry = DesktopEntry::from_path(path)?;
        Ok(ApplicationEntry {
            inner: desktop_entry,
        })
    }
}
