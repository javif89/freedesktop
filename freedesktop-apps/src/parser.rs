use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
pub enum ParseError {
    IoError(String),
    InvalidFormat(String),
    MissingRequiredKey(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String(String),
    #[allow(dead_code)] // Reserved for future localization features
    LocaleString(String),
    #[allow(dead_code)] // Reserved for future icon handling
    IconString(String),
    Boolean(bool),
    Numeric(f64),
    StringList(Vec<String>),
    #[allow(dead_code)] // Reserved for future localization features
    LocaleStringList(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct LocalizedKey {
    pub key: String,
    pub locale: Option<String>,
}

impl LocalizedKey {
    pub fn parse(input: &str) -> Self {
        if let Some(bracket_start) = input.find('[') {
            if let Some(bracket_end) = input.find(']') {
                if bracket_start < bracket_end {
                    let key = input[..bracket_start].to_string();
                    let locale = input[bracket_start + 1..bracket_end].to_string();
                    return Self {
                        key,
                        locale: Some(locale),
                    };
                }
            }
        }
        Self {
            key: input.to_string(),
            locale: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct DesktopEntryGroup {
    #[allow(dead_code)] // Reserved for future group name tracking
    pub name: String,
    pub fields: HashMap<String, ValueType>,
    pub localized_fields: HashMap<String, HashMap<String, ValueType>>,
}

impl DesktopEntryGroup {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: HashMap::new(),
            localized_fields: HashMap::new(),
        }
    }

    pub fn insert_field(&mut self, key: &str, value: ValueType) {
        let localized_key = LocalizedKey::parse(key);
        
        if let Some(locale) = localized_key.locale {
            self.localized_fields
                .entry(localized_key.key)
                .or_default()
                .insert(locale, value);
        } else {
            self.fields.insert(localized_key.key, value);
        }
    }

    pub fn get_field(&self, key: &str) -> Option<&ValueType> {
        self.fields.get(key)
    }

    pub fn get_localized_field(&self, key: &str, locale: Option<&str>) -> Option<&ValueType> {
        if let Some(locale) = locale {
            if let Some(localized_map) = self.localized_fields.get(key) {
                // Try exact match first
                if let Some(value) = localized_map.get(locale) {
                    return Some(value);
                }
                
                // Try fallback logic according to spec
                if let Some(value) = self.try_locale_fallback(localized_map, locale) {
                    return Some(value);
                }
            }
        }
        
        // Fall back to non-localized version
        self.fields.get(key)
    }

    fn try_locale_fallback<'a>(&self, localized_map: &'a HashMap<String, ValueType>, locale: &str) -> Option<&'a ValueType> {
        // Strip encoding part if present (everything after '.')
        let locale_without_encoding = if let Some(dot_pos) = locale.find('.') {
            &locale[..dot_pos]
        } else {
            locale
        };
        
        // Parse locale components: lang_COUNTRY@MODIFIER
        let (lang, country, modifier) = Self::parse_locale_components(locale_without_encoding);
        
        // Follow the spec fallback order exactly:
        // For lang_COUNTRY@MODIFIER: try lang_COUNTRY@MODIFIER, lang_COUNTRY, lang@MODIFIER, lang, default
        // For lang_COUNTRY: try lang_COUNTRY, lang, default  
        // For lang@MODIFIER: try lang@MODIFIER, lang, default
        // For lang: try lang, default
        
        if let (Some(country), Some(modifier)) = (country, modifier) {
            // Try lang_COUNTRY@MODIFIER
            let full_locale = format!("{}_{}{}", lang, country, modifier);
            if let Some(value) = localized_map.get(&full_locale) {
                return Some(value);
            }
            
            // Try lang_COUNTRY
            let lang_country = format!("{}_{}", lang, country);
            if let Some(value) = localized_map.get(&lang_country) {
                return Some(value);
            }
            
            // Try lang@MODIFIER
            let lang_modifier = format!("{}{}", lang, modifier);
            if let Some(value) = localized_map.get(&lang_modifier) {
                return Some(value);
            }
        } else if let Some(country) = country {
            // Try lang_COUNTRY
            let lang_country = format!("{}_{}", lang, country);
            if let Some(value) = localized_map.get(&lang_country) {
                return Some(value);
            }
        } else if let Some(modifier) = modifier {
            // Try lang@MODIFIER
            let lang_modifier = format!("{}{}", lang, modifier);
            if let Some(value) = localized_map.get(&lang_modifier) {
                return Some(value);
            }
        }
        
        // Try just lang
        localized_map.get(lang)
    }
    
    fn parse_locale_components(locale: &str) -> (&str, Option<&str>, Option<&str>) {
        let (base, modifier) = if let Some(at_pos) = locale.find('@') {
            (&locale[..at_pos], Some(&locale[at_pos..]))
        } else {
            (locale, None)
        };
        
        let (lang, country) = if let Some(under_pos) = base.find('_') {
            (&base[..under_pos], Some(&base[under_pos + 1..]))
        } else {
            (base, None)
        };
        
        (lang, country, modifier)
    }
}

#[derive(Debug, Default)]
pub struct DesktopEntry {
    pub path: PathBuf,
    pub groups: HashMap<String, DesktopEntryGroup>,
}

impl DesktopEntry {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        let file = File::open(path.as_ref())
            .map_err(|e| ParseError::IoError(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);
        
        let group_header_regex = Regex::new(r"^\[([^\[\]]+)\]$")
            .map_err(|e| ParseError::InvalidFormat(format!("Regex error: {}", e)))?;

        let mut current_group: Option<String> = None;
        let mut entry = DesktopEntry { 
            path: path.as_ref().to_path_buf(), 
            ..Default::default() 
        };
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| ParseError::IoError(format!("Failed to read line {}: {}", line_num + 1, e)))?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for group header
            if let Some(captures) = group_header_regex.captures(line) {
                let group_name = captures[1].to_string();
                current_group = Some(group_name.clone());
                entry.groups.entry(group_name.clone())
                    .or_insert_with(|| DesktopEntryGroup::new(group_name));
                continue;
            }

            // Parse key-value pair
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                if key.is_empty() {
                    continue; // Skip invalid entries
                }

                if !is_valid_key_name(key) {
                    return Err(ParseError::InvalidFormat(format!("Invalid key name: {}", key)));
                }

                if let Some(ref group_name) = current_group {
                    let parsed_value = parse_value(value)?;
                    if let Some(group) = entry.groups.get_mut(group_name) {
                        group.insert_field(key, parsed_value);
                    }
                } else {
                    return Err(ParseError::InvalidFormat("Key-value pair found before any group header".to_string()));
                }
            }
        }

        // Validate required keys
        entry.validate()?;
        
        Ok(entry)
    }

    fn validate(&self) -> Result<(), ParseError> {
        let desktop_entry = self.groups.get("Desktop Entry")
            .ok_or_else(|| ParseError::MissingRequiredKey("Desktop Entry group is required".to_string()))?;

        // Type is required
        let entry_type = desktop_entry.get_field("Type")
            .ok_or_else(|| ParseError::MissingRequiredKey("Type key is required".to_string()))?;

        // Name is required
        desktop_entry.get_field("Name")
            .ok_or_else(|| ParseError::MissingRequiredKey("Name key is required".to_string()))?;

        // For Application type, Exec is required unless DBusActivatable=true
        if let ValueType::String(type_val) = entry_type {
            if type_val == "Application" {
                let dbus_activatable = desktop_entry.get_field("DBusActivatable")
                    .and_then(|v| match v {
                        ValueType::Boolean(b) => Some(*b),
                        _ => None,
                    })
                    .unwrap_or(false);

                if !dbus_activatable {
                    desktop_entry.get_field("Exec")
                        .ok_or_else(|| ParseError::MissingRequiredKey("Exec key is required for Application type".to_string()))?;
                }
            } else if type_val == "Link" {
                // URL is required for Link type
                desktop_entry.get_field("URL")
                    .ok_or_else(|| ParseError::MissingRequiredKey("URL key is required for Link type".to_string()))?;
            }
        }

        Ok(())
    }

    pub fn get_desktop_entry_group(&self) -> Option<&DesktopEntryGroup> {
        self.groups.get("Desktop Entry")
    }
}

fn is_valid_key_name(key: &str) -> bool {
    // Remove locale part for validation
    let base_key = if let Some(bracket_pos) = key.find('[') {
        &key[..bracket_pos]
    } else {
        key
    };
    
    // Only A-Za-z0-9- allowed in key names
    base_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn parse_value(value: &str) -> Result<ValueType, ParseError> {
    // Handle escape sequences
    let unescaped = unescape_value(value);
    
    // Try to parse as boolean first
    match unescaped.to_lowercase().as_str() {
        "true" => return Ok(ValueType::Boolean(true)),
        "false" => return Ok(ValueType::Boolean(false)),
        _ => {}
    }
    
    // Try to parse as numeric
    if let Ok(num) = unescaped.parse::<f64>() {
        return Ok(ValueType::Numeric(num));
    }
    
    // Check if it's a list (contains unescaped semicolons)
    if value.contains(';') {
        let items = split_semicolon_list(value);
        return Ok(ValueType::StringList(items));
    }
    
    // Default to string
    Ok(ValueType::String(unescaped))
}

fn unescape_value(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    's' => result.push(' '),
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    ';' => result.push(';'),  // For escaped semicolons in lists
                    _ => {
                        // Unknown escape sequence, keep as-is
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

fn split_semicolon_list(value: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_item = String::new();
    let mut chars = value.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    // Escaped semicolon - add semicolon to current item
                    current_item.push(';');
                    chars.next(); // consume the semicolon
                } else {
                    // Other escape sequence - handle normally
                    current_item.push(ch);
                    if let Some(escaped_ch) = chars.next() {
                        current_item.push(escaped_ch);
                    }
                }
            } else {
                current_item.push(ch);
            }
        } else if ch == ';' {
            // Unescaped semicolon - end current item
            let trimmed = current_item.trim();
            if !trimmed.is_empty() {
                result.push(unescape_value(trimmed));
            }
            current_item.clear();
        } else {
            current_item.push(ch);
        }
    }
    
    // Add the last item
    let trimmed = current_item.trim();
    if !trimmed.is_empty() {
        result.push(unescape_value(trimmed));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_key_parsing() {
        let key = LocalizedKey::parse("Name");
        assert_eq!(key.key, "Name");
        assert_eq!(key.locale, None);

        let key = LocalizedKey::parse("Name[en_US]");
        assert_eq!(key.key, "Name");
        assert_eq!(key.locale, Some("en_US".to_string()));
    }

    #[test]
    fn test_value_parsing() {
        assert_eq!(parse_value("true").unwrap(), ValueType::Boolean(true));
        assert_eq!(parse_value("false").unwrap(), ValueType::Boolean(false));
        assert_eq!(parse_value("123.45").unwrap(), ValueType::Numeric(123.45));
        assert_eq!(parse_value("hello").unwrap(), ValueType::String("hello".to_string()));
        assert_eq!(
            parse_value("one;two;three").unwrap(),
            ValueType::StringList(vec!["one".to_string(), "two".to_string(), "three".to_string()])
        );
    }

    #[test]
    fn test_escape_sequences() {
        assert_eq!(unescape_value("hello\\sworld"), "hello world");
        assert_eq!(unescape_value("line1\\nline2"), "line1\nline2");
        assert_eq!(unescape_value("tab\\there"), "tab\there");
        assert_eq!(unescape_value("backslash\\\\"), "backslash\\");
    }

    #[test]
    fn test_key_validation() {
        assert!(is_valid_key_name("Name"));
        assert!(is_valid_key_name("Name[en_US]"));
        assert!(is_valid_key_name("X-Custom-Key"));
        assert!(!is_valid_key_name("Invalid Key"));
        assert!(!is_valid_key_name("Key=Value"));
    }
}