// nuvra-platform-win/src/power.rs
// Real-time battery percentage and charging telemetry via Win32 GetSystemPowerStatus.
// R-13: Unsafe Win32 call confined with // SAFETY: comment.
#![allow(unsafe_code)]

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemPowerStatus {
    pub ac_line_status: u8,
    pub battery_flag: u8,
    pub battery_life_percent: u8,
    pub system_status_flag: u8,
    pub battery_life_time: u32,
    pub battery_full_life_time: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemPowerStatus(lpSystemPowerStatus: *mut SystemPowerStatus) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryInfo {
    pub percentage: u32,
    pub is_charging: bool,
    pub has_battery: bool,
}

impl BatteryInfo {
    pub fn query() -> Self {
        let mut status = SystemPowerStatus::default();
        // SAFETY: Passing a valid pointer to a stack-allocated SystemPowerStatus struct.
        let success = unsafe { GetSystemPowerStatus(&mut status as *mut _) };

        if success != 0 {
            let has_battery = (status.battery_flag & 128) == 0 && status.battery_life_percent != 255;
            let percentage = if status.battery_life_percent <= 100 {
                status.battery_life_percent as u32
            } else {
                100
            };
            // AC connected (1) or BatteryFlag charging bit (8) set
            let is_charging = (status.battery_flag & 8) != 0 || status.ac_line_status == 1;

            Self {
                percentage,
                is_charging,
                has_battery,
            }
        } else {
            Self {
                percentage: 100,
                is_charging: false,
                has_battery: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_query_returns_valid_percentage() {
        let info = BatteryInfo::query();
        assert!(info.percentage <= 100);
    }
}
