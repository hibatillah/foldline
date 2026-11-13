//! Moving the notch between monitors, over a monitor list the test sets.

use std::cell::{Cell, RefCell};

use super::*;
use crate::host::{
    monitor::{MonitorId, MonitorInfo, MonitorMode, placement},
    notch::monitor_state::MonitorSource,
};

thread_local! {
    static MONITORS: RefCell<Vec<MonitorInfo>> = const { RefCell::new(Vec::new()) };
    static FOREGROUND: Cell<Option<MonitorId>> = const { Cell::new(None) };
    static FULLSCREEN_ON: Cell<Option<MonitorId>> = const { Cell::new(None) };
    static JUDGED_TIMES: Cell<usize> = const { Cell::new(0) };
}

fn fake_list() -> Vec<MonitorInfo> {
    MONITORS.with_borrow(Clone::clone)
}

fn fake_foreground() -> Option<MonitorId> {
    FOREGROUND.get()
}

fn fake_judge(monitor: Option<MonitorId>) -> ScreenMode {
    JUDGED_TIMES.set(JUDGED_TIMES.get() + 1);
    if monitor.is_some() && monitor == FULLSCREEN_ON.get() { ScreenMode::Fullscreen } else { ScreenMode::Normal }
}

/// Each test runs on its own thread, so the monitor list it sets is its own; an empty list moves nothing.
pub(super) const FAKE_MONITORS: MonitorSource =
    MonitorSource { list: fake_list, foreground: fake_foreground, judge: fake_judge };

const MAIN: MonitorId = MonitorId(1);
const SECOND: MonitorId = MonitorId(2);

fn monitor(id: MonitorId, left: i32, scale: f32) -> MonitorInfo {
    let rect = crate::host::monitor::ScreenRect { left, top: 0, right: left + 1920, bottom: 1080 };
    MonitorInfo { id, device_id: format!("device-{}", id.0), rect, scale, is_primary: id == MAIN }
}

/// A notch on the main monitor at the rig's scale, with a second monitor at `second_scale` right of it.
fn two_screen_rig(mode: MonitorMode, second_scale: f32) -> Notch {
    let main = monitor(MAIN, 0, SCALE);
    MONITORS.set(vec![main.clone(), monitor(SECOND, 1920, second_scale)]);
    let mut notch = rig(SampleOptions::default());
    notch.monitor.mode = mode;
    notch.monitor.current = Some(MAIN);
    notch.monitor.placed = Some(placement(&main, 0));
    notch
}

fn second_placement(scale: f32) -> crate::host::monitor::Placement {
    placement(&monitor(SECOND, 1920, scale), 0)
}

#[test]
fn a_move_to_a_monitor_of_another_scale_changes_the_scale_the_surface_size_and_position() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    let expected = second_placement(1.5);
    assert_eq!(notch.monitor.current, Some(SECOND));
    assert_eq!(notch.scale, 1.5);
    assert_eq!((notch.surface.width, notch.surface.height), (expected.width, expected.height));
    assert_eq!((notch.screen_position.x, notch.screen_position.y), (expected.left, expected.top));
}

#[test]
fn a_move_to_a_monitor_of_the_same_scale_only_moves_the_window() {
    let mut notch = two_screen_rig(MonitorMode::Focused, SCALE);
    let size_before = (notch.surface.width, notch.surface.height);
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    assert_eq!(notch.scale, SCALE);
    assert_eq!((notch.surface.width, notch.surface.height), size_before);
    assert_eq!(notch.screen_position.x, second_placement(SCALE).left);
}

#[test]
fn a_move_requested_while_the_notch_is_open_waits_until_it_closes() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.open.is_open = true;
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(MAIN), "moved while open");
    notch.set_open(false);
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn a_pinned_notch_never_moves_until_it_is_unpinned() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.open.is_open = true;
    notch.open.is_pinned = true;
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    // A screenshot overlay closes a pinned notch but keeps the pin.
    notch.set_open(false);
    assert_eq!(notch.monitor.current, Some(MAIN), "moved while pinned");
    notch.set_pinned(false);
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn a_move_is_not_waited_for_when_the_current_monitor_is_gone() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.open.is_open = true;
    notch.open.is_pinned = true;
    notch.monitor.current = Some(MonitorId(3));
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn nothing_happens_when_the_target_and_the_placement_are_unchanged() {
    let mut notch = two_screen_rig(MonitorMode::Main, 1.5);
    FOREGROUND.set(Some(SECOND));
    let position_before = (notch.screen_position.x, notch.screen_position.y);
    let was_judged = notch.relocate_if_needed();
    assert!(!was_judged);
    assert_eq!(JUDGED_TIMES.get(), 0);
    assert_eq!((notch.screen_position.x, notch.screen_position.y), position_before);
}

#[test]
fn a_foreground_on_the_desktop_leaves_the_notch_where_it_is() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    // `monitor::foreground` reports the desktop, a taskbar or the notch itself as no window.
    FOREGROUND.set(None);
    notch.foreground_changed(ScreenMode::Normal);
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn a_display_change_moves_a_focused_notch_to_a_remaining_monitor() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    notch.open.is_open = true;
    MONITORS.set(vec![monitor(MAIN, 0, SCALE)]);
    FOREGROUND.set(None);
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(MAIN));
    assert_eq!(notch.scale, SCALE);
}

#[test]
fn a_fullscreen_window_on_the_second_screen_does_not_hide_a_notch_on_the_main_screen() {
    let mut notch = two_screen_rig(MonitorMode::Main, 1.5);
    FULLSCREEN_ON.set(Some(SECOND));
    FOREGROUND.set(Some(SECOND));
    notch.foreground_changed(fake_judge(Some(MAIN)));
    assert_eq!(notch.monitor.current, Some(MAIN));
    assert!(!notch.is_suppressed());
}

#[test]
fn a_fullscreen_window_on_the_second_screen_hides_a_focused_notch_that_moved_there() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    FULLSCREEN_ON.set(Some(SECOND));
    FOREGROUND.set(Some(SECOND));
    notch.foreground_changed(fake_judge(Some(MAIN)));
    assert_eq!(notch.monitor.current, Some(SECOND));
    assert!(notch.is_suppressed());
}

#[test]
fn switching_the_setting_to_main_brings_a_focused_notch_back_to_the_primary_monitor() {
    let settings_file = SettingsFile::new("monitor-mode");
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.settings = settings_file.load();
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    fs::write(&settings_file.0, "notch.monitor=main\n").unwrap();
    notch.reload_settings();
    assert_eq!((notch.monitor.mode, notch.monitor.current), (MonitorMode::Main, Some(MAIN)));
}

#[test]
fn moving_between_monitors_of_different_scale_many_times_does_not_grow_the_gdi_object_count() {
    use windows::Win32::System::Threading::{GR_GDIOBJECTS, GetCurrentProcess, GetGuiResources};
    let gdi_object_count = || unsafe { GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS) };
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    let before = gdi_object_count();
    for cycle in 0..200 {
        FOREGROUND.set(Some(if cycle % 2 == 0 { SECOND } else { MAIN }));
        notch.relocate_if_needed();
    }
    // Parallel rig tests hold windows and surfaces for a while; one leaked object per move would add 200.
    const TOLERANCE: u32 = 20;
    let mut after = gdi_object_count();
    for _ in 0..50 {
        if after <= before + TOLERANCE {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        after = gdi_object_count();
    }
    assert_eq!(notch.monitor.current, Some(MAIN));
    assert!(after <= before + TOLERANCE, "GDI objects grew from {before} to {after}");
}

#[test]
fn unpinning_an_open_notch_moves_it_once_it_closes() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.open.is_open = true;
    notch.open.is_pinned = true;
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    notch.set_pinned(false);
    assert_eq!(notch.monitor.current, Some(MAIN), "moved while still open");
    // The close delay or the hover watch ends in this call.
    notch.set_open(false);
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn a_scale_change_forgets_the_hovered_header_item_and_page_badge_measured_at_the_old_scale() {
    use crate::gfx::{Point, Rect};
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.header_marks.hovered.set(Some(crate::host::notch::header_row::HeaderTarget::Pin));
    let badge = Rect::new(100.0, 0.0, 140.0, 41.0);
    notch.badge_pointer.route(crate::page::PointerEvent::Moved(Point::new(120.0, 20.0)), &[(0, badge)]);
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    assert_eq!(notch.header_marks.hovered.get(), None);
    assert!(!notch.badge_pointer.is_over_a_badge());
}

#[test]
fn a_scale_change_scales_the_painted_region_origin_used_for_pointer_input() {
    let mut notch = two_screen_rig(MonitorMode::Focused, 1.5);
    notch.region_origin.set((400, 0));
    // The suppressed notch is not repainted, so the origin is the converted one, not a fresh paint's.
    notch.hiding.screen_mode = ScreenMode::Capture;
    FOREGROUND.set(Some(SECOND));
    notch.relocate_if_needed();
    let expected = (400.0 * 1.5 / SCALE).round() as i32;
    assert_eq!(notch.region_origin.get(), (expected, 0));
}

#[test]
fn an_unplugged_chosen_screen_moves_the_notch_to_the_main_screen_and_replugging_moves_it_back() {
    let mut notch = two_screen_rig(MonitorMode::Chosen, 1.5);
    notch.monitor.chosen_id = Some("device-2".to_owned());
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(SECOND));
    MONITORS.set(vec![monitor(MAIN, 0, SCALE)]);
    notch.relocate_if_needed();
    assert_eq!((notch.monitor.current, notch.scale), (Some(MAIN), SCALE));
    MONITORS.set(vec![monitor(MAIN, 0, SCALE), monitor(SECOND, 1920, 1.5)]);
    notch.relocate_if_needed();
    assert_eq!((notch.monitor.current, notch.scale), (Some(SECOND), 1.5));
    assert_eq!(notch.monitor.chosen_id.as_deref(), Some("device-2"));
}

#[test]
fn a_chosen_notch_ignores_the_foreground_window() {
    let mut notch = two_screen_rig(MonitorMode::Chosen, 1.5);
    notch.monitor.chosen_id = Some("device-2".to_owned());
    FOREGROUND.set(Some(MAIN));
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(SECOND));
}

#[test]
fn writing_a_chosen_screen_to_the_settings_file_moves_the_notch_there_live() {
    let settings_file = SettingsFile::new("monitor-chosen");
    let mut notch = two_screen_rig(MonitorMode::Main, 1.5);
    notch.settings = settings_file.load();
    fs::write(
        &settings_file.0,
        "notch.monitor=chosen
notch.monitor_id=device-2
",
    )
    .unwrap();
    notch.reload_settings();
    assert_eq!((notch.monitor.mode, notch.monitor.current), (MonitorMode::Chosen, Some(SECOND)));
}

#[test]
fn a_chosen_move_waits_while_the_notch_is_open() {
    let mut notch = two_screen_rig(MonitorMode::Chosen, 1.5);
    notch.monitor.chosen_id = Some("device-2".to_owned());
    notch.open.is_open = true;
    notch.relocate_if_needed();
    assert_eq!(notch.monitor.current, Some(MAIN), "moved while open");
    notch.set_open(false);
    assert_eq!(notch.monitor.current, Some(SECOND));
}
