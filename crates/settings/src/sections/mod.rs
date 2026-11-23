mod about;
mod advanced;
mod behavior;
mod calendar;
mod countdown;
mod general;
mod language;
mod pages;
mod pause;
mod shalat;
mod snooze;
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
        NavEntry::new(NavGroup::Notch, icons::BEHAVIOR, behavior::page()),
        NavEntry::new(NavGroup::Notch, icons::PAGES, pages::page()),
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
    fn every_section_has_its_own_icon() {
        let mut icons = all().iter().map(|entry| entry.icon).collect::<Vec<_>>();
        let count = icons.len();
        icons.sort_unstable();
        icons.dedup();
        assert_eq!(icons.len(), count);
    }

    #[test]
    fn every_section_title_is_unique_and_a_description_is_a_whole_sentence() {
        let entries = all();
        let mut titles = entries.iter().map(|entry| entry.page.title).collect::<Vec<_>>();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), entries.len());
        for entry in &entries {
            if let Some(description) = entry.page.description() {
                assert!(description.trim().len() > 20, "{} needs a whole description sentence", entry.page.title);
            }
        }
    }

    #[test]
    fn the_sections_the_notch_ships_are_all_listed() {
        let entries = all();
        for icon in [
            icons::GENERAL,
            icons::PAGES,
            icons::BEHAVIOR,
            icons::ADVANCED,
            icons::CALENDAR,
            icons::TIMER,
            icons::SHALAT,
            icons::STATS,
            icons::WEATHER,
            icons::ABOUT,
        ] {
            assert!(entries.iter().any(|entry| entry.icon == icon), "no section has the icon {icon}");
        }
    }

    #[test]
    fn rebuilding_the_pages_in_indonesian_changes_their_titles() {
        use store::language::{Language, with_override};
        let title_of = |entries: &[NavEntry], icon: &str| {
            entries.iter().find(|entry| entry.icon == icon).map(|entry| entry.page.title).expect("section")
        };
        let english = all();
        let indonesian = with_override(Language::Indonesian, || {
            all().iter().map(|entry| (entry.icon, entry.page.title)).collect::<Vec<_>>()
        });
        let indonesian_title = |icon: &str| indonesian.iter().find(|(own, _)| *own == icon).map(|(_, title)| *title);
        assert_eq!(title_of(&english, icons::GENERAL), "General");
        assert_eq!(indonesian_title(icons::GENERAL), Some("Umum"));
        assert_eq!(indonesian_title(icons::CALENDAR), Some("Kalender"));
        assert_eq!(indonesian_title(icons::ABOUT), Some("Tentang"));
    }

    #[test]
    fn about_sits_in_the_footer_where_the_sidebar_pins_it_last() {
        let entries = all();
        let about = entries.iter().find(|entry| entry.icon == icons::ABOUT).expect("about entry");
        assert_eq!(about.page.title, "About");
        assert_eq!(about.group, NavGroup::Footer);
    }

    #[test]
    fn applying_several_keys_writes_each_of_them() {
        let path = std::env::temp_dir().join(format!("foldline-settings-apply-all-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        apply_all(&mut settings, &[("notch.first", "a".to_owned()), ("notch.second", "b".to_owned())]);
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.first=a") && text.contains("notch.second=b"), "{text}");
    }

    #[test]
    fn applying_a_key_writes_it_to_the_file() {
        let path = std::env::temp_dir().join(format!("foldline-settings-apply-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut settings = Settings::at(Some(path.clone()));
        apply(&mut settings, "notch.example", "on");
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("notch.example=on"), "{text}");
    }
}
