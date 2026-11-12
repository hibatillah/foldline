//! Renders a drawing on a transparent surface so tests can compare pixels.

use windows::{
    Win32::{
        Foundation::{HWND, POINT, RECT},
        UI::WindowsAndMessaging::{CreateWindowExW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP},
    },
    core::w,
};

use super::{Canvas, Fonts, Surface};

pub(crate) const SURFACE_WIDTH: i32 = 260;
pub(crate) const SURFACE_HEIGHT: i32 = 60;

pub(crate) fn test_window() -> HWND {
    let style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
    unsafe { CreateWindowExW(style, w!("STATIC"), w!("canvas"), WS_POPUP, 0, 0, 10, 10, None, None, None, None) }
        .unwrap()
}

/// Draws on a transparent surface at `scale` and returns its premultiplied BGRA pixels.
pub(crate) fn render(scale: f32, draw: impl FnOnce(&Canvas)) -> Vec<u8> {
    let fonts = Fonts::new(scale).unwrap();
    let surface = Surface::new(SURFACE_WIDTH, SURFACE_HEIGHT).unwrap();
    let region = RECT { left: 0, top: 0, right: SURFACE_WIDTH, bottom: SURFACE_HEIGHT };
    surface.render_and_present(test_window(), POINT { x: 0, y: 0 }, region, &fonts, scale, draw).unwrap();
    surface.read_pixels().to_vec()
}
