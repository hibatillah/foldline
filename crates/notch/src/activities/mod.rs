//! The only activity registry; optional activities persist their tray choice as activity.<id>=off.

mod battery;
mod bluetooth;
mod brightness;
mod camera;
mod caps_lock;
mod clipboard;
mod device_battery;
mod downloads;
mod microphone;
pub mod notifications;
mod privacy;
mod volume;

use crate::activity::Activity;

#[cfg(test)]
pub fn sample_notification(fonts: &crate::gfx::Fonts) -> Box<dyn Activity> {
    Box::new(notifications::NotificationsActivity::sample(fonts))
}

#[cfg(test)]
pub fn sample_download_started(fonts: &crate::gfx::Fonts) -> Box<dyn Activity> {
    Box::new(downloads::DownloadsActivity::sample_started(fonts))
}

#[cfg(test)]
pub fn sample_download_finished(fonts: &crate::gfx::Fonts) -> Box<dyn Activity> {
    Box::new(downloads::DownloadsActivity::sample_finished(fonts))
}

/// The finish banner under the pointer, with the real file icon uploaded through `upload`.
#[cfg(test)]
pub fn sample_download_finished_hovered(
    fonts: &crate::gfx::Fonts,
    upload: impl FnOnce(&crate::gfx::ImageData) -> Option<crate::gfx::ImageBrush>,
) -> Box<dyn Activity> {
    Box::new(downloads::DownloadsActivity::sample_finished(fonts).hovered().with_uploaded_icon(upload))
}

#[cfg(test)]
pub fn sample_download_running() -> Box<dyn Activity> {
    Box::new(downloads::DownloadsActivity::sample_running())
}

/// The volume activity in the middle of its level message.
#[cfg(test)]
pub fn sample_volume(volume: f32, is_muted: bool) -> Box<dyn Activity> {
    Box::new(volume::VolumeActivity::sample(volume, is_muted))
}

pub fn all() -> Vec<Box<dyn Activity>> {
    // Add a new activity as the last line below (`Box::new(<name>::<Name>Activity::new())`); order breaks ties of equal `header_status_rank`.
    vec![
        Box::new(volume::VolumeActivity::new()),
        Box::new(brightness::BrightnessActivity::new()),
        Box::new(battery::BatteryActivity::new()),
        Box::new(bluetooth::BluetoothActivity::new()),
        Box::new(device_battery::DeviceBatteryActivity::new()),
        Box::new(microphone::MicrophoneActivity::new()),
        Box::new(camera::CameraActivity::new()),
        Box::new(clipboard::ClipboardActivity::new()),
        Box::new(caps_lock::CapsLockActivity::new()),
        Box::new(notifications::NotificationsActivity::new()),
        Box::new(downloads::DownloadsActivity::new()),
    ]
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::source_scan::{imports_between_units, imports_of_modules};

    fn activities_directory() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("activities")
    }

    /// Modules that hold code two activities share and are not activities themselves (not in `all()`).
    const SHARED_HELPERS: &[&str] = &["privacy"];

    #[test]
    fn an_activity_imports_no_other_activity() {
        let offenders = imports_between_units(&activities_directory(), "activities", SHARED_HELPERS);
        assert!(offenders.is_empty(), "an activity may not import another activity: {offenders:#?}");
    }

    #[test]
    fn an_activity_imports_no_page() {
        let offenders = imports_of_modules(&activities_directory(), &["pages"]);
        assert!(offenders.is_empty(), "an activity may not import a page: {offenders:#?}");
    }
}
