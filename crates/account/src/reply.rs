//! Request bodies and the meaning of replies from the site; no network here.

use crate::json::string_fields;
use crate::schedule::CheckOutcome;

#[derive(Clone, PartialEq, Eq)]
pub struct Claimed {
    pub device_id: String,
    pub device_secret: String,
    pub token: String,
    pub policy: String,
}

impl std::fmt::Debug for Claimed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Claimed")
            .field("device_id", &self.device_id)
            .field("device_secret", &"<redacted>")
            .field("token", &"<redacted>")
            .field("policy", &self.policy)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientError {
    /// The site refused the login code (400).
    InvalidCode,
    /// No answer, an unexpected status or an unreadable body.
    Unavailable,
}

/// What a request got back: the HTTP status and the body text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub status: i32,
    pub body: String,
}

/// A JSON object of string fields, escaping `"`, `\` and control characters.
pub fn json_body(fields: &[(&str, &str)]) -> String {
    let members: Vec<String> = fields.iter().map(|(key, value)| format!("{}:{}", quote(key), quote(value))).collect();
    format!("{{{}}}", members.join(","))
}

fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            control if (control as u32) < 0x20 => quoted.push_str(&format!("\\u{:04x}", control as u32)),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

/// A value with a line break is refused, since it would add lines to the account file.
fn field(fields: &[(String, String)], name: &str) -> Option<String> {
    let value = fields.iter().find(|(key, _)| key == name).map(|(_, value)| value)?;
    (!value.contains(['\r', '\n'])).then(|| value.clone())
}

pub fn claim_result(reply: Option<Reply>) -> Result<Claimed, ClientError> {
    let reply = reply.ok_or(ClientError::Unavailable)?;
    match reply.status {
        200 => {
            let fields = string_fields(&reply.body).ok_or(ClientError::Unavailable)?;
            let read = |name: &str| field(&fields, name).ok_or(ClientError::Unavailable);
            Ok(Claimed {
                device_id: read("device_id")?,
                device_secret: read("device_secret")?,
                token: read("token")?,
                policy: read("policy")?,
            })
        }
        400 => Err(ClientError::InvalidCode),
        _ => Err(ClientError::Unavailable),
    }
}

/// `with_token` is false for the guest policy fetch, which has no token to renew.
pub fn check_result(reply: Option<Reply>, with_token: bool) -> CheckOutcome {
    let Some(reply) = reply else { return CheckOutcome::Failed };
    match reply.status {
        200 => {
            let Some(fields) = string_fields(&reply.body) else { return CheckOutcome::Failed };
            let Some(policy) = field(&fields, "policy") else { return CheckOutcome::Failed };
            if !with_token {
                return CheckOutcome::Renewed { token: None, policy };
            }
            match field(&fields, "token") {
                Some(token) => CheckOutcome::Renewed { token: Some(token), policy },
                None => CheckOutcome::Failed,
            }
        }
        401 => CheckOutcome::UnknownDevice,
        410 => CheckOutcome::SignedOut,
        _ => CheckOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(status: i32, body: &str) -> Option<Reply> {
        Some(Reply { status, body: body.to_owned() })
    }

    #[test]
    fn a_json_body_escapes_quotes_backslashes_and_controls() {
        assert_eq!(json_body(&[("code", "a\"b\\c"), ("verifier", "x\ny")]), r#"{"code":"a\"b\\c","verifier":"x\ny"}"#);
        assert_eq!(json_body(&[]), "{}");
    }

    #[test]
    fn other_control_characters_are_escaped_as_u00xx() {
        assert_eq!(json_body(&[("k", "a\u{1}b\u{1f}")]), r#"{"k":"a\u0001b\u001f"}"#);
    }

    #[test]
    fn a_value_with_a_line_break_makes_the_reply_unusable() {
        let claim = r#"{"device_id":"d","device_secret":"s\nx","token":"t","policy":"p"}"#;
        assert_eq!(claim_result(reply(200, claim)), Err(ClientError::Unavailable));
        let check = r#"{"token":"t\r","policy":"p"}"#;
        assert_eq!(check_result(reply(200, check), true), CheckOutcome::Failed);
        let policy = r#"{"policy":"p\nq"}"#;
        assert_eq!(check_result(reply(200, policy), false), CheckOutcome::Failed);
    }

    #[test]
    fn debug_output_never_shows_the_secret() {
        let claimed = Claimed {
            device_id: "d".into(),
            device_secret: "very-secret".into(),
            token: "t".into(),
            policy: "p".into(),
        };
        let shown = format!("{claimed:?}");
        assert!(!shown.contains("very-secret"));
        assert!(shown.contains("<redacted>"));
    }

    #[test]
    fn debug_output_never_shows_the_token() {
        let claimed = Claimed {
            device_id: "d".into(),
            device_secret: "s".into(),
            token: "signed-token-text".into(),
            policy: "p".into(),
        };
        assert!(!format!("{claimed:?}").contains("signed-token-text"));
    }

    #[test]
    fn a_claim_maps_200_400_and_everything_else() {
        let body = r#"{"device_id":"d","device_secret":"s","token":"t","policy":"p"}"#;
        let claimed = claim_result(reply(200, body)).expect("claimed");
        assert_eq!((claimed.device_id.as_str(), claimed.device_secret.as_str()), ("d", "s"));
        assert_eq!((claimed.token.as_str(), claimed.policy.as_str()), ("t", "p"));
        assert_eq!(claim_result(reply(400, r#"{"error":"invalid_code"}"#)), Err(ClientError::InvalidCode));
        assert_eq!(claim_result(reply(500, "")), Err(ClientError::Unavailable));
        assert_eq!(claim_result(reply(200, r#"{"device_id":"d"}"#)), Err(ClientError::Unavailable));
        assert_eq!(claim_result(reply(200, "not json")), Err(ClientError::Unavailable));
        assert_eq!(claim_result(None), Err(ClientError::Unavailable));
    }

    #[test]
    fn a_check_maps_200_401_410_and_everything_else() {
        let renewed = check_result(reply(200, r#"{"token":"t","policy":"p"}"#), true);
        assert_eq!(renewed, CheckOutcome::Renewed { token: Some("t".into()), policy: "p".into() });
        assert_eq!(check_result(reply(401, r#"{"error":"unknown_device"}"#), true), CheckOutcome::UnknownDevice);
        let gone = r#"{"error":"signed_out","reason":"replaced"}"#;
        assert_eq!(check_result(reply(410, gone), true), CheckOutcome::SignedOut);
        assert_eq!(check_result(reply(429, ""), true), CheckOutcome::Failed);
        assert_eq!(check_result(reply(200, r#"{"policy":"p"}"#), true), CheckOutcome::Failed);
        assert_eq!(check_result(None, true), CheckOutcome::Failed);
    }

    #[test]
    fn a_policy_fetch_needs_no_token() {
        let renewed = check_result(reply(200, r#"{"policy":"p"}"#), false);
        assert_eq!(renewed, CheckOutcome::Renewed { token: None, policy: "p".into() });
        assert_eq!(check_result(reply(200, "{}"), false), CheckOutcome::Failed);
    }
}
