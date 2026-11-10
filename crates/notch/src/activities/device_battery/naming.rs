const MAX_NAME_CHARACTERS: usize = 22;
/// Windows adds this to the name of a headset's call node.
const HANDS_FREE_SUFFIX: &str = " Hands-Free";

/// The device's own name: its nodes differ only by suffixes such as "Hands-Free".
pub(super) fn device_name(node_name: &str) -> String {
    let name = node_name.trim();
    name.strip_suffix(HANDS_FREE_SUFFIX).unwrap_or(name).to_owned()
}

fn short_name(name: &str) -> String {
    if name.chars().count() <= MAX_NAME_CHARACTERS {
        return name.to_owned();
    }
    let cut: String = name.chars().take(MAX_NAME_CHARACTERS - 1).collect();
    format!("{}\u{2026}", cut.trim_end())
}

pub(super) fn message(name: &str) -> String {
    crate::i18n::tr!("device_battery.low", name = short_name(name)).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hands_free_node_has_the_name_of_its_device() {
        assert_eq!(device_name("DAXA SL1 Hands-Free"), "DAXA SL1");
        assert_eq!(device_name("  DAXA SL1 "), "DAXA SL1");
    }

    #[test]
    fn short_names_are_kept_and_long_ones_are_cut_with_an_ellipsis() {
        assert_eq!(short_name("DAXA SL1"), "DAXA SL1");
        let long = short_name("Wireless Headphones Pro Max Edition");
        assert_eq!(long.chars().count(), MAX_NAME_CHARACTERS);
        assert!(long.ends_with('\u{2026}'));
    }

    #[test]
    fn the_message_names_the_device_then_the_warning() {
        assert_eq!(message("MX Master 3"), "MX Master 3 \u{b7} Low battery");
    }

    #[test]
    fn the_low_battery_message_reads_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(message("MX Master 3"), "MX Master 3 \u{b7} Baterai lemah");
        });
    }
}
