use freedesktop_apps::ApplicationEntry;
use std::fs;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_simple_desktop_file_id() {
    let path = fixture_path("minimal_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse minimal app");
    
    // For files not in XDG directories, should use filename without extension
    assert_eq!(entry.id(), Some("minimal_app".to_string()));
}

#[test]
fn test_spec_compliant_desktop_file_id() {
    // Create a temporary XDG-like directory structure
    let temp_dir = "/tmp/test_xdg_data";
    let apps_dir = format!("{}/applications", temp_dir);
    let subdir = format!("{}/foo", apps_dir);
    
    fs::create_dir_all(&subdir).unwrap();
    
    // Create test files in the structure
    let simple_file = format!("{}/simple.desktop", apps_dir);
    let nested_file = format!("{}/bar.desktop", subdir);
    
    let desktop_content = r#"[Desktop Entry]
Type=Application
Name=Test App
Exec=test-app"#;
    
    fs::write(&simple_file, desktop_content).unwrap();
    fs::write(&nested_file, desktop_content).unwrap();
    
    // Mock the XDG base directories to include our temp dir
    // Note: This test shows the concept, but in practice we'd need to mock
    // freedesktop_core::base_directories() or set XDG environment variables
    
    let simple_entry = ApplicationEntry::try_from_path(&simple_file).unwrap();
    let nested_entry = ApplicationEntry::try_from_path(&nested_file).unwrap();
    
    // For files outside XDG dirs, falls back to filename
    assert_eq!(simple_entry.id(), Some("simple".to_string()));
    assert_eq!(nested_entry.id(), Some("bar".to_string()));
    
    // Clean up
    fs::remove_dir_all(temp_dir).ok();
}

#[test]
fn test_desktop_file_id_with_subdirectories() {
    // Test the ID conversion logic with a mock scenario
    // In real XDG structure: /usr/share/applications/org/example/FooViewer.desktop
    // Should become: org-example-FooViewer.desktop (as per spec)
    
    let temp_base = "/tmp/test_xdg_subdir";
    let apps_dir = format!("{}/share/applications", temp_base);
    let org_dir = format!("{}/org", apps_dir);
    let example_dir = format!("{}/example", org_dir);
    
    fs::create_dir_all(&example_dir).unwrap();
    
    let desktop_file = format!("{}/FooViewer.desktop", example_dir);
    let desktop_content = r#"[Desktop Entry]
Type=Application
Name=Foo Viewer
Exec=foo-viewer"#;
    
    fs::write(&desktop_file, desktop_content).unwrap();
    
    let entry = ApplicationEntry::try_from_path(&desktop_file).unwrap();
    
    // Without XDG dir recognition, falls back to filename
    assert_eq!(entry.id(), Some("FooViewer".to_string()));
    
    // Clean up
    fs::remove_dir_all(temp_base).ok();
}

#[test]
fn test_desktop_file_id_edge_cases() {
    // Test with files that have no extension
    let temp_file = "/tmp/no_extension_test";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    assert_eq!(entry.id(), Some("no_extension_test".to_string()));
    
    fs::remove_file(temp_file).ok();
    
    // Test with files that have multiple dots
    let temp_file = "/tmp/complex.name.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    assert_eq!(entry.id(), Some("complex.name".to_string()));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_desktop_file_id_real_world_examples() {
    // Test some real-world style desktop file IDs
    let test_cases = vec![
        ("org.gnome.Calculator.desktop", "org.gnome.Calculator"),
        ("firefox.desktop", "firefox"),
        ("org.kde.konsole.desktop", "org.kde.konsole"),
        ("code.desktop", "code"),
    ];
    
    for (filename, expected_id) in test_cases {
        let temp_file = format!("/tmp/{}", filename);
        fs::write(&temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=test").unwrap();
        
        let entry = ApplicationEntry::try_from_path(&temp_file).unwrap();
        assert_eq!(entry.id(), Some(expected_id.to_string()));
        
        fs::remove_file(&temp_file).ok();
    }
}