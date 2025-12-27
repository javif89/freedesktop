use ini::Ini;
use std::{path::PathBuf, sync::LazyLock};

static CURRENT_ICON_THEME: LazyLock<IconTheme> = LazyLock::new(IconTheme::current);

#[derive(Debug, Clone)]
pub struct IconTheme {
    name: String,
    path: PathBuf,
    config: Ini,
}

impl IconTheme {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn config(&self) -> &Ini {
        &self.config
    }

    pub fn config_value<S: Into<String>, A: AsRef<str>>(
        &self,
        section_name: S,
        key: A,
    ) -> Option<String> {
        let cfg = self.config();
        let section = cfg.section(Some(section_name))?;
        let value = section.get(key)?;

        Some(value.to_string())
    }

    pub fn inherits(&self) -> Vec<String> {
        let Some(inherits) = &self.config_value("Icon Theme", "Inherits") else {
            return Vec::new();
        };

        inherits.split(",").map(String::from).collect()
    }

    // Per the spec, we will use $XDG_DATA_HOME/icons/[theme name]
    // as an "overlay" if it exists. Meaning that if a theme
    // has dir /some/system/install/path/[theme]/48x48
    // we will look in $XDG_DATA_HOME/icons/[theme]/48x48
    // if it exists before we look in the system path.
    pub fn icon_dirs(&self, size: u32, scale: u8) -> Vec<PathBuf> {
        let Some(dir_str) = &self.config_value("Icon Theme", "Directories") else {
            return Vec::new();
        };

        let dirs: Vec<String> = dir_str.split(",").map(String::from).collect();
        let overlay_dir = freedesktop_core::xdg_data_home()
            .join("icons")
            .join(&self.name);

        let mut paths: Vec<PathBuf> = Vec::new();

        for d in &dirs {
            let dir_size = &self
                .config_value(d, "Size")
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .unwrap_or(0);

            let dir_scale = match &self.config_value(d, "Scale") {
                Some(s) => s.parse::<u8>().unwrap_or(1),
                None => 1,
            };

            if (dir_size == &size) && (dir_scale == scale) {
                let overlay = overlay_dir.join(d);
                if overlay.exists() {
                    paths.push(overlay);
                }

                paths.push(self.path.join(d));
            }
        }

        paths
    }

    pub fn default_size(&self) -> Option<u32> {
        let Some(size_str) = &self.config_value("Icon Theme", "DesktopDefault") else {
            return None;
        };

        size_str.parse::<u32>().ok()
    }

    // Internal function for getting an icon from the
    // theme. The public get() function actually
    // traverses the inheritance chain.
    fn get_icon(&self, icon_name: &str, size: u32, scale: u8) -> Option<PathBuf> {
        let filenames = [
            format!("{}.{}", icon_name, "svg"),
            format!("{}.{}", icon_name, "png"),
            format!("{}.{}", icon_name, "xpm"),
        ];

        for d in &self.icon_dirs(size, scale) {
            for f in &filenames {
                let icon_path = d.join(f);
                if icon_path.exists() {
                    return Some(icon_path);
                }
            }
        }

        None
    }

    // First option in the process: We go through the current theme
    // and its inherited themes all the way down to see if we
    // can find the icon.
    fn get_through_inheritance(&self, icon_name: &str, size: u32, scale: u8) -> Option<PathBuf> {
        if let Some(icon_path) = &self.get_icon(icon_name, size, scale) {
            return Some(icon_path.to_owned());
        }

        // If we don't find it in the current theme, start recursing
        // into the inheritance chain loading the themes lazily
        for theme_name in &self.inherits() {
            let Some(theme) = IconTheme::from_name(theme_name) else {
                continue;
            };

            match theme.get_through_inheritance(icon_name, size, scale) {
                Some(icon_path) => return Some(icon_path),
                None => continue,
            }
        }

        None
    }

    /// Get an icon by name following the freedesktop icon theme specification
    /// Searches through the current theme and inherited themes for the icon
    pub fn get(&self, icon_name: &str) -> Option<PathBuf> {
        let size = self.default_size().unwrap_or(48);
        let scale: u8 = 1;

        if let Some(path) = &self.get_through_inheritance(icon_name, size, scale) {
            return Some(path.to_owned());
        }

        // Pixmaps are a last resort
        Pixmap::get(icon_name)
    }

    /// Get an icon by name following the freedesktop icon theme specification
    /// Searches through the current theme and inherited themes for the icon
    pub fn get_with_size(&self, icon_name: &str, size: u32) -> Option<PathBuf> {
        let scale: u8 = 1;

        if let Some(path) = &self.get_through_inheritance(icon_name, size, scale) {
            return Some(path.to_owned());
        }

        // Pixmaps are a last resort
        Pixmap::get(icon_name)
    }
}

impl IconTheme {
    /// According to the spec:
    /// First search $XDG_DATA_HOME/icons/[theme name]
    /// If not found, search $XDG_DATA_DIRS in order for
    /// [dir name]/icons/[theme name]
    /// The order of $XDG_DATA_DIRS needs to be respected, as the
    /// first hit counts as the "canonical path" of the theme.
    /// We will also check that an index.theme exists at the path
    /// since any valid theme must have this file.
    pub fn from_name<S: Into<String>>(name: S) -> Option<IconTheme> {
        let name: String = name.into();
        let xdg_home_path = freedesktop_core::xdg_data_home().join("icons").join(&name);

        if xdg_home_path.exists() {
            let config_path = xdg_home_path.join("index.theme");
            if config_path.exists() {
                let config = Ini::load_from_file(&config_path).unwrap_or_else(|_| Ini::new());
                return Some(IconTheme {
                    name,
                    path: xdg_home_path,
                    config,
                });
            }
        }

        for data_dir in freedesktop_core::xdg_data_dirs() {
            let theme_path = data_dir.join("icons").join(&name);

            if theme_path.exists() {
                let config_path = theme_path.join("index.theme");
                if config_path.exists() {
                    let config = Ini::load_from_file(&config_path).unwrap_or_else(|_| Ini::new());
                    return Some(IconTheme {
                        name,
                        path: theme_path,
                        config,
                    });
                }
            }
        }

        None
    }

    pub fn current() -> IconTheme {
        let home = std::env::var("HOME").expect("$HOME variable not set.");
        let config_path = freedesktop_core::xdg_config_home();
        let settings_paths = [
            PathBuf::from(&config_path)
                .join("gtk-4.0")
                .join("settings.ini"),
            PathBuf::from(&config_path)
                .join("gtk-3.0")
                .join("settings.ini"),
            PathBuf::from(&home).join("gtk-4.0").join("settings.ini"),
            PathBuf::from(&home).join("gtk-3.0").join("settings.ini"),
        ];
        let fallback_theme = || {
            IconTheme::from_name("hicolor").expect("The hicolor theme is not present. This is a required fallback theme and must be installed")
        };

        for p in &settings_paths {
            if !p.exists() {
                continue;
            }

            let Ok(conf) = Ini::load_from_file(p) else {
                continue;
            };

            if let Some(section) = conf.section(Some("Settings")) {
                if let Some(theme) = section.get("gtk-icon-theme-name") {
                    return IconTheme::from_name(theme).unwrap_or_else(fallback_theme);
                } else {
                    continue;
                }
            }
        }

        fallback_theme()
    }
}

/// Pixmaps are a fallback when an icon is not found
/// in a theme or any of it's inherited themes.
pub struct Pixmap;

impl Pixmap {
    pub fn get(icon_name: &str) -> Option<PathBuf> {
        let pixmap_paths = freedesktop_core::xdg_data_dirs()
            .into_iter()
            .map(|p| p.join("pixmaps"))
            .filter(|p| p.exists());

        let filenames = [
            format!("{}.{}", icon_name, "svg"),
            format!("{}.{}", icon_name, "png"),
            format!("{}.{}", icon_name, "xpm"),
        ];

        for d in pixmap_paths {
            for f in &filenames {
                let icon_path = d.join(f);
                if icon_path.exists() {
                    return Some(icon_path);
                }
            }
        }

        None
    }
}

/// Convenience function that will:
/// Get the current icon theme from IconTheme::current()
/// Call theme.get() which will get the icon for
/// the default size and scale set for the theme.
/// IconTheme::current() will be cached using LazyLock
/// so multiple calls to this function do not incurr
/// a performance penalty
pub fn get_icon(name: &str) -> Option<PathBuf> {
    CURRENT_ICON_THEME.get(name)
}

/// Convenience function that will:
/// Get the current icon theme from IconTheme::current()
/// Call theme.get_with_size() which will get the icon for
/// the the supplied size and default scale.
/// IconTheme::current() will be cached using LazyLock
/// so multiple calls to this function do not incurr
/// a performance penalty
pub fn get_icon_with_size(name: &str, size: u32) -> Option<PathBuf> {
    CURRENT_ICON_THEME.get_with_size(name, size)
}
