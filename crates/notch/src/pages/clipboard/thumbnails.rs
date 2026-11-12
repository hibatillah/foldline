//! The small pictures in the list rows: one upload per copy and device size, held here instead of the canvas's image
//! cache, which never drops an entry.

use std::{cell::RefCell, rc::Rc};

use super::item::Item;
use super::style::THUMBNAIL_OUTLINE;
use crate::gfx::{Canvas, ImageBrush, ImageData, Mask, Point, Rect};

/// The picture's corner radius in units, the same as the color swatch's.
pub(super) const THUMBNAIL_RADIUS: f32 = 4.0;

struct Thumbnail<Brush> {
    item_id: u64,
    side: u32,
    /// None when the upload failed; kept so a failing picture is not built again on every paint.
    brush: Option<Rc<Brush>>,
}

pub(super) struct Thumbnails<Brush = ImageBrush> {
    entries: RefCell<Vec<Thumbnail<Brush>>>,
    #[cfg(test)]
    builds: std::cell::Cell<usize>,
}

impl<Brush> Default for Thumbnails<Brush> {
    fn default() -> Self {
        Self {
            entries: RefCell::new(Vec::new()),
            #[cfg(test)]
            builds: std::cell::Cell::new(0),
        }
    }
}

impl<Brush> Thumbnails<Brush> {
    /// The uploaded picture of the copy at `side` device pixels, built once; None means the row shows the image icon.
    pub fn get(&self, item_id: u64, side: u32, build: impl FnOnce() -> Option<Brush>) -> Option<Rc<Brush>> {
        let mut entries = self.entries.borrow_mut();
        if let Some(entry) = entries.iter().find(|entry| entry.item_id == item_id && entry.side == side) {
            return entry.brush.clone();
        }
        #[cfg(test)]
        self.builds.set(self.builds.get() + 1);
        // A copy has one size at a time: a new scale replaces its old picture.
        entries.retain(|entry| entry.item_id != item_id);
        let brush = build().map(Rc::new);
        entries.push(Thumbnail { item_id, side, brush: brush.clone() });
        brush
    }

    /// Drops the pictures of copies no longer in the list.
    pub fn keep_only(&self, items: &[Item]) {
        self.entries.borrow_mut().retain(|entry| items.iter().any(|item| item.id == entry.item_id));
    }

    pub fn clear(&mut self) {
        self.entries.get_mut().clear();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.borrow().len()
    }
}

/// The square middle of the picture shrunk to `side` pixels; None for a picture with no pixels.
pub(super) fn thumbnail_pixels(image: &ImageData, side: u32) -> Option<ImageData> {
    if image.width == 0 || image.height == 0 || side == 0 {
        return None;
    }
    Some(image.centered_square().downscaled(side))
}

/// A one pixel ring just inside a rounded square of `side` pixels, so a picture close to the row's color keeps an edge.
fn outline(side: u32, radius: f32) -> ImageData {
    let half = side as f32 / 2.0;
    let radius = radius.min(half);
    let rounded_square = |point: Point| {
        let (x, y) = ((point.x - half).abs() - (half - radius), (point.y - half).abs() - (half - radius));
        x.max(0.0).hypot(y.max(0.0)) + x.max(y).min(0.0) - radius
    };
    let mask = Mask::from_distance(side, side, |point| {
        let distance = rounded_square(point);
        distance.max(-(distance + 1.0))
    });
    mask.tinted(THUMBNAIL_OUTLINE.r, THUMBNAIL_OUTLINE.g, THUMBNAIL_OUTLINE.b)
}

/// Draws the copy's picture as a `size` square centered on `center`, on whole device pixels; false when there is none.
pub(super) fn draw_thumbnail(
    canvas: &Canvas,
    thumbnails: &Thumbnails,
    item_id: u64,
    image: &ImageData,
    center: Point,
    size: f32,
    opacity: f32,
) -> bool {
    let raster_scale = canvas.raster_scale();
    // The square comes with a pixel of margin on each side, which the picture leaves out.
    let (side_with_margin, area) = canvas.pixel_square(center, size);
    let side = side_with_margin.saturating_sub(2);
    let margin = 1.0 / raster_scale;
    let area = Rect::new(area.left + margin, area.top + margin, area.right - margin, area.bottom - margin);
    let build = || thumbnail_pixels(image, side).and_then(|pixels| canvas.upload(&pixels));
    let Some(brush) = thumbnails.get(item_id, side, build) else { return false };
    canvas.image(&brush, area, THUMBNAIL_RADIUS, opacity);
    let device_radius = THUMBNAIL_RADIUS * raster_scale;
    let key = ("clipboard-thumbnail-outline", side, device_radius.to_bits());
    if let Some(ring) = canvas.cached_image(key, || outline(side, device_radius)) {
        canvas.image(&ring, area, 0.0, opacity * THUMBNAIL_OUTLINE.a);
    }
    true
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::pages::clipboard::item::{Content, Kind, StoredImage};

    /// Each pixel's blue byte is its column and its green byte its row.
    fn grid(width: u32, height: u32) -> ImageData {
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&[x as u8, y as u8, 0, 255]);
            }
        }
        ImageData::new(width, height, pixels)
    }

    fn picture_item(id: u64) -> Item {
        let content = Content::Image(StoredImage::new(grid(2, 2), 2, 2, "PNG"));
        Item { id, kind: Kind::Image, content, copied_at: Instant::now() }
    }

    fn first_pixel(image: &ImageData) -> [u8; 2] {
        [image.bgra_premultiplied[0], image.bgra_premultiplied[1]]
    }

    #[test]
    fn a_wide_picture_keeps_its_middle_square() {
        let thumbnail = thumbnail_pixels(&grid(9, 3), 3).unwrap();
        assert_eq!((thumbnail.width, thumbnail.height), (3, 3));
        assert_eq!(first_pixel(&thumbnail), [3, 0]);
    }

    #[test]
    fn a_tall_picture_keeps_its_middle_square() {
        let thumbnail = thumbnail_pixels(&grid(4, 10), 4).unwrap();
        assert_eq!((thumbnail.width, thumbnail.height), (4, 4));
        assert_eq!(first_pixel(&thumbnail), [0, 3]);
    }

    #[test]
    fn an_odd_difference_leaves_the_extra_column_on_the_right() {
        let thumbnail = thumbnail_pixels(&grid(6, 3), 3).unwrap();
        assert_eq!(first_pixel(&thumbnail), [1, 0]);
    }

    #[test]
    fn a_square_picture_is_only_shrunk() {
        let thumbnail = thumbnail_pixels(&grid(38, 38), 19).unwrap();
        assert_eq!((thumbnail.width, thumbnail.height), (19, 19));
    }

    #[test]
    fn a_one_pixel_picture_stays_one_pixel() {
        let thumbnail = thumbnail_pixels(&grid(1, 1), 19).unwrap();
        assert_eq!((thumbnail.width, thumbnail.height), (1, 1));
    }

    #[test]
    fn a_picture_without_pixels_falls_back_to_the_icon() {
        assert!(thumbnail_pixels(&ImageData::new(0, 0, Vec::new()), 19).is_none());
    }

    #[test]
    fn a_thumbnail_is_built_once_per_copy_and_size() {
        let thumbnails = Thumbnails::<u32>::default();
        assert_eq!(thumbnails.get(1, 19, || Some(7)).as_deref(), Some(&7));
        assert_eq!(thumbnails.get(1, 19, || Some(8)).as_deref(), Some(&7));
        thumbnails.get(2, 19, || Some(9));
        assert_eq!(thumbnails.builds.get(), 2);
        thumbnails.get(1, 23, || Some(10));
        assert_eq!((thumbnails.builds.get(), thumbnails.len()), (3, 2), "a new size replaces the copy's old one");
    }

    #[test]
    fn a_failed_upload_shows_the_icon_and_is_not_tried_again() {
        let thumbnails = Thumbnails::<u32>::default();
        assert!(thumbnails.get(1, 19, || None).is_none());
        assert!(thumbnails.get(1, 19, || Some(7)).is_none());
        assert_eq!(thumbnails.builds.get(), 1);
    }

    #[test]
    fn pictures_of_removed_copies_are_dropped() {
        let thumbnails = Thumbnails::<u32>::default();
        for id in 1..=3 {
            thumbnails.get(id, 19, || Some(0));
        }
        thumbnails.keep_only(&[picture_item(2)]);
        assert_eq!(thumbnails.len(), 1);
        assert_eq!(thumbnails.get(2, 19, || Some(5)).as_deref(), Some(&0));
    }

    #[test]
    fn clearing_drops_every_picture() {
        let mut thumbnails = Thumbnails::<u32>::default();
        thumbnails.get(1, 19, || Some(0));
        thumbnails.get(2, 19, || Some(0));
        thumbnails.clear();
        assert_eq!(thumbnails.len(), 0);
    }

    #[test]
    fn the_outline_covers_the_edge_pixels_and_leaves_the_middle_clear() {
        let ring = outline(19, 5.0);
        let alpha = |x: u32, y: u32| ring.bgra_premultiplied[((y * 19 + x) * 4 + 3) as usize];
        assert_eq!(alpha(9, 0), 255);
        assert_eq!(alpha(0, 9), 255);
        assert_eq!(alpha(9, 1), 0);
        assert_eq!(alpha(9, 9), 0);
        assert!(alpha(0, 0) < 64, "the corner is rounded off");
    }
}
