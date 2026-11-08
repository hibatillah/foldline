//! HID handles: a collection's ids and usage, and the overlapped short and long HID++ channel with a bounded wait.

use std::time::{Duration, Instant};

use windows::{
    Win32::{
        Devices::HumanInterfaceDevice::{
            HIDD_ATTRIBUTES, HIDP_CAPS, HIDP_STATUS_SUCCESS, HidD_FreePreparsedData, HidD_GetAttributes,
            HidD_GetPreparsedData, HidP_GetCaps, PHIDP_PREPARSED_DATA,
        },
        Foundation::{CloseHandle, ERROR_IO_PENDING, GENERIC_READ, GENERIC_WRITE, HANDLE, WAIT_OBJECT_0},
        Storage::FileSystem::{
            CreateFileW, FILE_FLAG_OVERLAPPED, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING, ReadFile, WriteFile,
        },
        System::{
            IO::{CancelIoEx, GetOverlappedResult, GetOverlappedResultEx, OVERLAPPED},
            Threading::{CreateEventW, WaitForMultipleObjects},
        },
    },
    core::HSTRING,
};

use super::{
    hidpp::{Answer, Request},
    logitech::Transport,
};

/// How long one request waits for its reply; a receiver answers in a few milliseconds, an empty slot at once.
const REPLY_TIMEOUT: Duration = Duration::from_millis(300);

/// A handle that is closed when dropped.
pub(super) struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if let Err(error) = unsafe { CloseHandle(self.0) } {
            crate::log::debug(&format!("could not close a HID handle: {error}"));
        }
    }
}

/// What a HID collection is: its USB ids, usage and report sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CollectionInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub usage_page: u16,
    pub usage: u16,
    pub input_length: usize,
    pub output_length: usize,
}

/// Opens a collection without read or write access, which Windows allows even for a mouse or keyboard it holds.
pub(super) fn collection_info(path: &str) -> windows::core::Result<CollectionInfo> {
    let handle = open(path, 0, FILE_FLAGS_AND_ATTRIBUTES(0))?;
    let mut attributes = HIDD_ATTRIBUTES { Size: size_of::<HIDD_ATTRIBUTES>() as u32, ..Default::default() };
    if !unsafe { HidD_GetAttributes(handle.0, &mut attributes) } {
        return Err(windows::core::Error::from_thread());
    }
    let mut preparsed = PHIDP_PREPARSED_DATA::default();
    if !unsafe { HidD_GetPreparsedData(handle.0, &mut preparsed) } {
        return Err(windows::core::Error::from_thread());
    }
    let mut capabilities = HIDP_CAPS::default();
    let status = unsafe { HidP_GetCaps(preparsed, &mut capabilities) };
    unsafe { HidD_FreePreparsedData(preparsed) };
    if status != HIDP_STATUS_SUCCESS {
        return Err(windows::core::Error::from_hresult(status.to_hresult()));
    }
    Ok(CollectionInfo {
        vendor_id: attributes.VendorID,
        product_id: attributes.ProductID,
        usage_page: capabilities.UsagePage,
        usage: capabilities.Usage,
        input_length: usize::from(capabilities.InputReportByteLength),
        output_length: usize::from(capabilities.OutputReportByteLength),
    })
}

fn open(path: &str, access: u32, flags: FILE_FLAGS_AND_ATTRIBUTES) -> windows::core::Result<OwnedHandle> {
    let handle = unsafe {
        CreateFileW(&HSTRING::from(path), access, FILE_SHARE_READ | FILE_SHARE_WRITE, None, OPEN_EXISTING, flags, None)
    }?;
    Ok(OwnedHandle(handle))
}

/// One HID++ collection opened for overlapped reads and writes, with the read that may be in flight.
struct Endpoint {
    /// The kernel writes a pending read here, so `Drop` cancels the read before the buffer is freed.
    buffer: Box<[u8]>,
    overlapped: Box<OVERLAPPED>,
    is_reading: bool,
    output_length: usize,
    event: OwnedHandle,
    handle: OwnedHandle,
}

impl Endpoint {
    fn open(path: &str, info: CollectionInfo) -> windows::core::Result<Self> {
        let handle = open(path, GENERIC_READ.0 | GENERIC_WRITE.0, FILE_FLAG_OVERLAPPED)?;
        let event = OwnedHandle(unsafe { CreateEventW(None, true, false, None) }?);
        let overlapped = Box::new(OVERLAPPED { hEvent: event.0, ..Default::default() });
        Ok(Self {
            buffer: vec![0; info.input_length.max(1)].into_boxed_slice(),
            overlapped,
            is_reading: false,
            output_length: info.output_length,
            event,
            handle,
        })
    }

    /// Starts a read unless one is in flight; `Some` holds a report that arrived at once.
    fn start_read(&mut self) -> Option<Vec<u8>> {
        if self.is_reading {
            return None;
        }
        *self.overlapped = OVERLAPPED { hEvent: self.event.0, ..Default::default() };
        match unsafe { ReadFile(self.handle.0, Some(&mut self.buffer), None, Some(&mut *self.overlapped)) } {
            Ok(()) => self.finish_read(),
            Err(error) if error.code() == ERROR_IO_PENDING.to_hresult() => {
                self.is_reading = true;
                None
            }
            Err(_) => None,
        }
    }

    /// The report of a completed read.
    fn finish_read(&mut self) -> Option<Vec<u8>> {
        self.is_reading = false;
        let mut transferred = 0u32;
        unsafe { GetOverlappedResult(self.handle.0, &*self.overlapped, &mut transferred, false) }.ok()?;
        Some(self.buffer[..transferred as usize].to_vec())
    }

    fn cancel_read(&mut self) {
        if !self.is_reading {
            return;
        }
        let _ = unsafe { CancelIoEx(self.handle.0, Some(&*self.overlapped)) };
        // Waits for the cancelled read to finish so the kernel no longer writes into the buffer.
        let mut transferred = 0u32;
        let _ = unsafe { GetOverlappedResult(self.handle.0, &*self.overlapped, &mut transferred, true) };
        self.is_reading = false;
    }

    fn write(&self, report: &[u8], deadline: Instant) -> bool {
        let mut bytes = report.to_vec();
        bytes.resize(self.output_length.max(report.len()), 0);
        let Ok(event) = (unsafe { CreateEventW(None, true, false, None) }).map(OwnedHandle) else { return false };
        let mut overlapped = OVERLAPPED { hEvent: event.0, ..Default::default() };
        match unsafe { WriteFile(self.handle.0, Some(&bytes), None, Some(&mut overlapped)) } {
            Ok(()) => return true,
            Err(error) if error.code() == ERROR_IO_PENDING.to_hresult() => {}
            Err(error) => {
                crate::log::debug(&format!("could not write a HID++ request: {error}"));
                return false;
            }
        }
        let mut transferred = 0u32;
        let wait = remaining_milliseconds(deadline);
        if unsafe { GetOverlappedResultEx(self.handle.0, &overlapped, &mut transferred, wait, false) }.is_ok() {
            return true;
        }
        let _ = unsafe { CancelIoEx(self.handle.0, Some(&overlapped)) };
        let _ = unsafe { GetOverlappedResult(self.handle.0, &overlapped, &mut transferred, true) };
        false
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.cancel_read();
    }
}

/// A Logitech receiver's HID++ collections: requests go out on the short one when they fit, and replies are read on
/// both, since Windows splits the two report ids into separate collections.
pub(super) struct Channel {
    short: Option<Endpoint>,
    long: Option<Endpoint>,
}

impl Channel {
    /// Opens whichever of the two collections exist; fails when neither opens.
    pub(super) fn open(
        short: Option<(&str, CollectionInfo)>,
        long: Option<(&str, CollectionInfo)>,
    ) -> windows::core::Result<Self> {
        let mut last_error = None;
        let mut open_endpoint = |collection: Option<(&str, CollectionInfo)>| {
            let (path, info) = collection?;
            Endpoint::open(path, info).map_err(|error| last_error = Some(error)).ok()
        };
        let channel = Self { short: open_endpoint(short), long: open_endpoint(long) };
        if channel.short.is_none() && channel.long.is_none() {
            return Err(last_error.unwrap_or_else(|| windows::core::Error::from_hresult(windows::core::HRESULT(-1))));
        }
        Ok(channel)
    }

    fn endpoints(&mut self) -> impl Iterator<Item = &mut Endpoint> {
        self.short.iter_mut().chain(self.long.iter_mut())
    }
}

impl Transport for Channel {
    fn exchange(&mut self, request: &Request) -> Option<Answer> {
        let deadline = Instant::now() + REPLY_TIMEOUT;
        for endpoint in self.endpoints() {
            if let Some(report) = endpoint.start_read()
                && let Some(answer) = request.answer(&report)
            {
                return Some(answer);
            }
        }
        let sent = match (&self.short, &self.long) {
            (Some(short), _) if request.fits_short() => short.write(&request.encode(true), deadline),
            (_, Some(long)) => long.write(&request.encode(false), deadline),
            (Some(short), None) => short.write(&request.encode(true), deadline),
            (None, None) => false,
        };
        let answer = if sent { self.await_answer(request, deadline) } else { None };
        for endpoint in self.endpoints() {
            endpoint.cancel_read();
        }
        answer
    }
}

impl Channel {
    /// Reads both collections until a report answers `request` or the deadline passes; other reports are dropped.
    fn await_answer(&mut self, request: &Request, deadline: Instant) -> Option<Answer> {
        loop {
            for endpoint in self.endpoints() {
                if let Some(report) = endpoint.start_read()
                    && let Some(answer) = request.answer(&report)
                {
                    return Some(answer);
                }
            }
            let events: Vec<HANDLE> =
                self.endpoints().filter(|endpoint| endpoint.is_reading).map(|endpoint| endpoint.event.0).collect();
            let wait = remaining_milliseconds(deadline);
            if events.is_empty() || wait == 0 {
                return None;
            }
            let signalled =
                unsafe { WaitForMultipleObjects(&events, false, wait) }.0.wrapping_sub(WAIT_OBJECT_0.0) as usize;
            let event = *events.get(signalled)?;
            let endpoint = self.endpoints().find(|endpoint| endpoint.event.0 == event)?;
            if let Some(report) = endpoint.finish_read()
                && let Some(answer) = request.answer(&report)
            {
                return Some(answer);
            }
        }
    }
}

fn remaining_milliseconds(deadline: Instant) -> u32 {
    u32::try_from(deadline.saturating_duration_since(Instant::now()).as_millis()).unwrap_or(u32::MAX)
}
