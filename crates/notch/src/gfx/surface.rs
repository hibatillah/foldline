use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, POINT, RECT, SIZE},
        Graphics::{
            Direct2D::{
                Common::{D2D_RECT_F, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT},
                D2D1_ANTIALIAS_MODE_ALIASED, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
                D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
                D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE, D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory,
                ID2D1SolidColorBrush,
            },
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            Gdi::{
                AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC,
                CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, HBITMAP, HDC, HGDIOBJ, SelectObject,
            },
        },
        UI::WindowsAndMessaging::{ULW_ALPHA, UpdateLayeredWindow},
    },
    core::Result,
};

use super::{
    canvas::Canvas,
    fonts::Fonts,
    image_brush::{self, ImageBrush, ImageCache},
    image_data::ImageData,
};

/// Off-screen premultiplied-alpha bitmap that Direct2D draws into and a layered window displays.
pub struct Surface {
    pub width: i32,
    pub height: i32,
    memory_context: HDC,
    bitmap: HBITMAP,
    /// The bitmap's pixels, premultiplied BGRA, `width * height * 4` bytes.
    #[cfg(test)]
    pixels: *const u8,
    previous_object: HGDIOBJ,
    target: ID2D1DCRenderTarget,
    brush: ID2D1SolidColorBrush,
    image_cache: ImageCache,
}

impl Surface {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let factory: ID2D1Factory = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }?;
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            // 96 dpi keeps one drawing unit equal to one pixel; callers apply display scaling themselves.
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_GDI_COMPATIBLE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let target = unsafe { factory.CreateDCRenderTarget(&properties) }?;
        let brush = unsafe { target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, None) }?;

        let memory_context = unsafe { CreateCompatibleDC(None) };
        let (bitmap, _pixel_pointer) = match create_bitmap(memory_context, width, height) {
            Ok(created) => created,
            Err(error) => {
                let _ = unsafe { DeleteDC(memory_context) };
                return Err(error);
            }
        };
        let previous_object = unsafe { SelectObject(memory_context, bitmap.into()) };

        Ok(Self {
            width,
            height,
            memory_context,
            bitmap,
            #[cfg(test)]
            pixels: _pixel_pointer.cast::<u8>(),
            previous_object,
            target,
            brush,
            image_cache: ImageCache::default(),
        })
    }

    /// Replaces the bitmap with one of the new size. The render target stays, so brushes uploaded through it stay
    /// valid; `render_and_present` binds the memory context again on every frame. A failure leaves the old size.
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        let (bitmap, _pixel_pointer) = create_bitmap(self.memory_context, width, height)?;
        // SAFETY: the memory context is ours, and selecting the new bitmap returns the old one, which is then unused.
        let old_bitmap = unsafe { SelectObject(self.memory_context, bitmap.into()) };
        let _ = unsafe { DeleteObject(old_bitmap) };
        self.bitmap = bitmap;
        #[cfg(test)]
        {
            self.pixels = _pixel_pointer.cast::<u8>();
        }
        self.width = width;
        self.height = height;
        // A new size means a new scale: images cached at the old device size would only take memory.
        self.image_cache.clear();
        Ok(())
    }

    /// The pixels as last drawn, premultiplied BGRA; only used to render the notch off-screen for review.
    #[cfg(test)]
    pub fn read_pixels(&self) -> &[u8] {
        // SAFETY: the pixels belong to the DIB section, which lives as long as the surface.
        unsafe { std::slice::from_raw_parts(self.pixels, self.width as usize * self.height as usize * 4) }
    }

    pub fn create_image_brush(&self, image: &ImageData) -> Result<ImageBrush> {
        image_brush::upload(&self.target, image)
    }

    /// Redraws only `region` of the bitmap, then shows just that region as the layered window.
    /// The window shrinks and grows with the pill, so the system composes far fewer pixels per frame.
    pub fn render_and_present(
        &self,
        window: HWND,
        screen_position: POINT,
        region: RECT,
        fonts: &Fonts,
        scale: f32,
        draw: impl FnOnce(&Canvas),
    ) -> Result<()> {
        let bounds = RECT { left: 0, top: 0, right: self.width, bottom: self.height };
        let clip = D2D_RECT_F {
            left: region.left as f32,
            top: region.top as f32,
            right: region.right as f32,
            bottom: region.bottom as f32,
        };
        unsafe {
            self.target.BindDC(self.memory_context, &bounds)?;
            self.target.BeginDraw();
            self.target.PushAxisAlignedClip(&clip, D2D1_ANTIALIAS_MODE_ALIASED);
            self.target.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
        }
        draw(&Canvas::new(&self.target, &self.brush, &self.image_cache, fonts, scale));
        unsafe {
            self.target.PopAxisAlignedClip();
            self.target.EndDraw(None, None)?;
        }

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        // SAFETY: the memory context holds the bitmap drawn above and stays alive for the call.
        unsafe {
            UpdateLayeredWindow(
                window,
                None,
                Some(&POINT { x: screen_position.x + region.left, y: screen_position.y + region.top }),
                Some(&SIZE { cx: region.right - region.left, cy: region.bottom - region.top }),
                Some(self.memory_context),
                Some(&POINT { x: region.left, y: region.top }),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            )
        }
    }
}

fn create_bitmap(memory_context: HDC, width: i32, height: i32) -> Result<(HBITMAP, *mut std::ffi::c_void)> {
    let bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            // A negative height makes the rows run top-down, like Direct2D expects.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixel_pointer = std::ptr::null_mut();
    let bitmap =
        unsafe { CreateDIBSection(Some(memory_context), &bitmap_info, DIB_RGB_COLORS, &mut pixel_pointer, None, 0) }?;
    Ok((bitmap, pixel_pointer))
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.memory_context, self.previous_object);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.memory_context);
        }
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Threading::{GR_GDIOBJECTS, GetCurrentProcess, GetGuiResources};

    use super::*;
    use crate::gfx::{color::Color, geometry::Rect, test_support::test_window};

    const SCALE: f32 = 1.0;

    fn fill_all(surface: &Surface, color: Color) {
        let fonts = Fonts::new(SCALE).unwrap();
        let region = RECT { left: 0, top: 0, right: surface.width, bottom: surface.height };
        let bounds = Rect::new(0.0, 0.0, surface.width as f32, surface.height as f32);
        surface
            .render_and_present(test_window(), POINT { x: 0, y: 0 }, region, &fonts, SCALE, |canvas| {
                canvas.fill_rounded_rect(bounds, 0.0, color);
            })
            .unwrap();
    }

    fn pixel_at(surface: &Surface, x: i32, y: i32) -> [u8; 4] {
        let offset = ((y * surface.width + x) * 4) as usize;
        surface.read_pixels()[offset..offset + 4].try_into().unwrap()
    }

    fn gdi_object_count() -> u32 {
        unsafe { GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS) }
    }

    #[test]
    fn a_surface_resized_larger_draws_into_its_whole_new_area() {
        let mut surface = Surface::new(40, 20).unwrap();
        fill_all(&surface, Color::rgb(1.0, 0.0, 0.0));
        surface.resize(120, 70).unwrap();
        fill_all(&surface, Color::rgb(0.0, 1.0, 0.0));

        assert_eq!(surface.read_pixels().len(), 120 * 70 * 4);
        assert_eq!(pixel_at(&surface, 119, 69), [0, 255, 0, 255]);
    }

    #[test]
    fn a_surface_resized_smaller_draws_into_its_new_area_and_keeps_the_row_length() {
        let mut surface = Surface::new(120, 70).unwrap();
        fill_all(&surface, Color::rgb(1.0, 0.0, 0.0));
        surface.resize(40, 20).unwrap();
        fill_all(&surface, Color::rgb(0.0, 0.0, 1.0));

        assert_eq!(surface.read_pixels().len(), 40 * 20 * 4);
        assert_eq!(pixel_at(&surface, 39, 19), [255, 0, 0, 255]);
    }

    #[test]
    fn an_image_brush_uploaded_before_a_resize_still_draws_after_it() {
        let mut surface = Surface::new(40, 20).unwrap();
        // Opaque green, premultiplied BGRA.
        let image = ImageData::new(2, 2, [0, 255, 0, 255].repeat(4));
        let brush = surface.create_image_brush(&image).unwrap();
        surface.resize(90, 50).unwrap();

        let fonts = Fonts::new(SCALE).unwrap();
        let region = RECT { left: 0, top: 0, right: 90, bottom: 50 };
        let bounds = Rect::new(0.0, 0.0, 90.0, 50.0);
        surface
            .render_and_present(test_window(), POINT { x: 0, y: 0 }, region, &fonts, SCALE, |canvas| {
                canvas.image(&brush, bounds, 0.0, 1.0);
            })
            .unwrap();

        assert_eq!(pixel_at(&surface, 80, 40), [0, 255, 0, 255]);
    }

    #[test]
    fn resizing_the_surface_many_times_does_not_grow_the_gdi_object_count() {
        let mut surface = Surface::new(100, 40).unwrap();
        let before = gdi_object_count();
        for cycle in 0..200 {
            let (width, height) = if cycle % 2 == 0 { (150, 60) } else { (100, 40) };
            surface.resize(width, height).unwrap();
        }
        // Other tests create and free GDI objects at the same time; a leak stays, their objects go.
        let mut after = gdi_object_count();
        for _ in 0..50 {
            if after <= before + 2 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
            after = gdi_object_count();
        }

        assert!(after <= before + 2, "GDI objects grew from {before} to {after}");
    }

    #[test]
    fn a_failed_resize_keeps_the_old_size_and_bitmap() {
        let mut surface = Surface::new(40, 20).unwrap();
        let result = surface.resize(i32::MAX, i32::MAX);

        assert!(result.is_err());
        assert_eq!((surface.width, surface.height), (40, 20));
    }

    fn cache_one_image(surface: &Surface) {
        let image = ImageData::new(2, 2, [0, 255, 0, 255].repeat(4));
        surface.image_cache.get_or_insert(7, || surface.create_image_brush(&image).ok());
    }

    #[test]
    fn a_resize_to_a_new_size_drops_the_images_cached_at_the_old_size() {
        let mut surface = Surface::new(40, 20).unwrap();
        cache_one_image(&surface);
        surface.resize(60, 30).unwrap();
        assert_eq!(surface.image_cache.len(), 0);
    }

    #[test]
    fn a_resize_to_the_same_size_keeps_the_cached_images() {
        let mut surface = Surface::new(40, 20).unwrap();
        cache_one_image(&surface);
        surface.resize(40, 20).unwrap();
        assert_eq!(surface.image_cache.len(), 1);
    }
}
