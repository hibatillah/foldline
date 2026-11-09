//! Opens the app behind a Windows notification the way a click on Windows' own banner does, through its toast
//! activator; a route counts only when a window comes to the front, otherwise the next one is tried.

mod activator;
mod click_limits;
mod foreground;
mod payload;
mod stored_payload;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use windows::{
    Win32::{
        Foundation::{ERROR_BUSY, ERROR_TIMEOUT},
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

use self::{
    click_limits::{ClickDeadline, OutstandingActivators},
    foreground::ForegroundWatch,
    payload::{ActivationKind, Route, ToastLaunch, application_id, routes, toast_launch},
};
use super::ComApartment;
use crate::log;

/// All the routes of one click wait at most this long together; a registered activator whose app never answers
/// (Discord's when its process is a second instance) would otherwise hold the click for COM's two-minute limit.
const CLICK_TIME_LIMIT: Duration = Duration::from_secs(5);
/// Long enough for a running app to show its window after a route answered; a cold start is waited for by the call.
const WINDOW_WAIT: Duration = Duration::from_secs(3);

/// A stuck activator call keeps its thread until its app answers, so each app has at most one at a time.
static OUTSTANDING_ACTIVATORS: OutstandingActivators = OutstandingActivators::new();

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
    let deadline = ClickDeadline::new(Instant::now(), CLICK_TIME_LIMIT);
    let watch = ForegroundWatch::start();
    let _apartment = ComApartment::enter();
    let launch = stored_payload::read(id).map_or_else(
        || {
            log::debug(&format!("notifications: no stored toast for {id}, so the app is only started"));
            ToastLaunch { arguments: String::new(), kind: ActivationKind::Foreground }
        },
        |payload| toast_launch(&payload),
    );
    log::debug(&format!(
        "notifications: opening {app_user_model_id}, {:?} activation, {} characters of arguments",
        launch.kind,
        launch.arguments.len()
    ));
    let routes = routes(&launch);
    let last = routes.len().saturating_sub(1);
    for (index, route) in routes.iter().enumerate() {
        let name = route_name(route);
        let is_last = index == last;
        if !deadline.allows(Instant::now(), is_last) {
            log::debug(&format!("notifications: the click's time is spent, so {name} is skipped"));
            continue;
        }
        match take_route(app_user_model_id, route, deadline) {
            // Starting the app is the last way left, so it is trusted without waiting.
            Ok(()) if is_last || watch.wait_for_a_new_window(deadline.wait_until(Instant::now(), WINDOW_WAIT)) => {
                log::debug(&format!("notifications: opened {app_user_model_id} by {name}"));
                return true;
            }
            Ok(()) => log::debug(&format!("notifications: {name} answered but no window came to the front")),
            Err(error) => log::debug(&format!("notifications: {name} did not open it: {error}")),
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

fn take_route(app_user_model_id: &str, route: &Route, deadline: ClickDeadline) -> Result<()> {
    match route {
        Route::Activator(arguments) => activate_in_time(app_user_model_id, arguments, deadline),
        Route::OpenUri(uri) => shell_open(uri),
        Route::LaunchWithArguments(arguments) => activate_package(app_user_model_id, arguments),
        // The activation manager brings a packaged app forward itself; the Apps folder starts any app it lists.
        Route::StartApp => activate_package(app_user_model_id, "").or_else(|error| {
            if application_id(app_user_model_id).is_some() {
                log::debug(&format!("notifications: the activation manager did not start the app: {error}"));
            }
            shell_open(&format!(r"shell:AppsFolder\{app_user_model_id}"))
        }),
    }
}

/// Calls the toast activator on a thread of its own and gives up waiting at the click's deadline; no thread is started
/// while an earlier one for the same app is still stuck.
fn activate_in_time(app_user_model_id: &str, arguments: &str, deadline: ClickDeadline) -> Result<()> {
    let Some(claim) = OUTSTANDING_ACTIVATORS.claim(app_user_model_id) else {
        log::debug(&format!("notifications: an earlier activator call to {app_user_model_id} is still waiting"));
        return Err(Error::from_hresult(ERROR_BUSY.to_hresult()));
    };
    let class = activator::find(app_user_model_id).ok_or_else(|| Error::from_hresult(not_found()))?;
    let (sender, receiver) = mpsc::channel();
    let (app_user_model_id, arguments) = (app_user_model_id.to_owned(), arguments.to_owned());
    thread::spawn(move || {
        let _claim = claim;
        let _apartment = ComApartment::enter();
        let result = (|| {
            let callback: INotificationActivationCallback =
                unsafe { CoCreateInstance(&class, None, CLSCTX_LOCAL_SERVER) }?;
            // The app is another process; without this its window would open behind the one in front.
            if let Err(error) = unsafe { CoAllowSetForegroundWindow(&callback, None) } {
                log::debug(&format!("could not let the app take the foreground: {error}"));
            }
            unsafe { callback.Activate(&HSTRING::from(&app_user_model_id), &HSTRING::from(&arguments), &[]) }
        })();
        // The receiver is gone when the time limit passed first; the result is only logged then.
        if let Err(unsent) = sender.send(result) {
            log::debug(&format!("notifications: the toast activator answered late: {:?}", unsent.0));
        }
    });
    receiver
        .recv_timeout(deadline.remaining(Instant::now()))
        .unwrap_or_else(|_| Err(Error::from_hresult(ERROR_TIMEOUT.to_hresult())))
}

/// Starts or brings forward a packaged app with `arguments` as its launch arguments; an unpackaged app has no package.
fn activate_package(app_user_model_id: &str, arguments: &str) -> Result<()> {
    if application_id(app_user_model_id).is_none() {
        return Err(Error::from_hresult(not_found()));
    }
    let manager: IApplicationActivationManager =
        unsafe { CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_ALL) }?;
    unsafe { manager.ActivateApplication(&HSTRING::from(app_user_model_id), &HSTRING::from(arguments), AO_NONE) }
        .map(|_process_id| ())
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
