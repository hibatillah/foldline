//! The settings keys the page reads and the values they become.

use super::times;
use super::times::{PRAYERS, Place, Rules};
use crate::host::Settings;
use places::Location;
use salah::Madhab;

pub(super) const METHOD_KEY: &str = "shalat.method";
pub(super) const HANAFI_KEY: &str = "shalat.hanafi";
pub(super) const MESSAGE_KEY: &str = "shalat.message";
pub(super) const BADGE_KEY: &str = "shalat.badge";

/// Everything read from the settings file.
#[derive(Clone, Debug)]
pub(super) struct Config {
    pub(super) city: String,
    pub(super) place: Option<Place>,
    pub(super) rules: Rules,
    pub(super) shows_message: bool,
    pub(super) shows_badge: bool,
}

impl Config {
    pub(super) fn read(get: &dyn Fn(&str) -> Option<String>) -> Self {
        let location = Location::read(get);
        let place =
            location.as_ref().map(|location| Place { latitude: location.latitude, longitude: location.longitude });
        let country = location.as_ref().map(|location| location.country.clone()).unwrap_or_default();
        let method = times::parse_method(get(METHOD_KEY).as_deref().unwrap_or("auto"), &country);
        let is_on = |key: &str, default: bool| match get(key).as_deref() {
            Some("on") => true,
            Some("off") => false,
            _ => default,
        };
        let mut offsets = [0i64; 6];
        for (offset, prayer) in offsets.iter_mut().zip(PRAYERS) {
            *offset =
                get(prayer.offset_key()).and_then(|text| text.trim().parse::<i64>().ok()).unwrap_or(0).clamp(-30, 30);
        }
        Self {
            city: location.map(|location| location.label).unwrap_or_default(),
            place,
            rules: Rules {
                method,
                madhab: if is_on(HANAFI_KEY, false) { Madhab::Hanafi } else { Madhab::Shafi },
                offsets,
            },
            shows_message: is_on(MESSAGE_KEY, true),
            shows_badge: is_on(BADGE_KEY, true),
        }
    }

    pub(super) fn from_settings(settings: &Settings) -> Self {
        Self::read(&|key| settings.get_string(key).map(str::to_owned))
    }
}
