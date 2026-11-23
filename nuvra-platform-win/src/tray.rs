// nuvra-platform-win/src/tray.rs
// Windows System Tray icon and context menu lifecycle (Section 14.7).
// Every unsafe block carries a // SAFETY: comment.

use windows::core::w;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadIconW, TrackPopupMenuEx, HMENU,
    IDI_APPLICATION, MF_SEPARATOR, MF_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
};

pub const WM_TRAY_CALLBACK: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 101;

pub const TRAY_CMD_TOGGLE_NOTCH: u32 = 1001;
pub const TRAY_CMD_SETTINGS: u32 = 1002;
pub const TRAY_CMD_QUIT: u32 = 1003;

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Result<Self, String> {
        // SAFETY: Initialize NOTIFYICONDATAW and call Shell_NotifyIconW.
        unsafe {
            let hicon =
                LoadIconW(None, IDI_APPLICATION).map_err(|e| format!("LoadIcon failed: {e}"))?;

            let mut tip = [0u16; 128];
            let tip_text = w!("Nuvra — Dynamic Notch");
            let len = tip_text.as_wide().len().min(127);
            tip[..len].copy_from_slice(&tip_text.as_wide()[..len]);

            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAY_CALLBACK,
                hIcon: hicon,
                szTip: tip,
                ..Default::default()
            };

            if Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
                Ok(Self { hwnd, nid })
            } else {
                Err("Shell_NotifyIconW NIM_ADD failed".into())
            }
        }
    }

    /// Display the context menu at the current cursor position.
    /// Returns the selected command ID, or 0 if dismissed.
    pub fn show_context_menu(&self) -> u32 {
        // SAFETY: CreatePopupMenu, TrackPopupMenuEx, and DestroyMenu.
        unsafe {
            let menu: HMENU = match CreatePopupMenu() {
                Ok(m) => m,
                Err(_) => return 0,
            };

            let _ = AppendMenuW(
                menu,
                MF_STRING,
                TRAY_CMD_TOGGLE_NOTCH as usize,
                w!("Show / Hide Notch"),
            );
            let _ = AppendMenuW(
                menu,
                MF_STRING,
                TRAY_CMD_SETTINGS as usize,
                w!("Settings..."),
            );
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, w!(""));
            let _ = AppendMenuW(menu, MF_STRING, TRAY_CMD_QUIT as usize, w!("Quit Nuvra"));

            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);

            let cmd = TrackPopupMenuEx(
                menu,
                TPM_RETURNCMD.0 | TPM_LEFTALIGN.0 | TPM_BOTTOMALIGN.0,
                pt.x,
                pt.y,
                self.hwnd,
                None,
            );

            let _ = DestroyMenu(menu);
            cmd.0 as u32
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        // SAFETY: Remove tray icon from shell taskbar notification area.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }
}
