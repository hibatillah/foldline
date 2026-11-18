use super::{draw::COLUMNS, forecast::rain_line, *};
use crate::gfx::Fonts;

#[test]
fn the_weather_is_a_page_to_add_with_no_collapsed_state() {
    let page = WeatherPage::new();
    assert!(page.is_opt_in());
    assert_eq!(page.priority(Mode::Resting), None);
    assert!(page.priority(Mode::Expanded).is_some());
    assert_eq!(page.presence(), None);
    assert_eq!(page.header_status_width(), None);
}

#[test]
fn the_rain_countdown_is_a_badge_with_a_header_mark_not_a_page_of_its_own() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = WeatherPage::sample_rain_badge(&fonts, "rain");
    assert!(matches!(page.presence(), Some(Presence::Badge { .. })));
    assert_eq!(page.header_status_width(), Some(Icon::CloudRain.size()));
    assert_eq!(page.countdown_seconds(), Some(25 * 60));
    assert!(page.short_presence_width().is_some());
    assert_eq!(page.header_status_page(), Some("weather"));
}

#[test]
fn the_header_mark_is_the_icon_of_the_coming_weather_whatever_the_state() {
    let fonts = Fonts::new(1.0).unwrap();
    let cases = [
        ("rain", Icon::CloudRain),
        ("heavyrain", Icon::CloudRainWind),
        ("rainshowers_day", Icon::CloudSunRain),
        ("rainshowers_night", Icon::CloudMoonRain),
        ("heavyrainandthunder", Icon::CloudLightning),
        ("snow", Icon::CloudSnow),
    ];
    for (code, icon) in cases {
        let page = WeatherPage::sample_rain_badge(&fonts, code);
        let now = page.now();
        // The open header and the collapsed row with lyrics both draw this icon through `draw_header_status`.
        assert!(page.alert.badge_icon(now) == Some(icon), "{code}");
        assert_eq!(page.header_status_width(), Some(icon.size()), "{code}");
    }
}

#[test]
fn the_banner_takes_the_pill_for_a_moment_before_the_badge() {
    let fonts = Fonts::new(1.0).unwrap();
    let page = WeatherPage::sample_alert(&fonts, "heavyrainandthunder");
    assert!(matches!(page.presence(), Some(Presence::Banner { width, .. }) if width == 340.0));
}

#[test]
fn the_sample_has_a_current_hour_days_and_rain_ahead() {
    let page = WeatherPage::sample(false);
    let forecast = page.forecast.as_ref().unwrap();
    assert!(forecast.hours.len() >= COLUMNS);
    assert!(!forecast.days.is_empty());
    assert!(forecast.upcoming_rain(page.now(), RAIN_WINDOW_HOURS).is_some());
    assert!(rain_line(forecast, page.now()).starts_with("Rain from "));
}
