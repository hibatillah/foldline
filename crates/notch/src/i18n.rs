//! Looks a text up in the language of `store::language::current()`; the texts are in `locales/app.yml` and in the split files under `locales/pages/` and `locales/activities/`.

/// `tr!("todo.clear_all")` or `tr!("shelf.files_added", n = 3)`; the result is a `Cow<'static, str>`.
macro_rules! tr {
    ($key:expr $(, $name:ident = $value:expr)* $(,)?) => {
        rust_i18n::t!($key, locale = store::language::current().code() $(, $name = $value)*)
    };
}
pub(crate) use tr;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use store::language::{Language, with_override};

    /// Keys built at run time, listed by prefix; each area adds its own.
    const DYNAMIC_KEY_PREFIXES: &[&str] =
        &["calendar.month.", "calendar.weekday.", "calendar.weekday_letter.", "weather.weekday."];

    #[test]
    fn tr_follows_the_language_of_its_thread() {
        assert_eq!(tr!("tray.settings"), "Settings");
        with_override(Language::Indonesian, || assert_eq!(tr!("tray.settings"), "Pengaturan"));
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

    /// Every `*.yml` under `locales/`, in any depth: `app.yml` and the split files in `pages/` and `activities/`.
    fn locale_files(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                locale_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "yml") {
                files.push(path);
            }
        }
        files.sort();
    }

    fn all_locale_files() -> Vec<PathBuf> {
        let mut files = Vec::new();
        locale_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("locales"), &mut files);
        files
    }

    /// One `(file, key, body)` per top-level key of every locale file; `body` holds the lines under the key.
    fn key_blocks() -> Vec<(PathBuf, String, String)> {
        let mut blocks: Vec<(PathBuf, String, String)> = Vec::new();
        for file in all_locale_files() {
            let text = std::fs::read_to_string(&file).unwrap();
            let first_block = blocks.len();
            for line in text.lines() {
                if !line.starts_with([' ', '_', '#']) && line.ends_with(':') {
                    blocks.push((file.clone(), line.trim_end_matches(':').to_owned(), String::new()));
                } else if blocks.len() > first_block
                    && let Some(block) = blocks.last_mut()
                {
                    block.2.push_str(line);
                    block.2.push('\n');
                }
            }
        }
        blocks
    }

    fn keys_in_the_files() -> Vec<String> {
        key_blocks().into_iter().map(|(_, key, _)| key).collect()
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
        for key in keys_in_the_files() {
            let by_prefix = DYNAMIC_KEY_PREFIXES.iter().any(|prefix| key.starts_with(prefix));
            assert!(by_prefix || used.contains(&key), "{key} is in a locale file but no tr! uses it");
        }
    }

    #[test]
    fn every_key_in_the_files_has_an_english_and_an_indonesian_line() {
        for (file, key, body) in key_blocks() {
            let name = file.display();
            assert!(body.contains("\n  en:") || body.starts_with("  en:"), "{key} in {name} has no en line");
            assert!(body.contains("\n  id:") || body.starts_with("  id:"), "{key} in {name} has no id line");
        }
    }

    #[test]
    fn a_key_is_defined_in_one_locale_file_only() {
        let mut seen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
        for (file, key, _) in key_blocks() {
            if let Some(first) = seen.insert(key.clone(), file.clone()) {
                panic!("{key} is in both {} and {}", first.display(), file.display());
            }
        }
    }

    #[test]
    fn every_locale_file_starts_with_the_version_line() {
        for file in all_locale_files() {
            let text = std::fs::read_to_string(&file).unwrap();
            assert_eq!(text.lines().next(), Some("_version: 2"), "{} needs `_version: 2` first", file.display());
        }
    }
}
