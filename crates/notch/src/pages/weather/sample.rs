use super::{WeatherPage, alert::Alert, forecast::Forecast};
use crate::gfx::Fonts;
use places::Location;
use salah::{DateTime, Duration, Utc};

impl WeatherPage {
    #[cfg(test)]
    pub fn sample(shows_days: bool) -> Self {
        let now = DateTime::parse_from_rfc3339("2026-11-05T09:05:00Z").unwrap().with_timezone(&Utc);
        let skies = ["partlycloudy_day", "partlycloudy_day", "rain", "heavyrainandthunder", "rain", "cloudy"];
        let hours: Vec<String> = (0..24)
            .map(|offset| {
                let (code, rain) = if (2..5).contains(&offset) { (skies[offset % 6], 1.4) } else { (skies[offset % 2], 0.0) };
                let time = (now + Duration::hours(offset as i64 - 1)).format("%Y-%m-%dT%H:00:00Z").to_string();
                format!(
                    r#"{{"time":"{time}","data":{{"instant":{{"details":{{"air_temperature":{},"apparent_air_temperature":36.0}}}},"next_1_hours":{{"summary":{{"symbol_code":"{code}"}},"details":{{"precipitation_amount":{rain}}}}},"next_6_hours":{{"summary":{{"symbol_code":"{code}"}},"details":{{"precipitation_amount":{rain}}}}}}}}}"#,
                    31 - (offset as i32 - 2).abs()
                )
            })
            .collect();
        let body = format!(r#"{{"properties":{{"timeseries":[{}]}}}}"#, hours.join(","));
        let mut page = Self::new();
        page.location = Some(Location {
            label: "Pekanbaru, ID".to_owned(),
            country: "ID".to_owned(),
            latitude: 0.517,
            longitude: 101.442,
        });
        page.forecast = Forecast::parse(&body, now);
        page.shows_days = shows_days;
        page.fixed_now = Some(now);
        page
    }

    #[cfg(test)]
    pub fn sample_alert(fonts: &Fonts) -> Self {
        let mut page = Self::sample(false);
        page.alert = Alert::sample_banner(fonts, page.sample_rain_time());
        page
    }

    /// The banner is gone and the countdown is in the pill.
    #[cfg(test)]
    pub fn sample_rain_badge(fonts: &Fonts) -> Self {
        let mut page = Self::sample(false);
        page.alert = Alert::sample_badge(fonts, page.sample_rain_time());
        page
    }

    /// 25 minutes after the sample's clock.
    #[cfg(test)]
    fn sample_rain_time(&self) -> DateTime<Utc> {
        self.now() + Duration::minutes(25)
    }

    #[cfg(test)]
    pub fn sample_offline(fonts: &Fonts) -> Self {
        let mut page = Self::sample(false);
        page.is_offline = true;
        page.refresh_message_width(fonts);
        page
    }

    #[cfg(test)]
    pub fn sample_no_location(fonts: &Fonts) -> Self {
        let mut page = Self::new();
        page.refresh_message_width(fonts);
        page
    }
}
