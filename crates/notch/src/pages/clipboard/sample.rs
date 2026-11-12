//! The design's example copies, one scenario per state of the page, for the snapshots.

use std::time::{Duration, Instant};

use super::{
    ClipboardPage,
    item::{Content, StoredImage},
    layout::Target,
    state::{Bar, Confirm},
};
use crate::{gfx::ImageData, pages::clipboard::capture::Skip};

pub(super) const TEXT_SELECTED: u8 = 0;
pub(super) const IMAGE_SELECTED: u8 = 1;
pub(super) const PASTED: u8 = 2;
pub(super) const COPIED: u8 = 3;
pub(super) const REMOVED: u8 = 4;
pub(super) const SKIPPED: u8 = 5;
pub(super) const EMPTY: u8 = 6;
/// Two copies pinned and an unpinned row hovered, which offers its pin.
pub(super) const PINNED: u8 = 7;
/// Three copies pinned and an unpinned row hovered, which offers no pin.
pub(super) const PINNED_FULL: u8 = 8;

const IMAGE_ROW: usize = 4;

fn sample_image() -> ImageData {
    let (width, height) = (360u32, 70u32);
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let amount = (x as f32 / width as f32 + y as f32 / height as f32) / 2.0;
            let channel = |from: f32, to: f32| (from + (to - from) * amount).round() as u8;
            bytes.extend_from_slice(&[channel(127.0, 44.0), channel(161.0, 106.0), channel(233.0, 238.0), 255]);
        }
    }
    ImageData::new(width, height, bytes)
}

impl ClipboardPage {
    /// Seven copies, newest first, as in the design.
    pub fn sample(state: u8) -> Self {
        let mut page = Self::new();
        page.is_enabled = true;
        if state == EMPTY {
            return page;
        }
        let now = Instant::now();
        let text = |value: &str| Content::Text(value.to_owned());
        let copies = [
            (text("Call back Sam after lunch"), 3600 * 3),
            (text("docs.example.com/shortcuts"), 3600),
            (Content::Image(StoredImage::new(sample_image(), 360, 70, "PNG")), 12 * 60),
            (text("#4dabf7"), 8 * 60),
            (text("Meeting moved to Friday 14:00, room B"), 4 * 60),
            (text("github.com/example/notch/issues/42"), 90),
            (text("npm run build -- --watch"), 0),
        ];
        for (content, seconds_ago) in copies {
            let copied_at = now.checked_sub(Duration::from_secs(seconds_ago)).unwrap_or(now);
            page.history.push(content, copied_at);
        }
        match state {
            IMAGE_SELECTED => page.history.select(IMAGE_ROW),
            PASTED => {
                page.history.select(IMAGE_ROW);
                page.confirm = Some(Confirm::Pasted);
            }
            COPIED => {
                page.history.move_to_front(IMAGE_ROW, now);
                page.confirm = Some(Confirm::Copied);
            }
            REMOVED => {
                page.history.select(IMAGE_ROW);
                page.history.remove(1);
                page.bar = Some(Bar::Removed);
            }
            SKIPPED => page.bar = Some(Bar::Skipped(Skip::Excluded)),
            PINNED | PINNED_FULL => {
                page.history.toggle_pin(2);
                page.history.toggle_pin(5);
                if state == PINNED_FULL {
                    page.history.toggle_pin(6);
                }
                page.hover.set(Some(Target::Row(4)));
            }
            _ => {}
        }
        page
    }
}
