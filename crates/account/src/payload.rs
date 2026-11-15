use crate::base64url;
use crate::grants::{ServerGrants, validate};
use crate::json::{self, RawGrants, RawPayload};
use crate::keys;
use crate::signature;
use crate::status::{LaunchFacts, Plan, Status, TokenFacts};

const SUPPORTED_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceToken {
    pub user_id: String,
    pub device_id: String,
    pub device_hash: String,
    pub email: String,
    pub plan: Plan,
    pub trial_ends_at: Option<i64>,
    pub issued_at: i64,
}

impl DeviceToken {
    pub fn facts(&self) -> TokenFacts {
        TokenFacts { plan: self.plan, trial_ends_at: self.trial_ends_at }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub launch: LaunchFacts,
    pub grants: Option<ServerGrants>,
    pub issued_at: i64,
}

/// A signed device token that names this device; anything else is `None`.
pub fn read_token(text: &str, this_device_hash: &str) -> Option<DeviceToken> {
    let payload = read_signed(text, "device")?;
    if payload.device_hash.as_deref()? != this_device_hash {
        return None;
    }
    let plan = match payload.plan.as_deref()? {
        "member" => Plan::Member,
        "trial" => Plan::Trial,
        "paid" => Plan::Paid,
        _ => return None,
    };
    let trial_ends_at = match payload.trial_ends_at {
        Some(text) => Some(parse_rfc3339(&text)?),
        None => None,
    };
    Some(DeviceToken {
        user_id: payload.user_id?,
        device_id: payload.device_id?,
        device_hash: payload.device_hash?,
        email: payload.email?,
        plan,
        trial_ends_at,
        issued_at: parse_rfc3339(&payload.issued_at?)?,
    })
}

/// A signed policy; a table of grants that fails to read or validate is dropped alone.
pub fn read_policy(text: &str) -> Option<Policy> {
    build_policy(read_signed(text, "policy")?)
}

fn build_policy(payload: RawPayload) -> Option<Policy> {
    let launch = payload.launch?;
    let ends_at = match launch.ends_at {
        Some(text) => Some(parse_rfc3339(&text)?),
        None => None,
    };
    let grants = payload.grants.and_then(server_grants_from).filter(|grants| validate(grants).is_ok());
    Some(Policy {
        launch: LaunchFacts { active: launch.active, ends_at },
        grants,
        issued_at: parse_rfc3339(&payload.issued_at?)?,
    })
}

fn read_signed(text: &str, expected_kind: &str) -> Option<RawPayload> {
    let (payload_part, signature_part) = text.split_once('.')?;
    if signature_part.contains('.') {
        return None;
    }
    let signature_bytes: [u8; 64] = base64url::decode(signature_part)?.try_into().ok()?;
    let payload_text = String::from_utf8(base64url::decode(payload_part)?).ok()?;
    let payload = json::parse_payload(&payload_text)?;
    if payload.version != SUPPORTED_VERSION || payload.kind != expected_kind {
        return None;
    }
    let public_key = keys::public_key(&payload.kid)?;
    signature::verify(&public_key, payload_part.as_bytes(), &signature_bytes).then_some(payload)
}

fn server_grants_from(raw: RawGrants) -> Option<ServerGrants> {
    raw.into_iter()
        .map(|(name, resources)| {
            let status = match name.as_str() {
                "guest" => Status::Guest,
                "member" => Status::Member,
                "trial" => Status::Trial,
                "paid" => Status::Paid,
                _ => return None,
            };
            Some((status, resources))
        })
        .collect()
}

/// Only `YYYY-MM-DDTHH:MM:SSZ` is accepted.
fn parse_rfc3339(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() != 20 || !text.is_ascii() {
        return None;
    }
    let separators = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':'), (19, b'Z')];
    if separators.iter().any(|&(index, character)| bytes[index] != character) {
        return None;
    }
    let number = |range: std::ops::Range<usize>| -> Option<i64> {
        let digits = &text[range];
        if digits.bytes().all(|byte| byte.is_ascii_digit()) { digits.parse().ok() } else { None }
    };
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 in the proleptic Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::RawLaunch;

    const DEVICE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ISSUED_AT: i64 = 1_794_528_000; // 2026-11-13T00:00:00Z

    fn valid_token() -> &'static str {
        include_str!("samples/token-valid.txt").trim()
    }

    #[test]
    fn the_sample_token_verifies_for_its_device() {
        let token = read_token(valid_token(), DEVICE_HASH).expect("sample token reads");
        assert_eq!(token.plan, Plan::Paid);
        assert_eq!(token.user_id, "user-1");
        assert_eq!(token.device_id, "device-1");
        assert_eq!(token.email, "sample@example.com");
        assert_eq!(token.issued_at, ISSUED_AT);
        assert_eq!(token.facts(), TokenFacts { plan: Plan::Paid, trial_ends_at: None });
    }

    #[test]
    fn a_tampered_token_a_wrong_kid_and_a_wrong_kind_are_refused() {
        for sample in [
            include_str!("samples/token-tampered.txt"),
            include_str!("samples/token-wrong-kid.txt"),
            include_str!("samples/token-wrong-kind.txt"),
        ] {
            assert_eq!(read_token(sample.trim(), DEVICE_HASH), None);
        }
    }

    #[test]
    fn a_token_for_another_device_is_refused() {
        assert_eq!(read_token(valid_token(), &"b".repeat(64)), None);
    }

    #[test]
    fn the_expired_trial_sample_reads_with_its_end_time() {
        let sample = include_str!("samples/token-trial-expired.txt").trim();
        let token = read_token(sample, DEVICE_HASH).expect("trial sample reads");
        assert_eq!(token.plan, Plan::Trial);
        assert_eq!(token.trial_ends_at, Some(1_767_830_400)); // 2026-01-08T00:00:00Z
    }

    #[test]
    fn launch_open_and_launch_ended_policies_read_their_end() {
        let open = read_policy(include_str!("samples/policy-launch-open.txt").trim()).expect("open policy");
        assert_eq!(open.launch, LaunchFacts { active: true, ends_at: None });
        assert_eq!(open.grants, None);
        assert_eq!(open.issued_at, ISSUED_AT);
        let ended = read_policy(include_str!("samples/policy-launch-ended.txt").trim()).expect("ended policy");
        assert_eq!(ended.launch, LaunchFacts { active: false, ends_at: Some(1_767_225_600) }); // 2026-01-01
    }

    #[test]
    fn the_sample_policy_with_grants_reads_its_table() {
        let policy = read_policy(include_str!("samples/policy-with-grants.txt").trim()).expect("policy reads");
        assert_eq!(policy.launch, LaunchFacts { active: false, ends_at: None });
        let grants = policy.grants.expect("grants read");
        assert_eq!(grants.len(), 2);
        let guest = grants.iter().find(|(status, _)| *status == Status::Guest).expect("guest table");
        let activity = guest.1.iter().find(|(resource, _)| resource == "activity").expect("activity resource");
        assert_eq!(activity.1, vec!["volume".to_string()]);
        let member = grants.iter().find(|(status, _)| *status == Status::Member).expect("member table");
        let media = member.1.iter().find(|(resource, _)| resource == "media").expect("media resource");
        assert!(media.1.contains(&"lyrics".to_string()));
    }

    #[test]
    fn a_policy_with_invalid_grants_keeps_its_launch() {
        let raw_policy = |grants: RawGrants| RawPayload {
            launch: Some(RawLaunch { active: true, ends_at: None }),
            grants: Some(grants),
            issued_at: Some("2026-11-13T00:00:00Z".to_string()),
            ..RawPayload::default()
        };
        // A guest table without any page, and an unknown status key.
        let emptied = ["media", "calendar", "shelf"].map(|name| (name.to_string(), vec![])).to_vec();
        let no_guest_page = vec![("guest".to_string(), emptied)];
        let unknown_status = vec![("vip".to_string(), vec![])];
        for grants in [no_guest_page, unknown_status] {
            let policy = build_policy(raw_policy(grants)).expect("launch survives");
            assert_eq!(policy.launch, LaunchFacts { active: true, ends_at: None });
            assert_eq!(policy.grants, None);
        }
    }

    #[test]
    fn a_malformed_text_is_refused() {
        for text in ["", ".", "a.b.c", "!!!.???", "e30.e30"] {
            assert_eq!(read_token(text, DEVICE_HASH), None);
            assert_eq!(read_policy(text), None);
        }
    }

    #[test]
    fn only_the_utc_second_precision_form_of_rfc_3339_is_read() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339("2026-11-13T00:00:00Z"), Some(ISSUED_AT));
        assert_eq!(parse_rfc3339("2024-02-29T12:00:00Z"), Some(1_709_208_000));
        for text in
            ["2026-11-13T00:00:00+00:00", "2026-11-13 00:00:00Z", "2026-02-30T00:00:00Z", "2026-11-13T24:00:00Z"]
        {
            assert_eq!(parse_rfc3339(text), None);
        }
    }

    #[test]
    fn a_signature_one_byte_long_or_short_is_refused() {
        let (payload_part, signature_part) = valid_token().split_once('.').expect("sample has two parts");
        let signature = base64url::decode(signature_part).expect("sample signature decodes");
        let mut longer = signature.clone();
        longer.push(0);
        let shorter = &signature[..signature.len() - 1];
        for changed in [longer.as_slice(), shorter] {
            let text = format!("{payload_part}.{}", base64url::encode(changed));
            assert_eq!(read_token(&text, DEVICE_HASH), None);
        }
    }

    #[test]
    fn a_policy_is_refused_as_a_token_and_a_token_as_a_policy() {
        assert_eq!(read_token(include_str!("samples/policy-launch-open.txt").trim(), DEVICE_HASH), None);
        assert_eq!(read_policy(valid_token()), None);
    }

    #[test]
    fn a_token_with_padding_or_plus_characters_is_refused() {
        for suffix in ["=", "+"] {
            assert_eq!(read_token(&format!("{}{suffix}", valid_token()), DEVICE_HASH), None);
        }
    }

    #[test]
    fn month_13_and_day_32_are_refused() {
        assert_eq!(parse_rfc3339("2026-13-01T00:00:00Z"), None);
        assert_eq!(parse_rfc3339("2026-01-32T00:00:00Z"), None);
    }
}
