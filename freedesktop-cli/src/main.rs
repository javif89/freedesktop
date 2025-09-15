use freedesktop_apps::ApplicationEntry;

fn main() {
    for app in ApplicationEntry::all() {
        println!("{:#?}", app.name());
    }
    // let app = ApplicationEntry::from_path("./btop.desktop");
}
