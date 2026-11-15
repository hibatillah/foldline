//! The rain countdown kept in the pill's group after the banner: a cloud and the minutes left, like the prayer mark.

use crate::i18n::tr;
use salah::{DateTime, Utc};

use super::symbol::Symbol;
use crate::gfx::{Canvas, Fonts, Point, Rect};
use crate::ui::palette::ACCENT_BLUE_LIGHT;
use crate::ui::type_styles::MEDIUM_13;

const ICON_SIZE: f32 = 14.0;
const ICON_GAP: f32 = 4.0;

pub(super) struct Badge {
    pub(super) symbol: Symbol,
    pub(super) rain_at: DateTime<Utc>,
    pub(super) width: f32,
    pub(super) short_width: f32,
}

/// Whole minutes left, rounded up, so the badge reads "1 min" until the rain starts.
pub(super) fn minutes_until(rain_at: DateTime<Utc>, now: DateTime<Utc>) -> i64 {
    (rain_at - now).num_seconds().max(0).saturating_add(59) / 60
}

pub(super) fn long_text(minutes: i64) -> String {
    tr!("weather.badge_long", minutes = minutes).into_owned()
}

pub(super) fn short_text(minutes: i64) -> String {
    tr!("weather.badge_short", minutes = minutes).into_owned()
}

/// When the minutes shown change next: just after the next whole-minute boundary.
pub(super) fn next_tick_milliseconds(rain_at: DateTime<Utc>, now: DateTime<Utc>) -> u32 {
    let seconds = (rain_at - now).num_seconds().max(1);
    let to_boundary = if seconds % 60 == 0 { 60 } else { seconds % 60 };
    (to_boundary * 1000 + 500) as u32
}

impl Badge {
    pub(super) fn new(fonts: &Fonts, symbol: Symbol, rain_at: DateTime<Utc>, widest_minutes: i64) -> Self {
        let text_width = |text: &str| fonts.measure(&MEDIUM_13, text).ceil();
        Self {
            symbol,
            rain_at,
            width: ICON_SIZE + ICON_GAP + text_width(&long_text(widest_minutes)),
            short_width: ICON_SIZE + ICON_GAP + text_width(&short_text(widest_minutes)),
        }
    }

    /// The same badge measured again, after a language change made its text longer or shorter.
    pub(super) fn remeasured(&self, fonts: &Fonts) -> Self {
        Self::new(fonts, self.symbol, self.rain_at, super::RAIN_ALERT_LEAD_MINUTES)
    }

    pub(super) fn is_over(&self, now: DateTime<Utc>) -> bool {
        self.rain_at <= now
    }

    pub(super) fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32, now: DateTime<Utc>, is_short: bool) {
        let minutes = minutes_until(self.rain_at, now);
        let text = if is_short { short_text(minutes) } else { long_text(minutes) };
        let center = Point::new(area.left + ICON_SIZE / 2.0, area.center_y());
        self.symbol.icon().draw(canvas, center, ICON_SIZE, ACCENT_BLUE_LIGHT, opacity);
        let text_area = Rect::new(area.left + ICON_SIZE + ICON_GAP, area.top, area.right, area.bottom);
        canvas.text(&MEDIUM_13, &text, text_area, ACCENT_BLUE_LIGHT.faded(opacity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn minutes_round_up_and_never_go_below_zero() {
        let rain = at("2026-11-05T10:00:00Z");
        assert_eq!(minutes_until(rain, at("2026-11-05T09:30:00Z")), 30);
        assert_eq!(minutes_until(rain, at("2026-11-05T09:30:01Z")), 30);
        assert_eq!(minutes_until(rain, at("2026-11-05T09:59:59Z")), 1);
        assert_eq!(minutes_until(rain, at("2026-11-05T10:05:00Z")), 0);
    }

    #[test]
    fn the_texts_say_the_minutes() {
        assert_eq!(long_text(25), "Rain in 25 min");
        assert_eq!(short_text(25), "25m");
    }

    #[test]
    fn the_next_tick_comes_just_after_the_minute_changes() {
        let rain = at("2026-11-05T10:00:00Z");
        assert_eq!(next_tick_milliseconds(rain, at("2026-11-05T09:30:00Z")), 60_500);
        assert_eq!(next_tick_milliseconds(rain, at("2026-11-05T09:30:20Z")), 40_500);
        assert_eq!(next_tick_milliseconds(rain, at("2026-11-05T09:59:50Z")), 10_500);
    }

    #[test]
    fn the_badge_ends_when_the_rain_starts() {
        let fonts = Fonts::new(1.0).unwrap();
        let badge = Badge::new(&fonts, Symbol::Rain, at("2026-11-05T10:00:00Z"), 30);
        assert!(!badge.is_over(at("2026-11-05T09:59:59Z")));
        assert!(badge.is_over(at("2026-11-05T10:00:00Z")));
        assert!(badge.short_width < badge.width);
    }

    #[test]
    fn the_rain_badge_reads_hujan_25_mnt_lagi() {
        assert_eq!(long_text(25), "Rain in 25 min");
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(long_text(25), "Hujan 25 mnt lagi");
            assert_eq!(short_text(25), "25m");
        });
    }
}

#[cfg(test)]
mod symbol_tests {
    use super::*;
    use crate::ui::Icon;

    #[test]
    fn the_rain_badge_draws_the_coming_weather() {
        let fonts = Fonts::new(1.0).unwrap();
        let rain_at = DateTime::parse_from_rfc3339("2026-11-05T10:00:00Z").unwrap().with_timezone(&Utc);
        let snow = Badge::new(&fonts, Symbol::from_code("lightsnow"), rain_at, 30);
        assert!(snow.symbol.icon() == Icon::CloudSnow);
        assert!(snow.remeasured(&fonts).symbol.icon() == Icon::CloudSnow, "a language change keeps the weather");
        let thunder = Badge::new(&fonts, Symbol::from_code("rainandthunder"), rain_at, 30);
        assert!(thunder.symbol.icon() == Icon::CloudLightning);
    }
}
