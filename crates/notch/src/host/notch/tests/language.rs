//! What a change of `general.language` does to the notch.

use std::cell::Cell;
use std::rc::Rc;

use store::language::{Language, with_override};

use super::*;
use crate::{
    gfx::Size,
    page::{Frame, Mode, Page, PageSizes, Priority},
};

struct Recorder {
    changes: Rc<Cell<u32>>,
}

impl Page for Recorder {
    fn id(&self) -> &'static str {
        "recorder"
    }

    fn priority(&self, _mode: Mode) -> Option<Priority> {
        None
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 100.0, expanded: Size::new(100.0, 86.0) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn on_language_changed(&mut self, _context: &mut Context) {
        self.changes.set(self.changes.get() + 1);
    }
}

struct ActivityRecorder {
    changes: Rc<Cell<u32>>,
}

impl Activity for ActivityRecorder {
    fn id(&self) -> &'static str {
        "recorder"
    }

    fn presence(&self) -> Option<Presence> {
        None
    }

    fn draw(&self, _canvas: &Canvas, _area: Rect, _scale: f32, _opacity: f32) {}

    fn on_language_changed(&mut self, _context: &mut Context) {
        self.changes.set(self.changes.get() + 1);
    }
}

#[test]
fn a_language_change_reaches_every_page_and_started_activity_once() {
    let settings_file = SettingsFile::new("language-hooks");
    let (page_changes, activity_changes) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
    let mut notch = custom_rig(
        vec![Box::new(Recorder { changes: page_changes.clone() })],
        settings_file.load(),
        vec![Box::new(ActivityRecorder { changes: activity_changes.clone() })],
    );
    with_override(Language::English, || {
        fs::write(&settings_file.0, "general.language=id\n").unwrap();
        notch.reload_settings();
        assert_eq!((page_changes.get(), activity_changes.get()), (1, 1));
        assert_eq!(store::language::current(), Language::Indonesian);
        notch.reload_settings();
        assert_eq!((page_changes.get(), activity_changes.get()), (1, 1), "an unchanged language makes no call");
    });
}

#[test]
fn a_changed_language_republishes_the_known_page_names() {
    let settings_file = SettingsFile::new("language-known-pages");
    let mut notch =
        configured_rig(SampleOptions { calendar: true, ..SampleOptions::default() }, settings_file.load(), Vec::new());
    notch.start();
    assert!(notch.settings.get_string("pages.known").unwrap().contains("calendar:Calendar"));
    with_override(Language::English, || {
        fs::write(&settings_file.0, "general.language=id\n").unwrap();
        notch.reload_settings();
    });
    assert!(notch.settings.get_string("pages.known").unwrap().contains("calendar:Kalender"));
}

#[test]
fn the_page_names_read_in_the_current_language() {
    let notch = rig(SampleOptions::default());
    let name_of = |id: &str| notch.pages.iter().find(|page| page.id() == id).unwrap().name();
    assert_eq!(name_of("calendar"), "Calendar");
    with_override(Language::Indonesian, || {
        assert_eq!(name_of("calendar"), "Kalender");
        assert_eq!(name_of("timer"), "Timer fokus");
        assert_eq!(name_of("todo"), "Todo");
        assert_eq!(name_of("shelf"), "File shelf");
    });
}

#[test]
fn starting_the_notch_rebuilds_the_text_pages_cached_before_the_language_was_set() {
    let changes = Rc::new(Cell::new(0));
    let mut notch = custom_rig(vec![Box::new(Recorder { changes: changes.clone() })], Settings::at(None), Vec::new());
    notch.start();
    assert_eq!(changes.get(), 1);
}
