//! Downloads the latest release of Foldline from GitHub and reads its tag.

use windows::{
    Data::Json::{JsonObject, JsonValueType},
    Foundation::Uri,
    Web::Http::HttpClient,
    core::HSTRING,
};

use crate::{
    links::{LATEST_RELEASE_API_URL, REPOSITORY_URL},
    version::displayed_version,
};

/// The site must be a comment in parentheses: a bare `github.com/...` is not a product token, so `HttpClient` drops
/// the header without an error and sends its own, and GitHub rejects a request without a User-Agent.
fn user_agent(version: &str) -> String {
    format!("Foldline/{version} (+{REPOSITORY_URL})")
}

/// Why no tag came back: the network is unreachable, the service refused or answered with an error status, or the
/// body held no tag. The strings are reasons for the log.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Failure {
    Network(String),
    Service(String),
    Unreadable,
}

/// `None` for a success status, otherwise the reason the service refused.
fn status_problem(status: i32) -> Option<String> {
    match status {
        200..=299 => None,
        403 | 429 => Some(format!("{status}: rate limited or refused")),
        404 => Some("404: no release published".to_owned()),
        other => Some(format!("status {other}")),
    }
}

fn tag_from_body(body: &str) -> Result<String, Failure> {
    let release = JsonObject::Parse(&HSTRING::from(body)).map_err(|_| Failure::Unreadable)?;
    let tag = release.GetNamedValue(&HSTRING::from("tag_name")).map_err(|_| Failure::Unreadable)?;
    if tag.ValueType().map_err(|_| Failure::Unreadable)? != JsonValueType::String {
        return Err(Failure::Unreadable);
    }
    let tag = tag.GetString().map_err(|_| Failure::Unreadable)?.to_string();
    if tag.is_empty() { Err(Failure::Unreadable) } else { Ok(tag) }
}

/// Blocking: asks GitHub for the latest release and returns its `tag_name`.
pub(super) fn latest_tag() -> Result<String, Failure> {
    let network = |error: windows::core::Error| Failure::Network(error.message());
    let client = HttpClient::new().map_err(network)?;
    let headers = client.DefaultRequestHeaders().map_err(network)?;
    let agent = HSTRING::from(user_agent(displayed_version()));
    headers.UserAgent().and_then(|headers| headers.TryParseAdd(&agent)).map_err(network)?;
    headers
        .Accept()
        .and_then(|accept| accept.TryParseAdd(&HSTRING::from("application/vnd.github+json")))
        .map_err(network)?;
    let uri = Uri::CreateUri(&HSTRING::from(LATEST_RELEASE_API_URL)).map_err(network)?;
    let response = client.GetAsync(&uri).map_err(network)?.join().map_err(network)?;
    let status = response.StatusCode().map_err(network)?.0;
    if let Some(problem) = status_problem(status) {
        return Err(Failure::Service(problem));
    }
    let text = response.Content().and_then(|content| content.ReadAsStringAsync()).map_err(network)?;
    tag_from_body(&text.join().map_err(network)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_agent_names_the_product_version_and_site_in_parentheses() {
        let agent = user_agent("0.6.0-alpha.2");
        assert!(agent.starts_with("Foldline/"), "{agent}");
        assert!(agent.contains("0.6.0-alpha.2"), "{agent}");
        assert!(agent.contains("(+https://github.com/"), "{agent}");
    }

    #[test]
    fn a_release_body_gives_its_tag() {
        assert_eq!(tag_from_body(r#"{"tag_name":"v0.7.0","html_url":"x"}"#), Ok("v0.7.0".to_owned()));
    }

    #[test]
    fn a_body_without_a_tag_is_unreadable() {
        for body in [r#"{"name":"x"}"#, r#"{"tag_name":null}"#, "[]", "not json"] {
            assert_eq!(tag_from_body(body), Err(Failure::Unreadable), "body: {body}");
        }
    }

    #[test]
    fn a_rate_limit_or_a_missing_release_is_a_problem() {
        assert_eq!(status_problem(200), None);
        for status in [403, 404, 429, 500] {
            let problem = status_problem(status).expect("an error status is a problem");
            assert!(problem.contains(&status.to_string()), "{problem}");
        }
    }
}
