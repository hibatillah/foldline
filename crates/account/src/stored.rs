use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::access::Access;
use crate::grants::Grants;
use crate::payload::{read_policy, read_token};
use crate::status::{Status, effective_status};
use crate::{base64url, keys, protect};

/// What survives a restart: the device identity, the last signed token and policy, and the last check time.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct StoredAccount {
    pub device_id: Option<String>,
    pub device_secret: Option<String>,
    pub token: Option<String>,
    pub policy: Option<String>,
    pub last_check_at: Option<i64>,
}

impl fmt::Debug for StoredAccount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StoredAccount")
            .field("device_id", &self.device_id)
            .field("device_secret", &self.device_secret.as_ref().map(|_| "<redacted>"))
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("policy", &self.policy)
            .field("last_check_at", &self.last_check_at)
            .finish()
    }
}

/// `%APPDATA%\notch\account.txt`; without `APPDATA` the path is relative to the working directory.
pub fn account_path() -> PathBuf {
    let directory = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    directory.join("notch").join("account.txt")
}

impl StoredAccount {
    /// A missing, unreadable or corrupt file reads as a signed-out account.
    pub fn load(path: &Path) -> StoredAccount {
        fs::read_to_string(path).map(|text| parse(&text)).unwrap_or_default()
    }

    /// Writes a `.tmp` beside the file, then renames it over; the secret is written DPAPI-protected.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)?;
        }
        let mut temporary_name = path.as_os_str().to_owned();
        temporary_name.push(".tmp");
        let temporary_path = PathBuf::from(temporary_name);
        let written = self.to_text().and_then(|text| {
            let mut file = fs::File::create(&temporary_path)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary_path, path)
        });
        if written.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        written
    }

    pub fn access(&self, device_hash: &str, now_unix: i64) -> Access {
        self.access_with(keys::is_enforced(), device_hash, now_unix)
    }

    /// Without an enforced key nothing can be verified, so everything opens as in the launch period.
    fn access_with(&self, is_enforced: bool, device_hash: &str, now_unix: i64) -> Access {
        if !is_enforced {
            return Access { status: Status::Launch, grants: Grants::built_in() };
        }
        let token = self.token.as_deref().and_then(|text| read_token(text, device_hash));
        let policy = self.policy.as_deref().and_then(read_policy);
        let grants = match policy.as_ref().and_then(|policy| policy.grants.as_ref()) {
            Some(server) => Grants::built_in().merged(server),
            None => Grants::built_in(),
        };
        let status = effective_status(
            token.as_ref().map(|token| token.facts()).as_ref(),
            policy.as_ref().map(|policy| &policy.launch),
            now_unix,
        );
        Access { status, grants }
    }

    fn to_text(&self) -> std::io::Result<String> {
        let has_line_break = [&self.device_id, &self.token, &self.policy]
            .into_iter()
            .flatten()
            .any(|value| value.contains(['\r', '\n']));
        if has_line_break {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "a value contains a line break"));
        }
        let mut text = String::new();
        let mut add_line = |key: &str, value: Option<&str>| {
            if let Some(value) = value {
                text.push_str(&format!("{key}={value}\n"));
            }
        };
        add_line("device_id", self.device_id.as_deref());
        add_line("token", self.token.as_deref());
        add_line("policy", self.policy.as_deref());
        add_line("last_check_at", self.last_check_at.map(|moment| moment.to_string()).as_deref());
        if let Some(secret) = &self.device_secret {
            let protected = protect::protect(secret.as_bytes())
                .ok_or_else(|| std::io::Error::other("could not protect the device secret"))?;
            add_line("device_secret", Some(&base64url::encode(&protected)));
        }
        Ok(text)
    }
}

fn parse(text: &str) -> StoredAccount {
    let mut account = StoredAccount::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key {
            "device_id" => account.device_id = Some(value.to_owned()),
            "device_secret" => account.device_secret = unprotect_secret(value),
            "token" => account.token = Some(value.to_owned()),
            "policy" => account.policy = Some(value.to_owned()),
            "last_check_at" => account.last_check_at = value.parse().ok(),
            _ => {}
        }
    }
    // A secret that cannot be unprotected (another Windows user, a copied file) leaves no usable device.
    if account.device_id.is_some() != account.device_secret.is_some() {
        account.device_id = None;
        account.device_secret = None;
        account.token = None;
    }
    account
}

fn unprotect_secret(text: &str) -> Option<String> {
    String::from_utf8(protect::unprotect(&base64url::decode(text)?)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn temporary_path(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("notch-account-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        directory.join("account.txt")
    }

    fn full_account() -> StoredAccount {
        StoredAccount {
            device_id: Some("device-1".into()),
            device_secret: Some("super-secret-value".into()),
            token: Some("payload.signature".into()),
            policy: Some("policy.signature".into()),
            last_check_at: Some(NOW),
        }
    }

    #[test]
    fn a_corrupt_account_file_reads_as_signed_out() {
        let path = temporary_path("corrupt");
        let hash = "b".repeat(64);
        assert_eq!(StoredAccount::load(&path).access_with(true, &hash, NOW).status, Status::Guest);

        fs::create_dir_all(path.parent().expect("has parent")).expect("directory");
        for content in ["", "garbage without equals\n===\n", "token=not.a.token\npolicy=nope\ndevice_secret=@@@\n"] {
            fs::write(&path, content).expect("write");
            assert_eq!(StoredAccount::load(&path).access_with(true, &hash, NOW).status, Status::Guest);
        }

        let valid_token = include_str!("samples/token-valid.txt").trim();
        fs::write(&path, format!("token={valid_token}\n")).expect("write");
        let account = StoredAccount::load(&path);
        assert_eq!(account.access_with(true, &hash, NOW).status, Status::Guest);
        assert_ne!(account.access_with(true, &"a".repeat(64), NOW).status, Status::Guest);
    }

    #[test]
    fn saving_and_loading_keeps_every_field() {
        let path = temporary_path("round-trip");
        full_account().save(&path).expect("save");
        assert_eq!(StoredAccount::load(&path), full_account());
        assert!(!path.with_extension("txt.tmp").exists());
    }

    #[test]
    fn debug_output_never_shows_the_secret() {
        let shown = format!("{:?}", full_account());
        assert!(!shown.contains("super-secret-value"));
        assert!(!shown.contains("payload.signature"), "the token is redacted too");
        assert!(shown.contains("<redacted>"));
        assert!(shown.contains("device-1"));
    }

    #[test]
    fn a_secret_that_cannot_be_unprotected_leaves_a_clean_guest() {
        let path = temporary_path("lost-secret");
        fs::create_dir_all(path.parent().expect("has parent")).expect("directory");
        let text = "device_id=device-1\ndevice_secret=AAAA\ntoken=t.s\npolicy=p.s\nlast_check_at=5\n";
        fs::write(&path, text).expect("write");
        let account = StoredAccount::load(&path);
        assert_eq!((account.device_id, account.device_secret, account.token), (None, None, None));
        assert_eq!(account.policy.as_deref(), Some("p.s"));
    }

    #[test]
    fn a_value_with_a_line_break_is_not_saved_and_leaves_no_temporary_file() {
        let path = temporary_path("line-break");
        let mut account = full_account();
        account.token = Some("a.b\nsecret=injected".into());
        assert!(account.save(&path).is_err());
        assert!(!path.exists());
        assert!(!path.with_extension("txt.tmp").exists());
    }

    #[test]
    fn saving_again_replaces_the_earlier_file() {
        let path = temporary_path("replace");
        full_account().save(&path).expect("save");
        StoredAccount::default().save(&path).expect("save");
        assert_eq!(StoredAccount::load(&path), StoredAccount::default());
    }

    #[test]
    fn the_secret_is_not_written_in_plain_text() {
        let path = temporary_path("plain");
        full_account().save(&path).expect("save");
        let text = fs::read_to_string(&path).expect("read");
        assert!(!text.contains("super-secret-value"));
        assert!(text.contains("device_secret="));
    }

    #[test]
    fn an_unenforced_build_opens_everything() {
        let account = StoredAccount::default();
        let access = account.access_with(false, &"b".repeat(64), NOW);
        assert_eq!(access.status, Status::Launch);
        assert!(access.allows_page("weather"));
        assert_eq!(account.access_with(true, &"b".repeat(64), NOW).status, Status::Guest);
    }

    #[test]
    fn a_signed_token_and_policy_give_the_status_and_merged_grants() {
        let account = StoredAccount {
            token: Some(include_str!("samples/token-valid.txt").trim().to_owned()),
            policy: Some(include_str!("samples/policy-with-grants.txt").trim().to_owned()),
            ..StoredAccount::default()
        };
        let access = account.access_with(true, &"a".repeat(64), NOW);
        assert_ne!(access.status, Status::Guest);
        assert!(access.allows_page("media"));
    }
}
