mod paths;
#[cfg(test)]
mod tests;
mod weather;

use self::paths::Paint;
use crate::gfx::{Canvas, Color, ImageData, Mask, Point};

const GRID: f32 = 24.0;
const LINE_WIDTH: f32 = 2.0;
const SIZE: f32 = 18.0;
const LARGE_SIZE: f32 = 22.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Lyrics,
    Previous,
    Play,
    Pause,
    Next,
    Speaker,
    SpeakerMuted,
    Sun,
    Bolt,
    Battery,
    BatteryLow,
    Microphone,
    Camera,
    Copy,
    Bell,
    Clock,
    Folder,
    Calendar,
    MoonStar,
    Bluetooth,
    CapsLock,
    Gear,
    Mouse,
    Keyboard,
    Headphones,
    Gamepad,
    Pen,
    CircleCheck,
    Pin,
    Download,
    CloudLightning,
    Cloudy,
    CloudRain,
    CloudRainWind,
    CloudSunRain,
    CloudSnow,
    CloudMoon,
    CloudMoonRain,
    CloudSun,
    CloudFog,
    /// Lucide "moon"; `MoonStar` is "moon-star".
    Moon,
}

impl Icon {
    /// Preferred edge length in device-independent pixels.
    pub fn size(self) -> f32 {
        match self {
            Icon::Play | Icon::Pause => LARGE_SIZE,
            _ => SIZE,
        }
    }

    /// Center and size are in the canvas's space: device pixels outside `in_units`.
    pub fn draw(self, canvas: &Canvas, center: Point, size: f32, color: Color, opacity: f32) {
        let (side, area) = canvas.pixel_square(center, size);
        let device_size = size * canvas.raster_scale();
        let key = ("icon", self as u8, side, color.rgb_bytes());
        let Some(brush) = canvas.cached_image(key, || self.render(device_size, side, color)) else { return };
        canvas.image(&brush, area, 0.0, opacity * color.a);
    }

    fn render(self, size: f32, side: u32, color: Color) -> ImageData {
        let middle = side as f32 / 2.0;
        let (path, paint) = self.path(Point::new(middle, middle), size);
        let figures = path.flatten();
        let mask = match paint {
            Paint::Fill => Mask::fill(side, side, &figures),
            Paint::Line => Mask::stroke(side, side, &figures, LINE_WIDTH * size / GRID / 2.0),
        };
        mask.tinted(color.r, color.g, color.b)
    }
}
