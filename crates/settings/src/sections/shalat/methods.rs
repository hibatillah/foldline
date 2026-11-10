//! The calculation method cards: Auto and the six methods, each showing where the sun is at Fajr and at Isha.

use crate::i18n::tr;
use gpui::{AnyElement, App, SharedString, prelude::*};
use gpui_component::v_flex;
use places::Location;
use store::Settings;

use super::{METHOD_KEY, pictures, pictures::Sun};
use crate::{card, field, layout::Item};

/// How Isha is found: the sun's depth in degrees, or a fixed time after Maghrib.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IshaRule {
    Degrees(f32),
    MinutesAfterMaghrib(u32),
}

pub struct Method {
    pub word: &'static str,
    pub label: &'static str,
    pub regions: fn() -> &'static str,
    pub fajr_degrees: f32,
    pub isha: IshaRule,
}

pub const AUTO_WORD: &str = "auto";
// A 90-minute Isha has no depth of its own; the picture shows the sun about as deep as the other methods put it.
const TIMED_ISHA_PICTURE_DEGREES: f32 = 18.0;

// The words come from `METHOD_OPTIONS` in `crates/notch/src/pages/shalat/times.rs` and the angles from the `salah` crate
// the notch calculates with; tests compare both. The labels and regions are this window's own.
pub const METHODS: [Method; 6] = [
    Method {
        word: "singapore",
        label: "Kemenag",
        regions: || tr!("shalat.region_kemenag"),
        fajr_degrees: 20.0,
        isha: IshaRule::Degrees(18.0),
    },
    Method {
        word: "mwl",
        label: "MWL",
        regions: || tr!("shalat.region_mwl"),
        fajr_degrees: 18.0,
        isha: IshaRule::Degrees(17.0),
    },
    Method {
        word: "northamerica",
        label: "ISNA",
        regions: || tr!("shalat.region_isna"),
        fajr_degrees: 15.0,
        isha: IshaRule::Degrees(15.0),
    },
    Method {
        word: "karachi",
        label: "Karachi",
        regions: || tr!("shalat.region_karachi"),
        fajr_degrees: 18.0,
        isha: IshaRule::Degrees(18.0),
    },
    Method {
        word: "ummalqura",
        label: "Umm al-Qura",
        regions: || tr!("shalat.region_umm_al_qura"),
        fajr_degrees: 18.5,
        isha: IshaRule::MinutesAfterMaghrib(90),
    },
    Method {
        word: "egyptian",
        label: "Egyptian",
        regions: || tr!("shalat.region_egyptian"),
        fajr_degrees: 19.5,
        isha: IshaRule::Degrees(17.5),
    },
];

// The countries the notch maps to a method when the setting is `auto`, in the order of `method_for_country`: the notch's
// `Method` variant, the countries and the name shown.
const AUTO_RULES: [(&str, &[&str], &str); 10] = [
    ("Singapore", &["ID", "MY", "SG", "BN"], "Kemenag"),
    ("UmmAlQura", &["SA"], "Umm al-Qura"),
    ("Dubai", &["AE"], "Dubai"),
    ("Qatar", &["QA"], "Qatar"),
    ("Kuwait", &["KW"], "Kuwait"),
    ("Egyptian", &["EG"], "Egyptian"),
    ("Karachi", &["PK", "IN", "BD", "AF"], "Karachi"),
    ("Tehran", &["IR"], "Tehran"),
    ("Turkey", &["TR"], "Turkey"),
    ("NorthAmerica", &["US", "CA"], "ISNA"),
];
const AUTO_FALLBACK: &str = "MWL";

/// The method name `auto` uses for a country code, as the notch's `method_for_country` picks it.
pub fn auto_method_name(country: &str) -> &'static str {
    AUTO_RULES.iter().find(|(_, countries, _)| countries.contains(&country)).map_or(AUTO_FALLBACK, |(_, _, name)| name)
}

/// "Fajr 18°"; a half degree keeps its decimal.
pub fn fajr_label(method: &Method) -> String {
    tr!("shalat.angle_fajr", degrees = method.fajr_degrees)
}

/// "Isha 18°" or "Isha 90 min".
pub fn isha_label(method: &Method) -> String {
    match method.isha {
        IshaRule::Degrees(degrees) => tr!("shalat.angle_isha", degrees = degrees),
        IshaRule::MinutesAfterMaghrib(minutes) => tr!("shalat.isha_minutes", minutes = minutes),
    }
}

fn isha_depth(method: &Method) -> f32 {
    match method.isha {
        IshaRule::Degrees(degrees) => degrees,
        IshaRule::MinutesAfterMaghrib(_) => TIMED_ISHA_PICTURE_DEGREES,
    }
}

/// The stored word when it names a method, else `auto`.
pub fn shown_word(settings: &Settings) -> &'static str {
    let stored = settings.get_string(METHOD_KEY).unwrap_or_default();
    METHODS.iter().find(|method| method.word == stored).map_or(AUTO_WORD, |method| method.word)
}

fn auto_sentence(country: &str) -> String {
    if country.is_empty() {
        tr!("shalat.auto_sentence_empty").to_owned()
    } else {
        tr!("shalat.auto_sentence", method = auto_method_name(country), country = country)
    }
}

fn method_card(
    word: &'static str,
    name: &'static str,
    note: &'static str,
    picture: AnyElement,
    selected: bool,
    cx: &App,
) -> AnyElement {
    card::frame(SharedString::from(format!("{METHOD_KEY}-{word}")), selected, 10.0, 10.0, cx)
        .cursor_pointer()
        .on_click(move |_event, _window, cx| field::write(METHOD_KEY, word, cx))
        .child(card::header(name, Some(note), selected, cx))
        .child(picture)
        .into_any_element()
}

/// The explanation and the cards, four equal columns.
pub fn item() -> Item {
    Item::custom(|_window, cx| {
        let settings = Settings::load();
        let shown = shown_word(&settings);
        let country = Location::read(&|key| settings.get_string(key).map(str::to_owned))
            .map(|location| location.country)
            .unwrap_or_default();
        let auto_name = if country.is_empty() { tr!("shalat.auto_by_country") } else { auto_method_name(&country) };
        let mut cards = vec![method_card(
            AUTO_WORD,
            tr!("shalat.auto"),
            tr!("shalat.auto_note"),
            pictures::globe_to(auto_name),
            shown == AUTO_WORD,
            cx,
        )];
        cards.extend(METHODS.iter().map(|method| {
            let picture = pictures::horizon(
                &Sun { label: fajr_label(method), depth_degrees: method.fajr_degrees },
                &Sun { label: isha_label(method), depth_degrees: isha_depth(method) },
            );
            method_card(method.word, method.label, (method.regions)(), picture, method.word == shown, cx)
        }));
        let explanation = tr!("shalat.method_explanation", auto = auto_sentence(&country));
        v_flex()
            .gap_3()
            .child(card::described(tr!("shalat.method_title"), explanation, cx))
            .child(card::grid(card::COLUMNS, cards))
            .into_any_element()
    })
}

#[cfg(test)]
mod tests {
    use salah::{Configuration, Madhab};

    use super::*;

    fn notch_source(file: &str) -> String {
        std::fs::read_to_string(format!("{}/../notch/src/pages/shalat/{file}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|error| panic!("notch source {file}: {error}"))
    }

    fn salah_method(word: &str) -> salah::Method {
        match word {
            "singapore" => salah::Method::Singapore,
            "mwl" => salah::Method::MuslimWorldLeague,
            "northamerica" => salah::Method::NorthAmerica,
            "karachi" => salah::Method::Karachi,
            "ummalqura" => salah::Method::UmmAlQura,
            "egyptian" => salah::Method::Egyptian,
            other => panic!("no method for {other}"),
        }
    }

    #[test]
    fn the_cards_follow_the_notchs_method_words_in_order() {
        let source = notch_source("times.rs");
        let options = source.split("METHOD_OPTIONS").nth(1).expect("METHOD_OPTIONS in times.rs");
        let options = options.split("];").next().expect("end of METHOD_OPTIONS");
        let notch_words: Vec<&str> =
            options.lines().filter_map(|line| line.trim().strip_prefix("(\"")?.split('"').next()).collect();
        let window_words: Vec<&str> =
            std::iter::once(AUTO_WORD).chain(METHODS.iter().map(|method| method.word)).collect();
        assert_eq!(window_words, notch_words);
    }

    #[test]
    fn the_cards_show_the_angles_the_notch_calculates_with() {
        for method in &METHODS {
            let parameters = Configuration::with(salah_method(method.word), Madhab::Shafi);
            assert_eq!(method.fajr_degrees, parameters.fajr_angle as f32, "{} Fajr", method.label);
            match method.isha {
                IshaRule::Degrees(degrees) => {
                    assert_eq!(degrees, parameters.isha_angle as f32, "{} Isha", method.label);
                    assert_eq!(parameters.isha_interval, 0, "{} Isha", method.label);
                }
                IshaRule::MinutesAfterMaghrib(minutes) => {
                    assert_eq!(i64::from(parameters.isha_interval), i64::from(minutes), "{} Isha", method.label);
                }
            }
        }
    }

    #[test]
    fn the_labels_read_in_degrees_or_minutes() {
        assert_eq!(fajr_label(&METHODS[0]), "Fajr 20°");
        assert_eq!(isha_label(&METHODS[0]), "Isha 18°");
        assert_eq!(fajr_label(&METHODS[4]), "Fajr 18.5°");
        assert_eq!(isha_label(&METHODS[4]), "Isha 90 min");
        assert_eq!(isha_label(&METHODS[5]), "Isha 17.5°");
    }

    #[test]
    fn auto_maps_each_country_the_way_the_notch_does() {
        let source = notch_source("times.rs");
        for (variant, countries, _) in &AUTO_RULES {
            let pattern = countries.iter().map(|country| format!("\"{country}\"")).collect::<Vec<_>>().join(" | ");
            assert!(source.contains(&format!("{pattern} => Method::{variant}")), "{variant} for {countries:?}");
        }
        assert!(source.contains("_ => Method::MuslimWorldLeague"));
    }

    #[test]
    fn auto_names_the_method_of_a_country_and_falls_back_to_mwl() {
        assert_eq!(auto_method_name("ID"), "Kemenag");
        assert_eq!(auto_method_name("US"), "ISNA");
        assert_eq!(auto_method_name("FR"), "MWL");
    }

    #[test]
    fn auto_explains_itself_for_a_known_and_an_unknown_country() {
        assert_eq!(auto_sentence("ID"), "Auto picks the method of your country: Kemenag for ID.");
        assert!(auto_sentence("").contains("Choose a city"));
    }

    #[test]
    fn a_missing_or_unknown_stored_method_shows_auto() {
        let path = std::env::temp_dir().join(format!("notch-settings-method-{}.txt", std::process::id()));
        std::fs::write(&path, "shalat.method=banana\n").unwrap();
        let unknown = Settings::at(Some(path.clone()));
        std::fs::write(&path, "shalat.method=karachi\n").unwrap();
        let stored = Settings::at(Some(path.clone()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(shown_word(&unknown), "auto");
        assert_eq!(shown_word(&stored), "karachi");
        assert_eq!(shown_word(&Settings::at(None)), "auto");
    }
}
