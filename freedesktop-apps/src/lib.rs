use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use std::{
    fs::File,
    io::{self, BufRead},
};

pub fn application_entry_paths() -> Vec<PathBuf> {
    freedesktop_core::base_directories()
        .iter()
        .map(|path| path.join("applications"))
        .filter(|path| path.exists())
        .collect()
}

#[derive(Debug, Default)]
pub struct ApplicationEntry {
    path: PathBuf,
    groups: HashMap<String, DesktopEntryGroup>,
}

impl ApplicationEntry {
    pub fn name(&self) -> String {
        self.groups
            .get("Desktop Entry")
            .unwrap()
            .fields
            .get("Name")
            .unwrap()
            .to_string()
    }
}

#[derive(Debug, Default)]
struct DesktopEntryGroup {
    name: String,
    fields: HashMap<String, String>,
}

impl DesktopEntryGroup {
    fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: HashMap::new(),
        }
    }
}

impl ApplicationEntry {
    pub fn all() -> Vec<ApplicationEntry> {
        let mut entries: Vec<ApplicationEntry> = Vec::new();
        for p in application_entry_paths() {
            let result = std::fs::read_dir(p);
            if let Err(_) = result {
                continue;
            }
            for e in result.unwrap().filter_map(|e| e.ok()) {
                if e.path().extension().is_some_and(|ext| ext == "desktop") {
                    entries.push(ApplicationEntry::from_path(e.path()));
                }
            }
        }

        entries
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path.as_ref()).expect("Failed to open file");
        let reader = io::BufReader::new(file);
        let is_group_header = Regex::new(r"^\[([A-Za-z0-9-]+(?: [A-Za-z0-9-]+)*)\]$").unwrap();

        let mut current_group: String = String::new();
        let mut app: ApplicationEntry = ApplicationEntry::default();
        app.path = path.as_ref().into();
        for line in reader.lines() {
            let l = line.expect("Error in line somehow");

            if l.trim().is_empty() {
                continue;
            } else if let Some(group_name) = is_group_header.captures(&l) {
                current_group = String::from(&group_name[1]);
            } else {
                let g = app
                    .groups
                    .entry(current_group.clone())
                    .or_insert_with(|| DesktopEntryGroup::new(current_group.clone()));
                let mut parts = l.splitn(2, "=");
                let k = parts.next().unwrap().trim();
                let v = parts.next().unwrap_or("").trim();
                g.fields.insert(k.into(), v.into());
            }
        }

        app
    }
}
