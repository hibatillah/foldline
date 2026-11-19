//! A `Notch` on an offscreen window, shared by the behavior tests and the image snapshots.

use std::{cell::Cell, fs, path::PathBuf, rc::Rc};

use windows::{
    Win32::{
        Foundation::{HWND, POINT, RECT},
        UI::WindowsAndMessaging::{CreateWindowExW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP},
    },
    core::w,
};

use super::*;
use crate::{
    activity::{Activity, Presence, Slot},
    gfx::{Canvas, Color, Fonts, Point, Rect, Surface},
    host::{chrome, navigation::PageList, screen_mode::ScreenMode, settings::Settings, visibility::Showing},
    page::{Context, Page, RESTING_HEIGHT},
    pages::{SampleOptions, samples},
    ui::{self, Icon},
};

mod badge_input;
mod clipboard_states;
mod header_hover;
mod header_row;
mod hiding_rules;
mod images;
mod language;
mod location;
mod page_navigation;
mod page_states;
mod pause;
mod pin;
mod priority_presences;
mod relocate;
mod resting_row;
mod row_motion;
mod screen_mode;
mod settings_reload;
mod snooze;
mod unit_presences;
mod units;

pub(super) const SCALE: f32 = 1.25;

pub(super) fn test_window() -> HWND {
    unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("STATIC"),
            w!("notch snapshot"),
            WS_POPUP,
            0,
            0,
            10,
            10,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }
}

pub(super) fn rig(options: SampleOptions) -> Notch {
    configured_rig(options, Settings::at(None), Vec::new())
}

pub(super) fn configured_rig(options: SampleOptions, settings: Settings, activities: Vec<Box<dyn Activity>>) -> Notch {
    let fonts = Fonts::new(SCALE).unwrap();
    let directory = std::env::temp_dir().join(format!("notch-snapshots-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    custom_rig(samples(&fonts, &directory, options), settings, activities)
}

/// A rig over pages the test builds itself; the Windows language counts as English.
pub(super) fn custom_rig(pages: Vec<Box<dyn Page>>, settings: Settings, activities: Vec<Box<dyn Activity>>) -> Notch {
    let count = pages.len();
    let width = (WINDOW_WIDTH * SCALE).round() as i32;
    let height = (WINDOW_HEIGHT * SCALE).round() as i32;
    let mut notch = Notch::new(NotchParts {
        window: test_window(),
        scale: SCALE,
        screen_position: POINT { x: 0, y: 0 },
        surface: Surface::new(width, height).unwrap(),
        fonts: Fonts::new(SCALE).unwrap(),
        settings,
        pages,
        activities,
        ticks_per_second: 10_000_000,
        system_language: store::language::Language::English,
        monitor: MonitorState::new(Default::default(), None, None, None),
    });
    notch.monitor.source = relocate::FAKE_MONITORS;
    notch.page_list = PageList::new(count);
    notch
}

pub(super) fn settle(notch: &mut Notch) {
    for _ in 0..3 {
        notch.refresh_layout();
        for spring in notch.layout_springs() {
            spring.snap_to_target();
        }
        notch.springs.expansion.snap_to_target();
        notch.page_swap.finish();
    }
}

pub(super) fn open(notch: &mut Notch, page_id: &str) {
    let index = notch.pages.iter().position(|page| page.id() == page_id).unwrap();
    notch.selected_page = Some(index);
    notch.open.is_open = true;
    notch.springs.expansion.target = 1.0;
    settle(notch);
}

pub(super) fn close(notch: &mut Notch) {
    notch.open.is_open = false;
    notch.springs.expansion.target = 0.0;
    settle(notch);
}

impl Notch {
    pub(super) fn page_at(&self, position: f32) -> usize {
        self.navigable[(position.round() as i64).rem_euclid(self.navigable.len() as i64) as usize]
    }
}

pub(super) struct SettingsFile(PathBuf);

impl SettingsFile {
    pub(super) fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("foldline-settings-{}-{name}.txt", std::process::id()));
        fs::write(&path, "").unwrap();
        Self(path)
    }

    pub(super) fn load(&self) -> Settings {
        Settings::at(Some(self.0.clone()))
    }
}

impl Drop for SettingsFile {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.0) {
            eprintln!("could not remove test settings {}: {error}", self.0.display());
        }
    }
}

pub(super) fn playing(lyrics_on: bool) -> Notch {
    rig(SampleOptions { media: Some(lyrics_on), ..SampleOptions::default() })
}

/// A stand-in activity whose pill presence and header mark a test sets; it draws a white disc.
pub(super) struct Mark {
    pub(super) presence: Rc<Cell<Option<Presence>>>,
    pub(super) rank: u8,
    pub(super) header_width: Option<f32>,
}

pub(super) const RING: Presence = Presence::Companion { slot: Slot::Ring, width: 14.0 };
pub(super) const DOT: Presence = Presence::Companion { slot: Slot::Dots, width: 6.0 };

impl Mark {
    pub(super) fn showing(presence: Option<Presence>) -> (Self, Rc<Cell<Option<Presence>>>) {
        let shared = Rc::new(Cell::new(presence));
        (Self { presence: shared.clone(), rank: 0, header_width: None }, shared)
    }
}

fn draw_disc(canvas: &Canvas, area: Rect, opacity: f32) {
    let size = area.width().min(14.0);
    let disc = Rect::around(Point::new(area.center_x(), area.center_y()), size / 2.0, size / 2.0);
    canvas.fill_rounded_rect(disc, size / 2.0, Color::WHITE.faded(opacity));
}

impl Activity for Mark {
    fn id(&self) -> &'static str {
        "mark"
    }

    fn presence(&self) -> Option<Presence> {
        self.presence.get()
    }

    fn header_status_rank(&self) -> u8 {
        self.rank
    }

    fn header_status_width(&self) -> Option<f32> {
        self.header_width
    }

    fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        draw_disc(canvas, area, opacity);
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        draw_disc(canvas, area, opacity);
    }
}

/// A settled rig whose activities are `marks`, all started.
pub(super) fn marked_rig(options: SampleOptions, marks: Vec<Mark>) -> Notch {
    let activities: Vec<Box<dyn Activity>> =
        marks.into_iter().map(|mark| Box::new(mark) as Box<dyn Activity>).collect();
    let mut notch = configured_rig(options, Settings::at(None), activities);
    for slot in &mut notch.activities {
        slot.is_started = true;
    }
    settle(&mut notch);
    notch
}
