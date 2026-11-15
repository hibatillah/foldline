/// A page or activity family; for a resource with a base, the first action is the base.
pub struct Resource {
    pub name: &'static str,
    pub actions: &'static [&'static str],
    pub has_base: bool,
}

pub const CATALOG: &[Resource] = &[
    Resource { name: "media", actions: &["page", "lyrics"], has_base: true },
    Resource { name: "calendar", actions: &["page", "google"], has_base: true },
    Resource { name: "shelf", actions: &["page"], has_base: true },
    Resource { name: "timer", actions: &["page"], has_base: true },
    Resource { name: "todo", actions: &["page"], has_base: true },
    Resource { name: "shalat", actions: &["page"], has_base: true },
    Resource { name: "stats", actions: &["page"], has_base: true },
    Resource { name: "devices", actions: &["page"], has_base: true },
    Resource { name: "clipboard", actions: &["page"], has_base: true },
    Resource { name: "weather", actions: &["page", "rain_alert"], has_base: true },
    Resource {
        name: "activity",
        actions: &[
            "volume",
            "brightness",
            "battery",
            "caps_lock",
            "microphone",
            "camera",
            "bluetooth",
            "device_battery",
            "notifications",
            "clipboard",
            "downloads",
            "account_notice",
        ],
        has_base: false,
    },
];

pub fn is_known(resource: &str, action: &str) -> bool {
    CATALOG.iter().any(|entry| entry.name == resource && entry.actions.contains(&action))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_pairs_are_found_and_others_are_not() {
        assert!(is_known("media", "lyrics"));
        assert!(is_known("activity", "camera"));
        assert!(!is_known("media", "rain_alert"));
        assert!(!is_known("nothing", "page"));
    }

    #[test]
    fn only_the_activity_resource_has_no_base() {
        for entry in CATALOG {
            assert_eq!(entry.has_base, entry.name != "activity", "{}", entry.name);
        }
    }
}
