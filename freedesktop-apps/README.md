# freedesktop-apps

Parse and execute desktop applications according to the freedesktop Desktop Entry Specification.

## Features

- **Desktop Entry parsing** - Robust parsing of `.desktop` files
- **Application execution** - Safe launching with field code expansion
- **Localization support** - Proper locale fallback for names and descriptions
- **Terminal applications** - Automatic terminal detection and wrapping
- **Spec-compliant** - Follows [Desktop Entry Specification v1.5](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html)

## Usage

### Basic Application Discovery

```rust
use freedesktop_apps::ApplicationEntry;

// List all installed applications
for app in ApplicationEntry::all() {
    if app.should_show() {
        println!("{}: {}", app.id().unwrap(), app.name().unwrap());
    }
}
```

### Application Information

```rust
let app = ApplicationEntry::try_from_path("/usr/share/applications/firefox.desktop")?;

println!("Name: {}", app.name().unwrap());
println!("Description: {}", app.comment().unwrap_or_default());
println!("Categories: {:?}", app.categories());
println!("Terminal app: {}", app.terminal());
```

### Application Execution

```rust
// Execute with no arguments
app.execute()?;

// Execute with files
app.execute_with_files(&["/path/to/file.txt"])?;

// Execute with URLs  
app.execute_with_urls(&["https://example.com"])?;
```

### Field Code Support

Supports all standard field codes:
- `%f` - Single file
- `%F` - Multiple files  
- `%u` - Single URL
- `%U` - Multiple URLs
- `%i` - Icon (`--icon iconname`)
- `%c` - Translated name
- `%k` - Desktop file location

### Localization

```rust
// Get localized strings with fallback
let name = app.get_localized_string("Name", Some("es_ES"));
// Falls back: es_ES → es → default
```

## Safety

- **Shell escaping** - All arguments are properly escaped
- **Input validation** - Malformed desktop files handled gracefully  
- **Process isolation** - Applications launched in detached processes
- **Error handling** - Comprehensive error types for all failure modes