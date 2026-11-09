use super::{dates::seconds_from_civil, ics::Clock};
use windows::Win32::{
    Foundation::FILETIME,
    System::{
        SystemInformation::GetLocalTime,
        Time::{FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime},
    },
};

pub(super) const UNIX_TO_FILE_TIME_SECONDS: i64 = 11_644_473_600;
pub(super) const FILE_TIME_TICKS_PER_SECOND: i64 = 10_000_000;

pub(super) struct SystemClock;

impl Clock for SystemClock {
    fn utc_to_local(&self, utc: i64) -> i64 {
        let ticks = (utc + UNIX_TO_FILE_TIME_SECONDS) * FILE_TIME_TICKS_PER_SECOND;
        let file_time = FILETIME { dwLowDateTime: ticks as u32, dwHighDateTime: (ticks >> 32) as u32 };
        let (mut universal, mut local, mut local_file_time) =
            (Default::default(), Default::default(), FILETIME::default());
        let converted = unsafe {
            FileTimeToSystemTime(&file_time, &mut universal).is_ok()
                && SystemTimeToTzSpecificLocalTime(None, &universal, &mut local).is_ok()
                && SystemTimeToFileTime(&local, &mut local_file_time).is_ok()
        };
        if !converted {
            return utc;
        }
        let local_ticks = (i64::from(local_file_time.dwHighDateTime) << 32) | i64::from(local_file_time.dwLowDateTime);
        local_ticks / FILE_TIME_TICKS_PER_SECOND - UNIX_TO_FILE_TIME_SECONDS
    }
}

pub(super) fn local_now() -> i64 {
    let time = unsafe { GetLocalTime() };
    seconds_from_civil(
        i64::from(time.wYear),
        u32::from(time.wMonth),
        u32::from(time.wDay),
        u32::from(time.wHour),
        u32::from(time.wMinute),
        u32::from(time.wSecond),
    )
}

pub(super) fn local_milliseconds_into_minute() -> i64 {
    let time = unsafe { GetLocalTime() };
    i64::from(time.wSecond) * 1000 + i64::from(time.wMilliseconds)
}

#[cfg(test)]
mod tests {
    use super::{
        super::dates::{SECONDS_PER_DAY, days_from_civil},
        *,
    };

    #[test]
    fn utc_converts_to_local_time_without_going_backwards() {
        let utc = days_from_civil(2026, 10, 24) * SECONDS_PER_DAY + 12 * 3600;
        let local = SystemClock.utc_to_local(utc);
        assert!((local - utc).abs() <= 14 * 3600);
        assert_eq!(SystemClock.utc_to_local(utc + 3600) - local, 3600);
    }
}
