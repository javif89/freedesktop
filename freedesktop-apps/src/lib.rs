use std::path::{Path, PathBuf};

mod parser;
use parser::{DesktopEntry, ValueType};

// Re-export the ParseError from parser
pub use parser::ParseError;

#[derive(Debug, Clone)]
pub enum ExecuteError {
    NotExecutable(String),
    TerminalNotFound,
    InvalidCommand(String),
    IoError(String),
    ValidationFailed(String),
}

pub fn application_entry_paths() -> Vec<PathBuf> {
    freedesktop_core::base_directories()
        .iter()
        .map(|path| path.join("applications"))
        .filter(|path| path.exists())
        .collect()
}

#[derive(Debug)]
#[derive(Default)]
pub struct ApplicationEntry {
    inner: DesktopEntry,
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

    /// Execute this application with no files
    pub fn execute(&self) -> Result<(), ExecuteError> {
        self.execute_with_files(&[])
    }

    /// Execute this application with the given files
    pub fn execute_with_files(&self, files: &[&str]) -> Result<(), ExecuteError> {
        self.execute_internal(files, &[])
    }

    /// Execute this application with the given URLs
    pub fn execute_with_urls(&self, urls: &[&str]) -> Result<(), ExecuteError> {
        self.execute_internal(&[], urls)
    }

    /// Prepare the command for execution without actually executing it (for testing)
    pub fn prepare_command(&self, files: &[&str], urls: &[&str]) -> Result<(String, Vec<String>), ExecuteError> {
        // Validate the application can be executed
        self.validate_executable()?;

        // Get the command and arguments
        let (program, args) = self.parse_exec_command(files, urls)?;

        // Handle terminal applications
        let (final_program, final_args) = if self.terminal() {
            self.wrap_with_terminal(&program, &args)?
        } else {
            (program, args)
        };

        Ok((final_program, final_args))
    }

    fn execute_internal(&self, files: &[&str], urls: &[&str]) -> Result<(), ExecuteError> {
        // Validate the application can be executed
        self.validate_executable()?;

        // Get the command and arguments
        let (program, args) = self.parse_exec_command(files, urls)?;

        // Handle terminal applications
        let (final_program, final_args) = if self.terminal() {
            self.wrap_with_terminal(&program, &args)?
        } else {
            (program, args)
        };

        // Set working directory if specified
        let working_dir = self.path_dir();
        
        // Spawn the process detached
        spawn_detached_with_env(&final_program, &final_args, working_dir.as_deref())
            .map_err(|e| ExecuteError::IoError(format!("Failed to spawn process: {}", e)))
    }

    fn validate_executable(&self) -> Result<(), ExecuteError> {
        // Check if we have an Exec key
        let exec = self.exec().ok_or_else(|| {
            ExecuteError::NotExecutable("No Exec key found".to_string())
        })?;

        if exec.trim().is_empty() {
            return Err(ExecuteError::NotExecutable("Exec key is empty".to_string()));
        }

        // Check TryExec if present
        if let Some(try_exec) = self.get_string("TryExec") {
            if !is_executable_available(&try_exec) {
                return Err(ExecuteError::ValidationFailed(
                    format!("TryExec '{}' not found or not executable", try_exec)
                ));
            }
        }

        Ok(())
    }

    fn parse_exec_command(&self, files: &[&str], urls: &[&str]) -> Result<(String, Vec<String>), ExecuteError> {
        let exec = self.exec().unwrap(); // Already validated in validate_executable
        
        // Expand field codes
        let expanded = self.expand_field_codes(&exec, files, urls);
        
        // Parse the command line
        parse_command_line(&expanded)
    }

    fn expand_field_codes(&self, exec: &str, files: &[&str], urls: &[&str]) -> String {
        let mut result = String::new();
        let mut chars = exec.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next_ch) = chars.peek() {
                    chars.next(); // consume the next character
                    match next_ch {
                        '%' => result.push('%'),
                        'f' => {
                            if let Some(file) = files.first() {
                                result.push_str(&shell_escape(file));
                            }
                        },
                        'F' => {
                            for (i, file) in files.iter().enumerate() {
                                if i > 0 { result.push(' '); }
                                result.push_str(&shell_escape(file));
                            }
                        },
                        'u' => {
                            if let Some(url) = urls.first() {
                                result.push_str(&shell_escape(url));
                            }
                        },
                        'U' => {
                            for (i, url) in urls.iter().enumerate() {
                                if i > 0 { result.push(' '); }
                                result.push_str(&shell_escape(url));
                            }
                        },
                        'i' => {
                            if let Some(icon) = self.icon() {
                                result.push_str("--icon ");
                                result.push_str(&shell_escape(&icon));
                            }
                        },
                        'c' => {
                            if let Some(name) = self.name() {
                                result.push_str(&shell_escape(&name));
                            }
                        },
                        'k' => {
                            let path = self.path().to_string_lossy();
                            result.push_str(&shell_escape(&path));
                        },
                        // Deprecated field codes - ignore
                        'd' | 'D' | 'n' | 'N' | 'v' | 'm' => {},
                        // Unknown field code - this is an error per spec
                        _ => {
                            return format!("{}%{}{}", result, next_ch, chars.collect::<String>());
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn wrap_with_terminal(&self, program: &str, args: &[String]) -> Result<(String, Vec<String>), ExecuteError> {
        let terminal = find_terminal().ok_or(ExecuteError::TerminalNotFound)?;
        
        // Build the command to run in terminal
        let mut terminal_args = vec!["-e".to_string()];
        terminal_args.push(program.to_string());
        terminal_args.extend(args.iter().cloned());
        
        Ok((terminal, terminal_args))
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

/// Spawn a process completely detached from the current process while preserving display environment
fn spawn_detached_with_env(program: &str, args: &[String], working_dir: Option<&str>) -> Result<(), std::io::Error> {
    use std::process::{Command, Stdio};
    
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Explicitly preserve important environment variables
        if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
            cmd.env("WAYLAND_DISPLAY", wayland_display);
        }
        if let Ok(display) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", display);
        }
        if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
        }
        if let Ok(xdg_session_type) = std::env::var("XDG_SESSION_TYPE") {
            cmd.env("XDG_SESSION_TYPE", xdg_session_type);
        }
        if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            cmd.env("XDG_CURRENT_DESKTOP", xdg_current_desktop);
        }

        unsafe {
            cmd.pre_exec(|| {
                // Start new process group but don't create new session
                // This allows detachment while preserving session environment
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        cmd.spawn()?;
        Ok(())
    }
    
    #[cfg(not(unix))]
    {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        
        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        cmd.spawn()?;
        Ok(())
    }
}

/// Check if an executable is available in PATH or as absolute path
fn is_executable_available(executable: &str) -> bool {
    use std::path::Path;
    
    if Path::new(executable).is_absolute() {
        // Absolute path - check if file exists and is executable
        Path::new(executable).exists()
    } else {
        // Relative path - check in PATH
        which_command(executable).is_some()
    }
}

/// Find an executable in PATH (simple implementation)
fn which_command(executable: &str) -> Option<String> {
    if let Ok(path_var) = std::env::var("PATH") {
        for path_dir in path_var.split(':') {
            let full_path = format!("{}/{}", path_dir, executable);
            if std::path::Path::new(&full_path).exists() {
                return Some(full_path);
            }
        }
    }
    None
}

/// Find an available terminal emulator
fn find_terminal() -> Option<String> {
    // First check TERMINAL environment variable
    if let Ok(terminal) = std::env::var("TERMINAL") {
        if is_executable_available(&terminal) {
            return Some(terminal);
        }
    }
    
    // Try common terminal emulators
    let terminals = [
        "x-terminal-emulator",  // Debian/Ubuntu alternative
        "gnome-terminal",
        "konsole",
        "xfce4-terminal", 
        "mate-terminal",
        "lxterminal",
        "rxvt-unicode",
        "rxvt",
        "xterm",
    ];
    
    for terminal in &terminals {
        if is_executable_available(terminal) {
            return Some(terminal.to_string());
        }
    }
    
    None
}

/// Escape a string for safe shell usage
fn shell_escape(s: &str) -> String {
    if s.chars().any(|c| " \t\n'\"\\$`()[]{}?*~&|;<>".contains(c)) {
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    } else {
        s.to_string()
    }
}

/// Parse a command line into program and arguments, handling quotes
fn parse_command_line(command: &str) -> Result<(String, Vec<String>), ExecuteError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    let mut chars = command.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            },
            ch if ch == quote_char && in_quotes => {
                in_quotes = false;
            },
            '\\' if in_quotes => {
                // Handle escape sequences in quotes
                if let Some(&next_ch) = chars.peek() {
                    chars.next();
                    match next_ch {
                        '"' | '\'' | '\\' | '$' | '`' => current.push(next_ch),
                        _ => {
                            current.push('\\');
                            current.push(next_ch);
                        }
                    }
                } else {
                    current.push('\\');
                }
            },
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
                // Skip multiple spaces
                while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                    chars.next();
                }
            },
            _ => current.push(ch),
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    if in_quotes {
        return Err(ExecuteError::InvalidCommand("Unterminated quote".to_string()));
    }
    
    if parts.is_empty() {
        return Err(ExecuteError::InvalidCommand("Empty command".to_string()));
    }
    
    let program = parts.remove(0);
    Ok((program, parts))
}
