//! Start with Windows: the Run key for an unpackaged build, the manifest's startup task for an MSIX build.
//!
//! Package identity is read from the settings process itself. `foldline.exe` starts this program with `CreateProcess`, and
//! a packaged full-trust process's children are expected to inherit the identity; that is not verified without a
//! packaged install. Without it, a program that sits beside an `AppxManifest.xml` shows the Run-key behaviour and a note.

mod packaged;
mod presentation;
mod registry;

use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

pub use presentation::{Note, Presentation};
use presentation::{TaskState, present_task};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Registry,
    /// No identity, but the program sits in a package folder.
    RegistryInPackageFolder,
    Task,
}

fn in_package_folder() -> bool {
    std::env::current_exe().is_ok_and(|path| path.with_file_name("AppxManifest.xml").is_file())
}

fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match (packaged::has_package_identity(), in_package_folder()) {
        (true, _) => Mode::Task,
        (false, true) => Mode::RegistryInPackageFolder,
        (false, false) => Mode::Registry,
    })
}

static TASK_STATE: Mutex<Option<TaskState>> = Mutex::new(None);
static LOAD_REQUESTED: AtomicBool = AtomicBool::new(false);

fn stored_state() -> Option<TaskState> {
    *TASK_STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn store_state(state: Option<TaskState>) {
    *TASK_STATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
}

/// What the switch shows now; cheap enough to call on every paint.
pub fn presentation() -> Presentation {
    match mode() {
        Mode::Task => present_task(stored_state()),
        Mode::Registry => Presentation { checked: registry::is_enabled(), interactive: true, note: None },
        Mode::RegistryInPackageFolder => {
            Presentation { checked: registry::is_enabled(), interactive: true, note: Some(Note::ClassicEntry) }
        }
    }
}

/// True once, when the task state still has to be read from Windows (call `load` on a background thread then).
pub fn needs_load() -> bool {
    mode() == Mode::Task && !LOAD_REQUESTED.swap(true, Ordering::AcqRel)
}

/// Blocking: reads the task state into the cache.
pub fn load() {
    store_state(packaged::read_state());
}

/// Blocking: applies the switch and refreshes the cache from what Windows reports.
pub fn set_enabled(enabled: bool) {
    match mode() {
        Mode::Task => store_state(packaged::write_state(enabled).or_else(packaged::read_state)),
        Mode::Registry | Mode::RegistryInPackageFolder => registry::set_enabled(enabled),
    }
}
