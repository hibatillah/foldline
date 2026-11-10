mod adjust;
mod asr;
mod methods;
mod pictures;
mod reminders;

use crate::i18n::tr;
use crate::layout::{Group, Page};

// The keys `crates/notch/src/pages/shalat/config.rs` reads; a test compares them.
const METHOD_KEY: &str = "shalat.method";
const HANAFI_KEY: &str = "shalat.hanafi";
const MESSAGE_KEY: &str = "shalat.message";
const BADGE_KEY: &str = "shalat.badge";

pub fn page() -> Page {
    Page::new(
        tr!("shalat.title"),
        tr!("shalat.description"),
        vec![
            Group::titled(tr!("shalat.group_method"), vec![methods::item()]),
            Group::titled(tr!("shalat.group_asr"), vec![asr::item()]),
            Group::titled(tr!("shalat.group_reminders"), vec![reminders::item()]),
            Group::titled(tr!("shalat.group_adjust"), vec![adjust::item()]),
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
