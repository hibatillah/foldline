//! The language of the notch and the settings window: the `general.language` setting, the Windows display language
//! and the value both programs read while they draw text.

use std::{
    cell::Cell,
    sync::atomic::{AtomicU8, Ordering},
};

use crate::Settings;

pub const LANGUAGE_KEY: &str = "general.language";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Indonesian,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Indonesian => "id",
        }
    }

    /// `id`, `id-ID` and `id_ID` in any case are Indonesian; every other tag, an empty one included, is English.
    pub fn from_tag(tag: &str) -> Language {
        let lower = tag.to_ascii_lowercase();
        match lower.strip_prefix("id") {
            Some("") => Language::Indonesian,
            Some(rest) if rest.starts_with(['-', '_']) => Language::Indonesian,
            _ => Language::English,
        }
    }

    fn to_byte(self) -> u8 {
        self as u8
    }

    fn from_byte(byte: u8) -> Language {
        if byte == Language::Indonesian as u8 { Language::Indonesian } else { Language::English }
    }
}

/// What the user picked: follow Windows, or one of the two languages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LanguageChoice {
    #[default]
    Auto,
    English,
    Indonesian,
}

impl LanguageChoice {
    /// A missing or unknown word reads as `Auto`.
    pub fn from_setting(text: Option<&str>) -> LanguageChoice {
        match text {
            Some("en") => LanguageChoice::English,
            Some("id") => LanguageChoice::Indonesian,
            _ => LanguageChoice::Auto,
        }
    }

    pub fn setting_word(self) -> &'static str {
        match self {
            LanguageChoice::Auto => "auto",
            LanguageChoice::English => "en",
            LanguageChoice::Indonesian => "id",
        }
    }

    pub fn resolve(self, system: Language) -> Language {
        match self {
            LanguageChoice::Auto => system,
            LanguageChoice::English => Language::English,
            LanguageChoice::Indonesian => Language::Indonesian,
        }
    }
}

/// Only the first preferred language counts; an empty list is English.
fn first_language_of(tags: &[&str]) -> Language {
    tags.first().map_or(Language::English, |tag| Language::from_tag(tag))
}

/// The Windows display language: the first of the user's preferred UI languages; any failure is English.
pub fn system_language() -> Language {
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
    use windows::core::PWSTR;

    let mut count = 0u32;
    let mut length = 0u32;
    // SAFETY: a null buffer asks only for the size of the multi-string.
    if unsafe { GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, None, &mut length) }.is_err() || length == 0
    {
        return Language::English;
    }
    let mut buffer = vec![0u16; length as usize];
    // SAFETY: the buffer holds `length` UTF-16 units, the size Windows asked for.
    if unsafe {
        GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, Some(PWSTR(buffer.as_mut_ptr())), &mut length)
    }
    .is_err()
    {
        return Language::English;
    }
    let first_end = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
    first_language_of(&[String::from_utf16_lossy(&buffer[..first_end]).as_str()])
}

static PROCESS_LANGUAGE: AtomicU8 = AtomicU8::new(0);

thread_local! {
    static OVERRIDE: Cell<Option<Language>> = const { Cell::new(None) };
}

/// The language texts are drawn in now: this thread's override when a test set one, else the process-wide value.
pub fn current() -> Language {
    OVERRIDE.with(Cell::get).unwrap_or_else(|| Language::from_byte(PROCESS_LANGUAGE.load(Ordering::Relaxed)))
}

/// Sets the process language; on a thread with an override (a test) it sets that override instead, so a test that
/// switches language never changes the text under the tests running beside it.
pub fn set_current(language: Language) {
    OVERRIDE.with(|slot| {
        if slot.get().is_some() {
            slot.set(Some(language));
        } else {
            PROCESS_LANGUAGE.store(language.to_byte(), Ordering::Relaxed);
        }
    });
}

/// Reads `general.language`, resolves it against the Windows language, makes it the process language and returns it.
pub fn apply_setting(settings: &Settings) -> Language {
    let language = resolve_setting(settings, system_language());
    set_current(language);
    language
}

/// What `general.language` means given the Windows display language `system`.
pub fn resolve_setting(settings: &Settings, system: Language) -> Language {
    LanguageChoice::from_setting(settings.get_string(LANGUAGE_KEY)).resolve(system)
}

/// Runs `body` with `language` as this thread's language, so a test can read Indonesian text without touching the
/// others that run in parallel.
pub fn with_override<T>(language: Language, body: impl FnOnce() -> T) -> T {
    struct Restore(Option<Language>);
    impl Drop for Restore {
        fn drop(&mut self) {
            OVERRIDE.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(OVERRIDE.with(|slot| slot.replace(Some(language))));
    body()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Settings;

    #[test]
    fn from_tag_reads_the_indonesian_codes_in_any_case() {
        for tag in ["id", "ID", "id-ID", "id_ID"] {
            assert_eq!(Language::from_tag(tag), Language::Indonesian, "{tag}");
        }
        for tag in ["en-US", "idx", "fr", ""] {
            assert_eq!(Language::from_tag(tag), Language::English, "{tag}");
        }
    }

    #[test]
    fn an_unknown_setting_reads_as_auto() {
        for text in [Some("fr"), Some(""), None] {
            assert_eq!(LanguageChoice::from_setting(text), LanguageChoice::Auto, "{text:?}");
        }
        assert_eq!(LanguageChoice::from_setting(Some("id")), LanguageChoice::Indonesian);
        assert_eq!(LanguageChoice::from_setting(Some("en")), LanguageChoice::English);
        assert_eq!(LanguageChoice::from_setting(Some("auto")), LanguageChoice::Auto);
    }

    #[test]
    fn a_choice_writes_the_word_it_reads_back() {
        for choice in [LanguageChoice::Auto, LanguageChoice::English, LanguageChoice::Indonesian] {
            assert_eq!(LanguageChoice::from_setting(Some(choice.setting_word())), choice);
        }
    }

    #[test]
    fn auto_follows_the_system_and_the_others_ignore_it() {
        assert_eq!(LanguageChoice::Auto.resolve(Language::Indonesian), Language::Indonesian);
        assert_eq!(LanguageChoice::Auto.resolve(Language::English), Language::English);
        assert_eq!(LanguageChoice::English.resolve(Language::Indonesian), Language::English);
        assert_eq!(LanguageChoice::Indonesian.resolve(Language::English), Language::Indonesian);
    }

    #[test]
    fn the_first_preferred_language_decides() {
        assert_eq!(first_language_of(&["id-ID", "en-US"]), Language::Indonesian);
        assert_eq!(first_language_of(&["en-US", "id-ID"]), Language::English);
        assert_eq!(first_language_of(&[]), Language::English);
    }

    #[test]
    fn the_override_wins_on_its_own_thread_only() {
        let before = current();
        with_override(Language::Indonesian, || {
            assert_eq!(current(), Language::Indonesian);
            with_override(Language::English, || assert_eq!(current(), Language::English));
            assert_eq!(current(), Language::Indonesian, "a nested override restores the outer one");
        });
        assert_eq!(current(), before);
    }

    #[test]
    fn set_current_changes_only_the_override_while_one_is_set() {
        let before = std::thread::spawn(current).join().unwrap();
        with_override(Language::English, || {
            set_current(Language::Indonesian);
            assert_eq!(current(), Language::Indonesian);
        });
        assert_eq!(std::thread::spawn(current).join().unwrap(), before, "the process value did not move");
    }

    #[test]
    fn the_setting_resolves_against_the_system_language() {
        let path = std::env::temp_dir().join(format!("notch-language-{}.txt", std::process::id()));
        std::fs::write(
            &path,
            "general.language=id
",
        )
        .unwrap();
        let settings = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(path);
        assert_eq!(resolve_setting(&settings, Language::English), Language::Indonesian);
        assert_eq!(resolve_setting(&Settings::at(None), Language::Indonesian), Language::Indonesian);
        assert_eq!(resolve_setting(&Settings::at(None), Language::English), Language::English);
    }
}
