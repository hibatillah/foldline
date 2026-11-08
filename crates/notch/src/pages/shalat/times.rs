//! Prayer times calculated on this PC; pure functions of a place, a date and the chosen method.

use salah::{
    Configuration, Coordinates, DateTime, Datelike, Duration, Madhab, Method, NaiveDate, Parameters,
    Prayer as SalahPrayer, PrayerTimes, TimeAdjustment, Utc,
};

/// The six times shown, in the order of the day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prayer {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

pub const PRAYERS: [Prayer; 6] =
    [Prayer::Fajr, Prayer::Sunrise, Prayer::Dhuhr, Prayer::Asr, Prayer::Maghrib, Prayer::Isha];

impl Prayer {
    pub fn name(self) -> &'static str {
        match self {
            Prayer::Fajr => "Fajr",
            Prayer::Sunrise => "Sunrise",
            Prayer::Dhuhr => "Dhuhr",
            Prayer::Asr => "Asr",
            Prayer::Maghrib => "Maghrib",
            Prayer::Isha => "Isha",
        }
    }

    pub fn offset_key(self) -> &'static str {
        match self {
            Prayer::Fajr => "shalat.offset_fajr",
            Prayer::Sunrise => "shalat.offset_sunrise",
            Prayer::Dhuhr => "shalat.offset_dhuhr",
            Prayer::Asr => "shalat.offset_asr",
            Prayer::Maghrib => "shalat.offset_maghrib",
            Prayer::Isha => "shalat.offset_isha",
        }
    }

    /// Sunrise ends the morning prayer; it is a marker, not a prayer.
    pub fn is_prayer(self) -> bool {
        self != Prayer::Sunrise
    }

    fn salah(self) -> SalahPrayer {
        match self {
            Prayer::Fajr => SalahPrayer::Fajr,
            Prayer::Sunrise => SalahPrayer::Sunrise,
            Prayer::Dhuhr => SalahPrayer::Dhuhr,
            Prayer::Asr => SalahPrayer::Asr,
            Prayer::Maghrib => SalahPrayer::Maghrib,
            Prayer::Isha => SalahPrayer::Isha,
        }
    }
}

/// The method words the settings window offers, which its tests compare against; `auto` picks one from the city's country.
#[cfg(test)]
pub const METHOD_OPTIONS: &[(&str, &str)] = &[
    ("auto", "Auto"),
    ("singapore", "Kemenag"),
    ("mwl", "MWL"),
    ("northamerica", "ISNA"),
    ("karachi", "Karachi"),
    ("ummalqura", "Makkah"),
    ("egyptian", "Egypt"),
];

/// The method used for a country when the setting is `auto`.
pub fn method_for_country(country: &str) -> Method {
    match country {
        "ID" | "MY" | "SG" | "BN" => Method::Singapore,
        "SA" => Method::UmmAlQura,
        "AE" => Method::Dubai,
        "QA" => Method::Qatar,
        "KW" => Method::Kuwait,
        "EG" => Method::Egyptian,
        "PK" | "IN" | "BD" | "AF" => Method::Karachi,
        "IR" => Method::Tehran,
        "TR" => Method::Turkey,
        "US" | "CA" => Method::NorthAmerica,
        _ => Method::MuslimWorldLeague,
    }
}

pub fn parse_method(word: &str, country: &str) -> Method {
    match word {
        "singapore" => Method::Singapore,
        "mwl" => Method::MuslimWorldLeague,
        "northamerica" => Method::NorthAmerica,
        "karachi" => Method::Karachi,
        "ummalqura" => Method::UmmAlQura,
        "egyptian" => Method::Egyptian,
        _ => method_for_country(country),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub method: Method,
    pub madhab: Madhab,
    /// Minutes added to each of `PRAYERS`, -30 to 30.
    pub offsets: [i64; 6],
}

/// Degrees the sun must stay beyond a threshold for the calculation to be trusted; 1.5 still panicked in a scan.
const SUN_MARGIN: f64 = 3.0;
/// The sun's altitude at sunrise and sunset, in degrees.
const HORIZON_ALTITUDE: f64 = 0.833;

/// The sun's declination in degrees on `date`, accurate to about a degree.
fn declination(date: NaiveDate) -> f64 {
    23.44 * (std::f64::consts::TAU * (284.0 + f64::from(date.ordinal())) / 365.0).sin()
}

/// The calculation settings for `date`, or `None` when the sun does not rise or set around then.
///
/// `salah` panics when a time it needs does not exist, and a panic closes the whole notch. Near the poles the sun can stay
/// up or down all day, and in summer at 45 degrees and beyond it never gets deep enough for the Fajr and Isha angles.
/// The first case has no prayer times; in the second the angles are capped at what the sun reaches, which hands the time
/// to the crate's night-portion rule.
fn guarded(date: NaiveDate, place: &Place, rules: &Rules) -> Option<Parameters> {
    let mut parameters = Configuration::with(rules.method, rules.madhab);
    let [fajr, sunrise, dhuhr, asr, maghrib, isha] = rules.offsets;
    parameters.adjustments = TimeAdjustment::new(fajr, sunrise, dhuhr, asr, maghrib, isha);
    let mut deepest = f64::MAX;
    // The crate also looks one day back and two days ahead.
    for days in -1..=2 {
        let declination = declination(date + Duration::days(days));
        let noon_altitude = 90.0 - (place.latitude - declination).abs();
        let midnight_depression = 90.0 - (place.latitude + declination).abs();
        if noon_altitude < SUN_MARGIN || midnight_depression < HORIZON_ALTITUDE + SUN_MARGIN {
            return None;
        }
        deepest = deepest.min(midnight_depression);
    }
    let reachable = deepest - SUN_MARGIN;
    parameters.fajr_angle = parameters.fajr_angle.min(reachable);
    parameters.isha_angle = parameters.isha_angle.min(reachable);
    parameters.maghrib_angle = parameters.maghrib_angle.min(reachable);
    Some(parameters)
}

fn day(date: NaiveDate, place: &Place, rules: &Rules) -> Option<[DateTime<Utc>; 6]> {
    let parameters = guarded(date, place, rules)?;
    let times = PrayerTimes::new(date, Coordinates::new(place.latitude, place.longitude), parameters);
    Some(PRAYERS.map(|prayer| times.time(prayer.salah())))
}

/// The six times of one day, in order; `None` when the sun does not rise and set that day.
pub fn day_times(date: NaiveDate, place: &Place, rules: &Rules) -> Option<[DateTime<Utc>; 6]> {
    day(date, place, rules)
}

/// The first time after `now`, looking from yesterday to tomorrow so the day's edges and far time zones work.
pub fn next_after(
    now: DateTime<Utc>,
    date: NaiveDate,
    place: &Place,
    rules: &Rules,
) -> Option<(Prayer, DateTime<Utc>)> {
    let days = [date - Duration::days(1), date, date + Duration::days(1)];
    days.into_iter()
        .filter_map(|date| day(date, place, rules))
        .flat_map(|times| PRAYERS.into_iter().zip(times))
        .filter(|&(_, time)| time > now)
        .min_by_key(|&(_, time)| time)
}

/// The last time at or before `now`, used to show the prayer whose time it currently is.
pub fn previous_before(
    now: DateTime<Utc>,
    date: NaiveDate,
    place: &Place,
    rules: &Rules,
) -> Option<(Prayer, DateTime<Utc>)> {
    let days = [date - Duration::days(1), date, date + Duration::days(1)];
    days.into_iter()
        .filter_map(|date| day(date, place, rules))
        .flat_map(|times| PRAYERS.into_iter().zip(times))
        .filter(|&(_, time)| time <= now)
        .max_by_key(|&(_, time)| time)
}

#[cfg(test)]
mod tests {
    use salah::Timelike;

    use super::*;

    fn rules(method: Method) -> Rules {
        Rules { method, madhab: Madhab::Shafi, offsets: [0; 6] }
    }

    fn local_minutes(time: DateTime<Utc>, hours: i32) -> i64 {
        (i64::from(time.hour()) * 60 + i64::from(time.minute()) + i64::from(hours) * 60).rem_euclid(24 * 60)
    }

    /// Aladhan's Kemenag method for Jakarta on 2026-11-03 gave 04:06, 05:26, 11:36, 14:54, 17:46, 18:58 (UTC+7).
    #[test]
    fn jakarta_matches_the_kemenag_method_within_a_few_minutes() {
        let jakarta = Place { latitude: -6.2087634, longitude: 106.845599 };
        let date = NaiveDate::from_ymd_opt(2026, 11, 3).expect("valid date");
        let times = day_times(date, &jakarta, &rules(Method::Singapore)).expect("times");
        let expected = [4 * 60 + 6, 5 * 60 + 26, 11 * 60 + 36, 14 * 60 + 54, 17 * 60 + 46, 18 * 60 + 58];
        for ((prayer, time), expected) in PRAYERS.iter().zip(times).zip(expected) {
            let difference = (local_minutes(time, 7) - expected).abs();
            assert!(difference <= 3, "{} off by {difference} minutes", prayer.name());
        }
    }

    #[test]
    fn an_offset_moves_only_its_own_time() {
        let place = Place { latitude: 0.517, longitude: 101.442 };
        let date = NaiveDate::from_ymd_opt(2026, 11, 3).expect("valid date");
        let plain = day_times(date, &place, &rules(Method::Singapore)).expect("times");
        let mut shifted_rules = rules(Method::Singapore);
        shifted_rules.offsets[2] = 2;
        let shifted = day_times(date, &place, &shifted_rules).expect("times");
        for (index, (a, b)) in plain.iter().zip(shifted).enumerate() {
            let expected = if index == 2 { Duration::minutes(2) } else { Duration::zero() };
            assert_eq!(b - *a, expected, "{}", PRAYERS[index].name());
        }
    }

    #[test]
    fn the_next_prayer_after_isha_is_tomorrows_fajr() {
        let place = Place { latitude: 0.517, longitude: 101.442 };
        let date = NaiveDate::from_ymd_opt(2026, 11, 3).expect("valid date");
        let today = day_times(date, &place, &rules(Method::Singapore)).expect("times");
        let after_isha = today[5] + Duration::minutes(1);
        let (prayer, time) = next_after(after_isha, date, &place, &rules(Method::Singapore)).expect("a next prayer");
        assert_eq!(prayer, Prayer::Fajr);
        assert!(time > after_isha && time - after_isha < Duration::hours(12));
    }

    #[test]
    fn the_next_prayer_is_the_first_time_still_ahead() {
        let place = Place { latitude: 0.517, longitude: 101.442 };
        let date = NaiveDate::from_ymd_opt(2026, 11, 3).expect("valid date");
        let today = day_times(date, &place, &rules(Method::Singapore)).expect("times");
        let between = today[2] + Duration::minutes(1);
        let (prayer, time) = next_after(between, date, &place, &rules(Method::Singapore)).expect("a next prayer");
        assert_eq!((prayer, time), (Prayer::Asr, today[3]));
        let (before, _) = previous_before(between, date, &place, &rules(Method::Singapore)).expect("a previous");
        assert_eq!(before, Prayer::Dhuhr);
    }

    #[test]
    fn no_latitude_or_date_crashes_the_calculation_and_ordinary_places_always_get_times() {
        let methods = [Method::Singapore, Method::MuslimWorldLeague, Method::UmmAlQura, Method::Tehran];
        for (tenth, longitude) in
            (-900..=900).step_by(5).flat_map(|tenth| [-120.0, 18.9, 150.0].map(|longitude| (tenth, longitude)))
        {
            let place = Place { latitude: f64::from(tenth) / 10.0, longitude };
            for day_of_year in (1..=365).step_by(11) {
                let date = NaiveDate::from_yo_opt(2026, day_of_year).expect("valid day");
                for method in methods {
                    let times = day_times(date, &place, &rules(method));
                    if place.latitude.abs() <= 60.0 {
                        assert!(times.is_some(), "{} {date} {method:?}", place.latitude);
                    }
                }
            }
        }
    }

    /// Upstream `salah` panicked when a time fell in the last half minute of a UTC day; this input did (see vendor/salah).
    #[test]
    fn a_time_in_the_last_half_minute_of_a_utc_day_does_not_crash() {
        let place = Place { latitude: -53.0, longitude: -120.0 };
        let date = NaiveDate::from_yo_opt(2026, 145).expect("valid day");
        let times = day_times(date, &place, &rules(Method::Singapore)).expect("times");
        assert!(times.windows(2).all(|pair| pair[0] < pair[1]), "{times:?}");
    }

    #[test]
    fn a_summer_in_london_has_times_in_order_even_though_the_sun_barely_dips() {
        let london = Place { latitude: 51.5, longitude: -0.12 };
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
        let times = day_times(date, &london, &rules(Method::MuslimWorldLeague)).expect("times");
        assert!(times.windows(2).all(|pair| pair[0] < pair[1]), "{times:?}");
    }

    #[test]
    fn the_midnight_sun_has_no_prayer_times() {
        let tromso = Place { latitude: 69.65, longitude: 18.96 };
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).expect("valid date");
        assert!(day_times(date, &tromso, &rules(Method::MuslimWorldLeague)).is_none());
        assert!(next_after(Utc::now(), date, &tromso, &rules(Method::MuslimWorldLeague)).is_none());
    }

    #[test]
    fn auto_picks_a_method_from_the_country_and_a_word_overrides_it() {
        assert_eq!(method_for_country("ID"), Method::Singapore);
        assert_eq!(method_for_country("GB"), Method::MuslimWorldLeague);
        assert_eq!(parse_method("auto", "SA"), Method::UmmAlQura);
        assert_eq!(parse_method("karachi", "ID"), Method::Karachi);
        assert!(METHOD_OPTIONS.iter().all(|(word, _)| *word == "auto"
            || parse_method(word, "ZZ") != Method::MuslimWorldLeague
            || *word == "mwl"));
    }
}
