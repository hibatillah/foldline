//! The weather page's two message states, what each says and how wide the page is for it.

use salah::Local;

use super::{WeatherPage, draw};
use crate::gfx::Fonts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MessageKind {
    /// No location is saved yet.
    NoLocation,
    /// The forecast could not be fetched.
    Offline,
}

pub(super) struct Message {
    pub kind: MessageKind,
    pub title: String,
    pub detail: String,
}

/// The content width of a message row: the badge, the gap and the wider of the two text lines.
pub(super) fn row_width(title_width: f32, detail_width: f32) -> f32 {
    draw::MESSAGE_BADGE + draw::MESSAGE_GAP + title_width.max(detail_width)
}

impl WeatherPage {
    /// The message the page shows instead of the forecast, if any.
    pub(super) fn message(&self) -> Option<Message> {
        if self.location.is_none() {
            return Some(Message {
                kind: MessageKind::NoLocation,
                title: "Choose a city".to_owned(),
                detail: "Pick a location in Settings to see the weather".to_owned(),
            });
        }
        if !self.is_offline {
            return None;
        }
        let title = if self.is_service_error { "Weather service did not answer" } else { "Weather is offline" };
        let detail = match &self.forecast {
            Some(forecast) => format!(
                "Last updated {} \u{b7} retrying on its own",
                forecast.fetched_at.with_timezone(&Local).format("%H:%M")
            ),
            None => "Retrying on its own".to_owned(),
        };
        Some(Message { kind: MessageKind::Offline, title: title.to_owned(), detail })
    }

    /// Measures the current message so the page can hug it; the forecast keeps the fixed width.
    pub(super) fn refresh_message_width(&mut self, fonts: &Fonts) {
        self.message_width = self.message().map(|message| {
            row_width(
                fonts.measure(&draw::MESSAGE_TITLE, &message.title),
                fonts.measure(&draw::MESSAGE_DETAIL, &message.detail),
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::Page;

    #[test]
    fn a_message_row_is_a_badge_a_gap_and_its_widest_line() {
        assert_eq!(row_width(100.0, 180.0), 36.0 + 16.0 + 180.0);
        assert_eq!(row_width(200.0, 120.0), 36.0 + 16.0 + 200.0);
    }

    #[test]
    fn no_saved_location_asks_for_a_city_and_the_forecast_has_no_message() {
        let message = WeatherPage::new().message().unwrap();
        assert_eq!(message.kind, MessageKind::NoLocation);
        assert_eq!(message.title, "Choose a city");
        assert_eq!(message.detail, "Pick a location in Settings to see the weather");
        assert!(WeatherPage::sample(false).message().is_none());
    }

    #[test]
    fn offline_keeps_its_wording() {
        let mut page = WeatherPage::sample(false);
        page.is_offline = true;
        let message = page.message().unwrap();
        assert_eq!((message.kind, message.title.as_str()), (MessageKind::Offline, "Weather is offline"));
        assert!(message.detail.starts_with("Last updated ") && message.detail.ends_with("retrying on its own"));
        page.is_service_error = true;
        assert_eq!(page.message().unwrap().title, "Weather service did not answer");
        page.forecast = None;
        assert_eq!(page.message().unwrap().detail, "Retrying on its own");
    }

    #[test]
    fn the_page_hugs_a_message_and_keeps_the_fixed_width_for_the_forecast() {
        let fonts = Fonts::new(1.0).unwrap();
        let mut page = WeatherPage::new();
        page.refresh_message_width(&fonts);
        let hugged = page.sizes().expanded.width;
        assert!(hugged < super::super::CONTENT_WIDTH && hugged > draw::MESSAGE_BADGE + draw::MESSAGE_GAP);
        let mut forecast = WeatherPage::sample(false);
        forecast.refresh_message_width(&fonts);
        assert_eq!(forecast.sizes().expanded.width, super::super::CONTENT_WIDTH);
    }
}
