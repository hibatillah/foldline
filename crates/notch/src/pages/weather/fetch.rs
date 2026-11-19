//! The forecast request runs off-thread and notifies the page when it is done.

use std::sync::{Arc, Mutex};

use windows::{Foundation::Uri, Web::Http::HttpClient, core::HSTRING};

use crate::{log, page::Notifier};

/// MET Norway asks every client to say who it is; a missing or generic name is blocked.
/// The site must be a comment in parentheses: a bare `github.com/...` is not a valid product token, so `HttpClient`
/// drops the header without an error and sends its own, which MET Norway answers with 403.
const USER_AGENT: &str = concat!("Foldline/", env!("CARGO_PKG_VERSION"), " (+https://github.com/hibatillah/foldline)");

/// Why a request gave no forecast: the network is unreachable (a real offline state), or the service answered with
/// something that is not a forecast (a status other than 200); both carry the reason for the log.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Failure {
    Network(String),
    Service(String),
}

impl Failure {
    pub(super) fn reason(&self) -> &str {
        match self {
            Self::Network(reason) | Self::Service(reason) => reason,
        }
    }
}

/// What a request found: the place it was for, so an answer that arrives after the place changed is dropped, and the
/// response body or the failure.
pub(super) type Outcome = Arc<Mutex<Option<((i64, i64), Result<String, Failure>)>>>;

/// A place rounded to about a kilometre, which is all the forecast distinguishes; also how an outcome is matched.
pub(super) fn place_key(latitude: f64, longitude: f64) -> (i64, i64) {
    ((latitude * 100.0).round() as i64, (longitude * 100.0).round() as i64)
}

fn address(latitude: f64, longitude: f64) -> String {
    format!("https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={latitude:.3}&lon={longitude:.3}")
}

/// `None` for a success status, otherwise the reason the service refused.
fn status_problem(status: i32) -> Option<String> {
    match status {
        200..=299 => None,
        403 => Some("403: the service rejected the User-Agent".to_owned()),
        429 => Some("429: too many requests".to_owned()),
        other => Some(format!("status {other}")),
    }
}

fn download(latitude: f64, longitude: f64) -> Result<String, Failure> {
    let network = |error: windows::core::Error| Failure::Network(error.message());
    let client = HttpClient::new().map_err(network)?;
    let accepted = client
        .DefaultRequestHeaders()
        .and_then(|headers| headers.UserAgent())
        .and_then(|agent| agent.TryParseAdd(&HSTRING::from(USER_AGENT)))
        .map_err(network)?;
    if !accepted {
        log::debug(&format!("weather: the User-Agent {USER_AGENT:?} was not accepted by HttpClient"));
    }
    let uri = Uri::CreateUri(&HSTRING::from(address(latitude, longitude))).map_err(network)?;
    let response = client.GetAsync(&uri).map_err(network)?.join().map_err(network)?;
    let status = response.StatusCode().map_err(network)?.0;
    if let Some(problem) = status_problem(status) {
        return Err(Failure::Service(problem));
    }
    let text = response.Content().and_then(|content| content.ReadAsStringAsync()).map_err(network)?;
    Ok(text.join().map_err(network)?.to_string())
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
        assert!(USER_AGENT.starts_with("Foldline/"));
        assert!(USER_AGENT.contains("github.com"));
    }

    #[test]
    fn the_user_agent_is_a_product_with_the_site_as_a_comment() {
        assert!(USER_AGENT.contains(" (+https://github.com/hibatillah/foldline)"));
        assert!(!USER_AGENT.contains(" github.com"));
    }

    #[test]
    fn only_a_success_status_is_accepted() {
        assert_eq!(status_problem(200), None);
        assert!(status_problem(403).unwrap().contains("User-Agent"));
        assert!(status_problem(429).unwrap().starts_with("429"));
        assert_eq!(status_problem(503).as_deref(), Some("status 503"));
    }

    #[test]
    fn places_a_kilometre_apart_are_one_place_and_cities_apart_are_not() {
        assert_eq!(place_key(0.5171, 101.4424), place_key(0.5180, 101.4390));
        assert_ne!(place_key(0.517, 101.442), place_key(-6.2, 106.8));
    }
}
