//! Unpackaged apps cannot subscribe to notification events; watch the database directory instead.
//! A new toast is found by asking for the ids above the highest seen, once at the first change and again when the
//! directory is still; only when that finds nothing is the whole list read, which takes seconds and rewrites the
//! database itself, so the changes it causes are waited out. The same thread removes toasts the banner dismissed.

use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
    thread,
    time::Instant,
};

use windows::{
    ApplicationModel::AppInfo,
    Foundation::Size,
    Graphics::Imaging::{
        BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat, BitmapTransform, ColorManagementMode, ExifOrientationMode,
    },
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, UserNotification,
    },
    Win32::{
        Foundation::{HANDLE, WAIT_OBJECT_0},
        Storage::FileSystem::{
            FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE, FindFirstChangeNotificationW,
            FindNextChangeNotification,
        },
        System::Threading::{CreateEventW, INFINITE, SetEvent, WaitForMultipleObjects, WaitForSingleObject},
    },
    core::{HSTRING, PCWSTR, Result},
};

use super::{
    items::Item,
    rounds::{Step, follow_ids, next_step, unseen_ids},
};
use crate::{gfx::ImageData, log, page::Notifier, platform::app_icon};

/// Changes must stop for this long before a toast is looked for again, so one toast is one wake.
const QUIET_TIME_MILLISECONDS: u32 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Unknown = 0,
    Allowed = 1,
    Denied = 2,
}

impl Access {
    fn from_code(code: u8) -> Self {
        match code {
            1 => Access::Allowed,
            2 => Access::Denied,
            _ => Access::Unknown,
        }
    }
}

pub struct Shared {
    arrived: Mutex<Vec<Item>>,
    access: AtomicU8,
    /// Ids the banner dismissed or opened, for the worker to remove from the Action Center.
    removals: Mutex<Vec<u32>>,
    // The event handle crosses threads as an integer, since the raw handle type is not thread-safe.
    removal_event_address: isize,
}

impl Shared {
    pub fn take_arrived(&self) -> Vec<Item> {
        self.arrived.lock().map(|mut arrived| std::mem::take(&mut *arrived)).unwrap_or_default()
    }

    pub fn access(&self) -> Access {
        Access::from_code(self.access.load(Ordering::SeqCst))
    }

    /// Takes notification `id` out of the Action Center, as Windows does when its own banner is clicked or closed.
    pub fn remove(&self, id: u32) {
        if let Ok(mut removals) = self.removals.lock() {
            removals.push(id);
        }
        if let Err(error) = unsafe { SetEvent(HANDLE(self.removal_event_address as *mut _)) } {
            log::debug(&format!("could not ask for notification {id} to be removed: {error}"));
        }
    }

    fn take_removals(&self) -> Vec<u32> {
        self.removals.lock().map(|mut removals| std::mem::take(&mut *removals)).unwrap_or_default()
    }
}

const LOGO_PIXELS: u32 = 64;

fn read_logo(app_info: &AppInfo) -> Result<ImageData> {
    let stream = app_info
        .DisplayInfo()?
        .GetLogo(Size { Width: LOGO_PIXELS as f32, Height: LOGO_PIXELS as f32 })?
        .OpenReadAsync()?
        .join()?;
    let decoder = BitmapDecoder::CreateAsync(&stream)?.join()?;
    let transform = BitmapTransform::new()?;
    transform.SetScaledWidth(LOGO_PIXELS)?;
    transform.SetScaledHeight(LOGO_PIXELS)?;
    let pixels = decoder
        .GetPixelDataTransformedAsync(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
            &transform,
            ExifOrientationMode::IgnoreExifOrientation,
            ColorManagementMode::DoNotColorManage,
        )?
        .join()?
        .DetachPixelData()?;
    Ok(ImageData::new(LOGO_PIXELS, LOGO_PIXELS, pixels.to_vec()))
}

type Logos = HashMap<String, Option<Arc<ImageData>>>;

/// The list as Windows held it when the read began, and the items for the notifications not seen before.
struct Reading {
    listed: HashSet<u32>,
    fresh: Vec<Item>,
}

fn build_item(notification: &UserNotification, logos: &mut Logos) -> Result<Item> {
    let app_info = notification.AppInfo()?;
    let mut texts = Vec::new();
    if let Ok(binding) = notification.Notification()?.Visual()?.GetBinding(&KnownNotificationBindings::ToastGeneric()?)
    {
        for text in binding.GetTextElements()? {
            texts.push(text.Text()?.to_string());
        }
    }
    let title = if texts.is_empty() { String::new() } else { texts.remove(0) };
    let app_id = app_info.AppUserModelId()?.to_string();
    let icon = logos
        .entry(app_id.clone())
        .or_insert_with(|| app_icon::load(&app_id).or_else(|| read_logo(&app_info).ok()).map(Arc::new))
        .clone();
    Ok(Item {
        id: notification.Id()?,
        app: app_info.DisplayInfo()?.DisplayName()?.to_string(),
        app_id,
        icon,
        title,
        body: texts.join("\n"),
        read_at: Instant::now(),
    })
}

fn build_or_skip(notification: &UserNotification, logos: &mut Logos) -> Option<Item> {
    build_item(notification, logos)
        .inspect_err(|error| log::debug(&format!("notifications: skipped a notification that cannot be read: {error}")))
        .ok()
}

/// Windows takes about 80 ms per notification in the list to answer (it rewrites its database for each), so only the
/// notifications not in `seen` have their texts and logo read, and one that cannot be read is skipped.
fn read_fresh(listener: &UserNotificationListener, seen: &HashSet<u32>, logos: &mut Logos) -> Result<Reading> {
    let list = listener.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
    let notifications: Vec<UserNotification> = list.into_iter().collect();
    let listed_ids: Vec<u32> = notifications.iter().filter_map(|notification| notification.Id().ok()).collect();
    let fresh_ids = unseen_ids(&listed_ids, seen);
    let fresh = notifications
        .iter()
        .filter(|notification| notification.Id().is_ok_and(|id| fresh_ids.contains(&id)))
        .filter_map(|notification| build_or_skip(notification, logos))
        .collect();
    Ok(Reading { listed: listed_ids.into_iter().collect(), fresh })
}

/// Only the ids: the notifications already waiting at start-up are never shown, so their texts and logos stay unread.
fn read_ids(listener: &UserNotificationListener) -> Result<HashSet<u32>> {
    let list = listener.GetNotificationsAsync(NotificationKinds::Toast)?.join()?;
    list.into_iter().map(|notification| notification.Id()).collect()
}

/// Returns once the directory has been still for `quiet_time_milliseconds`, which also clears what changed before.
fn wait_for_quiet(changes: HANDLE, quiet_time_milliseconds: u32) {
    loop {
        let _ = unsafe { FindNextChangeNotification(changes) };
        if unsafe { WaitForSingleObject(changes, quiet_time_milliseconds) } != WAIT_OBJECT_0 {
            return;
        }
    }
}

fn deliver(shared: &Shared, notifier: &Notifier, fresh: Vec<Item>) {
    if fresh.is_empty() {
        return;
    }
    if let Ok(mut arrived) = shared.arrived.lock() {
        arrived.extend(fresh);
    }
    notifier.notify();
}

struct Worker<'a> {
    shared: &'a Shared,
    notifier: Notifier,
    listener: UserNotificationListener,
    logos: Logos,
    seen: HashSet<u32>,
    high_water: u32,
}

impl Worker<'_> {
    /// Asks for the notifications numbered above the highest seen, which takes tens of milliseconds where reading the
    /// list takes seconds. Asking for an id that does not exist changes nothing in the directory; finding one does.
    /// `woke` is when the directory first changed, for the debug log.
    fn ask_by_id(&mut self, woke: Instant) -> bool {
        let started = Instant::now();
        let mut found = 0;
        let newest = follow_ids(self.high_water, |id| {
            let Ok(notification) = self.listener.GetNotification(id) else { return false };
            if let Some(item) = build_or_skip(&notification, &mut self.logos) {
                self.seen.insert(item.id);
                // The banner does not wait for the rest of the ids to be tried.
                deliver(self.shared, &self.notifier, vec![item]);
                found += 1;
            }
            true
        });
        self.high_water = newest;
        if found > 0 {
            log::debug(&format!(
                "notifications: {found} new by id in {} ms, {} ms after the folder changed",
                started.elapsed().as_millis(),
                woke.elapsed().as_millis()
            ));
        }
        found > 0
    }

    fn read_list(&mut self) -> bool {
        let started = Instant::now();
        match read_fresh(&self.listener, &self.seen, &mut self.logos) {
            Ok(reading) => {
                log::debug(&format!(
                    "notifications: read {} listed in {} ms, {} new",
                    reading.listed.len(),
                    started.elapsed().as_millis(),
                    reading.fresh.len()
                ));
                self.high_water = reading.listed.iter().copied().fold(self.high_water, u32::max);
                self.seen = reading.listed;
                let found_news = !reading.fresh.is_empty();
                deliver(self.shared, &self.notifier, reading.fresh);
                found_news
            }
            Err(error) => {
                log::debug(&format!("notifications: cannot read the list: {error}"));
                false
            }
        }
    }

    /// One wake of the directory watcher, once the directory is still. `list_was_read` is true when the list was read
    /// just now and `wake_found_news` when the toast was already found at the first change, so only the cheap round
    /// that finds toasts that arrived meanwhile is wanted.
    fn handle_change(&mut self, changes: HANDLE, woke: Instant, mut list_was_read: bool, mut wake_found_news: bool) {
        loop {
            let round_found_news = self.ask_by_id(woke);
            let step = next_step(round_found_news, wake_found_news, list_was_read);
            wake_found_news |= round_found_news;
            match step {
                Step::AskAgain => wait_for_quiet(changes, QUIET_TIME_MILLISECONDS),
                Step::ReadList => {
                    wake_found_news |= self.read_list();
                    list_was_read = true;
                    // The read rewrites the database as it goes; those changes are its own and must not wake us again.
                    wait_for_quiet(changes, QUIET_TIME_MILLISECONDS);
                }
                Step::Done => return,
            }
        }
    }

    /// Removing rewrites the database, so its changes are waited out; a toast that came in meanwhile is still found
    /// by the round after, without reading the whole list.
    fn remove_dismissed(&mut self, changes: HANDLE) {
        for id in self.shared.take_removals() {
            match self.listener.RemoveNotification(id) {
                Ok(()) => log::debug(&format!("notifications: removed {id} from the Action Center")),
                Err(error) => log::debug(&format!("could not remove notification {id}: {error}")),
            }
        }
        wait_for_quiet(changes, QUIET_TIME_MILLISECONDS);
        self.handle_change(changes, Instant::now(), true, false);
    }
}

fn run(shared: &Shared, notifier: Notifier) -> Result<()> {
    let listener = UserNotificationListener::Current()?;
    let mut status = listener.GetAccessStatus()?;
    if status == UserNotificationListenerAccessStatus::Unspecified {
        status = listener.RequestAccessAsync()?.join()?;
    }
    let access = if status == UserNotificationListenerAccessStatus::Allowed { Access::Allowed } else { Access::Denied };
    shared.access.store(access as u8, Ordering::SeqCst);
    notifier.notify();
    if access != Access::Allowed {
        return Ok(());
    }

    let directory = format!(r"{}\Microsoft\Windows\Notifications", std::env::var("LOCALAPPDATA").unwrap_or_default());
    // Watching starts before the first read, which takes seconds, so a toast that arrives meanwhile is not lost.
    let changes = unsafe {
        FindFirstChangeNotificationW(
            &HSTRING::from(directory),
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
        )?
    };
    let started = Instant::now();
    let seen = read_ids(&listener)?;
    let high_water = seen.iter().copied().max().unwrap_or(0);
    log::debug(&format!(
        "notifications: listening, {} waiting, newest id {high_water}, first read took {} ms",
        seen.len(),
        started.elapsed().as_millis()
    ));
    let mut worker = Worker { shared, notifier, listener, logos: Logos::new(), seen, high_water };
    wait_for_quiet(changes, QUIET_TIME_MILLISECONDS);
    worker.handle_change(changes, Instant::now(), true, false);
    let removal_event = HANDLE(shared.removal_event_address as *mut _);
    let waited_for = [changes, removal_event];
    let handles = if removal_event.is_invalid() { &waited_for[..1] } else { &waited_for[..] };
    loop {
        let signalled = unsafe { WaitForMultipleObjects(handles, false, INFINITE) };
        if signalled.0 == WAIT_OBJECT_0.0 + 1 {
            worker.remove_dismissed(changes);
            continue;
        }
        let woke = Instant::now();
        // Windows usually has the toast by the first change; finding it now saves waiting for the writes to stop.
        let found_at_first_change = worker.ask_by_id(woke);
        // One toast writes the database for about 200 ms; asking in the middle of that can miss it, so ask again after.
        wait_for_quiet(changes, QUIET_TIME_MILLISECONDS);
        worker.handle_change(changes, woke, false, found_at_first_change);
    }
}

pub fn start(notifier: Notifier) -> Arc<Shared> {
    // An auto-reset event: one wake per batch of removals.
    let removal_event = unsafe { CreateEventW(None, false, false, PCWSTR::null()) }
        .inspect_err(|error| log::debug(&format!("could not create the removal event: {error}")))
        .unwrap_or_default();
    let shared = Arc::new(Shared {
        arrived: Mutex::new(Vec::new()),
        access: AtomicU8::new(Access::Unknown as u8),
        removals: Mutex::new(Vec::new()),
        removal_event_address: removal_event.0 as isize,
    });
    let worker = shared.clone();
    thread::spawn(move || {
        if let Err(error) = run(&worker, notifier) {
            log::debug(&format!("notifications: stopped: {error}"));
        }
    });
    shared
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_codes_round_trip() {
        for access in [Access::Unknown, Access::Allowed, Access::Denied] {
            assert_eq!(Access::from_code(access as u8), access);
        }
        assert_eq!(Access::from_code(99), Access::Unknown);
    }
}
