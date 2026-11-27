// nuvra-platform-win/src/camera.rs
// Windows webcam sensor state monitor via CapabilityAccessManager registry (Section 14.3).
// Every unsafe block carries a // SAFETY: comment.

use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    REG_QWORD, REG_VALUE_TYPE,
};

pub struct CameraMonitor {
    hkey: HKEY,
}

unsafe fn query_last_used_stop_active(hkey: HKEY) -> bool {
    let mut value_type = REG_VALUE_TYPE(0);
    let mut stop_time = 0u64;
    let mut size = std::mem::size_of::<u64>() as u32;
    let res = RegQueryValueExW(
        hkey,
        w!("LastUsedTimeStop"),
        None,
        Some(&mut value_type),
        Some((&mut stop_time as *mut u64).cast::<u8>()),
        Some(&mut size),
    );
    res.is_ok()
        && value_type == REG_QWORD
        && size as usize == std::mem::size_of::<u64>()
        && stop_time == 0
}

unsafe fn check_child_keys_recursive(parent: HKEY, recurse_non_packaged: bool) -> bool {
    let mut index = 0u32;
    let mut name_buf = [0u16; 260];
    loop {
        let mut name_len = name_buf.len() as u32;
        let enum_res = RegEnumKeyExW(
            parent,
            index,
            PWSTR(name_buf.as_mut_ptr()),
            &mut name_len,
            None,
            PWSTR::null(),
            None,
            None,
        );
        if enum_res == ERROR_NO_MORE_ITEMS || enum_res.is_err() {
            break;
        }
        index = index.saturating_add(1);

        let child_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
        let mut child_key = HKEY::default();
        if RegOpenKeyExW(
            parent,
            PCWSTR(name_buf.as_ptr()),
            0,
            KEY_READ,
            &mut child_key,
        )
        .is_err()
        {
            continue;
        }

        let active = if recurse_non_packaged && child_name.eq_ignore_ascii_case("NonPackaged") {
            check_child_keys_recursive(child_key, false)
        } else {
            query_last_used_stop_active(child_key)
        };
        let _ = RegCloseKey(child_key);

        if active {
            return true;
        }
    }
    false
}

impl CameraMonitor {
    pub fn new() -> Option<Self> {
        let subkey = w!(
            r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam"
        );
        let mut hkey = HKEY::default();

        // SAFETY: RegOpenKeyExW query under HKCU.
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_ok() {
                Some(Self { hkey })
            } else {
                None
            }
        }
    }

    /// Check if any packaged or NonPackaged application child key reports an active camera session.
    pub fn is_camera_active(&self) -> bool {
        // SAFETY: Enumerates child subkeys under open ConsentStore\webcam handle and closes each child handle.
        unsafe { check_child_keys_recursive(self.hkey, true) }
    }
}

impl Drop for CameraMonitor {
    fn drop(&mut self) {
        // SAFETY: Close registry key.
        unsafe {
            let _ = RegCloseKey(self.hkey);
        }
    }
}

// SAFETY: Registry handles (HKEY) opened with KEY_READ are thread-safe to transfer across threads.
unsafe impl Send for CameraMonitor {}
unsafe impl Sync for CameraMonitor {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_monitor_child_key_query_safe() {
        if let Some(mon) = CameraMonitor::new() {
            let _active = mon.is_camera_active();
        }
    }
}
