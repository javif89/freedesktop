use std::env;

pub struct Info;

impl Info {
    pub fn current_desktop() -> Option<String> {
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            return Some(desktop);
        }

        None
    }
}
