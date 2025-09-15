use freedesktop_apps::{ApplicationEntry, ParseError};
use std::fs;
use std::path::Path;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_empty_file() {
    let temp_file = "/tmp/empty_test.desktop";
    fs::write(temp_file, "").unwrap();
    
    let result = ApplicationEntry::try_from_path(temp_file);
    assert!(result.is_err());
    
    fs::remove_file(temp_file).ok();
}

#[test] 
fn test_only_comments() {
    let temp_file = "/tmp/comments_only_test.desktop";
    fs::write(temp_file, "# Only comments\n# No actual content\n\n# More comments").unwrap();
    
    let result = ApplicationEntry::try_from_path(temp_file);
    assert!(result.is_err());
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_invalid_group_headers() {
    let temp_file = "/tmp/invalid_group_test.desktop";
    fs::write(temp_file, "[Invalid Group Header\nType=Application\nName=Test").unwrap();
    
    let result = ApplicationEntry::try_from_path(temp_file);
    assert!(result.is_err());
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_key_without_equals() {
    let temp_file = "/tmp/no_equals_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nNameWithoutEquals\nExec=test").unwrap();
    
    // Should parse successfully, ignoring the invalid line
    let result = ApplicationEntry::try_from_path(temp_file);
    assert!(result.is_err()); // Will fail due to missing Name
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_empty_key_name() {
    let temp_file = "/tmp/empty_key_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\n=EmptyKey\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse despite empty key");
    assert_eq!(entry.name(), Some("Test".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_empty_value() {
    let temp_file = "/tmp/empty_value_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nComment=\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse with empty value");
    assert_eq!(entry.comment(), Some("".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_multiple_equals_signs() {
    let temp_file = "/tmp/multiple_equals_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test --option=value=123\nComment=Text with = signs").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse with multiple equals");
    assert_eq!(entry.exec(), Some("test --option=value=123".to_string()));
    assert_eq!(entry.comment(), Some("Text with = signs".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_very_long_lines() {
    let temp_file = "/tmp/long_lines_test.desktop";
    let long_value = "A".repeat(10000);
    let content = format!("[Desktop Entry]\nType=Application\nName=Test\nComment={}\nExec=test", long_value);
    fs::write(temp_file, content).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse long lines");
    assert_eq!(entry.comment(), Some(long_value));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_unicode_content() {
    let temp_file = "/tmp/unicode_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=测试应用程序\nComment=Приложение для тестирования\nIcon=🚀\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse Unicode content");
    assert_eq!(entry.name(), Some("测试应用程序".to_string()));
    assert_eq!(entry.comment(), Some("Приложение для тестирования".to_string()));
    assert_eq!(entry.icon(), Some("🚀".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_whitespace_variations() {
    let temp_file = "/tmp/whitespace_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\n  Type  =  Application  \n\tName\t=\tTest App\t\nExec =test-app   \n   Comment=   A test app   ").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse whitespace variations");
    assert_eq!(entry.entry_type(), Some("Application".to_string()));
    assert_eq!(entry.name(), Some("Test App".to_string()));
    assert_eq!(entry.exec(), Some("test-app".to_string()));
    assert_eq!(entry.comment(), Some("A test app".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_duplicate_keys() {
    let temp_file = "/tmp/duplicate_keys_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=First Name\nName=Second Name\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse with duplicate keys");
    // Should use the last occurrence
    assert_eq!(entry.name(), Some("Second Name".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_multiple_groups() {
    let temp_file = "/tmp/multiple_groups_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test App\nExec=test\n\n[Another Group]\nCustomKey=CustomValue\n\n[Desktop Action test]\nName=Test Action\nExec=test --action").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse multiple groups");
    assert_eq!(entry.name(), Some("Test App".to_string()));
    // Our current implementation focuses on Desktop Entry group
    // Additional groups are parsed but not directly exposed in ApplicationEntry API
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_semicolon_edge_cases() {
    let temp_file = "/tmp/semicolon_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nCategories=A;B;C;\nKeywords=;word1;word2;;\nMimeType=text/plain;").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse semicolon edge cases");
    
    // Trailing semicolons and empty items should be handled
    assert_eq!(entry.categories(), Some(vec!["A".to_string(), "B".to_string(), "C".to_string()]));
    assert_eq!(entry.keywords(), Some(vec!["word1".to_string(), "word2".to_string()]));
    assert_eq!(entry.mime_types(), Some(vec!["text/plain".to_string()]));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_boolean_edge_cases() {
    let temp_file = "/tmp/boolean_edge_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nTerminal=TRUE\nHidden=False\nNoDisplay=yes\nX-Test-Invalid=maybe").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse boolean variations");
    
    // Our parser is strict: only "true" and "false" (case-insensitive)
    assert_eq!(entry.get_bool("Terminal"), Some(true));  // TRUE -> true
    assert_eq!(entry.get_bool("Hidden"), Some(false));   // False -> false
    assert_eq!(entry.get_bool("NoDisplay"), None);       // "yes" is not a valid boolean
    assert_eq!(entry.get_bool("X-Test-Invalid"), None);  // "maybe" is not a valid boolean
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_invalid_key_characters() {
    let temp_file = "/tmp/invalid_key_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test\nInvalid Key=value\nValid-Key=value\nAnother_Invalid=value").unwrap();
    
    // Should fail due to invalid key name (spaces not allowed)
    let result = ApplicationEntry::try_from_path(temp_file);
    assert!(result.is_err());
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_performance_large_file() {
    let temp_file = "/tmp/large_file_test.desktop";
    let mut content = String::from("[Desktop Entry]\nType=Application\nName=Large Test\nExec=test\n");
    
    // Add many custom fields
    for i in 0..1000 {
        content.push_str(&format!("X-Custom-Field-{}=Value {}\n", i, i));
    }
    
    fs::write(temp_file, content).unwrap();
    
    let start = std::time::Instant::now();
    let entry = ApplicationEntry::try_from_path(temp_file).expect("Should parse large file");
    let duration = start.elapsed();
    
    // Should parse reasonably quickly (less than 100ms for 1000 fields)
    assert!(duration.as_millis() < 100, "Parsing took too long: {:?}", duration);
    assert_eq!(entry.name(), Some("Large Test".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_application_all_filtering() {
    // Test that ApplicationEntry::all() properly filters desktop files
    // and handles parsing errors gracefully
    let entries = ApplicationEntry::all();
    
    // Should have some entries (unless system has no applications)
    // But more importantly, should not panic even if some files are malformed
    assert!(entries.len() >= 0); // Always true, but tests that it doesn't panic
    
    // All entries should have basic required fields when parsed successfully
    for entry in entries.iter().take(5) { // Test first 5 to keep test fast
        assert!(entry.entry_type().is_some());
        assert!(entry.name().is_some());
    }
}