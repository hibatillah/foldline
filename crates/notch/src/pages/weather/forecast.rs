//! The forecast from MET Norway's Locationforecast (`complete`): the next hours, the next days and when rain comes.

use crate::i18n::tr;
use std::collections::BTreeMap;

use salah::{DateTime, Local, NaiveDate, Timelike, Utc};
use windows::{
    Data::Json::{JsonArray, JsonObject},
    core::h,
};

use super::symbol::Symbol;

/// An hour with at least this much water counts as rain.
const RAIN_MILLIMETERS: f32 = 0.2;
const HOURS_SHOWN: usize = 24;
const DAYS_SHOWN: usize = 6;
/// The local hour whose sky stands for the whole day.
const MIDDAY_HOUR: i64 = 13;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Hour {
    pub(super) time: DateTime<Utc>,
    pub(super) temperature: f32,
    pub(super) symbol: Symbol,
    pub(super) rain_millimeters: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Day {
    pub(super) date: NaiveDate,
    pub(super) high: f32,
    pub(super) low: f32,
    pub(super) symbol: Symbol,
    pub(super) rain_millimeters: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Forecast {
    pub(super) fetched_at: DateTime<Utc>,
    pub(super) feels_like: f32,
    /// From the current hour on, one entry per hour.
    pub(super) hours: Vec<Hour>,
    pub(super) days: Vec<Day>,
}

/// One time step of the response: the air at that moment and the sky over the next hour or the next six.
#[derive(Clone, Debug)]
struct Entry {
    time: DateTime<Utc>,
    temperature: f32,
    apparent: Option<f32>,
    hour: Option<(Symbol, f32)>,
    six_hours: Option<(Symbol, f32)>,
}

impl Forecast {
    /// Reads the response body; `None` when it is not a forecast.
    pub(super) fn parse(text: &str, fetched_at: DateTime<Utc>) -> Option<Self> {
        let series = JsonObject::Parse(&text.into())
            .ok()?
            .GetNamedObject(h!("properties"))
            .ok()?
            .GetNamedArray(h!("timeseries"))
            .ok()?;
        let entries = entries_of(&series, fetched_at - chrono_hour());
        let first = entries.first()?;
        let hours = entries
            .iter()
            .filter_map(|entry| {
                let (symbol, rain_millimeters) = entry.hour?;
                Some(Hour { time: entry.time, temperature: entry.temperature, symbol, rain_millimeters })
            })
            .take(HOURS_SHOWN)
            .collect();
        Some(Self {
            fetched_at,
            feels_like: first.apparent.unwrap_or(first.temperature),
            hours,
            days: days_of(&entries),
        })
    }

    /// The index of the hour shown first, the one whose hour is still running at `now`.
    pub(super) fn first_hour_index(&self, now: DateTime<Utc>) -> usize {
        self.hours.iter().position(|hour| hour.time + salah::Duration::hours(1) > now).unwrap_or(0)
    }

    /// The hour whose weather is happening at `now`.
    pub(super) fn current(&self, now: DateTime<Utc>) -> Option<&Hour> {
        self.hours.iter().rev().find(|hour| hour.time <= now).or_else(|| self.hours.first())
    }

    /// The first hour from `now` on with rain, within the next `within_hours` hours.
    pub(super) fn first_rain(&self, now: DateTime<Utc>, within_hours: i64) -> Option<&Hour> {
        let end = now + salah::Duration::hours(within_hours);
        self.hours.iter().find(|hour| hour.time + salah::Duration::hours(1) > now && hour.time < end && is_rain(hour))
    }
}

impl Forecast {
    /// The first rainy hour that starts after `now`, within the next `within_hours` hours.
    pub(super) fn upcoming_rain(&self, now: DateTime<Utc>, within_hours: i64) -> Option<&Hour> {
        let end = now + salah::Duration::hours(within_hours);
        self.hours.iter().find(|hour| hour.time > now && hour.time < end && is_rain(hour))
    }
}

fn chrono_hour() -> salah::Duration {
    salah::Duration::hours(1)
}

pub(super) fn is_rain(hour: &Hour) -> bool {
    hour.rain_millimeters >= RAIN_MILLIMETERS || hour.symbol.is_wet()
}

fn number(object: &JsonObject, name: &windows::core::HSTRING) -> Option<f32> {
    object.GetNamedNumber(name).ok().map(|value| value as f32)
}

fn summary(period: &JsonObject) -> Option<(Symbol, f32)> {
    let code = period.GetNamedObject(h!("summary")).ok()?.GetNamedString(h!("symbol_code")).ok()?.to_string();
    let rain = period
        .GetNamedObject(h!("details"))
        .ok()
        .and_then(|details| number(&details, h!("precipitation_amount")))
        .unwrap_or(0.0);
    Some((Symbol::from_code(&code), rain))
}

fn entries_of(series: &JsonArray, earliest: DateTime<Utc>) -> Vec<Entry> {
    (0..series.Size().unwrap_or(0))
        .filter_map(|index| {
            let item = series.GetObjectAt(index).ok()?;
            let time = DateTime::parse_from_rfc3339(&item.GetNamedString(h!("time")).ok()?.to_string())
                .ok()?
                .with_timezone(&Utc);
            if time < earliest {
                return None;
            }
            let data = item.GetNamedObject(h!("data")).ok()?;
            let details = data.GetNamedObject(h!("instant")).ok()?.GetNamedObject(h!("details")).ok()?;
            Some(Entry {
                time,
                temperature: number(&details, h!("air_temperature"))?,
                apparent: number(&details, h!("apparent_air_temperature")),
                hour: data.GetNamedObject(h!("next_1_hours")).ok().and_then(|period| summary(&period)),
                six_hours: data.GetNamedObject(h!("next_6_hours")).ok().and_then(|period| summary(&period)),
            })
        })
        .collect()
}

/// Groups the entries by the PC's local date, with the extremes of the day, the sky around midday and the water that
/// falls (hourly where the response has it, in six-hour steps further out).
fn days_of(entries: &[Entry]) -> Vec<Day> {
    let mut by_date: BTreeMap<NaiveDate, Vec<&Entry>> = BTreeMap::new();
    for entry in entries {
        by_date.entry(entry.time.with_timezone(&Local).date_naive()).or_default().push(entry);
    }
    by_date
        .into_iter()
        .take(DAYS_SHOWN)
        .filter_map(|(date, day)| {
            let high = day.iter().map(|entry| entry.temperature).fold(f32::MIN, f32::max);
            let low = day.iter().map(|entry| entry.temperature).fold(f32::MAX, f32::min);
            let midday = day
                .iter()
                .filter(|entry| entry.hour.or(entry.six_hours).is_some())
                .min_by_key(|entry| (i64::from(entry.time.with_timezone(&Local).hour()) - MIDDAY_HOUR).abs())?;
            let (symbol, _) = midday.hour.or(midday.six_hours)?;
            let rain_millimeters = day
                .iter()
                .map(|entry| match (entry.hour, entry.six_hours) {
                    (Some((_, rain)), _) => rain,
                    (None, Some((_, rain))) => rain,
                    (None, None) => 0.0,
                })
                .sum();
            Some(Day { date, high, low, symbol, rain_millimeters })
        })
        .collect()
}

/// "Rain now", "Rain from 17:00" or "No rain expected" for the next twelve hours.
pub(super) fn rain_line(forecast: &Forecast, now: DateTime<Utc>) -> String {
    match forecast.first_rain(now, 12) {
        Some(hour) if hour.time <= now => tr!("weather.rain_now").into_owned(),
        Some(hour) => {
            tr!("weather.rain_from", time = hour.time.with_timezone(&Local).format("%H:%M").to_string()).into_owned()
        }
        None => tr!("weather.no_rain").into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(hour: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-05T00:00:00Z").unwrap().with_timezone(&Utc) + salah::Duration::hours(hour)
    }

    fn hour(offset: i64, rain: f32, symbol: Symbol) -> Hour {
        Hour { time: at(offset), temperature: 30.0, symbol, rain_millimeters: rain }
    }

    fn forecast(hours: Vec<Hour>) -> Forecast {
        Forecast { fetched_at: at(0), feels_like: 30.0, hours, days: Vec::new() }
    }

    const BODY: &str = r#"{"properties":{"timeseries":[
        {"time":"2026-11-05T14:00:00Z","data":{"instant":{"details":{"air_temperature":24.9,"apparent_air_temperature":27.0}},
         "next_1_hours":{"summary":{"symbol_code":"rain"},"details":{"precipitation_amount":0.5}},
         "next_6_hours":{"summary":{"symbol_code":"heavyrain"},"details":{"precipitation_amount":7.6}}}},
        {"time":"2026-11-05T15:00:00Z","data":{"instant":{"details":{"air_temperature":24.3}},
         "next_1_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}},
         "next_6_hours":{"summary":{"symbol_code":"cloudy"},"details":{"precipitation_amount":0.0}}}},
        {"time":"2026-11-09T06:00:00Z","data":{"instant":{"details":{"air_temperature":22.0}},
         "next_6_hours":{"summary":{"symbol_code":"partlycloudy_day"},"details":{"precipitation_amount":1.2}}}}
    ]}}"#;

    #[test]
    fn a_response_becomes_hours_a_feels_like_temperature_and_days() {
        let forecast = Forecast::parse(BODY, at(14)).unwrap();
        assert_eq!(forecast.hours.len(), 2);
        assert_eq!(forecast.hours[0].symbol, Symbol::Rain);
        assert_eq!(forecast.hours[0].rain_millimeters, 0.5);
        assert_eq!(forecast.hours[1].symbol, Symbol::Cloudy);
        assert_eq!(forecast.feels_like, 27.0);
        assert!(forecast.days.len() >= 2, "{:?}", forecast.days);
    }

    /// A real `complete` response for Pekanbaru, cut to its first three steps and the last one.
    #[test]
    fn a_saved_real_response_parses() {
        let sample = include_str!("sample_response.json");
        let fetched_at = DateTime::parse_from_rfc3339("2026-11-05T17:05:00Z").unwrap().with_timezone(&Utc);
        let forecast = Forecast::parse(sample, fetched_at).expect("the real response parses");
        assert_eq!(forecast.hours.len(), 3);
        assert_eq!(forecast.hours[0].symbol, Symbol::HeavyRain);
        assert_eq!(forecast.hours[0].rain_millimeters, 1.4);
        assert_eq!(forecast.feels_like, 23.7);
        assert!(!forecast.days.is_empty());
    }

    #[test]
    fn a_response_that_is_not_a_forecast_gives_nothing() {
        assert!(Forecast::parse("not json", at(0)).is_none());
        assert!(Forecast::parse(r#"{"properties":{"timeseries":[]}}"#, at(0)).is_none());
        assert!(Forecast::parse("{}", at(0)).is_none());
    }

    #[test]
    fn hours_that_are_already_over_are_left_out() {
        let forecast = Forecast::parse(BODY, at(16)).unwrap();
        assert_eq!(forecast.hours.len(), 1, "only 15:00 is within an hour of 16:00");
        assert_eq!(forecast.hours[0].symbol, Symbol::Cloudy);
    }

    #[test]
    fn the_first_rain_is_found_within_the_window() {
        let forecast =
            forecast(vec![hour(0, 0.0, Symbol::Cloudy), hour(1, 0.0, Symbol::Cloudy), hour(3, 1.0, Symbol::Rain)]);
        assert_eq!(forecast.first_rain(at(0), 12).map(|hour| hour.time), Some(at(3)));
        assert!(forecast.first_rain(at(0), 2).is_none());
    }

    #[test]
    fn a_wet_sky_counts_as_rain_even_with_little_water() {
        assert!(is_rain(&hour(0, 0.0, Symbol::Thunderstorm)));
        assert!(is_rain(&hour(0, 0.3, Symbol::Cloudy)));
        assert!(!is_rain(&hour(0, 0.1, Symbol::Cloudy)));
    }

    #[test]
    fn the_rain_line_says_now_a_time_or_none() {
        let dry = forecast(vec![hour(0, 0.0, Symbol::Clear)]);
        assert_eq!(rain_line(&dry, at(0)), "No rain expected");
        let raining = forecast(vec![hour(0, 2.0, Symbol::Rain)]);
        assert_eq!(rain_line(&raining, at(0) + salah::Duration::minutes(10)), "Rain now");
        let later = forecast(vec![hour(0, 0.0, Symbol::Clear), hour(5, 2.0, Symbol::Rain)]);
        assert!(rain_line(&later, at(0)).starts_with("Rain from "));
    }

    #[test]
    fn the_current_hour_is_the_latest_one_that_has_started() {
        let forecast =
            forecast(vec![hour(0, 0.0, Symbol::Clear), hour(1, 0.0, Symbol::Cloudy), hour(2, 0.0, Symbol::Rain)]);
        assert_eq!(forecast.current(at(1) + salah::Duration::minutes(20)).map(|hour| hour.time), Some(at(1)));
        assert_eq!(forecast.current(at(0) - salah::Duration::hours(3)).map(|hour| hour.time), Some(at(0)));
    }
}
