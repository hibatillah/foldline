//! Looks a text up in the language of `store::language::current()`; the texts are in `locales/app.yml`.

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
