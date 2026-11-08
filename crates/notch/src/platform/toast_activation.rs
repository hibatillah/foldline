//! Opens the app behind a Windows notification the way a click on Windows' own banner does: the toast's launch
//! arguments go to the app's toast activator, so a chat app opens the conversation the toast came from.

mod activator;
mod payload;
mod stored_payload;

use windows::{
    Win32::{
        System::Com::{CLSCTX_ALL, CLSCTX_LOCAL_SERVER, CoAllowSetForegroundWindow, CoCreateInstance},
        UI::{
            Notifications::INotificationActivationCallback,
            Shell::{
                AO_NONE, ApplicationActivationManager, IApplicationActivationManager, SEE_MASK_FLAG_NO_UI,
                SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW,
            },
            WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow, SW_SHOWNORMAL},
        },
    },
    core::{Error, HSTRING, PCWSTR, Result, w},
};

use self::payload::{ActivationKind, Route, ToastLaunch, application_id, routes, toast_launch};
use super::ComApartment;
use crate::log;

/// Lets the app that opens next take the foreground; call it on the UI thread while handling the click, because
/// only the process that received the last input may hand the foreground on.
pub fn allow_foreground_handoff() {
    if let Err(error) = unsafe { AllowSetForegroundWindow(ASFW_ANY) } {
        log::debug(&format!("could not hand on the foreground: {error}"));
    }
}

/// Opens notification `id` of app `app_user_model_id`; returns whether any way worked. Starting an app can take
/// seconds, so call it off the UI thread, and before the notification is removed, which deletes its stored XML.
pub fn open(app_user_model_id: &str, id: u32) -> bool {
    let _apartment = ComApartment::enter();
    let launch = stored_payload::read(id)
        .map_or(ToastLaunch { arguments: String::new(), kind: ActivationKind::Foreground }, |payload| {
            toast_launch(&payload)
        });
    log::debug(&format!(
        "notifications: opening {app_user_model_id}, {:?} activation, {} characters of arguments",
        launch.kind,
        launch.arguments.len()
    ));
    for route in routes(&launch) {
        match take_route(app_user_model_id, &route) {
            Ok(()) => {
                log::debug(&format!("notifications: opened {app_user_model_id} by {}", route_name(&route)));
                return true;
            }
            Err(error) => log::debug(&format!("notifications: {} did not open it: {error}", route_name(&route))),
        }
    }
    false
}

fn route_name(route: &Route) -> &'static str {
    match route {
        Route::Activator(_) => "the toast activator",
        Route::OpenUri(_) => "the toast's URI",
        Route::LaunchWithArguments(_) => "launch arguments",
        Route::StartApp => "starting the app",
    }
}

fn take_route(app_user_model_id: &str, route: &Route) -> Result<()> {
    match route {
        Route::Activator(arguments) => {
            let class = activator::find(app_user_model_id).ok_or_else(|| Error::from_hresult(not_found()))?;
            let callback: INotificationActivationCallback =
                unsafe { CoCreateInstance(&class, None, CLSCTX_LOCAL_SERVER) }?;
            // The app is another process; without this its window would open behind the one in front.
            if let Err(error) = unsafe { CoAllowSetForegroundWindow(&callback, None) } {
                log::debug(&format!("could not let the app take the foreground: {error}"));
            }
            unsafe { callback.Activate(&HSTRING::from(app_user_model_id), &HSTRING::from(arguments.as_str()), &[]) }
        }
        Route::OpenUri(uri) => shell_open(uri),
        Route::LaunchWithArguments(arguments) => {
            if application_id(app_user_model_id).is_none() {
                return Err(Error::from_hresult(not_found()));
            }
            let manager: IApplicationActivationManager =
                unsafe { CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_ALL) }?;
            unsafe {
                manager.ActivateApplication(
                    &HSTRING::from(app_user_model_id),
                    &HSTRING::from(arguments.as_str()),
                    AO_NONE,
                )
            }
            .map(|_process_id| ())
        }
        Route::StartApp => shell_open(&format!(r"shell:AppsFolder\{app_user_model_id}")),
    }
}

/// Opens a URI or shell path; with no app for it this fails instead of asking the user to find one.
fn shell_open(target: &str) -> Result<()> {
    let file = HSTRING::from(target);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_FLAG_NO_UI | SEE_MASK_NOASYNC,
        lpVerb: w!("open"),
        lpFile: PCWSTR(file.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut info) }
}

fn not_found() -> windows::core::HRESULT {
    windows::Win32::Foundation::ERROR_NOT_FOUND.to_hresult()
}
