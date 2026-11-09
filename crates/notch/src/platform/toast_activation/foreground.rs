//! Tells whether a route that reported success brought a window to the front, since an app's toast activator can
//! answer a call with success and show nothing.

use std::{
    cell::RefCell,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_SYSTEM_FOREGROUND, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId, KillTimer, MSG,
            PM_NOREMOVE, PeekMessageW, PostThreadMessageW, SetTimer, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
            WM_QUIT, WM_TIMER,
        },
    },
};

use super::payload::came_forward;
use crate::log;

thread_local! {
    /// Where the hook thread's callback sends each window that comes to the front.
    static FOREGROUND_SENDER: RefCell<Option<Sender<isize>>> = const { RefCell::new(None) };
}

/// The window in front at the click.
pub struct ForegroundWatch {
    window_at_click: isize,
}

impl ForegroundWatch {
    pub fn start() -> Self {
        Self { window_at_click: window_identity(unsafe { GetForegroundWindow() }).0 }
    }

    /// Waits until another window comes to the front, or `until` passes; returns whether one did.
    pub fn wait_for_a_new_window(&self, until: Instant) -> bool {
        // The app often comes forward during the route's own call, before any hook could see it.
        if self.came_forward_now() {
            log::debug("notifications: a new window was already in front");
            return true;
        }
        let Some(hook) = ForegroundHook::start(until) else { return false };
        let result = self.came_forward_now() || self.wait_for_events(&hook.windows, until);
        hook.stop();
        log::debug(&format!("notifications: a new window came forward: {result}"));
        result
    }

    fn came_forward_now(&self) -> bool {
        let (window_now, is_the_notch) = window_identity(unsafe { GetForegroundWindow() });
        came_forward(self.window_at_click, window_now, is_the_notch)
    }

    fn wait_for_events(&self, windows: &Receiver<isize>, until: Instant) -> bool {
        loop {
            let Ok(window) = windows.recv_timeout(until.saturating_duration_since(Instant::now())) else {
                return false;
            };
            let (window, is_the_notch) = window_identity(HWND(window as *mut _));
            if came_forward(self.window_at_click, window, is_the_notch) {
                return true;
            }
        }
    }
}

/// A foreground hook on a short-lived thread of its own; that thread unhooks before it ends, at `stop` or the deadline.
struct ForegroundHook {
    thread_id: u32,
    thread: JoinHandle<()>,
    windows: Receiver<isize>,
}

impl ForegroundHook {
    fn start(until: Instant) -> Option<Self> {
        let (window_sender, windows) = mpsc::channel();
        let (ready_sender, ready) = mpsc::channel();
        let spawned = thread::Builder::new().name("toast-foreground".into()).spawn(move || {
            let mut message = MSG::default();
            // Creates the message queue before the id is handed out, so the quit message is not lost.
            let _ = unsafe { PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE) };
            FOREGROUND_SENDER.with_borrow_mut(|sender| *sender = Some(window_sender));
            // Every process, since any app may come forward; the event fires only on a foreground switch.
            let hook = unsafe {
                SetWinEventHook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    None,
                    Some(on_foreground),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                )
            };
            if hook.is_invalid() {
                log::debug("could not hook foreground changes for a notification click");
                return;
            }
            let _ = ready_sender.send(unsafe { GetCurrentThreadId() });
            // Ends the loop at the deadline even if the quit message never arrives.
            let milliseconds = until.saturating_duration_since(Instant::now()).as_millis().clamp(1, u32::MAX as u128);
            let timer = unsafe { SetTimer(None, 0, milliseconds as u32, None) };
            // The hook's callbacks arrive inside GetMessageW; nothing else wakes this thread.
            while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() && message.message != WM_TIMER {}
            if timer != 0
                && let Err(error) = unsafe { KillTimer(None, timer) }
            {
                log::debug(&format!("could not stop the foreground hook's timer: {error}"));
            }
            if !unsafe { UnhookWinEvent(hook) }.as_bool() {
                log::debug("could not unhook foreground changes for a notification click");
            }
            log::debug("notifications: the foreground hook ended");
        });
        let thread =
            spawned.map_err(|error| log::debug(&format!("could not start the foreground hook: {error}"))).ok()?;
        let Ok(thread_id) = ready.recv() else {
            let _ = thread.join();
            return None;
        };
        log::debug("notifications: waiting for a new window to come forward");
        Some(Self { thread_id, thread, windows })
    }

    fn stop(self) {
        if let Err(error) = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) } {
            log::debug(&format!("could not stop the foreground hook: {error}"));
        }
        // Bounded: the thread's timer ends its loop at the deadline.
        if self.thread.join().is_err() {
            log::debug("could not wait for the foreground hook to end");
        }
    }
}

unsafe extern "system" fn on_foreground(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _event_time: u32,
) {
    FOREGROUND_SENDER.with_borrow(|sender| {
        if let Some(sender) = sender {
            let _ = sender.send(window.0 as isize);
        }
    });
}

/// A window as an integer, and whether it belongs to the notch.
fn window_identity(window: HWND) -> (isize, bool) {
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    (window.0 as isize, process_id == unsafe { GetCurrentProcessId() })
}
