// nuvra-platform-win/src/foreground.rs
// Foreground window class inspection (clipboard privacy), visible titles (call detection), and DND check.
// R-13: Unsafe confined here with // SAFETY: comments.

#![allow(unsafe_code)]

use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_ACCEPTS_NOTIFICATIONS, QUNS_APP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    IsWindowVisible,
};

/// Return the Win32 class name of the current foreground window (never logs text).
pub fn get_foreground_window_class() -> Option<String> {
    // SAFETY: GetForegroundWindow returns null or valid HWND; GetClassNameW writes bounded UTF-16 buffer.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 128];
        let len = GetClassNameW(hwnd, &mut buf);
        if len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}

/// Enumerate titles of top-level visible windows (for local CallDetector matching only).
pub fn list_visible_window_titles() -> Vec<String> {
    let mut titles: Vec<String> = Vec::new();
    let ptr = (&mut titles as *mut Vec<String>) as isize;
    // SAFETY: EnumWindows synchronously invokes callback with valid Vec pointer on current thread.
    unsafe {
        let _ = EnumWindows(Some(enum_win_cb), LPARAM(ptr));
    }
    titles
}

unsafe extern "system" fn enum_win_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 || len > 256 {
        return BOOL(1);
    }
    let mut buf = vec![0u16; (len as usize) + 1];
    let copied = GetWindowTextW(hwnd, &mut buf);
    if copied > 0 {
        let titles = &mut *(lparam.0 as *mut Vec<String>);
        titles.push(String::from_utf16_lossy(&buf[..copied as usize]));
    }
    BOOL(1)
}

/// Check if Windows Focus Assist / Presentation / Fullscreen DND suppresses toast popups.
pub fn is_dnd_active() -> bool {
    // SAFETY: SHQueryUserNotificationState queries current shell notification state safely.
    unsafe {
        match SHQueryUserNotificationState() {
            Ok(state) => state != QUNS_ACCEPTS_NOTIFICATIONS && state != QUNS_APP,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnd_and_window_queries_do_not_panic() {
        let _ = is_dnd_active();
        let _ = get_foreground_window_class();
        let _ = list_visible_window_titles();
    }
}
