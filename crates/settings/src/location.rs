//! The city search of the General page: a field with up to five matches below it, the saved one checked, and the
//! pages that use the city.

use crate::i18n::tr;

use gpui::{
    AnyElement, App, AppContext as _, Entity, Focusable as _, SharedString, Subscription, Window, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use places::{City, Location};
use store::Settings;

use crate::{field, sections};

const RESULT_LIMIT: usize = 5;
const FIELD_HEIGHT: f32 = 32.0;
const RESULTS_RADIUS: f32 = 9.0;
const RESULT_RADIUS: f32 = 6.0;
const RESULT_TEXT_SIZE: f32 = 13.0;
const CHIP_TEXT_SIZE: f32 = 11.5;

/// The saved city as the field shows it ("Pekanbaru, ID"), or nothing until one is chosen.
pub fn current_label(settings: &Settings) -> String {
    Location::read(&|key| settings.get_string(key).map(str::to_owned))
        .map(|location| location.label)
        .unwrap_or_default()
}

/// Saves the four location keys of the chosen city.
pub fn choose(settings: &mut Settings, city: &City) {
    sections::apply_all(settings, &places::values_of(city));
}

struct CitySearch {
    input: Entity<InputState>,
    matches: Vec<City>,
    _subscription: Subscription,
}

/// One match: the name, then " · country" muted; the saved city has an accent fill and a check.
fn result_row(id: &'static str, city: City, saved: bool, search: Entity<CitySearch>, cx: &App) -> AnyElement {
    let row_id = SharedString::from(format!("{id}-{}-{:.3}-{:.3}", city.label(), city.latitude, city.longitude));
    let accent = crate::palette::accent(cx);
    let muted = cx.theme().muted_foreground;
    let hover = cx.theme().foreground.opacity(0.08);
    let name =
        h_flex().min_w_0().gap(px(4.0)).child(div().flex_none().child(SharedString::from(city.name.clone()))).child(
            div().min_w_0().truncate().text_color(muted).child(SharedString::from(format!("· {}", city.country))),
        );
    h_flex()
        .id(row_id)
        .justify_between()
        .gap_2()
        .px(px(8.0))
        .py(px(6.0))
        .rounded(px(RESULT_RADIUS))
        .text_size(px(RESULT_TEXT_SIZE))
        .cursor_pointer()
        .when(saved, |row| row.bg(accent.opacity(0.18)))
        .when(!saved, |row| row.hover(move |row| row.bg(hover)))
        .on_click(move |_event, window, cx| {
            choose(&mut Settings::load(), &city);
            search.update(cx, |search, cx| {
                search.matches.clear();
                let label = city.label();
                search.input.update(cx, |input, cx| input.set_value(label, window, cx));
            });
            cx.refresh_windows();
        })
        .child(name)
        .children(saved.then(|| Icon::new(IconName::Check).size(px(13.0)).text_color(accent)))
        .into_any_element()
}

/// "Prayer times" and "Weather" as small chips, then "use this city".
fn users_of_the_city(cx: &App) -> AnyElement {
    let chip = |text: &'static str| {
        div().px(px(7.0)).py(px(1.0)).rounded(px(5.0)).bg(cx.theme().foreground.opacity(0.09)).child(text)
    };
    h_flex()
        .gap(px(6.0))
        .items_center()
        .text_size(px(CHIP_TEXT_SIZE))
        .text_color(cx.theme().muted_foreground)
        .child(chip(tr!("general.city_used_by_prayer")))
        .child(chip(tr!("general.city_used_by_weather")))
        .child(tr!("general.city_used_by_note"))
        .into_any_element()
}

/// The field, its matches and the pages that use the city; `id` keeps the field and its typed text apart.
pub fn city_search(id: &'static str, window: &mut Window, cx: &mut App) -> AnyElement {
    let label = current_label(&Settings::load());
    let search = window.use_keyed_state(SharedString::from(id), cx, {
        let label = label.clone();
        move |window, cx| {
            let input = cx.new(|cx| {
                InputState::new(window, cx).placeholder(tr!("general.city_placeholder")).default_value(label)
            });
            let _subscription = cx.subscribe_in(&input, window, |search: &mut CitySearch, input, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    search.matches = places::search(&input.read(cx).value(), RESULT_LIMIT);
                    window.refresh();
                }
            });
            CitySearch { input, matches: Vec::new(), _subscription }
        }
    });
    let input = search.read(cx).input.clone();
    let matches = search.read(cx).matches.clone();
    let focused = input.focus_handle(cx).is_focused(window);
    // The saved city comes back once the typed text has no matches left to pick from and the field lost focus.
    if matches.is_empty() && !focused && input.read(cx).value() != label {
        input.update(cx, |input, cx| input.set_value(label.clone(), window, cx));
    }
    let muted = cx.theme().muted_foreground;
    let field_box = field::framed(
        focused,
        Input::new(&input).appearance(false).prefix(Icon::new(IconName::Search).size(px(13.0)).text_color(muted)),
        cx,
    )
    .w_full()
    .h(px(FIELD_HEIGHT))
    .flex()
    .items_center();
    let results = (!matches.is_empty()).then(|| {
        let rows: Vec<AnyElement> = matches
            .into_iter()
            .map(|city| {
                let saved = city.label() == label;
                result_row(id, city, saved, search.clone(), cx)
            })
            .collect();
        v_flex().w_full().rounded(px(RESULTS_RADIUS)).bg(crate::palette::input_background(cx)).p(px(4.0)).children(rows)
    });
    v_flex().w_full().gap(px(8.0)).child(field_box).children(results).child(users_of_the_city(cx)).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_at(name: &str) -> (Settings, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("notch-settings-location-{}-{name}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        (Settings::at(Some(path.clone())), path)
    }

    #[test]
    fn choosing_a_city_saves_the_four_location_keys() {
        let (mut settings, path) = settings_at("choose");
        let city = places::search("Jakarta", 1).pop().expect("Jakarta is in the city list");
        choose(&mut settings, &city);
        let text = std::fs::read_to_string(&path).expect("settings file written");
        let _ = std::fs::remove_file(&path);
        for key in [places::CITY_KEY, places::LATITUDE_KEY, places::LONGITUDE_KEY, places::COUNTRY_KEY] {
            assert!(text.contains(&format!("{key}=")), "{key} missing in {text}");
        }
        assert!(text.contains(&format!("{}={}", places::CITY_KEY, city.label())), "{text}");
    }

    #[test]
    fn the_field_shows_the_saved_city_and_nothing_before_one_is_chosen() {
        let (mut settings, path) = settings_at("label");
        assert_eq!(current_label(&settings), "");
        let city = places::search("Pekanbaru", 1).pop().expect("Pekanbaru is in the city list");
        choose(&mut settings, &city);
        let shown = current_label(&Settings::at(Some(path.clone())));
        let _ = std::fs::remove_file(&path);
        assert_eq!(shown, city.label());
    }

    #[test]
    fn a_search_lists_at_most_five_cities_with_the_best_match_first() {
        let found = places::search("San", RESULT_LIMIT);
        assert_eq!(found.len(), RESULT_LIMIT);
        assert!(found[0].name.to_lowercase().starts_with("san"));
    }
}
