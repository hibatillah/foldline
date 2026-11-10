//! Drawing toolkit shared by every page: geometry, colors, fonts, images, springs and the canvas.

mod canvas;
mod color;
mod fonts;
mod geometry;
mod image_brush;
mod image_data;
mod lines;
mod path;
mod pixel_square;
mod raster;
mod spring;
mod surface;
mod svg_path;
#[cfg(test)]
pub(crate) mod test_support;
mod text_cache;

pub use canvas::Canvas;
pub use color::Color;
pub use fonts::{Align, FontStyle, Fonts, UI_FONT, Weight};
pub use geometry::{Point, Rect, Size, lerp};
pub use image_brush::{ImageBrush, ImageSlot};
pub use image_data::ImageData;
pub use lines::{balance_lines, clamp_lines};
pub use path::Path;
pub use raster::Mask;
pub use spring::Spring;
pub use surface::Surface;
