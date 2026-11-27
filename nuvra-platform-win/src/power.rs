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

pub fn parse_power_status(status: SystemPowerStatus) -> BatteryInfo {
    let unknown_flag = status.battery_flag == 255;
    let has_battery =
        !unknown_flag && (status.battery_flag & 128) == 0 && status.battery_life_percent != 255;
    let percentage = if status.battery_life_percent <= 100 {
        status.battery_life_percent as u32
    } else {
        100
    };
    // Only BatteryFlag bit 8 (Charging) with known flag (!= 255) indicates active charging
    let is_charging = !unknown_flag && (status.battery_flag & 8) != 0;

    BatteryInfo {
        percentage,
        is_charging,
        has_battery,
    }
}

impl BatteryInfo {
    pub fn query() -> Self {
        let mut status = SystemPowerStatus::default();
        // SAFETY: Passing a valid pointer to a stack-allocated SystemPowerStatus struct.
        let success = unsafe { GetSystemPowerStatus(&mut status as *mut _) };

        if success != 0 {
            parse_power_status(status)
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

    #[test]
    fn test_charging_flag_and_unknown_flag_behavior() {
        // AC online (1) without charging bit (8) -> not charging
        let plugged_idle = parse_power_status(SystemPowerStatus {
            ac_line_status: 1,
            battery_flag: 1,
            battery_life_percent: 80,
            ..Default::default()
        });
        assert!(!plugged_idle.is_charging);
        assert!(plugged_idle.has_battery);

        // BatteryFlag == 255 (unknown) has bit 8 set; must NOT report charging
        let unknown = parse_power_status(SystemPowerStatus {
            ac_line_status: 1,
            battery_flag: 255,
            battery_life_percent: 50,
            ..Default::default()
        });
        assert!(!unknown.is_charging);
        assert!(!unknown.has_battery);

        // BatteryFlag with bit 8 set -> charging
        let charging = parse_power_status(SystemPowerStatus {
            ac_line_status: 1,
            battery_flag: 8,
            battery_life_percent: 45,
            ..Default::default()
        });
        assert!(charging.is_charging);
    }
}
