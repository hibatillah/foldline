//! The forecast request runs off-thread and notifies the page when it is done.

use std::sync::{Arc, Mutex};

use windows::{Foundation::Uri, Web::Http::HttpClient, core::HSTRING};

use crate::page::Notifier;

/// MET Norway asks every client to say who it is; a missing or generic name is blocked.
const USER_AGENT: &str = concat!("Notch/", env!("CARGO_PKG_VERSION"), " github.com/hibatillah/notch");

/// What a request found: the place it was for, so an answer that arrives after the place changed is dropped, and the
/// response body or the reason it failed.
pub(super) type Outcome = Arc<Mutex<Option<((i64, i64), Result<String, String>)>>>;

/// A place rounded to about a kilometre, which is all the forecast distinguishes; also how an outcome is matched.
pub(super) fn place_key(latitude: f64, longitude: f64) -> (i64, i64) {
    ((latitude * 100.0).round() as i64, (longitude * 100.0).round() as i64)
}

fn address(latitude: f64, longitude: f64) -> String {
    format!("https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={latitude:.3}&lon={longitude:.3}")
}

fn download(latitude: f64, longitude: f64) -> Result<String, String> {
    let describe = |error: windows::core::Error| error.message();
    let client = HttpClient::new().map_err(describe)?;
    client
        .DefaultRequestHeaders()
        .and_then(|headers| headers.UserAgent())
        .and_then(|agent| agent.TryParseAdd(&HSTRING::from(USER_AGENT)))
        .map_err(describe)?;
    let uri = Uri::CreateUri(&HSTRING::from(address(latitude, longitude))).map_err(describe)?;
    let text = client.GetStringAsync(&uri).map_err(describe)?.join().map_err(describe)?;
    Ok(text.to_string())
}

pub(super) fn start(latitude: f64, longitude: f64, outcome: Outcome, notifier: Notifier) {
    std::thread::spawn(move || {
        let result = download(latitude, longitude);
        if let Ok(mut slot) = outcome.lock() {
            *slot = Some((place_key(latitude, longitude), result));
        }
        notifier.notify();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_names_the_place_with_three_decimals() {
        assert_eq!(
            address(0.5171, 101.4424),
            "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat=0.517&lon=101.442"
        );
    }

    #[test]
    fn the_client_says_who_it_is() {
        assert!(USER_AGENT.starts_with("Notch/"));
        assert!(USER_AGENT.contains("github.com"));
    }

    #[test]
    fn places_a_kilometre_apart_are_one_place_and_cities_apart_are_not() {
        assert_eq!(place_key(0.5171, 101.4424), place_key(0.5180, 101.4390));
        assert_ne!(place_key(0.517, 101.442), place_key(-6.2, 106.8));
    }
}
