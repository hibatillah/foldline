//! The icon Windows shows for a file extension, read without opening any file.

use windows::{
    Win32::{
        Graphics::Gdi::DeleteObject,
        Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        UI::{
            Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES, SHGetFileInfoW},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO},
        },
    },
    core::HSTRING,
};

use super::{ComApartment, bitmap};
use crate::gfx::ImageData;

/// The large (32 px) icon for `.{extension}` as premultiplied BGRA; an empty extension gives the generic file icon.
pub fn for_extension(extension: &str) -> Option<ImageData> {
    let extension = extension.trim_start_matches('.');
    // The shell looks only at the name, so a placeholder name stands in for "any file".
    let placeholder_name = if extension.is_empty() { "file".to_owned() } else { format!("file.{extension}") };
    let _apartment = ComApartment::enter();
    let mut info = SHFILEINFOW::default();
    let found = unsafe {
        SHGetFileInfoW(
            &HSTRING::from(placeholder_name),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
        )
    };
    if found == 0 || info.hIcon.is_invalid() {
        return None;
    }
    let image = icon_pixels(info.hIcon);
    // SAFETY: the shell gave this icon to the caller, and nothing else holds it.
    let _ = unsafe { DestroyIcon(info.hIcon) };
    image
}

/// Reads the icon's colour bitmap; the icon itself stays the caller's to destroy.
fn icon_pixels(icon: HICON) -> Option<ImageData> {
    let mut parts = ICONINFO::default();
    unsafe { GetIconInfo(icon, &mut parts) }.ok()?;
    // The mask bitmap is not needed (the colour bitmap carries its alpha) but still has to be released.
    if !parts.hbmMask.is_invalid() {
        let _ = unsafe { DeleteObject(parts.hbmMask.into()) };
    }
    if parts.hbmColor.is_invalid() {
        return None;
    }
    bitmap::take_pixels(parts.hbmColor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_extension_gives_a_square_icon_with_visible_pixels() {
        let icon = for_extension("txt").expect("a .txt icon");
        assert_eq!((icon.width, icon.height), (32, 32));
        assert!(icon.bgra_premultiplied.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0));
    }

    #[test]
    fn an_unknown_extension_still_gives_the_generic_icon() {
        assert!(for_extension("zzqqx").is_some());
    }
}
