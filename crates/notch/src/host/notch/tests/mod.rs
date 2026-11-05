//! A `Notch` on an offscreen window, shared by the behavior tests and the image snapshots.

use std::{fs, path::PathBuf};

use windows::{
    Win32::{
        Foundation::RECT,
        UI::WindowsAndMessaging::{CreateWindowExW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP},
    },
    core::w,
};

use std::cell::Cell;

use windows::Win32::Foundation::{HWND, POINT};

use super::*;
use crate::{
    activity::{Activity, Presence},
    gfx::{Canvas, Color, Fonts, Rect, Surface, lerp},
    host::{
        chrome, navigation::PageList, page_editor::PageEditor, screen_mode::ScreenMode, settings::Settings,
        settings_panel::Tab, visibility::Showing,
    },
    page::{Context, RESTING_HEIGHT},
    pages::{SampleOptions, samples},
    ui::{self, Icon},
};

mod context_menu;
mod hiding_rules;
mod images;
mod page_navigation;
mod screen_mode;
mod settings_reload;

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
    let pages = samples(&fonts, &directory, options);
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
    });
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
    }
}

pub(super) fn open(notch: &mut Notch, page_id: &str) {
    let index = notch.pages.iter().position(|page| page.id() == page_id).unwrap();
    notch.swipe = None;
    notch.holds_page_position = false;
    notch.selected_page = Some(index);
    notch.hover.is_hovering = true;
    notch.springs.expansion.target = 1.0;
    settle(notch);
}

pub(super) fn close(notch: &mut Notch) {
    notch.hover.is_hovering = false;
    notch.springs.expansion.target = 0.0;
    settle(notch);
}

pub(super) struct SettingsFile(PathBuf);

impl SettingsFile {
    pub(super) fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("notch-settings-{}-{name}.txt", std::process::id()));
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
    rig(SampleOptions { media: Some((lyrics_on, false)), ..SampleOptions::default() })
}
