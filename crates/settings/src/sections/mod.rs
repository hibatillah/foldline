mod advanced;
mod behavior;
mod calendar;
mod general;
mod pages;
mod shalat;

use store::Settings;

use crate::{layout::Page, notify::tell_notch_to_reload};

pub fn apply(settings: &mut Settings, key: &str, value: &str) {
    settings.set_string(key, value);
    tell_notch_to_reload();
}

/// Saves several keys, one atomic write each, and tells the notch once.
pub fn apply_all(settings: &mut Settings, values: &[(&str, String)]) {
    for (key, value) in values {
        settings.set_string(key, value);
    }
    tell_notch_to_reload();
}

pub fn all() -> Vec<Page> {
    vec![general::page(), pages::page(), behavior::page(), calendar::page(), shalat::page(), advanced::page()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applying_several_keys_writes_each_of_them() {
        let path = std::env::temp_dir().join(format!("notch-settings-apply-all-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        apply_all(&mut settings, &[("notch.first", "a".to_owned()), ("notch.second", "b".to_owned())]);
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.first=a") && text.contains("notch.second=b"), "{text}");
    }

    #[test]
    fn applying_a_key_writes_it_to_the_file() {
        let path = std::env::temp_dir().join(format!("notch-settings-apply-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        apply(&mut settings, "notch.example", "on");
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.example=on"), "{text}");
    }
}
