//! The only activity registry; optional activities persist their tray choice as activity.<id>=off.

mod account_notice;
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

/// The account notice, which nothing shows until the daily check calls its hook.
#[cfg(test)]
pub fn sample_account_notice() -> Box<dyn Activity> {
    Box::new(account_notice::AccountNotice::default())
}

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
        Box::new(account_notice::AccountNotice::default()),
    ]
}
