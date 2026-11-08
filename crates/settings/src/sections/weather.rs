//! The Weather page's one choice: the message in the pill before rain. The page itself is switched on in Pages and the
//! place is the shared location in General.

use crate::{
    field,
    layout::{Group, Item, Page},
};

pub const RAIN_ALERT_KEY: &str = "weather.rain_alert";

pub fn page() -> Page {
    Page::new(
        "Weather",
        "The message the notch shows before it rains. The weather page itself is switched on in Pages.",
        vec![Group::new(vec![Item::new("Rain alert", field::toggle(RAIN_ALERT_KEY, true)).description(
            "A message in the pill 30 minutes before the next rain. The weather page itself is switched on in Pages.",
        )])],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_is_called_weather_and_describes_itself() {
        let page = page();
        assert_eq!(page.title, "Weather");
        assert!(page.description().contains("rain"));
    }

    #[test]
    fn the_rain_alert_key_is_the_one_the_notch_reads() {
        assert_eq!(RAIN_ALERT_KEY, "weather.rain_alert");
    }
}
