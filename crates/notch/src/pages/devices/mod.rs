//! Battery of the wireless devices connected to Windows; read when the page opens (unless the last read is under a minute old) and once a minute while it shows.

mod draw;
mod layout;
mod merge;
mod model;
mod ring;
#[cfg(test)]
mod sample;
mod source;
#[cfg(test)]
mod tests;

use self::{
    draw::{draw_message, draw_tiles},
    layout::{CONTENT_HEIGHT, MESSAGE_WIDTH, tiles_width},
    model::{Listing, Prepared},
    source::Reader,
};
use std::time::{Duration, Instant};

use crate::{
    gfx::{Canvas, Size},
    page::{Context, Frame, Mode, Page, PageSizes, Priority, TimerId},
};

const RESTING_WIDTH: f32 = 150.0;
const REFRESH_TIMER: TimerId = TimerId(0);
/// Batteries change slowly, so one read a minute keeps the numbers right without keeping the CPU busy.
const REFRESH_INTERVAL_MILLISECONDS: u32 = 60_000;
/// Each read runs three WinRT enumerations, so opening the page again soon after a read reuses its answer.
const FRESH_FOR: Duration = Duration::from_secs(60);

pub struct DevicesPage {
    /// `None` until the first read finishes; the page draws nothing meanwhile.
    listing: Option<Listing>,
    /// The tiles and their words for `listing`, built when it changes; `None` while there are no devices to show.
    prepared: Option<Prepared>,
    last_read_at: Option<Instant>,
    reader: Reader,
}

impl DevicesPage {
    pub fn new() -> Self {
        Self { listing: None, prepared: None, last_read_at: None, reader: Reader::default() }
    }

    fn set_listing(&mut self, listing: Listing) {
        self.prepared = match &listing {
            Listing::Devices(devices) if !devices.is_empty() => Some(Prepared::new(devices)),
            _ => None,
        };
        self.listing = Some(listing);
    }
}

/// A read is due on open unless the last one is fresh and found devices; a message state is rechecked at once.
fn should_read_on_open(last_read_at: Option<Instant>, has_devices: bool, now: Instant) -> bool {
    match last_read_at {
        Some(read_at) if has_devices => now.saturating_duration_since(read_at) >= FRESH_FOR,
        _ => true,
    }
}

impl Page for DevicesPage {
    fn id(&self) -> &'static str {
        "devices"
    }

    fn name(&self) -> String {
        crate::i18n::tr!("page.devices").into_owned()
    }

    fn is_opt_in(&self) -> bool {
        true
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn uses_units(&self) -> bool {
        true
    }

    fn sizes(&self) -> PageSizes {
        let width = match &self.prepared {
            Some(prepared) => tiles_width(prepared.shown.devices.len()),
            None => MESSAGE_WIDTH,
        };
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(width, CONTENT_HEIGHT) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        match &self.listing {
            None => {}
            Some(Listing::BluetoothOff) => draw_message(
                canvas,
                frame.bounds,
                true,
                &crate::i18n::tr!("devices.bluetooth_off_title"),
                &crate::i18n::tr!("devices.bluetooth_off_detail"),
                opacity,
            ),
            Some(Listing::Devices(devices)) if devices.is_empty() => draw_message(
                canvas,
                frame.bounds,
                false,
                &crate::i18n::tr!("devices.none_title"),
                &crate::i18n::tr!("devices.none_detail"),
                opacity,
            ),
            Some(Listing::Devices(_)) => {
                if let Some(prepared) = &self.prepared {
                    draw_tiles(canvas, prepared, frame.bounds, opacity);
                }
            }
        }
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            if should_read_on_open(self.last_read_at, self.prepared.is_some(), Instant::now()) {
                self.reader.start(context.notifier());
            }
            context.set_timer(REFRESH_TIMER, REFRESH_INTERVAL_MILLISECONDS);
        } else {
            context.kill_timer(REFRESH_TIMER);
        }
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer == REFRESH_TIMER {
            self.reader.start(context.notifier());
        }
    }

    fn on_notify(&mut self, context: &mut Context) {
        if let Some(listing) = self.reader.take() {
            self.last_read_at = Some(Instant::now());
            if self.listing.as_ref() != Some(&listing) {
                self.set_listing(listing);
                context.request_repaint();
            }
        }
    }
}
