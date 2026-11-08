//! Reads a toast's XML from Windows' notification database, the only place that keeps its launch arguments.
//! The database is SQLite in WAL mode; it is opened read-only through `winsqlite3.dll`, which ships with Windows, so
//! the notch carries no SQLite of its own. The listener's notification id is the database's `Notification.Id`.

use std::ffi::{CStr, c_char, c_int, c_void};

use windows::{
    Win32::{
        Foundation::{FreeLibrary, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
    core::{PCSTR, s, w},
};

use crate::log;

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_OPEN_READONLY: c_int = 0x1;
const SQLITE_OPEN_URI: c_int = 0x40;
const PAYLOAD_QUERY: &CStr = c"select Payload from Notification where Id = ?1";

type Open = unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int, *const c_char) -> c_int;
type Prepare = unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut *mut c_void, *mut *const c_char) -> c_int;
type BindInt64 = unsafe extern "C" fn(*mut c_void, c_int, i64) -> c_int;
type Step = unsafe extern "C" fn(*mut c_void) -> c_int;
type ColumnBlob = unsafe extern "C" fn(*mut c_void, c_int) -> *const c_void;
type ColumnBytes = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type Release = unsafe extern "C" fn(*mut c_void) -> c_int;
/// What `GetProcAddress` gives for any export.
type Export = unsafe extern "system" fn() -> isize;

struct Sqlite {
    library: HMODULE,
    open: Open,
    prepare: Prepare,
    bind_int64: BindInt64,
    step: Step,
    column_blob: ColumnBlob,
    column_bytes: ColumnBytes,
    finalize: Release,
    close: Release,
}

impl Sqlite {
    fn load() -> Option<Self> {
        let library = unsafe { LoadLibraryW(w!("winsqlite3.dll")) }.ok()?;
        let find = |name: PCSTR| unsafe { GetProcAddress(library, name) };
        let functions = (|| {
            // SAFETY: each export has the C signature of the SQLite function of the same name.
            unsafe {
                Some(Self {
                    library,
                    open: std::mem::transmute::<Export, Open>(find(s!("sqlite3_open_v2"))?),
                    prepare: std::mem::transmute::<Export, Prepare>(find(s!("sqlite3_prepare_v2"))?),
                    bind_int64: std::mem::transmute::<Export, BindInt64>(find(s!("sqlite3_bind_int64"))?),
                    step: std::mem::transmute::<Export, Step>(find(s!("sqlite3_step"))?),
                    column_blob: std::mem::transmute::<Export, ColumnBlob>(find(s!("sqlite3_column_blob"))?),
                    column_bytes: std::mem::transmute::<Export, ColumnBytes>(find(s!("sqlite3_column_bytes"))?),
                    finalize: std::mem::transmute::<Export, Release>(find(s!("sqlite3_finalize"))?),
                    close: std::mem::transmute::<Export, Release>(find(s!("sqlite3_close"))?),
                })
            }
        })();
        if functions.is_none() {
            let _ = unsafe { FreeLibrary(library) };
        }
        functions
    }
}

impl Drop for Sqlite {
    fn drop(&mut self) {
        if let Err(error) = unsafe { FreeLibrary(self.library) } {
            log::debug(&format!("could not unload winsqlite3: {error}"));
        }
    }
}

/// `file:` URI of the database, opened read-only so Windows' own writer is never blocked.
fn database_uri(local_app_data: &str) -> String {
    // A `%`, `?` or `#` in a user name would otherwise end or change the path inside the URI.
    let folder = local_app_data.replace('%', "%25").replace('?', "%3f").replace('#', "%23").replace('\\', "/");
    format!("file:///{folder}/Microsoft/Windows/Notifications/wpndatabase.db?mode=ro\0")
}

/// The toast XML of notification `id`, or `None` when Windows no longer has it.
pub fn read(id: u32) -> Option<String> {
    let sqlite = Sqlite::load()?;
    let uri = database_uri(&std::env::var("LOCALAPPDATA").ok()?);
    let mut database = std::ptr::null_mut();
    let flags = SQLITE_OPEN_READONLY | SQLITE_OPEN_URI;
    let opened = unsafe { (sqlite.open)(uri.as_ptr().cast(), &mut database, flags, std::ptr::null()) };
    let payload = if opened == SQLITE_OK { query_payload(&sqlite, database, id) } else { None };
    if !database.is_null() {
        unsafe { (sqlite.close)(database) };
    }
    if opened != SQLITE_OK {
        log::debug(&format!("notifications: could not open the notification database: code {opened}"));
    }
    payload
}

fn query_payload(sqlite: &Sqlite, database: *mut c_void, id: u32) -> Option<String> {
    let mut statement = std::ptr::null_mut();
    let prepared =
        unsafe { (sqlite.prepare)(database, PAYLOAD_QUERY.as_ptr(), -1, &mut statement, std::ptr::null_mut()) };
    if prepared != SQLITE_OK {
        return None;
    }
    let payload = (|| {
        if unsafe { (sqlite.bind_int64)(statement, 1, i64::from(id)) } != SQLITE_OK
            || unsafe { (sqlite.step)(statement) } != SQLITE_ROW
        {
            return None;
        }
        let bytes = unsafe { (sqlite.column_bytes)(statement, 0) };
        let data = unsafe { (sqlite.column_blob)(statement, 0) };
        if data.is_null() || bytes <= 0 {
            return None;
        }
        // SAFETY: SQLite keeps the blob alive until the next step or finalize, and `bytes` is its length.
        let blob = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), bytes as usize) };
        Some(String::from_utf8_lossy(blob).into_owned())
    })();
    unsafe { (sqlite.finalize)(statement) };
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_database_is_opened_read_only_by_a_uri_with_forward_slashes() {
        assert_eq!(
            database_uri(r"C:\Users\someone\AppData\Local"),
            "file:///C:/Users/someone/AppData/Local/Microsoft/Windows/Notifications/wpndatabase.db?mode=ro\0"
        );
    }

    #[test]
    fn a_user_folder_with_uri_characters_keeps_its_whole_path() {
        assert!(database_uri(r"C:\Users\a#b?c%d\AppData\Local").starts_with("file:///C:/Users/a%23b%3fc%25d/AppData/"));
    }
}
