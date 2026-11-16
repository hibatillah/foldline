//! Ends the notch within a bounded time and puts back what it changed in Windows, on quit and on a panic.
//!
//! The UI thread must not drop the `Notch` after `run` returns: Rust runs the main thread's thread-local destructors
//! inside `ExitProcess`, after Windows has ended every other thread and while the loader lock is held, so a drop that
//! waits on a worker, a COM or WinRT release or an event-log subscription (`EvtClose`) never returns and the process
//! hangs with its window on screen.

use std::{
    sync::{
        atomic::{AtomicIsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use windows::Win32::Foundation::HWND;

use super::{notification_banners, volume_flyout};
use crate::log;

/// How long quitting waits for Windows' popups to come back before the process ends anyway.
const RESTORE_TIMEOUT: Duration = Duration::from_secs(3);

/// The notch window the panic hook compares against, so a newer notch that replaced this one keeps its popups hidden.
static OWN_WINDOW: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Finished {
    InTime,
    TimedOut,
}

/// Runs `job` on a short-lived thread and waits at most `timeout` for it; a job still running is left behind.
pub(super) fn run_bounded(name: &str, timeout: Duration, job: impl FnOnce() + Send + 'static) -> Finished {
    let (done_sender, done) = mpsc::channel();
    let spawned = thread::Builder::new().name(name.into()).spawn(move || {
        job();
        let _ = done_sender.send(());
    });
    match spawned {
        Ok(_) => match done.recv_timeout(timeout) {
            Err(RecvTimeoutError::Timeout) => {
                log::debug(&format!("could not finish {name} within {} ms; going on", timeout.as_millis()));
                Finished::TimedOut
            }
            // A job that panicked drops the sender: it has ended either way.
            Ok(()) | Err(RecvTimeoutError::Disconnected) => Finished::InTime,
        },
        Err(error) => {
            log::debug(&format!("could not start {name}: {error}"));
            Finished::TimedOut
        }
    }
}

/// Asks a worker to stop, then joins it only once it reports the end of its work, waiting at most `timeout`.
pub(super) fn stop_worker(
    name: &str,
    ask_to_stop: impl FnOnce(),
    finished: &Receiver<()>,
    thread: JoinHandle<()>,
    timeout: Duration,
) -> Finished {
    ask_to_stop();
    match finished.recv_timeout(timeout) {
        Ok(()) | Err(RecvTimeoutError::Disconnected) => {
            if thread.join().is_err() {
                log::debug(&format!("could not stop the {name}: it panicked"));
            }
            Finished::InTime
        }
        Err(RecvTimeoutError::Timeout) => {
            log::debug(&format!("could not stop the {name} within {} ms; leaving it", timeout.as_millis()));
            Finished::TimedOut
        }
    }
}

pub(super) fn remember_own_window(window: HWND) {
    OWN_WINDOW.store(window.0 as isize, Ordering::Relaxed);
}

/// Puts back the volume flyout and the notification banners, waiting at most `RESTORE_TIMEOUT`.
pub(super) fn restore_windows_popups() -> Finished {
    let own_window = OWN_WINDOW.load(Ordering::Relaxed);
    run_bounded("popup-restore", RESTORE_TIMEOUT, move || {
        let own_window = HWND(own_window as *mut _);
        notification_banners::release_at_exit(own_window);
        volume_flyout::release_at_exit(own_window);
    })
}

/// A release build aborts on a panic without running the exit path, so the hook puts the popups back first.
pub(super) fn restore_popups_on_panic() {
    if !cfg!(panic = "abort") {
        return;
    }
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |information| {
        previous_hook(information);
        log::debug(&format!("fatal panic: {information}"));
        restore_windows_popups();
    }));
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    const SHORT: Duration = Duration::from_millis(100);

    #[test]
    fn a_job_that_ends_quickly_finishes_in_time() {
        assert_eq!(run_bounded("test-quick", Duration::from_secs(5), || {}), Finished::InTime);
    }

    #[test]
    fn a_job_that_never_ends_lets_the_exit_go_on_after_the_timeout() {
        let (release_sender, release) = mpsc::channel::<()>();
        let started = Instant::now();
        let finished = run_bounded("test-stuck", SHORT, move || {
            let _ = release.recv();
        });
        let waited = started.elapsed();
        let _ = release_sender.send(());
        assert_eq!((finished, waited < Duration::from_secs(2)), (Finished::TimedOut, true));
    }

    #[test]
    fn a_job_that_panics_counts_as_finished() {
        assert_eq!(run_bounded("test-panic", Duration::from_secs(5), || panic!("job failed")), Finished::InTime);
    }

    fn worker(honors_stop: bool) -> (mpsc::Sender<()>, Receiver<()>, JoinHandle<()>) {
        let (stop_sender, stop) = mpsc::channel::<()>();
        let (finished_sender, finished) = mpsc::channel();
        let thread = thread::spawn(move || {
            let _ = stop.recv();
            if !honors_stop {
                thread::sleep(Duration::from_secs(1));
            }
            let _ = finished_sender.send(());
        });
        (stop_sender, finished, thread)
    }

    #[test]
    fn a_worker_that_honors_the_stop_signal_is_joined() {
        let (stop_sender, finished, thread) = worker(true);
        let result = stop_worker("test worker", move || drop(stop_sender), &finished, thread, Duration::from_secs(5));
        assert_eq!(result, Finished::InTime);
    }

    #[test]
    fn a_worker_stuck_in_its_work_is_left_behind_after_the_timeout() {
        let (stop_sender, finished, thread) = worker(false);
        let started = Instant::now();
        let result = stop_worker("test worker", move || drop(stop_sender), &finished, thread, SHORT);
        assert_eq!((result, started.elapsed() < Duration::from_millis(900)), (Finished::TimedOut, true));
    }

    #[test]
    fn a_worker_that_ended_on_its_own_is_joined_without_waiting() {
        let (finished_sender, finished) = mpsc::channel::<()>();
        let thread = thread::spawn(move || drop(finished_sender));
        assert_eq!(stop_worker("test worker", || {}, &finished, thread, Duration::from_secs(5)), Finished::InTime);
    }
}
