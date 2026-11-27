// nuvra-platform-win/src/theme_sync.rs
// Windows personalization sync (dark/light/high-contrast) and per-user startup Run key (Section 14.1, 14.7).
// R-13: Unsafe confined here with // SAFETY: comments.

#![allow(unsafe_code)]

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::UI::ViewManagement::{AccessibilitySettings, UIColorType, UISettings};

const RUN_SUBKEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const RUN_VALUE: PCWSTR = w!("Nuvra");

/// Query Windows dark mode preference via WinRT UISettings background color luminance.
pub fn is_system_dark_mode() -> bool {
    let Ok(settings) = UISettings::new() else {
        return true;
    };
    let Ok(bg) = settings.GetColorValue(UIColorType::Background) else {
        return true;
    };
    (u16::from(bg.R) + u16::from(bg.G) + u16::from(bg.B)) < 384
}

/// Query Windows system High Contrast mode via WinRT AccessibilitySettings.
pub fn is_high_contrast() -> bool {
    AccessibilitySettings::new()
        .and_then(|a| a.HighContrast())
        .unwrap_or(false)
}

/// Check if Nuvra is registered in HKCU\...\Run for launch at startup.
pub fn is_startup_enabled() -> bool {
    let mut hkey = HKEY::default();
    // SAFETY: Read-only HKCU registry open + value existence query + close.
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_READ, &mut hkey).is_err() {
            return false;
        }
        let exists = RegQueryValueExW(hkey, RUN_VALUE, None, None, None, None).is_ok();
        let _ = RegCloseKey(hkey);
        exists
    }
}

fn is_missing_reg_code(code: WIN32_ERROR) -> bool {
    code == ERROR_FILE_NOT_FOUND || code == ERROR_PATH_NOT_FOUND
}

/// Enable or disable launch at startup under HKCU\Software\Microsoft\Windows\CurrentVersion\Run.
pub fn set_startup_enabled(enable: bool) -> bool {
    let mut hkey = HKEY::default();
    // SAFETY: HKCU Run key creation/modification bounded strictly to the "Nuvra" value.
    unsafe {
        if enable {
            if RegCreateKeyExW(
                HKEY_CURRENT_USER,
                RUN_SUBKEY,
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut hkey,
                None,
            )
            .is_err()
            {
                return false;
            }
            let exe = std::env::current_exe()
                .ok()
                .map(|p| format!("\"{}\"", p.display()));
            let ok = if let Some(cmd) = exe {
                let mut wide: Vec<u16> = cmd.encode_utf16().collect();
                wide.push(0);
                let bytes = std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * 2);
                RegSetValueExW(hkey, RUN_VALUE, 0, REG_SZ, Some(bytes)).is_ok()
            } else {
                false
            };
            let _ = RegCloseKey(hkey);
            ok
        } else {
            let open_res = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_WRITE, &mut hkey);
            if open_res == ERROR_SUCCESS {
                let del_res = RegDeleteValueW(hkey, RUN_VALUE);
                let _ = RegCloseKey(hkey);
                del_res == ERROR_SUCCESS || is_missing_reg_code(del_res)
            } else {
                is_missing_reg_code(open_res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_and_startup_queries_safe() {
        let _ = is_system_dark_mode();
        let _ = is_high_contrast();
        let _ = is_startup_enabled();
        assert!(is_missing_reg_code(ERROR_FILE_NOT_FOUND));
        assert!(is_missing_reg_code(ERROR_PATH_NOT_FOUND));
        assert!(!is_missing_reg_code(ERROR_SUCCESS));
    }
}
