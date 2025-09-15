use freedesktop_apps::{ApplicationEntry, ParseError};
use std::path::Path;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_complete_application_entry() {
    let path = fixture_path("complete_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse complete app");

    // Basic required fields
    assert_eq!(entry.entry_type(), Some("Application".to_string()));
    assert_eq!(entry.name(), Some("Complete Test Application".to_string()));
    assert_eq!(entry.exec(), Some("test-app --mode=%f %F".to_string()));
    
    // Optional fields
    assert_eq!(entry.generic_name(), Some("Test App".to_string()));
    assert_eq!(entry.comment(), Some("A comprehensive test application demonstrating all features".to_string()));
    assert_eq!(entry.icon(), Some("test-complete-app".to_string()));
    assert_eq!(entry.path_dir(), Some("/tmp/test-workspace".to_string()));
    
    // Boolean fields
    assert!(!entry.terminal());
    assert!(!entry.is_hidden());
    assert!(!entry.no_display());
    assert_eq!(entry.get_bool("StartupNotify"), Some(true));
    assert_eq!(entry.get_bool("DBusActivatable"), Some(false));
    assert_eq!(entry.get_bool("PrefersNonDefaultGPU"), Some(false));
    assert_eq!(entry.get_bool("SingleMainWindow"), Some(false));
    
    // List fields
    assert_eq!(
        entry.mime_types(),
        Some(vec!["text/plain".to_string(), "application/x-test".to_string(), "image/test".to_string()])
    );
    assert_eq!(
        entry.categories(),
        Some(vec!["Development".to_string(), "Utility".to_string(), "Education".to_string()])
    );
    assert_eq!(
        entry.keywords(),
        Some(vec!["test".to_string(), "demo".to_string(), "example".to_string(), "complete".to_string()])
    );
    
    // Additional fields
    assert_eq!(entry.get_string("StartupWMClass"), Some("TestApp".to_string()));
    assert_eq!(entry.get_string("TryExec"), Some("/usr/bin/test-app".to_string()));
    assert_eq!(
        entry.get_vec("OnlyShowIn"),
        Some(vec!["GNOME".to_string(), "KDE".to_string()])
    );
    assert_eq!(
        entry.get_vec("Actions"),
        Some(vec!["new-window".to_string(), "preferences".to_string()])
    );
}

#[test]
fn test_minimal_application_entry() {
    let path = fixture_path("minimal_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse minimal app");

    assert_eq!(entry.entry_type(), Some("Application".to_string()));
    assert_eq!(entry.name(), Some("Minimal App".to_string()));
    assert_eq!(entry.exec(), Some("minimal-app".to_string()));
    
    // Optional fields should be None
    assert_eq!(entry.generic_name(), None);
    assert_eq!(entry.comment(), None);
    assert_eq!(entry.icon(), None);
    
    // Boolean fields should default to false
    assert!(!entry.terminal());
    assert!(!entry.is_hidden());
    assert!(!entry.no_display());
}

#[test]
fn test_link_entry() {
    let path = fixture_path("link_entry.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse link entry");

    assert_eq!(entry.entry_type(), Some("Link".to_string()));
    assert_eq!(entry.name(), Some("Test Website Link".to_string()));
    assert_eq!(entry.comment(), Some("A test link to example.com".to_string()));
    assert_eq!(entry.icon(), Some("web-browser".to_string()));
    assert_eq!(entry.get_string("URL"), Some("https://example.com".to_string()));
    
    // Link entries don't have Exec
    assert_eq!(entry.exec(), None);
}

#[test]
fn test_dbus_activatable_entry() {
    let path = fixture_path("dbus_activatable.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse D-Bus activatable app");

    assert_eq!(entry.entry_type(), Some("Application".to_string()));
    assert_eq!(entry.name(), Some("D-Bus Activatable App".to_string()));
    assert_eq!(entry.get_bool("DBusActivatable"), Some(true));
    assert_eq!(entry.exec(), Some("org.example.DBusApp".to_string()));
}

#[test]
fn test_boolean_parsing() {
    let path = fixture_path("boolean_test.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse boolean test");

    assert!(entry.terminal());
    assert!(!entry.is_hidden());
    assert!(entry.no_display());
    assert_eq!(entry.get_bool("StartupNotify"), Some(false));
    assert_eq!(entry.get_bool("DBusActivatable"), Some(true));
    assert_eq!(entry.get_bool("PrefersNonDefaultGPU"), Some(true));
    assert_eq!(entry.get_bool("SingleMainWindow"), Some(false));
}

#[test]
fn test_numeric_parsing() {
    let path = fixture_path("numeric_test.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse numeric test");

    assert_eq!(entry.get_numeric("X-Test-Integer"), Some(42.0));
    assert!((entry.get_numeric("X-Test-Float").unwrap() - 3.14159).abs() < 0.00001);
    assert_eq!(entry.get_numeric("X-Test-Negative"), Some(-123.45));
    assert!((entry.get_numeric("X-Test-Scientific").unwrap() - 0.000123).abs() < 0.0000001);
}

#[test]
fn test_escape_sequences() {
    let path = fixture_path("escape_sequences.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse escape sequences test");

    let comment = entry.comment().expect("Comment should be present");
    assert!(comment.contains("Testing escape sequences: newline:\nTab:\tCarriage return:\rBackslash:\\"));
    
    let exec = entry.exec().expect("Exec should be present");
    assert!(exec.contains("value with spaces"));
    assert!(exec.contains("$HOME/test\nfile.txt"));
    
    let categories = entry.categories().expect("Categories should be present");
    assert!(categories.contains(&"Test;Category".to_string()));
    assert!(categories.contains(&"Another".to_string()));
    
    let keywords = entry.keywords().expect("Keywords should be present");
    assert!(keywords.contains(&"escape;test".to_string()));
    assert!(keywords.contains(&"special chars".to_string()));
}

#[test]
fn test_comments_and_whitespace() {
    let path = fixture_path("comments_and_whitespace.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse comments test");

    assert_eq!(entry.name(), Some("Comments Test App".to_string()));
    assert_eq!(entry.exec(), Some("comments-test".to_string()));
    assert_eq!(entry.comment(), Some("Testing whitespace".to_string()));
    assert_eq!(entry.icon(), Some("test-icon    # Inline comment should be ignored".to_string()));
}

#[test]
fn test_malformed_missing_required_fields() {
    let path = fixture_path("malformed_missing_required.desktop");
    let result = ApplicationEntry::try_from_path(&path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::MissingRequiredKey(_) => {}, // Expected
        other => panic!("Expected MissingRequiredKey error, got: {:?}", other),
    }
}

#[test]
fn test_malformed_no_group() {
    let path = fixture_path("malformed_no_group.desktop");
    let result = ApplicationEntry::try_from_path(&path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::InvalidFormat(_) => {}, // Expected
        other => panic!("Expected InvalidFormat error, got: {:?}", other),
    }
}

#[test]
fn test_nonexistent_file() {
    let result = ApplicationEntry::try_from_path("/nonexistent/file.desktop");
    
    assert!(result.is_err());
    match result.unwrap_err() {
        ParseError::IoError(_) => {}, // Expected
        other => panic!("Expected IoError, got: {:?}", other),
    }
}

#[test]
fn test_from_path_fallback() {
    // This should not panic even with invalid file
    let path = fixture_path("malformed_missing_required.desktop");
    let entry = ApplicationEntry::from_path(&path);
    
    // Should get a default entry
    assert_eq!(entry.name(), None);
    assert_eq!(entry.entry_type(), None);
}

#[test]
fn test_path_method() {
    let path = fixture_path("minimal_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse");
    
    assert_eq!(entry.path(), Path::new(&path));
}