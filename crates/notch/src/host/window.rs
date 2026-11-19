use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW,
            Performance::QueryPerformanceFrequency,
            Power::POWERBROADCAST_SETTING,
            RemoteDesktop::{
                NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification, WTSUnRegisterSessionNotification,
            },
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
        },
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetSystemMetricsForDpi, SetProcessDpiAwarenessContext,
            },
            Input::{
                GetCurrentInputMessageSource, IMDT_TOUCHPAD, INPUT_MESSAGE_SOURCE,
                KeyboardAndMouse::{GetKeyState, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_CONTROL},
            },
            WindowsAndMessaging::*,
        },
    },
    core::{Result, w},
};

use super::{
    drop_target,
    icon::Icon,
    location_tracker::{WM_LOCATION_FOUND, WM_NETWORK_CHANGED},
    monitor,
    notch::{MonitorState, Notch, NotchParts},
    notification_banners, registry,
    screen_mode::{self, ScreenMode},
    settings::Settings,
    settings_launcher, shutdown, tray, volume_flyout,
};
use store::{pause::Pause, snooze::Snooze};

use crate::{
    activity::{Activity, SystemMessage, WM_ACTIVITY_NOTIFY},
    gfx::{Fonts, Surface},
    log,
    page::{Page, WM_PAGE_NOTIFY},
};

mod focus;

pub(super) use focus::{give_back_keyboard, raise_topmost, restore_topmost, take_keyboard};

// Not exported under WindowsAndMessaging by the windows crate.
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_POWERBROADCAST: u32 = 0x0218;
const WM_CLIPBOARDUPDATE: u32 = 0x031D;
/// The `wparam` of `WM_POWERBROADCAST` when a power setting registered with `listen_to_power_setting` changed.
const PBT_POWERSETTINGCHANGE: usize = 0x8013;
/// Posted to the `FoldlineWindow` window by another program after it changed settings.txt.
pub const WM_SETTINGS_CHANGED: u32 = WM_APP + 30;

/// Registered message id the shell broadcasts when Explorer restarts and the tray is rebuilt.
static TASKBAR_CREATED_MESSAGE: AtomicU32 = AtomicU32::new(0);
static SESSION_NOTIFICATIONS_REGISTERED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static NOTCH: RefCell<Option<Notch>> = const { RefCell::new(None) };
}

/// Runs `action` on the notch, skipping it if a message re-enters while the notch is already borrowed.
fn with_notch<R>(action: impl FnOnce(&mut Notch) -> R) -> Option<R> {
    NOTCH.with(|cell| cell.try_borrow_mut().ok().and_then(|mut slot| slot.as_mut().map(action)))
}

fn show_menu(window: HWND) {
    let (paused_line, snoozed_line) =
        with_notch(|notch| (tray::paused_line(notch.pause()), tray::snoozed_line(notch.snooze()))).unwrap_or_default();
    let state = tray::MenuState { paused_line, snoozed_line };
    match tray::show_context_menu(window, &state) {
        Some(tray::MenuChoice::OpenSettings) => settings_launcher::open_settings_window(),
        // Destroying first takes the pill and the tray icon away at once; WM_DESTROY then ends the message loop.
        Some(tray::MenuChoice::Quit) => {
            if let Err(error) = unsafe { DestroyWindow(window) } {
                log::debug(&format!("could not destroy the notch window: {error}"));
                unsafe { PostQuitMessage(0) };
            }
        }
        Some(tray::MenuChoice::Pause(choice)) => {
            let now_unix = store::pause::now_unix();
            with_notch(|notch| notch.set_pause(choice.pause_from(now_unix), now_unix));
        }
        Some(tray::MenuChoice::Resume) => {
            with_notch(|notch| notch.set_pause(Pause::Off, store::pause::now_unix()));
        }
        Some(tray::MenuChoice::Snooze(choice)) => {
            let now_unix = store::snooze::now_unix();
            with_notch(|notch| notch.set_snooze(choice.snooze_from(now_unix), now_unix));
        }
        Some(tray::MenuChoice::TurnOffSnooze) => {
            with_notch(|notch| notch.set_snooze(Snooze::Off, store::snooze::now_unix()));
        }
        None => {}
    }
}

fn power_setting_message(lparam: LPARAM) -> Option<SystemMessage> {
    let setting = unsafe { (lparam.0 as *const POWERBROADCAST_SETTING).as_ref()? };
    if (setting.DataLength as usize) < size_of::<u32>() {
        return None;
    }
    let value = unsafe { std::ptr::read_unaligned(setting.Data.as_ptr().cast::<u32>()) };
    Some(SystemMessage::PowerSettingChanged { setting: setting.PowerSetting, value })
}

fn low_word_signed(value: isize) -> i32 {
    i32::from((value & 0xFFFF) as u16 as i16)
}

fn high_word_signed(value: isize) -> i32 {
    i32::from(((value >> 16) & 0xFFFF) as u16 as i16)
}

/// Re-arming an occluded window immediately posts another leave, so check the actual pointer target first.
fn track_mouse_leave(window: HWND) {
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() || unsafe { WindowFromPoint(cursor) } != window {
        return;
    }
    let mut tracking = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: window,
        dwHoverTime: 0,
    };
    let _ = unsafe { TrackMouseEvent(&mut tracking) };
}

extern "system" fn window_procedure(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let (x, y) = (low_word_signed(lparam.0), high_word_signed(lparam.0));
    match message {
        WM_MOUSEMOVE => {
            track_mouse_leave(window);
            with_notch(|notch| notch.pointer_moved(x, y));
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            if with_notch(|notch| notch.pointer_left_window()) == Some(false) {
                track_mouse_leave(window);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            with_notch(|notch| notch.pointer_pressed(x, y));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            with_notch(|notch| notch.pointer_released(x, y));
            LRESULT(0)
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            let amount = high_word_signed(wparam.0 as isize);
            let mut source = INPUT_MESSAGE_SOURCE::default();
            // Drivers without source metadata still use the delta/active-gesture fallback in scrolled().
            let is_touchpad =
                unsafe { GetCurrentInputMessageSource(&mut source) }.is_ok() && source.deviceType == IMDT_TOUCHPAD;
            with_notch(|notch| notch.scrolled(amount, message == WM_MOUSEHWHEEL, is_touchpad));
            LRESULT(0)
        }
        WM_CAPTURECHANGED => {
            with_notch(|notch| notch.capture_lost());
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            with_notch(|notch| notch.right_clicked(x, y));
            LRESULT(0)
        }
        drop_target::WM_POINTER_MOVED => {
            with_notch(|notch| notch.pointer_moved(x, y));
            LRESULT(0)
        }
        drop_target::WM_DRAG_ENTERED => {
            with_notch(|notch| notch.drag_entered());
            LRESULT(0)
        }
        drop_target::WM_DRAG_LEFT => {
            with_notch(|notch| notch.drag_left());
            LRESULT(0)
        }
        drop_target::WM_DRAG_DROPPED => {
            let paths = drop_target::take_dropped_paths();
            with_notch(|notch| notch.files_dropped(paths));
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_HOTKEY => {
            with_notch(|notch| notch.hotkey_pressed(wparam.0 as i32));
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let is_control_down = unsafe { GetKeyState(i32::from(VK_CONTROL.0)) } < 0;
            if with_notch(|notch| notch.key_pressed(wparam.0 as u16, is_control_down)) == Some(true) {
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(window, message, wparam, lparam) }
            }
        }
        WM_CHAR => {
            with_notch(|notch| notch.character_typed(wparam.0 as u16));
            LRESULT(0)
        }
        WM_ACTIVATE if (wparam.0 & 0xFFFF) as u32 == WA_INACTIVE => {
            with_notch(|notch| notch.deactivated());
            LRESULT(0)
        }
        WM_TIMER => {
            with_notch(|notch| notch.timer_fired(wparam.0));
            LRESULT(0)
        }
        WM_PAGE_NOTIFY => {
            with_notch(|notch| notch.page_notified(wparam.0));
            LRESULT(0)
        }
        WM_ACTIVITY_NOTIFY => {
            with_notch(|notch| notch.activity_notified(wparam.0));
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            if wparam.0 == PBT_POWERSETTINGCHANGE
                && let Some(message) = power_setting_message(lparam)
            {
                with_notch(|notch| notch.system_message(message));
            }
            if wparam.0 == PBT_APMRESUMEAUTOMATIC as usize {
                screen_mode::refresh_foreground();
                with_notch(|notch| {
                    notch.apply_stored_pause(store::pause::now_unix());
                    notch.reset_session_interaction();
                    notch.relocate_if_needed();
                    notch.request_location();
                });
            }
            LRESULT(1)
        }
        WM_WTSSESSION_CHANGE if wparam.0 == WTS_SESSION_LOCK as usize || wparam.0 == WTS_SESSION_UNLOCK as usize => {
            log::debug(if wparam.0 == WTS_SESSION_LOCK as usize { "session locked" } else { "session unlocked" });
            if wparam.0 == WTS_SESSION_UNLOCK as usize {
                screen_mode::refresh_foreground();
            }
            with_notch(|notch| {
                notch.apply_stored_pause(store::pause::now_unix());
                notch.reset_session_interaction();
                notch.relocate_if_needed();
            });
            LRESULT(0)
        }
        // A screen plugged in or out, a new resolution or primary, or a new scale of the notch's monitor.
        WM_DISPLAYCHANGE => {
            with_notch(|notch| notch.relocate_if_needed());
            LRESULT(0)
        }
        // The notch sizes itself per monitor, so Windows' suggested rectangle is not applied; during the notch's own
        // move the notch is borrowed and `with_notch` skips, which keeps this from looping.
        WM_DPICHANGED => {
            with_notch(|notch| notch.relocate_if_needed());
            LRESULT(0)
        }
        WM_LOCATION_FOUND => {
            with_notch(|notch| notch.location_found());
            LRESULT(0)
        }
        WM_NETWORK_CHANGED => {
            with_notch(|notch| notch.network_changed());
            LRESULT(0)
        }
        WM_SETTINGS_CHANGED => {
            log::debug("settings changed by another program");
            with_notch(|notch| notch.reload_settings());
            LRESULT(0)
        }
        WM_CLIPBOARDUPDATE => {
            with_notch(|notch| notch.system_message(SystemMessage::ClipboardChanged));
            LRESULT(0)
        }
        tray::WM_TRAY => {
            // Tray messages carry the originating mouse message in the low word of lparam.
            let mouse_message = (lparam.0 & 0xFFFF) as u32;
            if mouse_message == WM_RBUTTONUP || mouse_message == WM_CONTEXTMENU {
                show_menu(window);
            }
            LRESULT(0)
        }
        // Windows ends the process right after this message, so the message loop's exit path never runs.
        WM_ENDSESSION if wparam.0 != 0 => {
            notification_banners::release_at_session_end();
            volume_flyout::release();
            LRESULT(0)
        }
        WM_DESTROY => {
            if SESSION_NOTIFICATIONS_REGISTERED.swap(false, Ordering::Relaxed)
                && let Err(error) = unsafe { WTSUnRegisterSessionNotification(window) }
            {
                log::debug(&format!("could not unregister session notifications: {error}"));
            }
            tray::remove(window);
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ if message != 0 && message == TASKBAR_CREATED_MESSAGE.load(Ordering::Relaxed) => {
            let _ = tray::add(window);
            volume_flyout::explorer_restarted();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn on_screen_mode(mode: ScreenMode) {
    with_notch(|notch| notch.foreground_changed(mode));
}

/// Returns the class icons, which must stay alive while the window runs.
fn register_window_class(scale: f32, instance: HINSTANCE) -> Result<(Icon, Icon)> {
    let dpi = (scale * 96.0).round() as u32;
    let large_icon = Icon::load(unsafe { GetSystemMetricsForDpi(SM_CXICON, dpi) })?;
    let small_icon = Icon::load(unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) })?;
    let class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(window_procedure),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
        hIcon: large_icon.handle(),
        hIconSm: small_icon.handle(),
        lpszClassName: w!("FoldlineWindow"),
        ..Default::default()
    };
    unsafe { RegisterClassExW(&class) };
    Ok((large_icon, small_icon))
}

pub fn run(pages: Vec<Box<dyn Page>>, activities: Vec<Box<dyn Activity>>) -> Result<()> {
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }?;
    shutdown::restore_popups_on_panic();

    // Before any file read: the banner backup and the pages' files move with settings.txt.
    if store::migrate_legacy_data_directory() {
        crate::log::debug("copied the data folder from %APPDATA%\\notch to %APPDATA%\\Foldline");
    }
    let settings = Settings::load();
    let monitor_mode = monitor::read_mode(&settings, Default::default());
    let chosen_id = monitor::read_chosen_id(&settings, None);
    let (start_monitor, start) = monitor::start_placement(monitor_mode, chosen_id.as_deref());
    let scale = start.scale;
    let instance: HINSTANCE = unsafe { GetModuleHandleW(None) }?.into();
    let _class_icons = register_window_class(scale, instance)?;
    TASKBAR_CREATED_MESSAGE.store(unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) }, Ordering::Relaxed);

    let (width, height) = (start.width, start.height);
    let screen_position = POINT { x: start.left, y: start.top };
    // A previous instance is asked to close only after this one is visible, so a rebuild swaps without a gap.
    let previous_window = unsafe { FindWindowW(w!("FoldlineWindow"), None) }.ok();
    let window = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("FoldlineWindow"),
            w!("Foldline"),
            WS_POPUP,
            screen_position.x,
            screen_position.y,
            width,
            height,
            None,
            None,
            Some(instance),
            None,
        )
    }?;
    shutdown::remember_own_window(window);
    if previous_window.is_none() {
        volume_flyout::release_left_behind();
    }

    drop_target::start(window, scale, screen_position.x + width / 2, screen_position.y);

    let mut ticks_per_second = 0;
    unsafe { QueryPerformanceFrequency(&mut ticks_per_second) }?;

    let system_language = store::language::system_language();
    store::language::set_current(store::language::resolve_setting(&settings, system_language));
    let pages = registry::enabled_pages(pages, &settings);
    let notch = Notch::new(NotchParts {
        window,
        scale,
        screen_position,
        surface: Surface::new(width, height)?,
        fonts: Fonts::new(scale)?,
        settings,
        pages,
        activities,
        ticks_per_second,
        system_language,
        monitor: MonitorState::new(monitor_mode, chosen_id, start_monitor, start_monitor.map(|_| start)),
    });
    NOTCH.with(|cell| *cell.borrow_mut() = Some(notch));
    match unsafe { WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION) } {
        Ok(()) => SESSION_NOTIFICATIONS_REGISTERED.store(true, Ordering::Relaxed),
        Err(error) => {
            log::debug(&format!("session notifications unavailable; foreground recovery remains active: {error}"))
        }
    }
    with_notch(|notch| notch.start());

    let _ = unsafe { ShowWindow(window, SW_SHOWNOACTIVATE) };
    screen_mode::set_judged_monitor(start_monitor);
    screen_mode::start(on_screen_mode);
    if let Err(error) = tray::add(window) {
        log::debug(&format!("tray icon unavailable: {error}"));
    }
    if let Some(previous_window) = previous_window {
        let _ = unsafe { PostMessageW(Some(previous_window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    }

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        let _ = unsafe { TranslateMessage(&message) };
        unsafe { DispatchMessageW(&message) };
    }
    shutdown::restore_windows_popups();
    // Dropped in a thread-local destructor, the notch would run inside ExitProcess after every other thread ended.
    let notch = NOTCH.with(|cell| cell.try_borrow_mut().ok().and_then(|mut slot| slot.take()));
    std::mem::forget(notch);
    log::debug("notch exited");
    Ok(())
}
