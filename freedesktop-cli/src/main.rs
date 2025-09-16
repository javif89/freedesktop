use freedesktop_apps::ApplicationEntry;

fn main() {
    for app in ApplicationEntry::all() {
        if app.should_show() {
            println!("{}", app.path().display());
        }
    }
    let app =
        ApplicationEntry::from_path("/home/javi/.nix-profile/share/applications/obsidian.desktop");
    app.execute();
}
