// nuvra-platform-win/src/camera.rs
// Windows webcam sensor state monitor via CapabilityAccessManager registry (Section 14.3).
// Every unsafe block carries a // SAFETY: comment.

use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE,
};

pub struct CameraMonitor {
    hkey: HKEY,
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

    /// Check if any active application has open access to the camera.
    pub fn is_camera_active(&self) -> bool {
        // SAFETY: Read subkeys or global values under ConsentStore\webcam
        unsafe {
            let mut value_type = REG_VALUE_TYPE(0);
            let mut stop_time = 0u64;
            let mut size = std::mem::size_of::<u64>() as u32;

            let res = RegQueryValueExW(
                self.hkey,
                w!("LastUsedTimeStop"),
                None,
                Some(&mut value_type),
                Some(&mut stop_time as *mut _ as *mut u8),
                Some(&mut size),
            );

            // In Windows 10/11, stop_time == 0 indicates active capture session
            res.is_ok() && stop_time == 0
        }
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
