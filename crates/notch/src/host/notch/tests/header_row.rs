//! The open header's dots, clock, group and gear, and the clicks on them.

use super::*;
use crate::gfx::Point;
use crate::host::{
    clock,
    notch::{header_row::HeaderTarget, layout::WIDTH_FLOOR_PAGE_ID, row_motion::RowItem},
    pill_row,
};

fn header_icon(rank: u8) -> Mark {
    let (mut mark, _) = Mark::showing(None);
    (mark.rank, mark.header_width) = (rank, Some(16.0));
    mark
}

/// Muted (rank 2, no page) as activity 0, the battery ring as activity 1 and a camera dot as activity 2.
fn open_header(options: SampleOptions) -> Notch {
    let (ring, _) = Mark::showing(Some(RING));
    let (dot, _) = Mark::showing(Some(DOT));
    let mut notch = marked_rig(options, vec![header_icon(2), ring, dot]);
    open(&mut notch, "media");
    notch
}

fn header(notch: &Notch) -> Rect {
    chrome::header_rect(notch.frame().bounds, notch.scale)
}

/// The device-pixel point at the middle of the header item that starts `left` DIP from the surface origin.
fn at_item(notch: &Notch, left: f32, width: f32) -> Point {
    Point::new((left + width / 2.0) * notch.scale, header(notch).center_y())
}

fn page_index(notch: &Notch, id: &str) -> usize {
    notch.pages.iter().position(|page| page.id() == id).unwrap()
}

/// Where the header item sits and how wide it is, from the settled layout.
fn item_place(notch: &Notch, item: RowItem) -> (f32, f32) {
    let row = notch.header_layout(notch.frame().bounds);
    let group = notch.shown_header_group(&row);
    let position = group.iter().position(|&(known, _)| known == item).expect("the item is in the header");
    (row.group_lefts[position], group[position].1)
}

#[test]
fn the_gear_is_the_last_item_at_the_right_edge_and_a_click_on_it_opens_the_settings_window() {
    let notch = open_header(SampleOptions::default());
    let row = notch.header_layout(notch.frame().bounds);
    assert!((row.gear_left + pill_row::GEAR_SIZE - header(&notch).right / notch.scale).abs() < 0.001);
    let point = at_item(&notch, row.gear_left, pill_row::GEAR_SIZE);
    assert_eq!(notch.header_target_at(point), Some(HeaderTarget::Settings));
}

#[test]
fn the_page_dots_start_at_the_left_edge_of_the_header() {
    let notch = open_header(SampleOptions::default());
    let dots = chrome::dot_rects(notch.navigable.len(), notch.track.position, notch.frame().bounds, notch.scale);
    assert!((dots[0].left - header(&notch).left).abs() < 0.001);
}

#[test]
fn the_gear_spring_settles_at_the_right_edge_of_the_header() {
    let notch = open_header(SampleOptions::default());
    assert_eq!(notch.row_motion.header.position(RowItem::Gear), Some(pill_row::GEAR_SIZE));
}

#[test]
fn a_collapsed_notch_has_no_gear_to_press() {
    let mut notch = open_header(SampleOptions::default());
    let row = notch.header_layout(notch.frame().bounds);
    let point = at_item(&notch, row.gear_left, pill_row::GEAR_SIZE);
    close(&mut notch);
    assert!(!notch.press_header(point));
}

#[test]
fn the_header_group_keeps_the_idle_order_and_its_dots_end_10_before_the_gear() {
    let notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let items: Vec<RowItem> = notch.header_group().iter().map(|&(item, _)| item).collect();
    let timer = RowItem::Source(Source::Page(page_index(&notch, "timer")));
    let muted = RowItem::Source(Source::Activity(0));
    assert_eq!(items, [muted, timer, RowItem::Source(Source::Activity(1)), RowItem::Dots]);
    let (dots_left, dots_width) = item_place(&notch, RowItem::Dots);
    let gear_left = notch.header_layout(notch.frame().bounds).gear_left;
    assert!((gear_left - (dots_left + dots_width) - pill_row::HEADER_ITEM_GAP).abs() < 0.001);
}

#[test]
fn the_header_springs_settle_on_the_header_layout() {
    let notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let right = header(&notch).right / notch.scale;
    let row = notch.header_layout(notch.frame().bounds);
    for (item, _) in notch.shown_header_group(&row) {
        let (left, _) = item_place(&notch, item);
        let inset = notch.row_motion.header.position(item).unwrap();
        assert!((right - inset - left).abs() < 0.01, "{item:?} sits where the layout puts it");
    }
}

#[test]
fn clicking_the_timer_icon_opens_the_timer_page() {
    let mut notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let timer = page_index(&notch, "timer");
    let (left, width) = item_place(&notch, RowItem::Source(Source::Page(timer)));
    let point = at_item(&notch, left, width);
    assert_eq!(notch.header_target_at(point), Some(HeaderTarget::Page("timer")));
    assert!(notch.press_header(point));
    assert_eq!(notch.selected_page, Some(timer));
}

#[test]
fn the_hit_area_is_24_wide_and_24_high_around_the_icon() {
    let notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let (left, width) = item_place(&notch, RowItem::Source(Source::Page(page_index(&notch, "timer"))));
    let center = at_item(&notch, left, width);
    let scale = notch.scale;
    let shifted = |dx: f32, dy: f32| Point::new(center.x + dx * scale, center.y + dy * scale);
    let expected = Some(HeaderTarget::Page("timer"));
    assert_eq!(notch.header_target_at(shifted(0.0, -11.5)), expected, "inside 12 above");
    let beyond_the_icon = width / 2.0 + 3.5;
    assert_eq!(notch.header_target_at(shifted(-beyond_the_icon, 0.0)), expected, "left of the icon, inside its area");
    assert_eq!(notch.header_target_at(shifted(0.0, 13.0)), None, "below the 24 high area");
}

#[test]
fn muted_the_battery_ring_and_the_dots_take_the_click_but_open_nothing() {
    let mut notch = open_header(SampleOptions::default());
    for item in [RowItem::Source(Source::Activity(0)), RowItem::Source(Source::Activity(1)), RowItem::Dots] {
        let (left, width) = item_place(&notch, item);
        let point = at_item(&notch, left, width);
        assert_eq!(notch.header_target_at(point), Some(HeaderTarget::Nothing), "{item:?}");
        let before = notch.selected_page;
        assert!(notch.press_header(point));
        assert_eq!(notch.selected_page, before);
    }
}

#[test]
fn the_header_clock_sits_alone_in_the_middle_and_a_press_on_it_hits_no_item() {
    let mut notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let kept = [page_index(&notch, "media"), page_index(&notch, "timer")];
    for index in (0..notch.pages.len()).filter(|index| !kept.contains(index)) {
        notch.page_list.set_enabled(index, false);
    }
    settle(&mut notch);
    let row = notch.header_layout(notch.frame().bounds);
    let header = header(&notch);
    assert_eq!(row.clock_center, (header.left + header.right) / 2.0 / notch.scale);
    let point = Point::new(row.clock_center * notch.scale, header.center_y());
    assert_eq!(notch.header_target_at(point), None);
}

fn timer_open_width(notch: &Notch) -> f32 {
    notch.pages[page_index(notch, WIDTH_FLOOR_PAGE_ID)].sizes().expanded.width + chrome::WIDTH_EXTRA
}

#[test]
fn the_header_floor_is_the_centered_clock_with_the_wider_side_twice_and_the_side_padding() {
    let notch = open_header(SampleOptions::default());
    let end: Vec<f32> = notch.resting_row.right_end().iter().map(|&(_, width)| width).collect();
    let content = pill_row::header_content_width(
        chrome::dots_width(notch.navigable.len()),
        clock::layout_width(&notch.fonts),
        &end,
    );
    assert!((notch.header_min_width() - (content + chrome::WIDTH_EXTRA)).abs() < 0.001);
}

#[test]
fn the_open_notch_is_never_narrower_than_the_timer_pages_open_width() {
    let notch = open_header(SampleOptions::default());
    assert!(notch.header_min_width() < timer_open_width(&notch), "the sample header is narrower than the timer");
    assert!((notch.open_min_width() - timer_open_width(&notch)).abs() < 0.001);
}

#[test]
fn a_page_narrower_than_the_timer_opens_the_notch_as_wide_as_the_timer_page() {
    let notch = open_header(SampleOptions::default());
    let media_width = notch.pages[page_index(&notch, "media")].sizes().expanded.width;
    assert!(media_width + chrome::WIDTH_EXTRA < timer_open_width(&notch), "the sample media page is narrow");
    assert!((notch.springs.expanded_width.target - timer_open_width(&notch)).abs() < 0.001);
    assert!((notch.frame().bounds.width() / notch.scale - timer_open_width(&notch)).abs() < 0.01);
}

#[test]
fn a_page_wider_than_the_timer_keeps_its_own_open_width() {
    let mut notch = open_header(SampleOptions { calendar: true, ..SampleOptions::default() });
    open(&mut notch, "calendar");
    let calendar_width = notch.pages[page_index(&notch, "calendar")].sizes().expanded.width + chrome::WIDTH_EXTRA;
    assert!(calendar_width > notch.open_min_width());
    assert!((notch.springs.expanded_width.target - calendar_width).abs() < 0.001);
}

#[test]
fn with_the_timer_page_switched_off_the_floor_falls_back_to_the_header_content() {
    let mut notch = open_header(SampleOptions::default());
    notch.page_list.set_enabled(page_index(&notch, WIDTH_FLOOR_PAGE_ID), false);
    settle(&mut notch);
    assert!((notch.open_min_width() - notch.header_min_width()).abs() < 0.001);
    let media_width = notch.pages[page_index(&notch, "media")].sizes().expanded.width + chrome::WIDTH_EXTRA;
    let expected = media_width.max(notch.header_min_width());
    assert!((notch.springs.expanded_width.target - expected).abs() < 0.001);
}

/// Prints each open width; `cargo test every_sample_page -- --nocapture` shows them.
#[test]
fn every_sample_page_opens_at_least_as_wide_as_the_timer_page() {
    let options =
        SampleOptions { media: Some(true), timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() };
    let mut notch = open_header(options);
    let floor = timer_open_width(&notch);
    for id in ["media", "shelf", "calendar", "timer", "stats", "todo", "shalat", "weather"] {
        open(&mut notch, id);
        let width = notch.springs.expanded_width.target;
        println!("open_widths {id}: {width}");
        assert!(width >= floor - 0.001, "{id} opens at {width}, narrower than the timer's {floor}");
    }
}

#[test]
fn a_crowded_header_keeps_the_clock_centered_with_room_to_the_dots_and_the_icons() {
    let options =
        SampleOptions { media: Some(false), timer: true, calendar: true, shalat_mark: 1, ..SampleOptions::default() };
    let notch = open_header(options);
    assert!(notch.springs.expanded_width.target >= notch.header_min_width());
    let bounds = notch.frame().bounds;
    let row = notch.header_layout(bounds);
    let scale = notch.scale;
    let dots = chrome::dot_rects(notch.navigable.len(), notch.track.position, bounds, scale);
    let half_clock = clock::layout_width(&notch.fonts) / 2.0;
    let clock_left = row.clock_center - half_clock;
    let header = header(&notch);
    assert!((row.clock_center - (header.left + header.right) / 2.0 / scale).abs() < 0.001);
    assert!(clock_left - dots.last().unwrap().right / scale >= pill_row::PAGE_DOTS_CLOCK_GAP - 0.01);
    assert!(row.group_lefts[0] - (row.clock_center + half_clock) >= pill_row::HEADER_CLOCK_GAP - 0.01);
}

/// Twelve icons of rank 0 to 11 (muted's width, no page) as activities 0 to 11, then the ring and a camera dot.
fn crowded_header() -> Notch {
    let mut marks: Vec<Mark> = (0..12).map(header_icon).collect();
    marks.push(Mark::showing(Some(RING)).0);
    marks.push(Mark::showing(Some(DOT)).0);
    let mut notch = marked_rig(SampleOptions::default(), marks);
    open(&mut notch, "media");
    notch
}

#[test]
fn a_header_without_room_drops_the_icons_nearest_the_clock_first() {
    let notch = crowded_header();
    let row = notch.header_layout(notch.frame().bounds);
    assert!(row.dropped_icons > 0, "twelve icons do not fit beside the centered clock");
    let shown = notch.shown_header_group(&row);
    let first_shown = RowItem::Source(Source::Activity(row.dropped_icons));
    assert_eq!(shown[0].0, first_shown, "the lowest ranks go first");
    assert_eq!(shown.last().map(|&(item, _)| item), Some(RowItem::Dots), "the ring and the dots stay");
}

#[test]
fn a_dropped_icon_has_no_spring_and_a_press_where_it_would_sit_hits_nothing() {
    let notch = crowded_header();
    let row = notch.header_layout(notch.frame().bounds);
    let dropped = RowItem::Source(Source::Activity(0));
    assert_eq!(notch.row_motion.header.position(dropped), None);
    let half_clock = clock::layout_width(&notch.fonts) / 2.0;
    let between = (row.clock_center + half_clock + row.group_lefts[0]) / 2.0;
    let point = Point::new(between * notch.scale, header(&notch).center_y());
    assert_eq!(notch.header_target_at(point), None);
}

#[test]
fn switching_a_page_off_drops_its_header_icon_at_the_next_layout_refresh() {
    let mut notch = open_header(SampleOptions { timer: true, ..SampleOptions::default() });
    let timer = page_index(&notch, "timer");
    let shows_timer = |notch: &Notch| notch.header_marks.icons.iter().any(|&(_, source)| source == Source::Page(timer));
    assert!(shows_timer(&notch));
    notch.page_list.set_enabled(timer, false);
    notch.refresh_layout();
    assert!(!shows_timer(&notch));
}
