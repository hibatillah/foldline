//! Looks a text up in the language of `store::language::current()`; the texts are in `locales/app.yml`.

use gpui::App;
use store::language::{LANGUAGE_KEY, LanguageChoice};

/// `tr!("general.title")` is a `&'static str` borrowed from the compiled table, so it fits every `&'static str` title and
/// description the window builds; `tr!("key", n = 3)` builds a `String` for a text with a value in it.
macro_rules! tr {
    ($key:literal) => {
        $crate::i18n::text($key)
    };
    ($key:literal, $($name:ident = $value:expr),+ $(,)?) => {
        rust_i18n::t!($key, locale = store::language::current().code(), $($name = $value),+).into_owned()
    };
}
pub(crate) use tr;

/// A found key is borrowed from the static table; a missing one is leaked once (a bug the parity test catches).
pub fn text(key: &'static str) -> &'static str {
    match rust_i18n::t!(key, locale = store::language::current().code()) {
        std::borrow::Cow::Borrowed(text) => text,
        std::borrow::Cow::Owned(text) => Box::leak(text.into_boxed_str()),
    }
}

/// Saves the choice, tells the notch, makes it the language of this window and of gpui-component's own text, and repaints.
pub fn apply_language(choice: LanguageChoice, cx: &mut App) {
    crate::field::write(LANGUAGE_KEY, choice.setting_word(), cx);
    let language = store::language::apply_setting(&store::Settings::load());
    rust_i18n::set_locale(language.code());
    cx.refresh_windows();
}

/// The key line and its `en` and `id` lines in a locale file; a checkout with CRLF line endings reads the same as LF.
#[cfg(test)]
fn locale_entry(file: &std::path::Path, key: &str) -> Option<String> {
    entry_in(&std::fs::read_to_string(file).ok()?, key)
}

#[cfg(test)]
fn entry_in(text: &str, key: &str) -> Option<String> {
    let header = format!("{key}:");
    let mut lines = text.lines();
    lines.find(|line| *line == header)?;
    Some(std::iter::once(header.as_str()).chain(lines.take(2)).collect::<Vec<_>>().join("\n"))
}

/// Whether a picture's word reads the same here as in the notch, in both languages.
#[cfg(test)]
pub(crate) fn same_in_the_notch(key: &str) -> bool {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let own = locale_entry(&manifest.join("locales/app.yml"), key);
    own.is_some() && own == locale_entry(&manifest.join("../notch/locales/app.yml"), key)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use store::language::{Language, with_override};

    /// Keys built at run time, listed by prefix.
    const DYNAMIC_KEY_PREFIXES: &[&str] = &[];

    #[test]
    fn a_locale_entry_reads_the_same_with_crlf_and_lf_line_endings() {
        let unix = "a:\n  en: \"A\"\n  id: \"B\"\nb:\n  en: \"C\"\n  id: \"D\"\n";
        let windows = unix.replace('\n', "\r\n");
        assert_eq!(super::entry_in(&windows, "b"), super::entry_in(unix, "b"));
        assert_eq!(super::entry_in(unix, "b").as_deref(), Some("b:\n  en: \"C\"\n  id: \"D\""));
        assert_eq!(super::entry_in(unix, "c"), None);
    }

    #[test]
    fn a_locale_entry_that_differs_between_files_is_not_the_same() {
        let first = "a:\n  en: \"A\"\n  id: \"B\"\n";
        let second = "a:\n  en: \"A\"\n  id: \"X\"\n";
        assert_ne!(super::entry_in(first, "a"), super::entry_in(second, "a"));
    }

    #[test]
    fn a_text_follows_the_language_of_its_thread() {
        assert_eq!(tr!("general.title"), "General");
        with_override(Language::Indonesian, || assert_eq!(tr!("general.title"), "Umum"));
    }

    #[test]
    fn a_looked_up_text_is_borrowed_from_the_table_and_never_copied() {
        assert!(std::ptr::eq(tr!("general.title"), tr!("general.title")));
    }

    fn source_files(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                source_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") && !path.ends_with("i18n.rs") {
                files.push(path);
            }
        }
    }

    fn keys_used_in_the_source() -> Vec<String> {
        let mut files = Vec::new();
        source_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
        let mut keys = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(file).unwrap();
            for (start, _) in text.match_indices("tr!(") {
                // `include_str!(..)` and the like end in `tr!(` too; only a bare `tr!` is ours.
                let is_ours =
                    !text[..start].ends_with(|character: char| character.is_alphanumeric() || character == '_');
                let rest = text[start + 4..].trim_start();
                if let (true, Some(quoted)) = (is_ours, rest.strip_prefix('"'))
                    && let Some(end) = quoted.find('"')
                {
                    keys.push(quoted[..end].to_owned());
                }
            }
        }
        keys
    }

    fn keys_in_the_file() -> Vec<String> {
        let text =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("locales").join("app.yml")).unwrap();
        text.lines()
            .filter(|line| !line.starts_with([' ', '_', '#']) && line.ends_with(':'))
            .map(|line| line.trim_end_matches(':').to_owned())
            .collect()
    }

    #[test]
    fn every_key_used_in_the_source_exists_in_both_languages() {
        for key in keys_used_in_the_source() {
            for locale in ["en", "id"] {
                let text = rust_i18n::t!(key.as_str(), locale = locale);
                assert!(text != key.as_str(), "{key} has no {locale} text");
            }
        }
    }

    #[test]
    fn no_key_in_the_file_is_unused() {
        let used = keys_used_in_the_source();
        for key in keys_in_the_file() {
            let by_prefix = DYNAMIC_KEY_PREFIXES.iter().any(|prefix| key.starts_with(prefix));
            assert!(by_prefix || used.contains(&key), "{key} is in app.yml but no tr! uses it");
        }
    }

    #[test]
    fn every_key_in_the_file_has_an_english_and_an_indonesian_line() {
        let text =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("locales").join("app.yml")).unwrap();
        let mut blocks: Vec<(String, String)> = Vec::new();
        for line in text.lines() {
            if !line.starts_with([' ', '_', '#']) && line.ends_with(':') {
                blocks.push((line.trim_end_matches(':').to_owned(), String::new()));
            } else if let Some(block) = blocks.last_mut() {
                block.1.push_str(line);
                block.1.push('\n');
            }
        }
        for (key, body) in blocks {
            assert!(body.contains("\n  en:") || body.starts_with("  en:"), "{key} has no en line");
            assert!(body.contains("\n  id:") || body.starts_with("  id:"), "{key} has no id line");
        }
    }
}
