//! One read of the device position from Windows' location service. It blocks, so call it from a worker thread.

use windows::{
    Devices::Geolocation::{Geolocator, PositionAccuracy},
    Foundation::TimeSpan,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reading {
    Found {
        latitude: f64,
        longitude: f64,
    },
    /// Windows location is off for desktop apps, or for this user.
    Denied,
    Unavailable,
}

const E_ACCESSDENIED: i32 = 0x8007_0005_u32 as i32;
/// A position this old is as good as a new one for telling cities apart.
const MAXIMUM_AGE: TimeSpan = TimeSpan { Duration: 30 * 60 * 10_000_000 };
const TIMEOUT: TimeSpan = TimeSpan { Duration: 15 * 10_000_000 };

/// Asks once and returns; the notch never keeps Windows' location service open, which would show its "in use" icon.
pub fn read_position() -> Reading {
    try_read().unwrap_or_else(|error| reading_for_error(error.code().0))
}

fn try_read() -> windows::core::Result<Reading> {
    let locator = Geolocator::new()?;
    locator.SetDesiredAccuracy(PositionAccuracy::Default)?;
    let position = locator.GetGeopositionAsyncWithAgeAndTimeout(MAXIMUM_AGE, TIMEOUT)?.join()?;
    let point = position.Coordinate()?.Point()?.Position()?;
    Ok(Reading::Found { latitude: point.Latitude, longitude: point.Longitude })
}

fn reading_for_error(code: i32) -> Reading {
    if code == E_ACCESSDENIED { Reading::Denied } else { Reading::Unavailable }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_access_denied_error_means_windows_location_is_off_for_this_app() {
        assert_eq!(reading_for_error(0x8007_0005_u32 as i32), Reading::Denied);
    }

    #[test]
    #[ignore = "asks Windows where this machine is; run on purpose with --nocapture"]
    fn reading_this_machine_gives_a_position_or_says_why_not() {
        println!("{:?}", read_position());
    }

    #[test]
    fn any_other_error_means_no_position_was_available() {
        assert_eq!(reading_for_error(0x8000_4005_u32 as i32), Reading::Unavailable);
        assert_eq!(reading_for_error(0x8007_05B4_u32 as i32), Reading::Unavailable);
    }
}
