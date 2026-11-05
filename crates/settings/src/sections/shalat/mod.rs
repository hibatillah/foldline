mod adjust;
mod asr;
mod methods;
mod pictures;
mod reminders;

use crate::layout::{Group, Page};

// The keys `crates/notch/src/pages/shalat/config.rs` reads; a test compares them.
const METHOD_KEY: &str = "shalat.method";
const HANAFI_KEY: &str = "shalat.hanafi";
const MESSAGE_KEY: &str = "shalat.message";
const BADGE_KEY: &str = "shalat.badge";

pub fn page() -> Page {
    Page::new(
        "Shalat",
        "Prayer times use the location set in General. They show on the Shalat page and in the collapsed pill.",
        vec![
            Group::titled("Calculation method", vec![methods::item()]),
            Group::titled("Asr", vec![asr::item()]),
            Group::titled("Reminders", vec![reminders::item()]),
            Group::titled("Adjust the times", vec![adjust::item()]),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_keys_match_the_ones_the_notch_reads() {
        let config =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/shalat/config.rs"))
                .expect("notch config source");
        for key in [METHOD_KEY, HANAFI_KEY, MESSAGE_KEY, BADGE_KEY] {
            assert!(config.contains(&format!("\"{key}\"")), "{key} missing in config.rs");
        }
    }
}
