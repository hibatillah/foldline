use windows::Data::Json::{IJsonValue, JsonObject, JsonValueType};
use windows::core::HSTRING;

/// Raw grants as written in a policy: status name, then resource, then its actions.
pub type RawGrants = Vec<(String, Vec<(String, Vec<String>)>)>;

#[derive(Debug, Default, PartialEq)]
pub struct RawLaunch {
    pub active: bool,
    pub ends_at: Option<String>,
}

/// Every payload field as plain data; the checks on its meaning live in `payload.rs`.
#[derive(Debug, Default, PartialEq)]
pub struct RawPayload {
    pub version: i64,
    pub kind: String,
    pub kid: String,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub device_hash: Option<String>,
    pub plan: Option<String>,
    pub trial_ends_at: Option<String>,
    pub email: Option<String>,
    pub issued_at: Option<String>,
    pub launch: Option<RawLaunch>,
    /// `None` when absent or malformed, so a bad table never costs the launch.
    pub grants: Option<RawGrants>,
}

/// `None` when the text is not a JSON object or a field has the wrong type; a null counts as the wrong type, except `grants`, which then reads as absent.
pub fn parse_payload(text: &str) -> Option<RawPayload> {
    let object = JsonObject::Parse(&HSTRING::from(text)).ok()?;
    let launch = match value_of(&object, "launch")? {
        None => None,
        Some(value) => Some(read_launch(&value.GetObject().ok()?)?),
    };
    Some(RawPayload {
        version: read_version(&object)?,
        kind: optional_string(&object, "kind")??,
        kid: optional_string(&object, "kid")??,
        user_id: optional_string(&object, "user_id")?,
        device_id: optional_string(&object, "device_id")?,
        device_hash: optional_string(&object, "device_hash")?,
        plan: optional_string(&object, "plan")?,
        trial_ends_at: optional_string(&object, "trial_ends_at")?,
        email: optional_string(&object, "email")?,
        issued_at: optional_string(&object, "issued_at")?,
        launch,
        grants: read_grants(&object),
    })
}

fn read_launch(object: &JsonObject) -> Option<RawLaunch> {
    let active = object.GetNamedBoolean(&HSTRING::from("active")).ok()?;
    Some(RawLaunch { active, ends_at: optional_string(object, "ends_at")? })
}

/// `Some(None)` is an absent key; `None` is a value that could not be read.
fn value_of(object: &JsonObject, key: &str) -> Option<Option<IJsonValue>> {
    let key = HSTRING::from(key);
    if !object.HasKey(&key).ok()? {
        return Some(None);
    }
    Some(Some(object.GetNamedValue(&key).ok()?.into()))
}

/// Outer `None` is a wrong type; inner `None` is an absent key.
fn optional_string(object: &JsonObject, key: &str) -> Option<Option<String>> {
    let Some(value) = value_of(object, key)? else {
        return Some(None);
    };
    if value.ValueType().ok()? != JsonValueType::String {
        return None;
    }
    Some(Some(value.GetString().ok()?.to_string()))
}

fn read_grants(object: &JsonObject) -> Option<RawGrants> {
    let table = value_of(object, "grants").flatten()?.GetObject().ok()?;
    let mut grants = RawGrants::new();
    for status_entry in table {
        let resources = status_entry.Value().ok()?.GetObject().ok()?;
        let mut resource_list = Vec::new();
        for resource_entry in resources {
            let mut actions = Vec::new();
            for action in resource_entry.Value().ok()?.GetArray().ok()? {
                actions.push(action.GetString().ok()?.to_string());
            }
            resource_list.push((resource_entry.Key().ok()?.to_string(), actions));
        }
        grants.push((status_entry.Key().ok()?.to_string(), resource_list));
    }
    Some(grants)
}

/// Only a whole number counts, so `1.9` is not version 1.
fn read_version(object: &JsonObject) -> Option<i64> {
    let number = object.GetNamedNumber(&HSTRING::from("v")).ok()?;
    (number.fract() == 0.0).then_some(number as i64)
}

/// The string fields of a top-level JSON object in no fixed order (other types are skipped); `None` when the text is not an object.
pub fn string_fields(text: &str) -> Option<Vec<(String, String)>> {
    let object = JsonObject::Parse(&HSTRING::from(text)).ok()?;
    let mut fields = Vec::new();
    for entry in object.First().ok()? {
        let value = entry.Value().ok()?;
        if value.ValueType().ok()? == JsonValueType::String {
            fields.push((entry.Key().ok()?.to_string(), value.GetString().ok()?.to_string()));
        }
    }
    Some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_fields_skip_other_types_and_refuse_non_objects() {
        let mut fields = string_fields(r#"{"a":"x","n":1,"b":"y","o":{"c":"z"}}"#).expect("object");
        fields.sort();
        assert_eq!(fields, vec![("a".to_owned(), "x".to_owned()), ("b".to_owned(), "y".to_owned())]);
        assert_eq!(string_fields("[1]"), None);
        assert_eq!(string_fields("nope"), None);
    }

    const BASE: &str = r#""kind":"device","kid":"test-1""#;

    #[test]
    fn a_fractional_version_is_refused() {
        assert!(parse_payload(&format!(r#"{{"v":1,{BASE}}}"#)).is_some());
        assert_eq!(parse_payload(&format!(r#"{{"v":1.9,{BASE}}}"#)), None);
    }

    #[test]
    fn a_null_field_refuses_the_payload() {
        assert_eq!(parse_payload(&format!(r#"{{"v":1,{BASE},"email":null}}"#)), None);
        assert_eq!(parse_payload(r#"{"v":1,"kind":null,"kid":"test-1"}"#), None);
        let null_grants = parse_payload(&format!(r#"{{"v":1,{BASE},"grants":null}}"#));
        assert_eq!(null_grants.map(|payload| payload.grants), Some(None));
    }
}
