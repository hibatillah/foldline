//! Turns what one copy left on the clipboard into something the history keeps, within its limits.

use super::{
    dib,
    history::{MAX_IMAGE_PIXELS, MAX_TEXT_BYTES},
    item::{Content, StoredImage},
};
use crate::platform::clipboard::Snapshot;

/// The largest bitmap read off the clipboard before it is shrunk: about 4K at 32 bits per pixel.
pub(super) const MAX_DIB_BYTES: usize = 40 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Skip {
    /// The copy asked not to be recorded.
    Excluded,
    TooLarge,
}

pub(super) enum Captured {
    Nothing,
    Skipped(Skip),
    Content(Content),
}

pub(super) fn capture(snapshot: Snapshot) -> Captured {
    if snapshot.is_excluded {
        return Captured::Skipped(Skip::Excluded);
    }
    if snapshot.is_text_too_long || snapshot.is_image_too_large {
        return Captured::Skipped(Skip::TooLarge);
    }
    if let Some(text) = snapshot.text {
        return match text {
            text if text.len() > MAX_TEXT_BYTES => Captured::Skipped(Skip::TooLarge),
            text if text.trim().is_empty() => Captured::Nothing,
            text => Captured::Content(Content::Text(text)),
        };
    }
    let Some(image) = snapshot.dib.as_deref().and_then(dib::decode) else { return Captured::Nothing };
    let (width, height) = (image.width, image.height);
    let pixels = dib::fit_pixels(image, MAX_IMAGE_PIXELS);
    let format = if snapshot.has_png { "PNG" } else { "Bitmap" };
    Captured::Content(Content::Image(StoredImage::new(pixels, width, height, format)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitmap(width: i32, height: i32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&40u32.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(&[0; 24]);
        bytes.extend_from_slice(&vec![7; (width * height * 4) as usize]);
        bytes
    }

    #[test]
    fn a_copy_that_asks_not_to_be_recorded_is_skipped() {
        let snapshot = Snapshot { is_excluded: true, text: Some("secret".into()), ..Snapshot::default() };
        assert!(matches!(capture(snapshot), Captured::Skipped(Skip::Excluded)));
    }

    #[test]
    fn text_over_the_limit_is_skipped_and_blank_text_is_ignored() {
        let long = Snapshot { text: Some("a".repeat(MAX_TEXT_BYTES + 1)), ..Snapshot::default() };
        assert!(matches!(capture(long), Captured::Skipped(Skip::TooLarge)));
        let blank = Snapshot { text: Some(" \n ".into()), ..Snapshot::default() };
        assert!(matches!(capture(blank), Captured::Nothing));
        let at_limit = Snapshot { text: Some("a".repeat(MAX_TEXT_BYTES)), ..Snapshot::default() };
        assert!(matches!(capture(at_limit), Captured::Content(Content::Text(_))));
    }

    #[test]
    fn a_picture_keeps_its_original_size_in_the_details_and_is_shrunk_to_the_pixel_limit() {
        let snapshot = Snapshot { dib: Some(bitmap(1500, 1000)), has_png: true, ..Snapshot::default() };
        let Captured::Content(Content::Image(image)) = capture(snapshot) else { panic!("a picture is kept") };
        assert_eq!((image.original_width, image.original_height, image.format), (1500, 1000, "PNG"));
        assert!(u64::from(image.pixels.width) * u64::from(image.pixels.height) <= MAX_IMAGE_PIXELS);
    }

    #[test]
    fn a_picture_that_cannot_be_read_gives_nothing() {
        let snapshot = Snapshot { dib: Some(vec![1, 2, 3]), ..Snapshot::default() };
        assert!(matches!(capture(snapshot), Captured::Nothing));
    }
}
