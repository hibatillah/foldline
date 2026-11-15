//! Signing in through the browser: the site's login address, the loopback callback it redirects to, and finishing the
//! sign-in with the code it carries.

mod complete;
mod flow;
mod listener;

pub use complete::{Completion, complete, sign_out};
pub use flow::{Phase, SignInFlow};
pub use listener::{Listener, Stopper, Waited};

/// What the browser's redirect to `/callback` says.
#[derive(Clone, PartialEq, Eq)]
pub enum Callback {
    Code(String),
    Cancelled,
    /// Not a callback, a wrong `state`, or neither a code nor a cancel.
    Refused,
}

impl std::fmt::Debug for Callback {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callback::Code(_) => formatter.write_str("Code(<redacted>)"),
            Callback::Cancelled => formatter.write_str("Cancelled"),
            Callback::Refused => formatter.write_str("Refused"),
        }
    }
}

/// Reads `GET /callback?code&state` or `GET /callback?error=cancelled&state`; the state must match the one sent.
pub fn parse_callback(request_line: &str, expected_state: &str) -> Callback {
    let mut words = request_line.split(' ');
    let (Some("GET"), Some(target)) = (words.next(), words.next()) else { return Callback::Refused };
    let Some(query) = target.strip_prefix("/callback?") else { return Callback::Refused };
    let mut code = None;
    let mut state = None;
    let mut error = None;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let Some(value) = percent_decode(value) else { return Callback::Refused };
        match name {
            "code" => code = Some(value),
            "state" => state = Some(value),
            "error" => error = Some(value),
            _ => {}
        }
    }
    if expected_state.is_empty() || state.as_deref() != Some(expected_state) {
        return Callback::Refused;
    }
    match (code, error.as_deref()) {
        (_, Some("cancelled")) => Callback::Cancelled,
        (Some(code), None) if !code.is_empty() => Callback::Code(code),
        _ => Callback::Refused,
    }
}

/// The values the site's login page needs, besides the address of the site.
pub struct LoginRequest<'a> {
    pub port: u16,
    pub challenge: &'a str,
    pub state: &'a str,
    pub device_hash: &'a str,
    pub device_name: &'a str,
}

/// `<site>/device/login?port&challenge&state&device_hash&name`, every value percent-encoded.
pub fn login_url(site: &str, request: &LoginRequest) -> String {
    format!(
        "{site}/device/login?port={}&challenge={}&state={}&device_hash={}&name={}",
        request.port,
        percent_encode(request.challenge),
        percent_encode(request.state),
        percent_encode(request.device_hash),
        percent_encode(request.device_name),
    )
}

/// Keeps RFC 3986's unreserved characters and encodes every other UTF-8 byte as `%XX`.
fn percent_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

/// `None` for a broken `%` escape or bytes that are not UTF-8.
fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATE: &str = "state-123_abc";

    #[test]
    fn a_callback_with_a_code_and_the_right_state_returns_the_code() {
        let line = format!("GET /callback?code=the-code&state={STATE} HTTP/1.1");
        assert_eq!(parse_callback(&line, STATE), Callback::Code("the-code".to_owned()));
    }

    #[test]
    fn a_callback_with_a_wrong_state_is_refused() {
        let line = "GET /callback?code=the-code&state=other HTTP/1.1";
        assert_eq!(parse_callback(line, STATE), Callback::Refused);
        let cancelled = "GET /callback?error=cancelled&state=other HTTP/1.1";
        assert_eq!(parse_callback(cancelled, STATE), Callback::Refused);
        let missing = "GET /callback?code=the-code HTTP/1.1";
        assert_eq!(parse_callback(missing, STATE), Callback::Refused);
    }

    #[test]
    fn a_cancel_with_the_right_state_is_a_cancel() {
        let line = format!("GET /callback?error=cancelled&state={STATE} HTTP/1.1");
        assert_eq!(parse_callback(&line, STATE), Callback::Cancelled);
    }

    #[test]
    fn other_paths_methods_and_empty_codes_are_refused() {
        for line in [
            format!("POST /callback?code=c&state={STATE} HTTP/1.1"),
            format!("GET /favicon.ico?code=c&state={STATE} HTTP/1.1"),
            format!("GET /callback?code=&state={STATE} HTTP/1.1"),
            format!("GET /callback?error=denied&state={STATE} HTTP/1.1"),
            format!("GET /callback?code=%zz&state={STATE} HTTP/1.1"),
            String::new(),
        ] {
            assert_eq!(parse_callback(&line, STATE), Callback::Refused, "{line}");
        }
        assert_eq!(parse_callback("GET /callback?code=c&state= HTTP/1.1", ""), Callback::Refused);
    }

    #[test]
    fn escaped_values_are_decoded_before_the_state_is_compared() {
        let line = "GET /callback?code=a%2Fb&state=s%20t HTTP/1.1";
        assert_eq!(parse_callback(line, "s t"), Callback::Code("a/b".to_owned()));
    }

    #[test]
    fn a_code_never_shows_in_debug_output() {
        assert_eq!(format!("{:?}", Callback::Code("secret-code".to_owned())), "Code(<redacted>)");
    }

    #[test]
    fn the_login_address_carries_every_value_and_encodes_the_name() {
        let request = LoginRequest {
            port: 50123,
            challenge: "chal-lenge_1",
            state: "st",
            device_hash: "ab12",
            device_name: "Hiba's PC & co",
        };
        assert_eq!(
            login_url("http://127.0.0.1:8787", &request),
            "http://127.0.0.1:8787/device/login?port=50123&challenge=chal-lenge_1&state=st&device_hash=ab12\
             &name=Hiba%27s%20PC%20%26%20co"
        );
    }

    #[test]
    fn encoding_then_decoding_gives_the_text_back() {
        for text in ["", "plain", "a b&c=d/é"] {
            assert_eq!(percent_decode(&percent_encode(text)).as_deref(), Some(text));
        }
    }
}
