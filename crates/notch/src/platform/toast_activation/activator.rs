//! Finds the COM class an app registered to receive clicks on its notifications (its toast activator).
//! Windows keeps no list of them: packaged apps declare it in their manifest, others in the registry or on their
//! Start menu shortcut, so each place is looked at in turn.

use std::path::{Path, PathBuf};

use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::{ERROR_SUCCESS, PROPERTYKEY},
        System::{
            Com::{
                CoTaskMemFree,
                StructuredStorage::{PROPVARIANT, PropVariantClear, PropVariantToGUID, PropVariantToStringAlloc},
            },
            Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW},
        },
        UI::Shell::PropertiesSystem::{GPS_DEFAULT, IPropertyStore, SHGetPropertyStoreFromParsingName},
    },
    core::{GUID, HSTRING},
};

use super::payload::{application_id, manifest_activator};
use crate::log;

/// `System.AppUserModel.ID` and `System.AppUserModel.ToastActivatorCLSID`, the two shortcut properties read here.
const APP_ID_PROPERTY: PROPERTYKEY =
    PROPERTYKEY { fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3), pid: 5 };
const ACTIVATOR_PROPERTY: PROPERTYKEY =
    PROPERTYKEY { fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3), pid: 26 };
/// The Start menu holds a few hundred shortcuts at most; deeper folders than this are not app shortcuts.
const MAX_FOLDER_DEPTH: usize = 4;

pub fn find(app_user_model_id: &str) -> Option<GUID> {
    from_registry(app_user_model_id)
        .or_else(|| from_manifest(app_user_model_id))
        .or_else(|| from_shortcuts(app_user_model_id))
}

/// `{xxxxxxxx-...}` or the bare form.
fn parse_guid(text: &str) -> Option<GUID> {
    GUID::try_from(text.trim().trim_start_matches('{').trim_end_matches('}')).ok()
}

/// Where the Windows App SDK and others put it for unpackaged apps.
fn from_registry(app_user_model_id: &str) -> Option<GUID> {
    let key = HSTRING::from(format!(r"Software\Classes\AppUserModelId\{app_user_model_id}"));
    let mut buffer = [0u16; 64];
    let mut size = (buffer.len() * 2) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &HSTRING::from("CustomActivator"),
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let length = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
    parse_guid(&String::from_utf16_lossy(&buffer[..length]))
}

fn from_manifest(app_user_model_id: &str) -> Option<GUID> {
    let application = application_id(app_user_model_id)?;
    let app_info = AppInfo::GetFromAppUserModelId(&HSTRING::from(app_user_model_id)).ok()?;
    let folder = app_info.Package().ok()?.InstalledPath().ok()?.to_string();
    let manifest = std::fs::read_to_string(Path::new(&folder).join("AppxManifest.xml")).ok()?;
    parse_guid(&manifest_activator(&manifest, application)?)
}

fn from_shortcuts(app_user_model_id: &str) -> Option<GUID> {
    let start_menus = [
        ("APPDATA", r"Microsoft\Windows\Start Menu\Programs"),
        ("ProgramData", r"Microsoft\Windows\Start Menu\Programs"),
    ];
    let mut shortcuts = Vec::new();
    for (variable, folder) in start_menus {
        if let Ok(root) = std::env::var(variable) {
            collect_shortcuts(&Path::new(&root).join(folder), 0, &mut shortcuts);
        }
    }
    shortcuts.iter().find_map(|path| shortcut_activator(path, app_user_model_id))
}

fn collect_shortcuts(folder: &Path, depth: usize, shortcuts: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < MAX_FOLDER_DEPTH {
                collect_shortcuts(&path, depth + 1, shortcuts);
            }
        } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("lnk")) {
            shortcuts.push(path);
        }
    }
}

fn shortcut_activator(path: &Path, app_user_model_id: &str) -> Option<GUID> {
    let path = HSTRING::from(path.as_os_str());
    let store: IPropertyStore = unsafe { SHGetPropertyStoreFromParsingName(&path, None, GPS_DEFAULT) }.ok()?;
    let matches = read_property(&store, &APP_ID_PROPERTY, |value| {
        let text = unsafe { PropVariantToStringAlloc(value) }.ok()?;
        let matches = unsafe { text.to_string() }.is_ok_and(|id| id.eq_ignore_ascii_case(app_user_model_id));
        unsafe { CoTaskMemFree(Some(text.0 as *const _)) };
        Some(matches)
    })?;
    if !matches {
        return None;
    }
    read_property(&store, &ACTIVATOR_PROPERTY, |value| unsafe { PropVariantToGUID(value) }.ok())
        .filter(|guid| *guid != GUID::zeroed())
}

/// Reads one property and frees the value afterwards, since this `PROPVARIANT` does not free itself.
fn read_property<T>(
    store: &IPropertyStore,
    key: &PROPERTYKEY,
    read: impl FnOnce(&PROPVARIANT) -> Option<T>,
) -> Option<T> {
    let mut value = unsafe { store.GetValue(key) }.ok()?;
    let result = read(&value);
    if let Err(error) = unsafe { PropVariantClear(&mut value) } {
        log::debug(&format!("could not free a shortcut property: {error}"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guid_is_read_with_or_without_braces() {
        let expected = GUID::from_u128(0xbebd6a5e_1b14_4a57_a195_c772d28b6d15);
        assert_eq!(parse_guid("{bebd6a5e-1b14-4a57-a195-c772d28b6d15}"), Some(expected));
        assert_eq!(parse_guid("bebd6a5e-1b14-4a57-a195-c772d28b6d15"), Some(expected));
        assert_eq!(parse_guid("not a guid"), None);
    }
}
