//! Browser downloads: a banner when one starts, an icon while any runs, a banner with the file when one finishes.
//! The Downloads folder's file names are watched; file contents are never read.

mod card;
mod format;
mod tracker;
mod watcher;

#[cfg(test)]
mod tests;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use self::{
    card::{Card, Shown, WIDTH, draw_icon},
    format::size_text,
    tracker::{Outcome, Tracker},
    watcher::Shared,
};
use crate::{
    activity::{Activity, FLASH_TIMER, Flash, Presence},
    gfx::{Canvas, Rect},
    i18n::tr,
    log,
    page::{Context, PointerEvent, TimerId},
    platform::{downloads_folder, file_icon, shell_open, toast_activation},
    ui::{BannerButton, Icon},
};

const START_SHOW_MILLISECONDS: u32 = 4000;
const FINISH_SHOW_MILLISECONDS: u32 = 5000;
/// After the pointer leaves the finish banner, so a pointer that slips off the edge on the way to a button keeps it.
const LINGER_MILLISECONDS: u32 = 1500;
const STALE_TIMER: TimerId = TimerId(1);
const STALE_CHECK_MILLISECONDS: u32 = 60_000;

/// What one download outcome asks the activity to show.
#[derive(Debug, PartialEq, Eq)]
enum Reaction {
    Nothing,
    ShowStarted { title: Option<String> },
    ShowFinished { path: PathBuf },
}

/// Where a vanished download's bytes are: the real file named by a Firefox `.part` name, when it holds any.
fn finished_path(folder: &Path, final_name: Option<&str>) -> Option<PathBuf> {
    let path = folder.join(final_name?);
    std::fs::metadata(&path).ok().filter(|metadata| metadata.len() > 0).map(|_| path)
}

fn react(outcome: Outcome, folder: &Path) -> Reaction {
    match outcome {
        Outcome::Nothing => Reaction::Nothing,
        Outcome::Started { display_name } => Reaction::ShowStarted { title: display_name },
        Outcome::Finished { file_name } => Reaction::ShowFinished { path: folder.join(file_name) },
        Outcome::Vanished { final_name } => finished_path(folder, final_name.as_deref())
            .map_or(Reaction::Nothing, |path| Reaction::ShowFinished { path }),
    }
}

pub struct DownloadsActivity {
    folder: Option<PathBuf>,
    shared: Option<Arc<Shared>>,
    tracker: Tracker,
    flash: Flash,
    card: Option<Card>,
    is_pointer_over: bool,
    is_stale_timer_running: bool,
}

impl DownloadsActivity {
    pub fn new() -> Self {
        Self {
            folder: None,
            shared: None,
            tracker: Tracker::default(),
            flash: Flash::default(),
            card: None,
            is_pointer_over: false,
            is_stale_timer_running: false,
        }
    }

    fn shows_buttons(&self) -> bool {
        self.is_pointer_over
    }

    fn show_started(&mut self, title: Option<String>, context: &mut Context) {
        if self.is_pointer_over && self.card.as_ref().is_some_and(Card::is_finished) {
            return;
        }
        let title = title.unwrap_or_else(|| tr!("downloads.preparing").into_owned());
        self.card = Some(Card::new(Shown::Started { title }, context.fonts()));
        self.is_pointer_over = false;
        self.flash.show(context, START_SHOW_MILLISECONDS);
    }

    fn show_finished(&mut self, path: PathBuf, context: &mut Context) {
        let size = match std::fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                log::debug(&format!("downloads: could not read the size of a finished download: {error}"));
                return;
            }
        };
        let title = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let extension = path.extension().and_then(|extension| extension.to_str()).unwrap_or("");
        let shown = Shown::Finished {
            title,
            detail: tr!("downloads.done", size = size_text(size)).into_owned(),
            icon: file_icon::for_extension(extension),
            path,
        };
        let mut card = Card::new(shown, context.fonts());
        card.upload_icon(|image| context.upload_image(image));
        self.card = Some(card);
        self.flash.show(context, FINISH_SHOW_MILLISECONDS);
        if self.is_pointer_over {
            context.kill_timer(FLASH_TIMER);
        }
    }

    fn hide(&mut self, context: &mut Context) {
        self.is_pointer_over = false;
        if self.flash.is_visible() {
            self.flash.on_timer(FLASH_TIMER, context);
        }
    }

    fn pointer_left(&mut self, context: &mut Context) {
        if !self.is_pointer_over {
            return;
        }
        self.is_pointer_over = false;
        if self.flash.is_visible() {
            self.flash.show(context, LINGER_MILLISECONDS);
        }
    }

    fn clicked(&mut self, button: BannerButton, context: &mut Context) {
        let Some(path) = self.card.as_ref().and_then(|card| card.path()).map(Path::to_path_buf) else { return };
        let action: fn(&Path) = match button {
            BannerButton::Dismiss => shell_open::show_in_folder,
            BannerButton::Open => shell_open::open_file,
            BannerButton::Reply => return,
        };
        // Only now, inside the click, may the opened app come to the front.
        toast_activation::allow_foreground_handoff();
        thread::spawn(move || action(&path));
        self.hide(context);
    }

    /// The stale check runs once a minute, and only while a download is tracked.
    fn update_stale_timer(&mut self, context: &mut Context) {
        let has_downloads = self.tracker.active_count() > 0;
        if has_downloads && !self.is_stale_timer_running {
            context.set_timer(STALE_TIMER, STALE_CHECK_MILLISECONDS);
        } else if !has_downloads && self.is_stale_timer_running {
            context.kill_timer(STALE_TIMER);
        }
        self.is_stale_timer_running = has_downloads;
    }
}

impl Activity for DownloadsActivity {
    fn id(&self) -> &'static str {
        "downloads"
    }

    fn is_optional(&self) -> bool {
        true
    }

    fn header_status_rank(&self) -> u8 {
        7
    }

    fn presence(&self) -> Option<Presence> {
        if let (Some(card), true) = (&self.card, self.flash.is_visible()) {
            return Some(Presence::Banner { width: WIDTH, height: card.height(self.shows_buttons()) });
        }
        (self.tracker.active_count() > 0).then_some(Presence::Badge { width: Icon::Download.size() })
    }

    fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        match (&self.card, self.flash.is_visible()) {
            (Some(card), true) => card.draw(canvas, area, opacity, self.shows_buttons()),
            _ => self.draw_header_status(canvas, area, opacity),
        }
    }

    fn header_status_width(&self) -> Option<f32> {
        (self.tracker.active_count() > 0).then_some(Icon::Download.size())
    }

    fn draw_header_status(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        draw_icon(canvas, area, opacity);
    }

    fn dismiss_banner(&mut self, context: &mut Context) {
        self.hide(context);
    }

    fn on_start(&mut self, context: &mut Context) {
        let Some(folder) = downloads_folder::path() else {
            log::debug("downloads: no Downloads folder");
            return;
        };
        self.shared = Some(watcher::start(folder.clone(), context.notifier()));
        self.folder = Some(folder);
    }

    fn on_notify(&mut self, context: &mut Context) {
        let (Some(shared), Some(folder)) = (self.shared.clone(), self.folder.clone()) else { return };
        for event in shared.take_events() {
            for outcome in self.tracker.apply(&event) {
                match react(outcome, &folder) {
                    Reaction::Nothing => {}
                    Reaction::ShowStarted { title } => self.show_started(title, context),
                    Reaction::ShowFinished { path } => self.show_finished(path, context),
                }
            }
        }
        self.update_stale_timer(context);
        context.request_repaint();
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == FLASH_TIMER {
            self.is_pointer_over = false;
            self.flash.on_timer(timer, context);
        } else if timer == STALE_TIMER {
            if let Some(folder) = &self.folder {
                self.tracker
                    .check_stale(|name| std::fs::metadata(folder.join(name)).ok().map(|metadata| metadata.len()));
            }
            self.update_stale_timer(context);
            context.request_repaint();
        }
    }

    /// Leaving the notch is leaving the card, also when the notch had opened meanwhile and the card got no `Left`.
    fn on_open_changed(&mut self, is_open: bool, context: &mut Context) {
        if !is_open {
            self.pointer_left(context);
        }
    }

    fn on_pointer(&mut self, event: PointerEvent, area: Rect, context: &mut Context) {
        if !self.flash.is_visible() || !self.card.as_ref().is_some_and(Card::is_finished) {
            return;
        }
        match event {
            PointerEvent::Moved(_) | PointerEvent::Pressed(_) if !self.is_pointer_over => {
                self.is_pointer_over = true;
                context.kill_timer(FLASH_TIMER);
                context.request_repaint();
            }
            PointerEvent::Left | PointerEvent::CaptureLost => self.pointer_left(context),
            _ => {}
        }
        let Some(card) = &mut self.card else { return };
        let (clicked, needs_repaint) = card.on_pointer(event, area);
        if needs_repaint {
            context.request_repaint();
        }
        if let Some(button) = clicked {
            self.clicked(button, context);
        }
    }
}

#[cfg(test)]
impl DownloadsActivity {
    pub fn sample_started(fonts: &crate::gfx::Fonts) -> Self {
        let mut activity = Self::sample_running();
        let title = "calendar-windows-x64.msi".to_owned();
        activity.card = Some(Card::new(Shown::Started { title }, fonts));
        activity.flash = Flash::visible();
        activity
    }

    pub fn sample_finished(fonts: &crate::gfx::Fonts) -> Self {
        let mut activity = Self::new();
        let shown = Shown::Finished {
            title: "calendar-windows-x64.msi".to_owned(),
            detail: tr!("downloads.done", size = size_text(18_661_376)).into_owned(),
            path: PathBuf::from("C:/Downloads/calendar-windows-x64.msi"),
            icon: file_icon::for_extension("msi"),
        };
        activity.card = Some(Card::new(shown, fonts));
        activity.flash = Flash::visible();
        activity
    }

    pub fn sample_running() -> Self {
        let mut activity = Self::new();
        activity.tracker.apply(&tracker::FolderEvent::Added("Unconfirmed 1.crdownload".to_owned()));
        activity
    }

    pub fn hovered(mut self) -> Self {
        self.is_pointer_over = true;
        self
    }

    pub fn with_uploaded_icon(
        mut self,
        upload: impl FnOnce(&crate::gfx::ImageData) -> Option<crate::gfx::ImageBrush>,
    ) -> Self {
        if let Some(card) = &mut self.card {
            card.upload_icon(upload);
        }
        self
    }
}
