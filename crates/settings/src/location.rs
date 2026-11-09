//! The city search both the General and the Shalat page show: a field with up to five matches below it.

use std::rc::Rc;

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

use crate::{field, layout::Render, sections};

const RESULT_LIMIT: usize = 5;
const FIELD_WIDTH: f32 = 260.0;

/// The saved city as the field shows it ("Pekanbaru, ID"), or nothing until one is chosen.
fn current_label(settings: &Settings) -> String {
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

/// The search field for the item's control; `id` keeps each page's field and its typed text apart.
pub fn city_search(id: &'static str) -> Render {
    Rc::new(move |window, cx| city_search_element(id, window, cx))
}

fn city_search_element(id: &'static str, window: &mut Window, cx: &mut App) -> AnyElement {
    let label = current_label(&Settings::load());
    let search = window.use_keyed_state(SharedString::from(id), cx, {
        let label = label.clone();
        move |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search for a city").default_value(label));
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
    // The saved city comes back once the typed text has no matches left to pick from and the field lost focus.
    if matches.is_empty() && !input.focus_handle(cx).is_focused(window) && input.read(cx).value() != label {
        input.update(cx, |input, cx| input.set_value(label, window, cx));
    }
    let muted = cx.theme().muted_foreground;
    let field_box = field::framed(
        FIELD_WIDTH,
        Input::new(&input).appearance(false).prefix(Icon::new(IconName::Search).size(px(13.0)).text_color(muted)),
        cx,
    );
    let results = (!matches.is_empty()).then(|| {
        let rows = matches.into_iter().map(|city| {
            let row_id =
                SharedString::from(format!("{id}-{}-{:.3}-{:.3}", city.label(), city.latitude, city.longitude));
            let search = search.clone();
            let chosen = city.clone();
            h_flex()
                .id(row_id)
                .justify_between()
                .gap_2()
                .px(px(10.0))
                .py(px(6.0))
                .text_size(px(13.0))
                .cursor_pointer()
                .hover(|row| row.bg(cx.theme().foreground.opacity(0.08)))
                .on_click(move |_event, window, cx| {
                    choose(&mut Settings::load(), &chosen);
                    search.update(cx, |search, cx| {
                        search.matches.clear();
                        let label = chosen.label();
                        search.input.update(cx, |input, cx| input.set_value(label, window, cx));
                    });
                    cx.refresh_windows();
                })
                .child(div().min_w_0().truncate().child(SharedString::from(city.name.clone())))
                .child(div().flex_none().text_color(muted).child(SharedString::from(city.country.clone())))
        });
        v_flex()
            .w(px(FIELD_WIDTH))
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().input)
            .bg(crate::palette::input_background(cx))
            .py(px(3.0))
            .children(rows)
    });
    v_flex().w(px(FIELD_WIDTH)).gap(px(6.0)).child(field_box).children(results).into_any_element()
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
