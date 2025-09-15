use freedesktop_apps::ApplicationEntry;

fn main() {
    for app in ApplicationEntry::all() {
        if let Some(name) = app.name()
            && app.should_show()
        {
            println!("{}: {}", app.id().unwrap(), name);
        }
    }
}
