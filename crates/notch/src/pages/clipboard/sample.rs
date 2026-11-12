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

/// The wide screenshot's row in both lists.
const IMAGE_ROW: usize = 4;

/// Premultiplied BGRA pixels from `paint(x, y)`, which returns red, green, blue and alpha bytes.
fn picture(width: u32, height: u32, paint: impl Fn(u32, u32) -> [u8; 4]) -> ImageData {
    let mut bytes = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let [red, green, blue, alpha] = paint(x, y);
            let premultiply = |channel: u8| (u32::from(channel) * u32::from(alpha) / 255) as u8;
            bytes.extend_from_slice(&[premultiply(blue), premultiply(green), premultiply(red), alpha]);
        }
    }
    ImageData::new(width, height, bytes)
}

fn image(pixels: ImageData) -> Content {
    let (width, height) = (pixels.width, pixels.height);
    Content::Image(StoredImage::new(pixels, width, height, "PNG"))
}

/// A wide window: a gray title bar, a white page and lines of dark and blue text.
fn screenshot() -> ImageData {
    picture(360, 70, |x, y| match (y, x % 60) {
        (0..12, _) => [222, 222, 228, 255],
        (20..24 | 34..38 | 48..52, 6..50) if x < 300 => [40, 40, 48, 255],
        (27..31, 6..40) => [34, 139, 230, 255],
        _ => [255, 255, 255, 255],
    })
}

/// A page of text: almost all white with a few light gray lines.
fn near_white_page() -> ImageData {
    picture(
        240,
        180,
        |x, y| if y % 20 < 3 && (24..216).contains(&x) { [214, 214, 218, 255] } else { [250, 250, 248, 255] },
    )
}

/// A terminal: almost all black with a few dim green lines.
fn near_black_terminal() -> ImageData {
    picture(200, 150, |x, y| if y % 16 < 3 && x < 120 { [40, 90, 60, 255] } else { [10, 10, 12, 255] })
}

/// A sunset: orange to purple from top to bottom with a yellow sun.
fn photo() -> ImageData {
    picture(320, 240, |x, y| {
        let amount = y as f32 / 240.0;
        let mix = |from: f32, to: f32| (from + (to - from) * amount).round() as u8;
        let (dx, dy) = (x as f32 - 200.0, y as f32 - 110.0);
        if dx.hypot(dy) < 40.0 {
            [255, 214, 90, 255]
        } else {
            [mix(255.0, 70.0), mix(140.0, 40.0), mix(60.0, 140.0), 255]
        }
    })
}

/// A logo: a blue disc on a transparent background.
fn logo() -> ImageData {
    picture(128, 128, |x, y| {
        let distance = (x as f32 - 63.5).hypot(y as f32 - 63.5);
        let coverage = (44.0 - distance).clamp(0.0, 1.0);
        [77, 171, 247, (coverage * 255.0).round() as u8]
    })
}

impl ClipboardPage {
    /// Seven copies, newest first, as in the design; the picture states list five kinds of picture.
    pub fn sample(state: u8) -> Self {
        let mut page = Self::new();
        page.is_enabled = true;
        if state == EMPTY {
            return page;
        }
        let now = Instant::now();
        let text = |value: &str| Content::Text(value.to_owned());
        let copies = if matches!(state, TEXT_SELECTED | SKIPPED) {
            [
                (text("Call back Sam after lunch"), 3600 * 3),
                (text("docs.example.com/shortcuts"), 3600),
                (image(screenshot()), 12 * 60),
                (text("#4dabf7"), 8 * 60),
                (text("Meeting moved to Friday 14:00, room B"), 4 * 60),
                (text("github.com/example/notch/issues/42"), 90),
                (text("npm run build -- --watch"), 0),
            ]
        } else {
            [
                (text("#4dabf7"), 3600 * 3),
                (image(near_black_terminal()), 3600),
                (image(screenshot()), 12 * 60),
                (image(logo()), 8 * 60),
                (image(photo()), 4 * 60),
                (image(near_white_page()), 90),
                (text("npm run build -- --watch"), 0),
            ]
        };
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
