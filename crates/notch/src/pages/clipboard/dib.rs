//! Converts between clipboard bitmaps (`CF_DIB`, `CF_DIBV5`) and premultiplied BGRA pictures.

use crate::gfx::ImageData;

const INFO_HEADER_SIZE: usize = 40;
const V5_HEADER_SIZE: usize = 124;
const COMPRESSION_RGB: u32 = 0;
const COMPRESSION_BITFIELDS: u32 = 3;
const COLOR_SPACE_SRGB: u32 = 0x7352_4742;
const INTENT_IMAGES: u32 = 4;
const MAX_SIDE: i64 = 32_768;

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?))
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    read_u32(bytes, offset).map(|value| value as i32)
}

/// Reads a 24 or 32 bit bitmap. Palette and 16 bit bitmaps are not handled and give `None`.
pub(super) fn decode(bytes: &[u8]) -> Option<ImageData> {
    let header_size = read_u32(bytes, 0)? as usize;
    if header_size < INFO_HEADER_SIZE {
        return None;
    }
    let width = i64::from(read_i32(bytes, 4)?);
    let signed_height = i64::from(read_i32(bytes, 8)?);
    let bits = read_u16(bytes, 14)?;
    let compression = read_u32(bytes, 16)?;
    let colors_used = read_u32(bytes, 32)? as usize;
    let height = signed_height.abs();
    let is_supported = matches!(bits, 24 | 32) && matches!(compression, COMPRESSION_RGB | COMPRESSION_BITFIELDS);
    if !is_supported || width <= 0 || height <= 0 || width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    // A 40 byte header with bit fields is followed by three masks that this reader assumes are the usual BGR ones.
    let masks = if compression == COMPRESSION_BITFIELDS && header_size == INFO_HEADER_SIZE { 12 } else { 0 };
    let pixels_start = header_size + masks + colors_used * 4;
    let (width, height) = (width as usize, height as usize);
    let bytes_per_pixel = usize::from(bits / 8);
    let stride = (width * usize::from(bits)).div_ceil(32) * 4;
    let data = bytes.get(pixels_start..pixels_start.checked_add(stride.checked_mul(height)?)?)?;
    let is_top_down = signed_height < 0;
    let mut output = vec![0u8; width * height * 4];
    for row in 0..height {
        let source_row = if is_top_down { row } else { height - 1 - row };
        let source = &data[source_row * stride..source_row * stride + width * bytes_per_pixel];
        let target = &mut output[row * width * 4..(row + 1) * width * 4];
        for (pixel, out) in source.chunks_exact(bytes_per_pixel).zip(target.as_chunks_mut::<4>().0.iter_mut()) {
            out[..3].copy_from_slice(&pixel[..3]);
            out[3] = if bytes_per_pixel == 4 { pixel[3] } else { 255 };
        }
    }
    normalize_alpha(&mut output);
    Some(ImageData::new(width as u32, height as u32, output))
}

/// Many programs write 32 bit bitmaps whose fourth byte is unused and zero; such a picture is opaque. Otherwise the
/// colors are straight and become premultiplied.
fn normalize_alpha(bytes: &mut [u8]) {
    let (pixels, _) = bytes.as_chunks_mut::<4>();
    if pixels.iter().all(|pixel| pixel[3] == 0) {
        pixels.iter_mut().for_each(|pixel| pixel[3] = 255);
        return;
    }
    for pixel in pixels {
        let alpha = u32::from(pixel[3]);
        if alpha == 255 {
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * alpha + 127) / 255) as u8;
        }
    }
}

/// A `CF_DIBV5` structure with straight 32 bit BGRA pixels, bottom row first.
pub(super) fn encode(image: &ImageData) -> Vec<u8> {
    let (width, height) = (image.width as usize, image.height as usize);
    let pixel_bytes = width * height * 4;
    let mut bytes = Vec::with_capacity(V5_HEADER_SIZE + pixel_bytes);
    let mut put = |value: u32| bytes.extend_from_slice(&value.to_le_bytes());
    put(V5_HEADER_SIZE as u32);
    put(image.width);
    put(image.height);
    put(1 | (32 << 16));
    put(COMPRESSION_BITFIELDS);
    put(pixel_bytes as u32);
    put(0);
    put(0);
    put(0);
    put(0);
    for mask in [0x00FF_0000u32, 0x0000_FF00, 0x0000_00FF, 0xFF00_0000] {
        put(mask);
    }
    put(COLOR_SPACE_SRGB);
    // Color space endpoints and gamma are unused for sRGB.
    (0..12).for_each(|_| put(0));
    put(INTENT_IMAGES);
    put(0);
    put(0);
    put(0);
    debug_assert_eq!(bytes.len(), V5_HEADER_SIZE);
    for row in (0..height).rev() {
        for pixel in image.bgra_premultiplied[row * width * 4..(row + 1) * width * 4].as_chunks::<4>().0 {
            let alpha = u32::from(pixel[3]);
            let straight = |channel: u8| {
                (u32::from(channel) * 255 + alpha / 2).checked_div(alpha).map_or(0, |value| value.min(255) as u8)
            };
            bytes.extend_from_slice(&[straight(pixel[0]), straight(pixel[1]), straight(pixel[2]), pixel[3]]);
        }
    }
    bytes
}

/// Shrinks a picture until it has at most `max_pixels` pixels; one that already fits is kept as it is.
pub(super) fn fit_pixels(image: ImageData, max_pixels: u64) -> ImageData {
    let pixels = u64::from(image.width) * u64::from(image.height);
    if pixels <= max_pixels {
        return image;
    }
    let factor = (max_pixels as f64 / pixels as f64).sqrt();
    let target_width = ((f64::from(image.width) * factor).floor() as u32).max(1);
    image.downscaled(target_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info_header(width: i32, height: i32, bits: u16, compression: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&40u32.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&bits.to_le_bytes());
        bytes.extend_from_slice(&compression.to_le_bytes());
        bytes.extend_from_slice(&[0; 20]);
        bytes
    }

    #[test]
    fn a_24_bit_bitmap_is_read_bottom_row_first_with_padded_rows() {
        let mut bytes = info_header(1, 2, 24, 0);
        // Rows are padded to four bytes; the bottom row comes first.
        bytes.extend_from_slice(&[1, 2, 3, 0]);
        bytes.extend_from_slice(&[4, 5, 6, 0]);
        let image = decode(&bytes).unwrap();
        assert_eq!((image.width, image.height), (1, 2));
        assert_eq!(image.bgra_premultiplied, vec![4, 5, 6, 255, 1, 2, 3, 255]);
    }

    #[test]
    fn a_negative_height_means_the_top_row_comes_first() {
        let mut bytes = info_header(1, -2, 24, 0);
        bytes.extend_from_slice(&[1, 2, 3, 0, 4, 5, 6, 0]);
        assert_eq!(decode(&bytes).unwrap().bgra_premultiplied, vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn a_32_bit_bitmap_with_an_unused_fourth_byte_is_opaque() {
        let mut bytes = info_header(1, 1, 32, 0);
        bytes.extend_from_slice(&[10, 20, 30, 0]);
        assert_eq!(decode(&bytes).unwrap().bgra_premultiplied, vec![10, 20, 30, 255]);
    }

    #[test]
    fn a_32_bit_bitmap_with_alpha_becomes_premultiplied() {
        let mut bytes = info_header(2, 1, 32, 3);
        bytes.extend_from_slice(&[0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00]);
        bytes.extend_from_slice(&[200, 100, 50, 128, 1, 2, 3, 255]);
        let image = decode(&bytes).unwrap();
        assert_eq!(image.bgra_premultiplied, vec![100, 50, 25, 128, 1, 2, 3, 255]);
    }

    #[test]
    fn bitmaps_that_are_cut_short_or_use_a_palette_give_nothing() {
        let mut short = info_header(2, 2, 32, 0);
        short.extend_from_slice(&[0; 8]);
        assert!(decode(&short).is_none());
        let mut palette = info_header(1, 1, 8, 0);
        palette.extend_from_slice(&[0; 8]);
        assert!(decode(&palette).is_none());
        assert!(decode(&[1, 2, 3]).is_none());
    }

    #[test]
    fn writing_then_reading_a_picture_gives_the_same_pixels() {
        let pixels = vec![10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 1, 2, 3, 255];
        let image = ImageData::new(2, 2, pixels.clone());
        let encoded = encode(&image);
        assert_eq!(encoded.len(), V5_HEADER_SIZE + 16);
        let decoded = decode(&encoded).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 2));
        assert_eq!(decoded.bgra_premultiplied, pixels);
    }

    #[test]
    fn a_picture_over_the_pixel_limit_is_shrunk_to_fit_and_a_small_one_is_kept() {
        let big = ImageData::new(400, 200, vec![9; 400 * 200 * 4]);
        let fitted = fit_pixels(big, 20_000);
        assert!(u64::from(fitted.width) * u64::from(fitted.height) <= 20_000);
        assert!(fitted.width > 100);
        let small = ImageData::new(4, 4, vec![9; 64]);
        assert_eq!(fit_pixels(small, 20_000).width, 4);
    }
}
