//! The web forecast a click opens: weather.com, which resolves the place from the coordinates alone.

const BASE: &str = "https://weather.com/weather";

/// weather.com has no address for one hour or one day, so each click opens the closest view of the place for the cell.
/// Only numbers go into the address, so nothing needs escaping and a place name never reaches it.
fn url(view: &str, latitude: f64, longitude: f64) -> String {
    format!("{BASE}/{view}/l/{},{}", coordinate(latitude), coordinate(longitude))
}

/// The hour by hour view: an hour of the Today view opens it.
pub(super) fn hourly_url(latitude: f64, longitude: f64) -> String {
    url("hourbyhour", latitude, longitude)
}

/// The ten day view: a day of the Week view opens it.
pub(super) fn tenday_url(latitude: f64, longitude: f64) -> String {
    url("tenday", latitude, longitude)
}

/// Four decimals is about 11 m, with no trailing zeros.
fn coordinate(degrees: f64) -> String {
    let text = format!("{degrees:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" || trimmed.is_empty() { "0".to_owned() } else { trimmed.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_hour_opens_the_hour_by_hour_view() {
        assert_eq!(hourly_url(0.517, 101.442), "https://weather.com/weather/hourbyhour/l/0.517,101.442");
    }

    #[test]
    fn a_day_opens_the_ten_day_view() {
        assert_eq!(tenday_url(59.9139, 10.7522), "https://weather.com/weather/tenday/l/59.9139,10.7522");
    }

    #[test]
    fn coordinates_keep_four_decimals_and_a_sign() {
        assert_eq!(coordinate(-6.2088123), "-6.2088");
        assert_eq!(coordinate(106.8), "106.8");
        assert_eq!(coordinate(10.0), "10");
        assert_eq!(coordinate(-0.00001), "0");
    }
}
