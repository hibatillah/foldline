//! The General page: startup, the city, notifications, the screen the notch sits on, and Windows' popups.

mod city_map;
mod idle_pictures;
mod monitor;
mod notification_cards;
mod screen;

use std::rc::Rc;

use gpui::{div, prelude::*, px};
use gpui_component::h_flex;

use super::lock;
use crate::i18n::tr;
use crate::{
    field,
    layout::{Group, Item, Page},
    location, startup,
};

/// The city search and the map side by side.
const LOCATION_GAP: f32 = 14.0;

/// A row with a title and a description and nothing at the right, for a row whose control follows below it.
fn heading_row(title: &'static str, description: &'static str) -> Item {
    Item::new(title, Rc::new(|_window, _cx| div().into_any_element())).description(description)
}

fn city_row() -> Item {
    Item::picture_row(|window, cx| {
        h_flex()
            .w_full()
            .items_start()
            .gap(px(LOCATION_GAP))
            .child(div().flex_1().min_w_0().child(location::city_search("general-city", window, cx)))
            .child(div().flex_1().min_w_0().child(city_map::city_map(cx)))
            .into_any_element()
    })
}

pub fn page() -> Page {
    let start_with_windows = field::switch("start-with-windows", startup::is_enabled, |enabled, cx| {
        startup::set_enabled(enabled);
        cx.refresh_windows();
    });
    Page::plain(
        tr!("general.title"),
        vec![
            Group::titled(
                tr!("general.group_startup"),
                vec![
                    Item::new(tr!("general.start_with_windows"), start_with_windows)
                        .description(tr!("general.start_with_windows_note")),
                    Item::new(tr!("general.hide_when_idle"), field::toggle("notch.hide_when_idle", false))
                        .description(tr!("general.hide_when_idle_note")),
                    idle_pictures::item(),
                ],
            ),
            Group::titled(
                tr!("general.group_location"),
                vec![
                    heading_row(tr!("general.city"), tr!("general.city_note")),
                    city_row(),
                    Item::new(tr!("general.track_location"), field::toggle(places::TRACK_KEY, true))
                        .description(tr!("general.track_location_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_notifications"),
                vec![
                    Item::new(
                        tr!("general.show_notifications"),
                        lock::locked("activity", "notifications", field::toggle("activity.notifications", true)),
                    )
                    .description(tr!("general.show_notifications_note")),
                    Item::new(tr!("general.show_notification_text"), field::toggle("notifications.show_text", true))
                        .description(tr!("general.show_notification_text_note")),
                    notification_cards::item(),
                    Item::new(
                        tr!("general.device_battery"),
                        lock::locked("activity", "device_battery", field::toggle("activity.device_battery", true)),
                    )
                    .description(tr!("general.device_battery_note")),
                    Item::new(
                        tr!("general.downloads"),
                        lock::locked("activity", "downloads", field::toggle("activity.downloads", true)),
                    )
                    .description(tr!("general.downloads_note")),
                ],
            ),
            Group::titled(tr!("general.monitor"), vec![monitor::item(), screen::item()])
                .description(tr!("general.monitor_note")),
            Group::titled(
                tr!("general.group_windows_popups"),
                vec![
                    Item::new(tr!("general.hide_volume_popup"), field::toggle("notch.hide_windows_volume_popup", true))
                        .description(tr!("general.hide_volume_popup_note")),
                    Item::new(tr!("general.hide_banners"), field::toggle("notch.hide_windows_notifications", true))
                        .description(tr!("general.hide_banners_note")),
                ],
            ),
        ],
    )
}
