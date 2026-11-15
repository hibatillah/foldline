//! Calls to the site over `Windows.Web.Http`, each blocking its caller for at most 15 s; call them off the UI thread.

use std::sync::mpsc;
use std::time::Duration;

use windows::Foundation::Uri;
use windows::Storage::Streams::UnicodeEncoding;
use windows::Web::Http::{HttpClient, HttpResponseMessage, HttpStringContent};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
use windows::core::HSTRING;

pub use crate::reply::{Claimed, ClientError};
use crate::reply::{Reply, check_result, claim_result, json_body};
use crate::schedule::CheckOutcome;

const SITE_URL: &str = "https://fold.encure.id";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Keeps the calling thread in the multithreaded apartment; dropping it leaves again.
pub struct Apartment;

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: only built by a successful `RoInitialize` on this thread, so this call balances it.
        unsafe { RoUninitialize() };
    }
}

/// Makes the thread's membership of the multithreaded apartment explicit instead of relying on the implicit one.
/// S_FALSE (already joined) counts as success; RPC_E_CHANGED_MODE comes back as an error the caller may ignore.
pub fn join_multithreaded_apartment() -> windows::core::Result<Apartment> {
    // SAFETY: plain initialization call with no pointers.
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map(|()| Apartment)
}

/// Joins the operation on a helper thread and cancels it after the limit, so a stalled network never holds the caller.
/// A macro because the async types live in `windows_future`, which this crate does not depend on directly.
macro_rules! wait_for_response {
    ($operation:expr) => {{
        let operation = $operation;
        let waiting = operation.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            // A refused join (apartment already set up another way) is not fatal; the call still runs.
            let _apartment = join_multithreaded_apartment();
            let _ = sender.send(waiting.join());
        });
        match receiver.recv_timeout(REQUEST_TIMEOUT) {
            Ok(Ok(response)) => Some(response),
            Ok(Err(_)) => None,
            Err(_) => {
                let _ = operation.Cancel();
                None
            }
        }
    }};
}

/// Release builds use the site; debug builds use `NOTCH_ACCOUNT_URL` or the local mock server.
pub fn site_url() -> String {
    site_url_for(cfg!(debug_assertions), std::env::var("NOTCH_ACCOUNT_URL").ok())
}

fn site_url_for(is_debug: bool, override_url: Option<String>) -> String {
    if !is_debug {
        return SITE_URL.to_owned();
    }
    override_url
        .map(|url| url.trim_end_matches('/').to_owned())
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_owned())
}

pub fn claim(code: &str, verifier: &str) -> Result<Claimed, ClientError> {
    let body = json_body(&[("code", code), ("verifier", verifier)]);
    claim_result(post("/api/device/claim", &body))
}

pub fn check(device_id: &str, device_secret: &str) -> CheckOutcome {
    let body = json_body(&[("device_id", device_id), ("device_secret", device_secret)]);
    check_result(post("/api/device/check", &body), true)
}

/// The guest's daily refresh: the policy alone, with no identifier.
pub fn fetch_policy() -> CheckOutcome {
    check_result(get("/api/policy"), false)
}

/// Best effort: the app forgets the device whether or not the site hears about it.
pub fn sign_out(device_id: &str, device_secret: &str) {
    let body = json_body(&[("device_id", device_id), ("device_secret", device_secret)]);
    let _ = post("/api/device/sign-out", &body);
}

fn post(path: &str, json: &str) -> Option<Reply> {
    let content = HttpStringContent::CreateFromStringWithEncodingAndMediaType(
        &HSTRING::from(json),
        UnicodeEncoding::Utf8,
        &HSTRING::from("application/json"),
    )
    .ok()?;
    let uri = Uri::CreateUri(&HSTRING::from(format!("{}{path}", site_url()))).ok()?;
    let operation = HttpClient::new().ok()?.PostAsync(&uri, &content).ok()?;
    reply_from(wait_for_response!(operation)?)
}

fn get(path: &str) -> Option<Reply> {
    get_address(&format!("{}{path}", site_url()))
}

fn get_address(address: &str) -> Option<Reply> {
    let uri = Uri::CreateUri(&HSTRING::from(address)).ok()?;
    let operation = HttpClient::new().ok()?.GetAsync(&uri).ok()?;
    reply_from(wait_for_response!(operation)?)
}

fn reply_from(response: HttpResponseMessage) -> Option<Reply> {
    // The request already buffered the whole content, so this read does not wait on the network and needs no limit.
    let body = response.Content().ok()?.ReadAsStringAsync().ok()?.join().ok()?.to_string();
    Some(Reply { status: response.StatusCode().ok()?.0, body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joining_the_apartment_again_on_the_same_thread_still_succeeds() {
        let result = std::thread::spawn(|| {
            let first = join_multithreaded_apartment().is_ok();
            let second = join_multithreaded_apartment().is_ok();
            first && second
        })
        .join()
        .unwrap();
        assert!(result);
    }

    #[test]
    fn release_uses_the_site_and_debug_uses_the_override_or_the_mock() {
        assert_eq!(site_url_for(false, Some("http://x".into())), "https://fold.encure.id");
        assert_eq!(site_url_for(true, Some("http://example.test:9/".into())), "http://example.test:9");
        assert_eq!(site_url_for(true, None), "http://127.0.0.1:8787");
        assert_eq!(site_url_for(true, Some(String::new())), "http://127.0.0.1:8787");
    }

    #[test]
    fn a_request_to_a_closed_port_is_a_failed_call() {
        // Port 1 on loopback refuses at once, so this needs no server and no wait.
        assert_eq!(get_address("http://127.0.0.1:1/"), None);
    }
}
