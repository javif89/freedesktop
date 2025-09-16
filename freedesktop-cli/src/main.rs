use freedesktop_apps::ApplicationEntry;

fn main() {
    for app in ApplicationEntry::all() {
        if app.should_show() {
            println!("{app:#?}");
        }
    }
}
