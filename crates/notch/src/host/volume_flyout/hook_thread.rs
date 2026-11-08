//! The show hook runs on a thread of its own, so a notch frame being drawn never delays hiding the flyout.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
        mpsc,
    },
    thread,
};

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::{GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY_HIGHEST},
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, WM_APP,
    },
};

use crate::log;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Command {
    /// Find the flyout (or listen to Explorer until it appears) and hide it at every show.
    Follow,
    /// Explorer restarted: forget the old window, then follow the new Explorer.
    FollowNewExplorer,
    StopHiding,
}

const FOLLOW_MESSAGE: u32 = WM_APP + 1;
const FOLLOW_NEW_EXPLORER_MESSAGE: u32 = WM_APP + 2;
const STOP_HIDING_MESSAGE: u32 = WM_APP + 3;

static THREAD_ID: AtomicU32 = AtomicU32::new(0);
static STARTING: Mutex<()> = Mutex::new(());

pub(super) fn send(command: Command) {
    let message = match command {
        Command::Follow => FOLLOW_MESSAGE,
        Command::FollowNewExplorer => FOLLOW_NEW_EXPLORER_MESSAGE,
        Command::StopHiding => STOP_HIDING_MESSAGE,
    };
    let thread_id = match THREAD_ID.load(Ordering::Acquire) {
        0 if command == Command::StopHiding => return,
        0 => match start() {
            Some(thread_id) => thread_id,
            None => return,
        },
        thread_id => thread_id,
    };
    if let Err(error) = unsafe { PostThreadMessageW(thread_id, message, WPARAM(0), LPARAM(0)) } {
        log::debug(&format!("could not reach the volume flyout thread: {error}"));
    }
}

fn start() -> Option<u32> {
    let _starting = STARTING.lock().ok()?;
    if let thread_id @ 1.. = THREAD_ID.load(Ordering::Acquire) {
        return Some(thread_id);
    }
    let (sender, receiver) = mpsc::channel();
    let spawned = thread::Builder::new().name("volume-flyout".into()).spawn(move || {
        let mut message = MSG::default();
        // Creates the message queue before the id is handed out, so the first command is not lost.
        let _ = unsafe { PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE) };
        if let Err(error) = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST) } {
            log::debug(&format!("could not raise the volume flyout thread's priority: {error}"));
        }
        let _ = sender.send(unsafe { GetCurrentThreadId() });
        // The hook's callbacks arrive inside GetMessageW; nothing else wakes this thread.
        while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
            match message.message {
                FOLLOW_MESSAGE => super::follow_explorer(),
                FOLLOW_NEW_EXPLORER_MESSAGE => super::follow_new_explorer(),
                STOP_HIDING_MESSAGE => super::stop_hiding(),
                _ => unsafe {
                    DispatchMessageW(&message);
                },
            }
        }
    });
    if let Err(error) = spawned {
        log::debug(&format!("could not start the volume flyout thread: {error}"));
        return None;
    }
    let thread_id = receiver.recv().ok()?;
    THREAD_ID.store(thread_id, Ordering::Release);
    Some(thread_id)
}
