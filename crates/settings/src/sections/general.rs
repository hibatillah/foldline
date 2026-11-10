use std::ops::RangeInclusive;

use crate::i18n::tr;
use crate::{
    field::{self, Kind},
    layout::{Group, Item, Page},
    location, startup,
};

fn minutes(
    title: &'static str,
    key: &'static str,
    default: u32,
    range: RangeInclusive<u32>,
    placeholder: &'static str,
) -> Item {
    Item::new(title, field::input(key, Kind::WholeNumber { default, range }, placeholder))
}

pub fn page() -> Page {
    let start_with_windows = field::switch("start-with-windows", startup::is_enabled, |enabled, cx| {
        startup::set_enabled(enabled);
        cx.refresh_windows();
    });
    Page::new(
        tr!("general.title"),
        tr!("general.description"),
        vec![
            Group::titled(tr!("general.language_title"), vec![super::language::item()]),
            Group::titled(
                tr!("general.group_startup"),
                vec![
                    Item::new(tr!("general.start_with_windows"), start_with_windows)
                        .description(tr!("general.start_with_windows_note")),
                    Item::new(tr!("general.hide_when_idle"), field::toggle("notch.hide_when_idle", false))
                        .description(tr!("general.hide_when_idle_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_location"),
                vec![
                    Item::new(tr!("general.track_location"), field::toggle(places::TRACK_KEY, true))
                        .description(tr!("general.track_location_note")),
                    Item::new(tr!("general.city"), location::city_search("general-city"))
                        .description(tr!("general.city_note"))
                        .top_aligned(),
                ],
            ),
            Group::titled(
                tr!("general.group_notifications"),
                vec![
                    Item::new(tr!("general.show_notifications"), field::toggle("activity.notifications", true))
                        .description(tr!("general.show_notifications_note")),
                    Item::new(tr!("general.show_notification_text"), field::toggle("notifications.show_text", true))
                        .description(tr!("general.show_notification_text_note")),
                    Item::new(tr!("general.device_battery"), field::toggle("activity.device_battery", true))
                        .description(tr!("general.device_battery_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_windows_popups"),
                vec![
                    Item::new(tr!("general.hide_volume_popup"), field::toggle("notch.hide_windows_volume_popup", true))
                        .description(tr!("general.hide_volume_popup_note")),
                    Item::new(tr!("general.hide_banners"), field::toggle("notch.hide_windows_notifications", true))
                        .description(tr!("general.hide_banners_note")),
                ],
            ),
            Group::titled(
                tr!("general.group_timer"),
                vec![
                    minutes(tr!("general.focus_minutes"), "timer.focus_minutes", 25, 5..=120, "25")
                        .description(tr!("general.focus_minutes_note")),
                    minutes(tr!("general.short_break_minutes"), "timer.short_break_minutes", 5, 1..=120, "5")
                        .description(tr!("general.short_break_note")),
                    minutes(tr!("general.long_break_minutes"), "timer.long_break_minutes", 15, 1..=120, "15")
                        .description(tr!("general.long_break_note")),
                ],
            ),
        ],
    )
}
