//! Finds the icon of an app from its Windows app identifier.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_SUCCESS, PROPERTYKEY, SIZE},
        Graphics::Gdi::HBITMAP,
        System::Com::{CoTaskMemFree, StructuredStorage::PropVariantToStringAlloc},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
        },
        System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW},
        UI::Shell::{
            IShellItem, IShellItemImageFactory,
            PropertiesSystem::{GPS_DEFAULT, SHGetPropertyStoreFromParsingName},
            SHCreateItemFromParsingName, SIIGBF_ICONONLY,
        },
    },
    core::{GUID, HSTRING, Interface, w},
};

use super::{ComApartment, bitmap, window_info};
use crate::gfx::ImageData;

const REQUESTED_PIXELS: i32 = 256;
/// Retain roughly twice the drawn size for smooth downscaling.
const STORED_PIXELS: u32 = 64;

/// Loads the icon of an app, from its Windows app identifier.
/// Packaged apps resolve through the Apps folder; classic apps (such as `Spotify.exe`) through their running process.
pub fn load(app_user_model_id: &str) -> Option<ImageData> {
    if app_user_model_id.is_empty() {
        return None;
    }
    // The shell image factory needs COM initialised on the calling thread.
    let _apartment = ComApartment::enter();
    // An app without an icon of its own can come back as the picture Windows draws for any file; that is no icon.
    let icon = image_for(&format!("shell:AppsFolder\\{app_user_model_id}"))
        .and_then(bitmap::take_pixels)
        .filter(|icon| !is_generic_file_icon(icon))
        .or_else(|| executable_path(app_user_model_id).and_then(|path| image_for(&path)).and_then(bitmap::take_pixels))
        .or_else(|| {
            // An app that registers its toasts under its own id (Firefox) names its icon file in the registry.
            let file = icon_file_from_uri(&registered_icon_uri(app_user_model_id)?)?;
            image_for(&file).and_then(bitmap::take_pixels)
        })
        .or_else(|| {
            // An app installed the classic way with a stub package (Edge) has no icon under its package identity; its main program does.
            let program = largest_executable(Path::new(&package_install_path(app_user_model_id)?))?;
            image_for(&program.to_string_lossy()).and_then(bitmap::take_pixels)
        });
    icon.map(|icon| icon.downscaled(STORED_PIXELS))
}

/// The folder a packaged app is installed in, from its Windows app identifier (`System.AppUserModel.PackageInstallPath`).
pub fn package_install_path(app_user_model_id: &str) -> Option<String> {
    const PACKAGE_INSTALL_PATH: PROPERTYKEY =
        PROPERTYKEY { fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3), pid: 15 };
    let _apartment = ComApartment::enter();
    let store: windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore = unsafe {
        SHGetPropertyStoreFromParsingName(
            &HSTRING::from(format!("shell:AppsFolder\\{app_user_model_id}")),
            None,
            GPS_DEFAULT,
        )
    }
    .ok()?;
    let value = unsafe { store.GetValue(&PACKAGE_INSTALL_PATH) }.ok()?;
    let text = unsafe { PropVariantToStringAlloc(&value) }.ok()?;
    // SAFETY: the call returned a valid, NUL-terminated string that this code owns and frees once.
    let path = unsafe { text.to_string() }.ok().filter(|path| !path.is_empty());
    unsafe { CoTaskMemFree(Some(text.0 as *const _)) };
    path
}

/// The `IconUri` an app registered under `AppUserModelId\<id>`, per user first, then for the machine.
fn registered_icon_uri(app_user_model_id: &str) -> Option<String> {
    let sub_key = HSTRING::from(format!("SOFTWARE\\Classes\\AppUserModelId\\{app_user_model_id}"));
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .find_map(|root| {
            let mut buffer = [0u16; 520];
            let mut bytes = std::mem::size_of_val(&buffer) as u32;
            // SAFETY: `buffer` holds `bytes` bytes and outlives the call.
            let status = unsafe {
                RegGetValueW(
                    root,
                    &sub_key,
                    w!("IconUri"),
                    RRF_RT_REG_SZ,
                    None,
                    Some(buffer.as_mut_ptr().cast()),
                    Some(&mut bytes),
                )
            };
            let length = (bytes as usize / 2).saturating_sub(1);
            (status == ERROR_SUCCESS).then(|| String::from_utf16_lossy(&buffer[..length.min(buffer.len())]))
        })
        .filter(|uri| !uri.is_empty())
}

/// The file an `IconUri` names: a plain path or a `file:///` address; `ms-appx:` and other schemes name nothing readable here.
fn icon_file_from_uri(uri: &str) -> Option<String> {
    let path = match uri.strip_prefix("file:///") {
        Some(rest) => rest.replace('/', "\\"),
        None if uri.contains("://") => return None,
        None => uri.to_string(),
    };
    Path::new(&path).is_file().then_some(path)
}

/// The biggest `.exe` directly inside a folder, which is the app's main program rather than its helpers.
fn largest_executable(directory: &Path) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension.eq_ignore_ascii_case("exe")))
        .filter_map(|entry| Some((entry.metadata().ok()?.len(), entry.path())))
        .max_by_key(|(size, _)| *size)
        .map(|(_, path)| path)
}

/// What Windows draws for a file type it knows nothing about, taken once from an empty probe file in the temp folder.
fn generic_file_icon() -> Option<&'static ImageData> {
    static GENERIC: OnceLock<Option<ImageData>> = OnceLock::new();
    GENERIC
        .get_or_init(|| {
            let probe = std::env::temp_dir().join("foldline-icon-probe.foldlineprobe");
            std::fs::write(&probe, b"").ok()?;
            let icon = image_for(&probe.to_string_lossy()).and_then(bitmap::take_pixels);
            let _ = std::fs::remove_file(&probe);
            icon
        })
        .as_ref()
}

fn is_generic_file_icon(icon: &ImageData) -> bool {
    generic_file_icon().is_some_and(|generic| same_picture(icon, generic))
}

fn same_picture(first: &ImageData, second: &ImageData) -> bool {
    first.width == second.width
        && first.height == second.height
        && first.bgra_premultiplied == second.bgra_premultiplied
}

fn image_for(parsing_name: &str) -> Option<HBITMAP> {
    let item: IShellItem = unsafe { SHCreateItemFromParsingName(&HSTRING::from(parsing_name), None) }.ok()?;
    let factory: IShellItemImageFactory = item.cast().ok()?;
    unsafe { factory.GetImage(SIZE { cx: REQUESTED_PIXELS, cy: REQUESTED_PIXELS }, SIIGBF_ICONONLY) }.ok()
}

fn executable_path(file_name: &str) -> Option<String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
    let mut path = None;
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more && path.is_none() {
        let length = entry.szExeFile.iter().position(|&unit| unit == 0).unwrap_or(entry.szExeFile.len());
        if String::from_utf16_lossy(&entry.szExeFile[..length]).eq_ignore_ascii_case(file_name) {
            path = window_info::process_image_path(entry.th32ProcessID);
        }
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    let _ = unsafe { CloseHandle(snapshot) };
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_icon_uri_names_a_file_only_when_it_is_a_path_or_a_file_address_to_an_existing_file() {
        let existing = std::env::current_exe().unwrap().to_string_lossy().into_owned();
        assert_eq!(icon_file_from_uri(&existing), Some(existing.clone()));
        assert_eq!(icon_file_from_uri(&format!("file:///{}", existing.replace('\\', "/"))), Some(existing));
        assert_eq!(icon_file_from_uri("ms-appx:///Assets/logo.png"), None);
        assert_eq!(icon_file_from_uri(r"C:\no\such\icon.png"), None);
    }

    #[test]
    fn the_main_program_is_the_biggest_executable_in_the_folder() {
        let directory = std::env::temp_dir().join(format!("foldline-largest-exe-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("helper.exe"), [0u8; 10]).unwrap();
        fs::write(directory.join("main.EXE"), [0u8; 100]).unwrap();
        fs::write(directory.join("notes.txt"), [0u8; 1000]).unwrap();
        assert_eq!(largest_executable(&directory), Some(directory.join("main.EXE")));
        assert_eq!(largest_executable(&directory.join("missing")), None);
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn two_images_with_the_same_pixels_are_the_same_picture_whatever_their_versions() {
        let first = ImageData::new(1, 1, vec![1, 2, 3, 4]);
        let second = ImageData::new(1, 1, vec![1, 2, 3, 4]);
        assert!(same_picture(&first, &second));
    }

    #[test]
    fn images_that_differ_in_a_pixel_or_a_size_are_different_pictures() {
        let image = ImageData::new(1, 1, vec![1, 2, 3, 4]);
        assert!(!same_picture(&image, &ImageData::new(1, 1, vec![1, 2, 3, 5])));
        assert!(!same_picture(&image, &ImageData::new(2, 1, vec![1, 2, 3, 4, 0, 0, 0, 0])));
    }
}
