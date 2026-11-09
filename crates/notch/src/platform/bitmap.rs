//! The pixels of a GDI bitmap the shell hands out, as premultiplied BGRA.

use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits, GetObjectW, HBITMAP,
    ReleaseDC,
};

use crate::gfx::ImageData;

/// Reads the bitmap's pixels and releases it.
pub(super) fn take_pixels(bitmap: HBITMAP) -> Option<ImageData> {
    let image = pixels(bitmap);
    let _ = unsafe { DeleteObject(bitmap.into()) };
    image
}

fn pixels(bitmap: HBITMAP) -> Option<ImageData> {
    let mut header = BITMAP::default();
    let described =
        unsafe { GetObjectW(bitmap.into(), size_of::<BITMAP>() as i32, Some((&raw mut header).cast::<c_void>())) };
    let (width, height) = (header.bmWidth, header.bmHeight);
    if described == 0 || width <= 0 || height <= 0 {
        return None;
    }
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            // Negative height: the rows run from the top.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bytes = vec![0u8; width as usize * height as usize * 4];
    let screen = unsafe { GetDC(None) };
    let rows = unsafe {
        GetDIBits(screen, bitmap, 0, height as u32, Some(bytes.as_mut_ptr().cast()), &mut info, DIB_RGB_COLORS)
    };
    unsafe { ReleaseDC(None, screen) };
    if rows == 0 {
        return None;
    }
    premultiplied(&mut bytes);
    Some(ImageData::new(width as u32, height as u32, bytes))
}

/// Shell bitmaps may lack alpha or use straight alpha; Direct2D needs premultiplied BGRA.
fn premultiplied(bytes: &mut [u8]) {
    let (pixels, _) = bytes.as_chunks_mut::<4>();
    if pixels.iter().all(|pixel| pixel[3] == 0) {
        pixels.iter_mut().for_each(|pixel| pixel[3] = 255);
        return;
    }
    let is_premultiplied = pixels.iter().all(|pixel| pixel[..3].iter().all(|&channel| channel <= pixel[3]));
    if is_premultiplied {
        return;
    }
    for pixel in pixels {
        let alpha = u32::from(pixel[3]);
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_with_no_alpha_become_opaque() {
        let mut bytes = vec![10, 20, 30, 0, 40, 50, 60, 0];
        premultiplied(&mut bytes);
        assert_eq!(bytes, vec![10, 20, 30, 255, 40, 50, 60, 255]);
    }

    #[test]
    fn pixels_already_multiplied_by_their_alpha_stay_as_they_are() {
        let mut bytes = vec![10, 20, 30, 128, 0, 0, 0, 0];
        premultiplied(&mut bytes);
        assert_eq!(bytes, vec![10, 20, 30, 128, 0, 0, 0, 0]);
    }

    #[test]
    fn pixels_brighter_than_their_alpha_are_multiplied() {
        let mut bytes = vec![200, 100, 0, 128];
        premultiplied(&mut bytes);
        assert_eq!(bytes, vec![100, 50, 0, 128]);
    }
}
