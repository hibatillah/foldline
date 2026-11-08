use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

/// `BatteryFlag` bit that Windows sets when the machine has no battery.
const NO_SYSTEM_BATTERY: u8 = 128;
/// `BatteryFlag` value for "status unknown", which also has the no-battery bit set.
const UNKNOWN_STATUS: u8 = 255;

/// Whether this machine has a battery; true when Windows cannot say, so a laptop never loses its ring.
pub fn has_system_battery() -> bool {
    let mut status = SYSTEM_POWER_STATUS::default();
    match unsafe { GetSystemPowerStatus(&mut status) } {
        Ok(()) => battery_present(status.BatteryFlag),
        Err(error) => {
            crate::log::debug(&format!("could not read the power status: {error}"));
            true
        }
    }
}

fn battery_present(battery_flag: u8) -> bool {
    battery_flag == UNKNOWN_STATUS || battery_flag & NO_SYSTEM_BATTERY == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_no_battery_flag_means_a_desktop() {
        assert!(!battery_present(128));
        assert!(!battery_present(128 | 4));
    }

    #[test]
    fn charging_low_and_unknown_flags_still_mean_a_battery() {
        assert!(battery_present(1));
        assert!(battery_present(8));
        assert!(battery_present(UNKNOWN_STATUS));
    }
}
