use freedesktop_apps::ApplicationEntry;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_basic_localization() {
    let path = fixture_path("complete_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse complete app");

    // Default (non-localized) values
    assert_eq!(entry.name(), Some("Complete Test Application".to_string()));
    assert_eq!(entry.generic_name(), Some("Test App".to_string()));
    assert_eq!(entry.comment(), Some("A comprehensive test application demonstrating all features".to_string()));

    // English US localization
    assert_eq!(
        entry.get_localized_string("Name", Some("en_US")),
        Some("Complete Test Application (US)".to_string())
    );
    
    // Spanish localization
    assert_eq!(
        entry.get_localized_string("Name", Some("es")),
        Some("Aplicación de Prueba Completa".to_string())
    );
    assert_eq!(
        entry.get_localized_string("GenericName", Some("es")),
        Some("App de Prueba".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Comment", Some("es")),
        Some("Una aplicación de prueba que demuestra todas las características".to_string())
    );
    
    // French localization
    assert_eq!(
        entry.get_localized_string("Name", Some("fr")),
        Some("Application de Test Complète".to_string())
    );
    
    // Non-existent localization should fall back to default
    assert_eq!(
        entry.get_localized_string("Name", Some("de")),
        Some("Complete Test Application".to_string())
    );
    
    // List field localization
    assert_eq!(
        entry.get_vec("Keywords"),
        Some(vec!["test".to_string(), "demo".to_string(), "example".to_string(), "complete".to_string()])
    );
}

#[test]
fn test_complex_localization_fallback() {
    let path = fixture_path("complex_localization.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse localization test");

    // Test exact matches
    assert_eq!(
        entry.get_localized_string("Name", Some("en_US")),
        Some("American English Name".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("en_GB")),
        Some("British English Name".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("es_ES")),
        Some("Nombre en España".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("es_MX")),
        Some("Nombre en México".to_string())
    );

    // Test language fallback (when country-specific doesn't exist)
    assert_eq!(
        entry.get_localized_string("Name", Some("en_CA")),  // No en_CA, should fallback to en
        Some("English Name".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("es_AR")),  // No es_AR, should fallback to es
        Some("Nombre en Español".to_string())
    );

    // Test complete fallback to default when language doesn't exist
    assert_eq!(
        entry.get_localized_string("Name", Some("ja_JP")),  // No Japanese, fallback to default
        Some("Localization Test".to_string())
    );

    // Test fields that don't have localization
    assert_eq!(
        entry.get_localized_string("Comment", Some("zh_CN")),  // No Chinese comment, fallback to default
        Some("Base comment".to_string())
    );

    // Test existing Chinese localization
    assert_eq!(
        entry.get_localized_string("Name", Some("zh_CN")),
        Some("中文名称".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("zh_TW")),
        Some("中文名稱".to_string())
    );
}

#[test]
fn test_localization_priority_order() {
    let path = fixture_path("complex_localization.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse localization test");

    // When we have fr_CA defined, requesting fr_CA should return it directly
    assert_eq!(
        entry.get_localized_string("Name", Some("fr_CA")),
        Some("Nom Canada".to_string())  // Should get exact match "fr_CA"
    );

    // When we have both de and de_DE, requesting de_AT should try:
    // 1. de_AT (doesn't exist)  
    // 2. de (exists) - should return this
    assert_eq!(
        entry.get_localized_string("Name", Some("de_AT")),
        Some("Deutscher Name".to_string())  // Should get "de" not "de_DE"
    );
}

#[test]
fn test_no_localization_fallback() {
    let path = fixture_path("minimal_app.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse minimal app");

    // No localized versions exist, should always return default
    assert_eq!(
        entry.get_localized_string("Name", Some("es")),
        Some("Minimal App".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("fr")),
        Some("Minimal App".to_string())
    );
    assert_eq!(
        entry.get_localized_string("Name", Some("de_DE")),
        Some("Minimal App".to_string())
    );
}

#[test]
fn test_localized_keywords() {
    let path = fixture_path("complex_localization.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse localization test");

    // For now, our implementation returns the default keywords
    // In a more advanced implementation, we might want to support localized lists
    let keywords = entry.keywords().expect("Keywords should exist");
    assert_eq!(keywords, vec!["test".to_string(), "localization".to_string()]);
    
    // Test that localized keywords exist in the raw data
    // Note: Our current implementation doesn't expose localized list access
    // but we can verify the base implementation works
    assert!(keywords.contains(&"test".to_string()));
    assert!(keywords.contains(&"localization".to_string()));
}

#[test]
fn test_locale_with_encoding() {
    let path = fixture_path("complex_localization.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse localization test");

    // Test that locale with encoding strips the encoding part
    // en_US.UTF-8 should match en_US
    assert_eq!(
        entry.get_localized_string("Name", Some("en_US.UTF-8")),
        Some("American English Name".to_string())
    );
    
    // es_ES.ISO-8859-1 should match es_ES  
    assert_eq!(
        entry.get_localized_string("Name", Some("es_ES.ISO-8859-1")),
        Some("Nombre en España".to_string())
    );
}

#[test]
fn test_locale_with_modifier() {
    let path = fixture_path("complex_localization.desktop");
    let entry = ApplicationEntry::try_from_path(&path).expect("Failed to parse localization test");

    // Test locale with modifier fallback
    // de_DE@euro should fallback to de_DE, then de
    assert_eq!(
        entry.get_localized_string("Name", Some("de_DE@euro")),
        Some("Deutscher Name Deutschland".to_string())
    );
    
    // fr_CA@euro should fallback to fr_CA (exists), not fr  
    assert_eq!(
        entry.get_localized_string("Name", Some("fr_CA@euro")),
        Some("Nom Canada".to_string())
    );
}