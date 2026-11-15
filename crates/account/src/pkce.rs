use crate::{base64url, hash, random};

/// One sign-in attempt: the app keeps `verifier`, sends `challenge` and `state` to the site.
#[derive(Clone, PartialEq, Eq)]
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

impl std::fmt::Debug for Pkce {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Pkce")
            .field("verifier", &"<redacted>")
            .field("challenge", &self.challenge)
            .field("state", &self.state)
            .finish()
    }
}

impl Pkce {
    /// `None` only when Windows CNG fails to give random bytes or a hash.
    pub fn new() -> Option<Pkce> {
        let verifier = base64url::encode(&random::random_bytes::<32>()?);
        let challenge = base64url::encode(&hash::sha256(verifier.as_bytes())?);
        let state = base64url::encode(&random::random_bytes::<32>()?);
        Some(Pkce { verifier, challenge, state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pkce_challenge_is_the_sha256_of_the_verifier() {
        let pkce = Pkce::new().expect("cng works");
        let digest = hash::sha256(pkce.verifier.as_bytes()).expect("cng works");
        assert_eq!(pkce.challenge, base64url::encode(&digest));
        assert_eq!(pkce.challenge.len(), 43);
    }

    #[test]
    fn debug_output_never_shows_the_verifier() {
        let pkce = Pkce::new().expect("cng works");
        let shown = format!("{pkce:?}");
        assert!(!shown.contains(&pkce.verifier));
        assert!(shown.contains("<redacted>") && shown.contains(&pkce.challenge));
    }

    #[test]
    fn every_value_is_32_random_bytes_in_base64url() {
        let first = Pkce::new().expect("cng works");
        let second = Pkce::new().expect("cng works");
        assert_eq!(first.verifier.len(), 43);
        assert_eq!(first.state.len(), 43);
        assert_ne!(first.verifier, second.verifier);
        assert_ne!(first.verifier, first.state);
    }
}
