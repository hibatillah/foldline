use std::{cell::RefCell, collections::HashMap, rc::Rc};

use windows::{
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT},
            D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES, ID2D1BitmapBrush, ID2D1DCRenderTarget,
        },
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
    core::Result,
};

use super::image_data::ImageData;

pub struct ImageBrush {
    brush: ID2D1BitmapBrush,
    pub width: u32,
}

impl ImageBrush {
    pub(super) fn brush(&self) -> &ID2D1BitmapBrush {
        &self.brush
    }
}

pub(super) fn upload(target: &ID2D1DCRenderTarget, image: &ImageData) -> Result<ImageBrush> {
    let properties = D2D1_BITMAP_PROPERTIES {
        pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
        dpiX: 96.0,
        dpiY: 96.0,
    };
    // SAFETY: the pixel pointer is valid for `height * width * 4` bytes, and Direct2D copies it before returning.
    let bitmap = unsafe {
        target.CreateBitmap(
            D2D_SIZE_U { width: image.width, height: image.height },
            Some(image.bgra_premultiplied.as_ptr().cast()),
            image.width * 4,
            &properties,
        )
    }?;
    let brush = unsafe { target.CreateBitmapBrush(&bitmap, None, None) }?;
    unsafe { brush.SetInterpolationMode(D2D1_BITMAP_INTERPOLATION_MODE_LINEAR) };
    Ok(ImageBrush { brush, width: image.width })
}

#[derive(Default)]
pub(super) struct ImageCache {
    entries: RefCell<HashMap<u64, Rc<ImageBrush>>>,
}

impl ImageCache {
    pub(super) fn get_or_insert(&self, key: u64, build: impl FnOnce() -> Option<ImageBrush>) -> Option<Rc<ImageBrush>> {
        if let Some(brush) = self.entries.borrow().get(&key) {
            return Some(Rc::clone(brush));
        }
        let brush = Rc::new(build()?);
        self.entries.borrow_mut().insert(key, Rc::clone(&brush));
        Some(brush)
    }

    /// Keys hold the device size, so after a scale change the old entries are never hit again.
    pub(super) fn clear(&self) {
        self.entries.borrow_mut().clear();
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.borrow().len()
    }
}

/// Keeps an uploaded copy of an image that changes over time, re-uploading only when the image is replaced.
#[derive(Default)]
pub struct ImageSlot {
    version: u64,
    brush: Option<ImageBrush>,
}

impl ImageSlot {
    pub fn brush(&self) -> Option<&ImageBrush> {
        self.brush.as_ref()
    }

    pub fn sync(&mut self, source: Option<&ImageData>, upload: impl FnOnce(&ImageData) -> Option<ImageBrush>) {
        let version = source.map_or(0, |image| image.version);
        if version == self.version {
            return;
        }
        self.version = version;
        self.brush = source.and_then(upload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> ImageData {
        ImageData::new(1, 1, vec![0; 4])
    }

    #[test]
    fn a_slot_uploads_only_when_the_image_changes() {
        let (first, second) = (image(), image());
        let mut slot = ImageSlot::default();
        let mut uploads = 0;

        slot.sync(Some(&first), |_| {
            uploads += 1;
            None
        });
        slot.sync(Some(&first), |_| {
            uploads += 1;
            None
        });
        assert_eq!(uploads, 1, "the same image must not be uploaded twice");

        slot.sync(Some(&second), |_| {
            uploads += 1;
            None
        });
        assert_eq!(uploads, 2);

        slot.sync(None, |_| {
            uploads += 1;
            None
        });
        assert_eq!(uploads, 2, "clearing needs no upload");
        assert!(slot.brush().is_none());
    }
}
