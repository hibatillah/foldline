//! One thread blocked in `ReadDirectoryChangesW` on the Downloads folder; it wakes the activity on file name changes.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use windows::{
    Win32::{
        Foundation::CloseHandle,
        Storage::FileSystem::{
            CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_FILE_NAME,
            FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadDirectoryChangesW,
        },
    },
    core::HSTRING,
};

use super::tracker::FolderEvent;
use crate::{log, page::Notifier};

const BUFFER_WORDS: usize = 4096;
const ACTION_ADDED: u32 = 1;
const ACTION_REMOVED: u32 = 2;
const ACTION_RENAMED_OLD_NAME: u32 = 4;
const ACTION_RENAMED_NEW_NAME: u32 = 5;
/// `NextEntryOffset`, `Action` and `FileNameLength`, each four bytes, come before the UTF-16 name.
const RECORD_HEADER_BYTES: usize = 12;

#[derive(Default)]
pub struct Shared {
    events: Mutex<Vec<FolderEvent>>,
}

impl Shared {
    pub fn take_events(&self) -> Vec<FolderEvent> {
        self.events.lock().map(|mut events| std::mem::take(&mut *events)).unwrap_or_default()
    }

    fn push(&self, new_events: Vec<FolderEvent>) {
        if let Ok(mut events) = self.events.lock() {
            events.extend(new_events);
        }
    }
}

/// Starts the thread named "downloads-watcher"; it ends when the folder cannot be watched any more.
pub fn start(folder: PathBuf, notifier: Notifier) -> Arc<Shared> {
    start_with(folder, move || notifier.notify())
}

fn start_with(folder: PathBuf, on_events: impl Fn() + Send + 'static) -> Arc<Shared> {
    let shared = Arc::new(Shared::default());
    let worker_shared = shared.clone();
    let spawned = thread::Builder::new().name("downloads-watcher".into()).spawn(move || {
        if let Err(error) = watch(&folder, &worker_shared, &on_events) {
            log::debug(&format!("could not watch the Downloads folder: {error}"));
        }
    });
    if let Err(error) = spawned {
        log::debug(&format!("could not start the Downloads watcher: {error}"));
    }
    shared
}

fn watch(folder: &Path, shared: &Shared, on_events: &impl Fn()) -> windows::core::Result<()> {
    let directory = unsafe {
        CreateFileW(
            &HSTRING::from(folder.as_os_str()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }?;
    // Words, so the buffer is aligned as `ReadDirectoryChangesW` requires.
    let mut buffer = vec![0u32; BUFFER_WORDS];
    let result = loop {
        let mut returned_bytes = 0u32;
        // SAFETY: the buffer outlives the blocking call and is `BUFFER_WORDS * 4` bytes long.
        let read = unsafe {
            ReadDirectoryChangesW(
                directory,
                buffer.as_mut_ptr().cast(),
                (BUFFER_WORDS * size_of::<u32>()) as u32,
                false,
                FILE_NOTIFY_CHANGE_FILE_NAME,
                Some(&mut returned_bytes),
                None,
                None,
            )
        };
        if let Err(error) = read {
            break Err(error);
        }
        // SAFETY: a `u32` buffer is valid to view as bytes, and the call wrote only `returned_bytes` of them.
        let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), returned_bytes as usize) };
        // No bytes with a success means the buffer overflowed and the events were lost.
        let events =
            if returned_bytes == 0 { vec![FolderEvent::Relisted(list_names(folder))] } else { events_from(bytes) };
        if !events.is_empty() {
            shared.push(events);
            on_events();
        }
    };
    if let Err(error) = unsafe { CloseHandle(directory) } {
        log::debug(&format!("could not close the Downloads folder: {error}"));
    }
    result
}

fn list_names(folder: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
    entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect()
}

fn read_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]])
}

/// The records of one `ReadDirectoryChangesW` buffer as events, renames paired.
pub(super) fn events_from(buffer: &[u8]) -> Vec<FolderEvent> {
    let mut events = Vec::new();
    let mut old_name: Option<String> = None;
    let mut offset = 0usize;
    while offset + RECORD_HEADER_BYTES <= buffer.len() {
        let next_offset = read_u32(buffer, offset) as usize;
        let action = read_u32(buffer, offset + 4);
        let name_bytes = read_u32(buffer, offset + 8) as usize;
        let name_start = offset + RECORD_HEADER_BYTES;
        let Some(name_end) = name_start.checked_add(name_bytes).filter(|&end| end <= buffer.len()) else { break };
        let units: Vec<u16> =
            buffer[name_start..name_end].as_chunks::<2>().0.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
        let name = String::from_utf16_lossy(&units);
        match action {
            ACTION_RENAMED_OLD_NAME => {
                events.extend(old_name.replace(name).map(FolderEvent::Removed));
            }
            ACTION_RENAMED_NEW_NAME => match old_name.take() {
                Some(from) => events.push(FolderEvent::Renamed { from, to: name }),
                None => events.push(FolderEvent::Added(name)),
            },
            ACTION_ADDED | ACTION_REMOVED => {
                events.extend(old_name.take().map(FolderEvent::Removed));
                events.push(if action == ACTION_ADDED { FolderEvent::Added(name) } else { FolderEvent::Removed(name) });
            }
            _ => {}
        }
        if next_offset == 0 {
            break;
        }
        offset += next_offset;
    }
    events.extend(old_name.map(FolderEvent::Removed));
    events
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    use super::*;

    /// One `FILE_NOTIFY_INFORMATION` record; `is_last` leaves `NextEntryOffset` at zero.
    fn record(action: u32, name: &str, is_last: bool) -> Vec<u8> {
        let units: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let length = (RECORD_HEADER_BYTES + units.len()).next_multiple_of(4);
        let next_offset = if is_last { 0 } else { length as u32 };
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&next_offset.to_le_bytes());
        bytes.extend_from_slice(&action.to_le_bytes());
        bytes.extend_from_slice(&(units.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&units);
        bytes.resize(length, 0);
        bytes
    }

    #[test]
    fn an_added_and_a_removed_record_become_two_events() {
        let buffer = [record(ACTION_ADDED, "a.crdownload", false), record(ACTION_REMOVED, "b.txt", true)].concat();
        assert_eq!(
            events_from(&buffer),
            vec![FolderEvent::Added("a.crdownload".into()), FolderEvent::Removed("b.txt".into())]
        );
    }

    #[test]
    fn an_old_name_and_a_new_name_become_one_rename() {
        let buffer =
            [record(ACTION_RENAMED_OLD_NAME, "x.crdownload", false), record(ACTION_RENAMED_NEW_NAME, "x.bin", true)]
                .concat();
        assert_eq!(
            events_from(&buffer),
            vec![FolderEvent::Renamed { from: "x.crdownload".into(), to: "x.bin".into() }]
        );
    }

    #[test]
    fn an_old_name_without_a_new_name_is_a_removal() {
        let buffer = [record(ACTION_RENAMED_OLD_NAME, "x.crdownload", true)].concat();
        assert_eq!(events_from(&buffer), vec![FolderEvent::Removed("x.crdownload".into())]);
        let followed = [record(ACTION_RENAMED_OLD_NAME, "x.part", false), record(ACTION_ADDED, "y.txt", true)].concat();
        assert_eq!(
            events_from(&followed),
            vec![FolderEvent::Removed("x.part".into()), FolderEvent::Added("y.txt".into())]
        );
    }

    #[test]
    fn a_new_name_without_an_old_name_is_an_addition() {
        let buffer = [record(ACTION_RENAMED_NEW_NAME, "x.bin", true)].concat();
        assert_eq!(events_from(&buffer), vec![FolderEvent::Added("x.bin".into())]);
    }

    #[test]
    fn an_empty_buffer_gives_no_events() {
        assert_eq!(events_from(&[]), Vec::new());
    }

    #[test]
    fn a_record_that_runs_past_the_buffer_ends_the_reading() {
        let mut buffer = record(ACTION_ADDED, "a.txt", true);
        buffer.truncate(buffer.len() - 6);
        assert_eq!(events_from(&buffer), Vec::new());
    }

    #[test]
    fn a_file_created_and_renamed_in_a_watched_folder_arrives_as_events() {
        let folder = std::env::temp_dir().join(format!("notch-downloads-watcher-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let (sender, receiver) = mpsc::channel();
        let shared = start_with(folder.clone(), move || {
            let _ = sender.send(());
        });
        // The watch is armed by the thread's first call; give it a moment before changing files.
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(folder.join("x.crdownload"), b"1").unwrap();
        std::fs::rename(folder.join("x.crdownload"), folder.join("x.bin")).unwrap();
        let mut events = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(2);
        let wanted = FolderEvent::Renamed { from: "x.crdownload".into(), to: "x.bin".into() };
        while !events.contains(&wanted) && Instant::now() < deadline {
            let _ = receiver.recv_timeout(Duration::from_millis(100));
            events.extend(shared.take_events());
        }
        std::fs::remove_dir_all(&folder).unwrap();
        let added = events.iter().position(|event| *event == FolderEvent::Added("x.crdownload".into()));
        let renamed = events.iter().position(|event| *event == wanted);
        assert!(added.is_some() && added < renamed, "events: {events:?}");
    }
}
