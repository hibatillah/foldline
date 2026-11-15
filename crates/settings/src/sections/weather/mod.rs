//! The Weather page's one choice: the rain alert (a banner, then a badge beside the clock). It needs the page
//! switched on in Pages; the place is the shared location in General.

use crate::i18n::tr;
use crate::{
    field,
    layout::{Group, Item, Page},
};

const RAIN_ALERT_KEY: &str = "weather.rain_alert";

pub fn page() -> Page {
    Page::new(
        tr!("weather.title"),
        tr!("weather.description"),
        vec![Group::new(vec![
            Item::new(tr!("weather.rain_alert"), field::toggle(RAIN_ALERT_KEY, true))
                .description(tr!("weather.rain_alert_note")),
        ])],
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
}
