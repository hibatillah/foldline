//! The connected screens and the identifier the notch and the settings window store for a chosen one.

use windows::{
    Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{
            DISPLAY_DEVICEW, EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
            MONITORINFOEXW,
        },
    },
    core::{BOOL, PCWSTR},
};

const MONITORINFOF_PRIMARY: u32 = 1;
// The `windows` crate does not export this `EnumDisplayDevicesW` flag.
const EDD_GET_DEVICE_INTERFACE_NAME: u32 = 1;
// Windows' stand-in name when no driver names the monitor; it says nothing to a user.
const GENERIC_MODEL: &str = "Generic PnP Monitor";

/// A connected screen as the settings window lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    pub id: String,
    pub model: Option<String>,
    pub is_primary: bool,
    /// The top-left corner on the virtual desktop; the primary screen's is 0, 0 and others can be negative.
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

/// What names a monitor: the stored identifier and the model Windows gives, when it gives one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenIdentity {
    pub id: String,
    pub model: Option<String>,
}

/// The device interface path when Windows gives one, else the GDI device name, which Windows can renumber.
fn pick_id(device_interface_path: &str, gdi_device_name: &str) -> String {
    if device_interface_path.is_empty() { gdi_device_name.to_owned() } else { device_interface_path.to_owned() }
}

fn model_of(device_string: &str) -> Option<String> {
    let name = device_string.trim();
    (!name.is_empty() && name != GENERIC_MODEL).then(|| name.to_owned())
}

fn text_of(units: &[u16]) -> String {
    let end = units.iter().position(|&unit| unit == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}

fn extended_info(monitor: HMONITOR) -> Option<MONITORINFOEXW> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: `MONITORINFOEXW` starts with a `MONITORINFO`, and `cbSize` tells Windows to fill the longer struct.
    let read = unsafe { GetMonitorInfoW(monitor, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO) };
    read.as_bool().then_some(info)
}

fn identity_of(info: &MONITORINFOEXW) -> ScreenIdentity {
    let gdi_device_name = text_of(&info.szDevice);
    let mut device = DISPLAY_DEVICEW { cb: size_of::<DISPLAY_DEVICEW>() as u32, ..Default::default() };
    // SAFETY: `info.szDevice` is a null-terminated name that outlives the call, and `device` is a sized, writable struct.
    let found =
        unsafe { EnumDisplayDevicesW(PCWSTR(info.szDevice.as_ptr()), 0, &mut device, EDD_GET_DEVICE_INTERFACE_NAME) };
    if !found.as_bool() {
        return ScreenIdentity { id: gdi_device_name, model: None };
    }
    ScreenIdentity {
        id: pick_id(&text_of(&device.DeviceID), &gdi_device_name),
        model: model_of(&text_of(&device.DeviceString)),
    }
}

/// The identifier of the monitor behind `monitor` (an `HMONITOR` value); `None` when Windows cannot read it.
pub fn identify(monitor: isize) -> Option<ScreenIdentity> {
    extended_info(HMONITOR(monitor as *mut _)).map(|info| identity_of(&info))
}

unsafe extern "system" fn collect(monitor: HMONITOR, _context: HDC, _rect: *mut RECT, list: LPARAM) -> BOOL {
    // SAFETY: `list` is the `Vec` that `list()` passed and keeps alive for the whole enumeration.
    let screens = unsafe { &mut *(list.0 as *mut Vec<Screen>) };
    if let Some(info) = extended_info(monitor) {
        let ScreenIdentity { id, model } = identity_of(&info);
        let rect = info.monitorInfo.rcMonitor;
        let is_primary = info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0;
        screens.push(Screen {
            id,
            model,
            is_primary,
            left: rect.left,
            top: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        });
    }
    true.into()
}

/// Every attached screen with its identifier; a read, so the caller decides when to ask.
pub fn list() -> Vec<Screen> {
    let mut screens: Vec<Screen> = Vec::new();
    let list = LPARAM(&mut screens as *mut Vec<Screen> as isize);
    // SAFETY: `collect` only runs during this call, while `screens` is alive.
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(collect), list) };
    screens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_device_path_is_the_identifier_when_windows_gives_one() {
        assert_eq!(pick_id(r"\?\DISPLAY#GSM5B09#5&1#{x}", r"\.\DISPLAY2"), r"\?\DISPLAY#GSM5B09#5&1#{x}");
    }

    #[test]
    fn the_gdi_name_is_the_identifier_when_no_device_path_can_be_read() {
        assert_eq!(pick_id("", r"\.\DISPLAY2"), r"\.\DISPLAY2");
    }

    #[test]
    fn a_generic_or_empty_model_name_is_left_out() {
        assert_eq!(model_of("Generic PnP Monitor"), None);
        assert_eq!(model_of("  "), None);
        assert_eq!(model_of(" LG UltraGear "), Some("LG UltraGear".to_owned()));
    }

    #[test]
    fn the_list_has_one_primary_screen_and_every_identifier_is_filled() {
        let screens = list();
        assert_eq!(screens.iter().filter(|screen| screen.is_primary).count(), 1);
        let primary = screens.iter().find(|screen| screen.is_primary).expect("primary");
        assert_eq!((primary.left, primary.top), (0, 0), "the primary screen starts the virtual desktop");
        assert!(screens.iter().all(|screen| !screen.id.is_empty() && screen.width > 0 && screen.height > 0));
    }

    #[test]
    fn a_monitor_reads_to_the_same_identifier_the_list_gives() {
        let listed = list();
        let primary = listed.iter().find(|screen| screen.is_primary).unwrap();
        let primary_handle = {
            use windows::Win32::Foundation::POINT;
            use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTOPRIMARY, MonitorFromPoint};
            unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) }
        };
        assert_eq!(identify(primary_handle.0 as isize).map(|identity| identity.id), Some(primary.id.clone()));
    }
}
