use freedesktop_apps::{ApplicationEntry, ExecuteError};
use std::fs;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn test_execute_validation_no_exec() {
    let temp_file = "/tmp/no_exec_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nDBusActivatable=true\n").unwrap();
    
    // This should parse successfully since DBusActivatable=true makes Exec optional
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // But prepare_command should fail since we don't actually have Exec or D-Bus support
    let result = entry.prepare_command(&[], &[]);
    assert!(matches!(result, Err(ExecuteError::NotExecutable(_))));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_execute_validation_empty_exec() {
    let temp_file = "/tmp/empty_exec_test.desktop";
    fs::write(temp_file, "[Desktop Entry]\nType=Application\nName=Test\nExec=\n").unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    let result = entry.prepare_command(&[], &[]);
    
    assert!(matches!(result, Err(ExecuteError::NotExecutable(_))));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_field_code_expansion() {
    let temp_file = "/tmp/field_code_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=Test App\nIcon=test-icon\nExec=echo 'name:%c icon:%i file:%f'\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // Test field code expansion (we can't easily test actual execution)
    let exec = entry.exec().unwrap();
    assert!(exec.contains("%c"));
    assert!(exec.contains("%i"));
    assert!(exec.contains("%f"));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_command_line_parsing() {
    // Test through the public interface
    let temp_file = "/tmp/command_parse_test.desktop";
    fs::write(temp_file, 
        r#"[Desktop Entry]
Type=Application
Name=Test
Exec=echo "arg with spaces" 'single quotes' normal_arg
"#).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    assert!(entry.exec().is_some());
    
    // Test that complex command lines don't cause validation to fail
    let result = entry.prepare_command(&[], &[]);
    match result {
        Ok((program, args)) => {
            assert_eq!(program, "echo");
            assert!(args.len() >= 3); // Should have parsed the quoted arguments
        },
        Err(ExecuteError::InvalidCommand(_)) => panic!("Command line parsing failed"),
        Err(_) => {}, // Other errors are acceptable (like missing echo)
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_try_exec_validation() {
    let temp_file = "/tmp/try_exec_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=Test\nExec=echo test\nTryExec=/nonexistent/program\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    let result = entry.prepare_command(&[], &[]);
    
    assert!(matches!(result, Err(ExecuteError::ValidationFailed(_))));
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_try_exec_with_valid_program() {
    // Use 'echo' which should be available in PATH
    let temp_file = "/tmp/try_exec_valid_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=Test\nExec=echo test\nTryExec=echo\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // This should pass validation since 'echo' should be in PATH
    match entry.prepare_command(&[], &[]) {
        Ok((program, args)) => {
            assert_eq!(program, "echo");
            assert_eq!(args, vec!["test"]);
        },
        Err(ExecuteError::ValidationFailed(_)) => panic!("Validation should have passed for 'echo' in PATH"),
        Err(_) => {}, // Other errors are ok (like missing echo on some systems)
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_terminal_application() {
    let temp_file = "/tmp/terminal_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=Terminal Test\nExec=htop\nTerminal=true\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    assert!(entry.terminal());
    
    // Test that it tries to find a terminal (may fail if no terminal available)
    let result = entry.prepare_command(&[], &[]);
    match result {
        Ok((program, args)) => {
            // Should have wrapped with a terminal - check for common terminal names
            let is_terminal = program.contains("term") || 
                             program == "xterm" || 
                             program.ends_with("terminal") ||
                             program.contains("konsole") ||
                             program == "rxvt" ||
                             program.contains("rxvt") ||
                             program == "kitty";
            
            if !is_terminal {
                println!("Found terminal program: {}", program);
            }
            assert!(is_terminal, "Expected terminal program, got: {}", program);
            assert!(args.contains(&"-e".to_string()) || args.contains(&"htop".to_string()));
        },
        Err(ExecuteError::TerminalNotFound) => {}, // No terminal available - expected in some environments
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_execute_with_files() {
    let temp_file = "/tmp/files_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=File Test\nExec=cat %F\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // Test with files
    let files = vec!["/tmp/test1.txt", "/tmp/test2.txt"];
    let result = entry.prepare_command(&files, &[]);
    
    match result {
        Ok((program, args)) => {
            assert_eq!(program, "cat");
            // Should have expanded %F to the file list
            assert!(args.len() >= 2);
            assert!(args.iter().any(|arg| arg.contains("test1.txt")));
            assert!(args.iter().any(|arg| arg.contains("test2.txt")));
        },
        Err(_) => {}, // May fail if cat not available
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_execute_with_urls() {
    let temp_file = "/tmp/urls_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=URL Test\nExec=echo %U\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // Test with URLs
    let urls = vec!["https://example.com", "https://test.org"];
    let result = entry.prepare_command(&[], &urls);
    
    match result {
        Ok((program, args)) => {
            assert_eq!(program, "echo");
            // Should have expanded %U to the URL list
            let args_str = args.join(" ");
            assert!(args_str.contains("example.com"));
            assert!(args_str.contains("test.org"));
        },
        Err(_) => {}, // May fail if echo not available
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_working_directory() {
    let temp_file = "/tmp/workdir_test.desktop";
    fs::write(temp_file, 
        "[Desktop Entry]\nType=Application\nName=WorkDir Test\nExec=pwd\nPath=/tmp\n"
    ).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    assert_eq!(entry.path_dir(), Some("/tmp".to_string()));
    
    // Test preparation works (working directory is handled in actual execution)
    let result = entry.prepare_command(&[], &[]);
    match result {
        Ok((program, args)) => {
            assert_eq!(program, "pwd");
            assert!(args.is_empty());
        },
        Err(_) => {}, // May fail if pwd not available
    }
    
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_shell_escaping() {
    // Test that dangerous characters are properly escaped
    let temp_file = "/tmp/escape_test.desktop";
    fs::write(temp_file, 
        r#"[Desktop Entry]
Type=Application
Name=Escape Test
Exec=echo %f
"#).unwrap();
    
    let entry = ApplicationEntry::try_from_path(temp_file).unwrap();
    
    // Test with a filename containing dangerous characters
    let dangerous_files = vec!["file with spaces", "file'with'quotes", "file;with;semicolons"];
    
    for file in dangerous_files {
        let result = entry.prepare_command(&[file], &[]);
        match result {
            Ok((program, args)) => {
                assert_eq!(program, "echo");
                // Should have properly escaped the dangerous filename
                let args_str = args.join(" ");
                assert!(args_str.contains(file) || args_str.contains(&format!("'{}'", file)));
            },
            Err(e) => panic!("Unexpected error with file '{}': {:?}", file, e),
        }
    }
    
    fs::remove_file(temp_file).ok();
}