// nuvra-platform-win/src/monitor.rs
// Multi-monitor geometry, DPI calculation, and fullscreen auto-hide (D-WIN-06, D-WIN-07).
// Every unsafe block carries a // SAFETY: comment.

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetShellWindow, GetWindowRect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl MonitorBounds {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Retrieve the bounding rectangle of the primary monitor.
pub fn get_primary_monitor_bounds() -> MonitorBounds {
    // SAFETY: MonitorFromWindow and GetMonitorInfoW with valid MONITORINFO structure.
    unsafe {
        let hmonitor = MonitorFromWindow(HWND(std::ptr::null_mut()), MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            MonitorBounds {
                left: info.rcMonitor.left,
                top: info.rcMonitor.top,
                right: info.rcMonitor.right,
                bottom: info.rcMonitor.bottom,
            }
        } else {
            // Fallback default 1920x1080
            MonitorBounds {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            }
        }
    }
}

/// Check if the currently active foreground window is occupying the entire screen in fullscreen (D-WIN-07).
/// Used by the Notch engine to auto-hide the notch unless pinned.
pub fn is_foreground_fullscreen() -> bool {
    // SAFETY: GetForegroundWindow, GetDesktopWindow, GetShellWindow, and GetWindowRect queries.
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd.0.is_null() {
            return false;
        }

        let desktop = GetDesktopWindow();
        let shell = GetShellWindow();
        if fg_hwnd == desktop || fg_hwnd == shell {
            return false;
        }

        // Filter out desktop background (WorkerW, Progman) and Windows taskbars
        let mut class_name = [0u16; 256];
        let len = GetClassNameW(fg_hwnd, &mut class_name);
        if len > 0 {
            let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
            if class_str == "WorkerW" || class_str == "Progman" || class_str.starts_with("Shell_") {
                return false;
            }
        }

        let mut window_rect = RECT::default();
        if GetWindowRect(fg_hwnd, &mut window_rect).is_err() {
            return false;
        }

        let hmonitor = MonitorFromWindow(fg_hwnd, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            return false;
        }

        // Fullscreen match: covers or exceeds the monitor rect
        window_rect.left <= info.rcMonitor.left
            && window_rect.top <= info.rcMonitor.top
            && window_rect.right >= info.rcMonitor.right
            && window_rect.bottom >= info.rcMonitor.bottom
    }
}
