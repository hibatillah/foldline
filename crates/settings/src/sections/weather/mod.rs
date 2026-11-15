//! The Weather page's one choice: the rain alert (a banner, then a badge beside the clock). It needs the page
//! switched on in Pages; the place is the shared location in General.

mod pictures;

use super::lock;
use crate::i18n::tr;
use crate::{
    field,
    layout::{Group, Item, Page},
};

const RAIN_ALERT_KEY: &str = "weather.rain_alert";

pub fn page() -> Page {
    Page::plain(
        tr!("weather.title"),
        vec![
            Group::titled(
                tr!("weather.group_alerts"),
                vec![
                    Item::new(
                        tr!("weather.rain_alert"),
                        lock::locked("weather", "rain_alert", field::toggle(RAIN_ALERT_KEY, true)),
                    )
                    .description(tr!("weather.rain_alert_note")),
                    Item::picture_row(pictures::rain_alert),
                ],
            )
            .description(tr!("weather.group_alerts_note")),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rain_alert_key_is_the_one_the_notch_reads() {
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../notch/src/pages/weather/mod.rs"))
            .expect("notch weather source");
        assert!(source.contains(&format!("\"{RAIN_ALERT_KEY}\"")));
    }

    #[test]
    fn the_page_has_one_section_and_no_description_sentence() {
        let page = page();
        assert_eq!(page.group_titles(), [tr!("weather.group_alerts")]);
        assert_eq!(page.description(), None);
    }
}
