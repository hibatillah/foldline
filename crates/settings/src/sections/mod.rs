mod about;
mod advanced;
mod behavior;
mod calendar;
mod general;
mod language;
mod pages;
mod pause;
mod shalat;
mod stats;
mod timer;
mod weather;

use store::Settings;

use crate::{
    icons,
    notify::tell_notch_to_reload,
    sidebar::{NavEntry, NavGroup},
};

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

/// Every section with its icon and sidebar group; `sidebar::grouped` sets the order they are shown in.
pub fn all() -> Vec<NavEntry> {
    vec![
        NavEntry::new(NavGroup::Notch, icons::GENERAL, general::page()),
        NavEntry::new(NavGroup::Notch, icons::PAGES, pages::page()),
        NavEntry::new(NavGroup::Notch, icons::BEHAVIOR, behavior::page()),
        NavEntry::new(NavGroup::Notch, icons::ADVANCED, advanced::page()),
        NavEntry::new(NavGroup::PageSettings, icons::CALENDAR, calendar::page()),
        NavEntry::new(NavGroup::PageSettings, icons::TIMER, timer::page()),
        NavEntry::new(NavGroup::PageSettings, icons::SHALAT, shalat::page()),
        NavEntry::new(NavGroup::PageSettings, icons::STATS, stats::page()),
        NavEntry::new(NavGroup::PageSettings, icons::WEATHER, weather::page()),
        NavEntry::new(NavGroup::Footer, icons::ABOUT, about::page()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_has_its_own_icon_and_a_description_is_a_whole_sentence() {
        let entries = all();
        let icons = entries.iter().map(|entry| entry.icon).collect::<Vec<_>>();
        assert_eq!(
            icons,
            [
                icons::GENERAL,
                icons::PAGES,
                icons::BEHAVIOR,
                icons::ADVANCED,
                icons::CALENDAR,
                icons::TIMER,
                icons::SHALAT,
                icons::STATS,
                icons::WEATHER,
                icons::ABOUT
            ]
        );
        let mut unique = icons.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), icons.len());
        for entry in &entries {
            if let Some(description) = entry.page.description() {
                assert!(description.trim().len() > 20, "{} needs a whole description sentence", entry.page.title);
            }
        }
    }

    #[test]
    fn rebuilding_the_pages_in_indonesian_changes_their_titles() {
        use store::language::{Language, with_override};
        let titles = |entries: Vec<NavEntry>| entries.iter().map(|entry| entry.page.title).collect::<Vec<_>>();
        let english = titles(all());
        let indonesian = with_override(Language::Indonesian, || titles(all()));
        assert_eq!(english[0], "General");
        assert_eq!(indonesian[0], "Umum");
        assert_eq!(indonesian[4], "Kalender");
        assert_eq!(indonesian[9], "Tentang");
    }

    #[test]
    fn about_is_the_last_sidebar_item_and_sits_in_the_footer() {
        let entries = all();
        let last = entries.last().expect("entries");
        assert_eq!(last.page.title, "About");
        assert_eq!(last.icon, icons::ABOUT);
        assert_eq!(last.group, NavGroup::Footer);
    }

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
