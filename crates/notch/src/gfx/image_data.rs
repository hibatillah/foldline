use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_VERSION: AtomicU64 = AtomicU64::new(1);

/// Decoded image pixels in premultiplied BGRA, ready to upload. Every image gets a unique `version`.
pub struct ImageData {
    pub version: u64,
    pub width: u32,
    pub height: u32,
    pub bgra_premultiplied: Vec<u8>,
}

impl ImageData {
    pub fn new(width: u32, height: u32, bgra_premultiplied: Vec<u8>) -> Self {
        Self { version: NEXT_VERSION.fetch_add(1, Ordering::Relaxed), width, height, bgra_premultiplied }
    }

    pub fn centered_square(&self) -> ImageData {
        let side = self.width.min(self.height);
        let (left, top) = ((self.width - side) / 2, (self.height - side) / 2);
        let row_bytes = (side * 4) as usize;
        let mut pixels = Vec::with_capacity(row_bytes * side as usize);
        for row in top..top + side {
            let start = ((row * self.width + left) * 4) as usize;
            pixels.extend_from_slice(&self.bgra_premultiplied[start..start + row_bytes]);
        }
        ImageData::new(side, side, pixels)
    }

    /// Average source pixels before shrinking; renderer downscaling skips pixels and looks jagged.
    pub fn downscaled(&self, target_width: u32) -> ImageData {
        let target_width = target_width.max(1);
        if target_width >= self.width {
            return ImageData::new(self.width, self.height, self.bgra_premultiplied.clone());
        }
        let ratio_x = self.width as f32 / target_width as f32;
        let target_height = ((self.height as f32 / ratio_x).round() as u32).max(1);
        let ratio_y = self.height as f32 / target_height as f32;

        let mut pixels = vec![0u8; (target_width * target_height * 4) as usize];
        for target_y in 0..target_height {
            let (top, bottom) = (target_y as f32 * ratio_y, (target_y + 1) as f32 * ratio_y);
            for target_x in 0..target_width {
                let (left, right) = (target_x as f32 * ratio_x, (target_x + 1) as f32 * ratio_x);
                let mut sum = [0.0f32; 4];
                let mut total_weight = 0.0f32;
                for source_y in top.floor() as u32..(bottom.ceil() as u32).min(self.height) {
                    let weight_y = (bottom.min((source_y + 1) as f32) - top.max(source_y as f32)).max(0.0);
                    for source_x in left.floor() as u32..(right.ceil() as u32).min(self.width) {
                        let weight = weight_y * (right.min((source_x + 1) as f32) - left.max(source_x as f32)).max(0.0);
                        let index = ((source_y * self.width + source_x) * 4) as usize;
                        for (channel, total) in sum.iter_mut().enumerate() {
                            *total += f32::from(self.bgra_premultiplied[index + channel]) * weight;
                        }
                        total_weight += weight;
                    }
                }
                let index = ((target_y * target_width + target_x) * 4) as usize;
                for (channel, total) in sum.iter().enumerate() {
                    pixels[index + channel] = (total / total_weight.max(f32::EPSILON)).round() as u8;
                }
            }
        }
        ImageData::new(target_width, target_height, pixels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(width: u32, height: u32) -> ImageData {
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&[x as u8, y as u8, 0, 255]);
            }
        }
        ImageData::new(width, height, pixels)
    }

    #[test]
    fn shrinking_averages_the_pixels_under_each_new_pixel() {
        let pixels = vec![0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255];
        let shrunk = ImageData::new(2, 2, pixels).downscaled(1);
        assert_eq!((shrunk.width, shrunk.height), (1, 1));
        assert_eq!(shrunk.bgra_premultiplied, vec![128, 128, 128, 255]);
    }

    #[test]
    fn shrinking_keeps_a_flat_color_unchanged() {
        let flat: Vec<u8> = [10, 20, 30, 255].repeat(36 * 36);
        let shrunk = ImageData::new(36, 36, flat).downscaled(10);
        assert_eq!((shrunk.width, shrunk.height), (10, 10));
        assert!(shrunk.bgra_premultiplied.as_chunks::<4>().0.iter().all(|pixel| *pixel == [10, 20, 30, 255]));
    }

    #[test]
    fn an_image_already_small_enough_keeps_its_size() {
        assert_eq!(gradient(1, 1).downscaled(8).width, 1);
    }

    #[test]
    fn every_image_gets_its_own_version() {
        assert_ne!(gradient(1, 1).version, gradient(1, 1).version);
    }

    #[test]
    fn a_wide_image_loses_its_left_and_right_edges() {
        let square = gradient(8, 4).centered_square();
        assert_eq!((square.width, square.height), (4, 4));
        assert_eq!(&square.bgra_premultiplied[0..2], &[2, 0]);
    }

    #[test]
    fn a_tall_image_loses_its_top_and_bottom_edges() {
        let square = gradient(4, 8).centered_square();
        assert_eq!((square.width, square.height), (4, 4));
        assert_eq!(&square.bgra_premultiplied[0..2], &[0, 2]);
    }

    #[test]
    fn a_square_image_is_unchanged() {
        let square = gradient(4, 4).centered_square();
        assert_eq!(square.bgra_premultiplied, gradient(4, 4).bgra_premultiplied);
    }
}
