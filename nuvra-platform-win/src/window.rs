// nuvra-platform-win/src/window.rs
// Topmost borderless Win32 notch window with 2D repositioning & compositor integration (UI-1, DECIDED-007).
// R-13: ONLY file in nuvra-platform-win using unsafe for Win32 window APIs.
// Every unsafe block carries a // SAFETY: comment.
#![allow(unsafe_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
};
use windows::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, PeekMessageW, PostQuitMessage,
    RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_DBLCLKS, GWLP_USERDATA,
    HTCLIENT, HTTRANSPARENT, MSG, PM_REMOVE, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SW_SHOWNA,
    WM_CANCELMODE, WM_CAPTURECHANGED, WM_DESTROY, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_ERASEBKGND,
    WM_HOTKEY, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_MOUSEMOVE, WM_NCHITTEST, WM_PAINT, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

pub const WM_MOUSELEAVE: u32 = 0x02A3;
pub const WM_DROPFILES_MSG: u32 = 0x0233;

/// Interaction state communicated from the Win32 message pump to the Notch controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowEvent {
    MouseEnter,
    MouseLeave,
    Click { x: i32, y: i32 },
    DragStart { x: i32, y: i32 },
    DoubleClick { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    Moved { x: i32, y: i32 },
    HotKey { id: i32 },
    DpiChanged { new_dpi: u32 },
    DisplayChanged,
    TrayClick,
    FileDropped(PathBuf),
}

struct WindowSharedState {
    hit_x: AtomicI32,
    hit_y: AtomicI32,
    hit_width: AtomicI32,
    hit_height: AtomicI32,
    hover_x: AtomicI32,
    hover_y: AtomicI32,
    is_hovered: AtomicBool,
    event_tx: flume::Sender<WindowEvent>,
    current_dpi: AtomicU32,
    drag_active: AtomicBool,
    drag_start_x: AtomicI32,
    drag_start_y: AtomicI32,
    drag_start_win_x: AtomicI32,
    drag_start_win_y: AtomicI32,
    l_down_x: AtomicI32,
    l_down_y: AtomicI32,
    l_is_down: AtomicBool,
    r_down_x: AtomicI32,
    r_down_y: AtomicI32,
    r_is_down: AtomicBool,
    user_center_x: AtomicI32,
    user_offset_y: AtomicI32,
    current_x: AtomicI32,
    current_y: AtomicI32,
    current_width: AtomicI32,
    current_height: AtomicI32,
}

pub struct NotchWindow {
    hwnd: HWND,
    shared: Arc<WindowSharedState>,
    event_rx: flume::Receiver<WindowEvent>,
}

impl NotchWindow {
    /// Create a new topmost, borderless, layered Notch window centered on the primary monitor.
    pub fn new(initial_width: u32, initial_height: u32) -> Result<Self, String> {
        let (tx, rx) = flume::bounded(64);
        let shared = Arc::new(WindowSharedState {
            hit_x: AtomicI32::new(0),
            hit_y: AtomicI32::new(0),
            hit_width: AtomicI32::new(initial_width as i32),
            hit_height: AtomicI32::new(initial_height as i32),
            hover_x: AtomicI32::new(-1),
            hover_y: AtomicI32::new(-1),
            is_hovered: AtomicBool::new(false),
            event_tx: tx,
            current_dpi: AtomicU32::new(96),
            drag_active: AtomicBool::new(false),
            drag_start_x: AtomicI32::new(0),
            drag_start_y: AtomicI32::new(0),
            drag_start_win_x: AtomicI32::new(0),
            drag_start_win_y: AtomicI32::new(0),
            l_down_x: AtomicI32::new(0),
            l_down_y: AtomicI32::new(0),
            l_is_down: AtomicBool::new(false),
            r_down_x: AtomicI32::new(0),
            r_down_y: AtomicI32::new(0),
            r_is_down: AtomicBool::new(false),
            user_center_x: AtomicI32::new(0),
            user_offset_y: AtomicI32::new(0),
            current_x: AtomicI32::new(0),
            current_y: AtomicI32::new(0),
            current_width: AtomicI32::new(initial_width as i32),
            current_height: AtomicI32::new(initial_height as i32),
        });

        // SAFETY: RegisterClassExW and CreateWindowExW with verified class names and styles.
        let hwnd = unsafe {
            let hinstance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW: {e}"))?;
            let class_name = w!("NuvraNotchWindowClass");

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(notch_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassExW(&wnd_class);

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name,
                w!("NuvraNotch"),
                WS_POPUP,
                0,
                0,
                initial_width as i32,
                initial_height as i32,
                HWND(std::ptr::null_mut()),
                None,
                hinstance,
                None,
            )
            .map_err(|e| format!("CreateWindowExW failed: {e}"))?;

            // Store shared state pointer in GWLP_USERDATA and enable file drag-drop
            let ptr = Arc::into_raw(Arc::clone(&shared)) as isize;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr);
            DragAcceptFiles(hwnd, true);

            let dpi = GetDpiForWindow(hwnd);
            if dpi > 0 {
                shared.current_dpi.store(dpi, Ordering::Relaxed);
            }

            let _ = ShowWindow(hwnd, SW_SHOWNA);
            hwnd
        };

        Ok(Self {
            hwnd,
            shared,
            event_rx: rx,
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
    pub fn event_receiver(&self) -> &flume::Receiver<WindowEvent> {
        &self.event_rx
    }
    pub fn current_dpi(&self) -> u32 {
        self.shared.current_dpi.load(Ordering::Relaxed)
    }
    pub fn user_center_x(&self) -> i32 {
        self.shared.user_center_x.load(Ordering::Acquire)
    }
    pub fn set_user_center_x(&self, cx: i32) {
        self.shared.user_center_x.store(cx, Ordering::Release);
    }
    pub fn user_offset_y(&self) -> i32 {
        self.shared.user_offset_y.load(Ordering::Acquire)
    }
    pub fn set_user_offset_y(&self, oy: i32) {
        self.shared.user_offset_y.store(oy, Ordering::Release);
    }
    pub fn is_dragging(&self) -> bool {
        self.shared.drag_active.load(Ordering::Acquire)
    }

    pub fn update_hit_bounds(&self, x: i32, y: i32, width: u32, height: u32) {
        self.shared.hit_x.store(x, Ordering::Release);
        self.shared.hit_y.store(y, Ordering::Release);
        self.shared.hit_width.store(width as i32, Ordering::Release);
        self.shared
            .hit_height
            .store(height as i32, Ordering::Release);
    }

    pub fn hover_pos(&self) -> Option<(f32, f32)> {
        let hx = self.shared.hover_x.load(Ordering::Acquire);
        let hy = self.shared.hover_y.load(Ordering::Acquire);
        if hx >= 0 && hy >= 0 {
            Some((hx as f32, hy as f32))
        } else {
            None
        }
    }

    /// Directly sample live OS cursor screen coordinates for jitter-free hover detection.
    pub fn cursor_screen_pos(&self) -> Option<(i32, i32)> {
        let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
        // SAFETY: Valid POINT pointer passed to GetCursorPos.
        unsafe { GetCursorPos(&mut pt).ok().map(|()| (pt.x, pt.y)) }
    }

    /// Position the physical Notch window on screen.
    pub fn set_geometry(&self, x: i32, y: i32, width: u32, height: u32) {
        self.shared.current_x.store(x, Ordering::Release);
        self.shared.current_y.store(y, Ordering::Release);
        self.shared
            .current_width
            .store(width as i32, Ordering::Release);
        self.shared
            .current_height
            .store(height as i32, Ordering::Release);
        self.shared
            .user_center_x
            .store(x + (width as i32) / 2, Ordering::Release);
        self.shared.user_offset_y.store(y, Ordering::Release);
        // SAFETY: SetWindowPos with SWP_NOACTIVATE and SWP_NOZORDER preserves topmost layer.
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                HWND(std::ptr::null_mut()),
                x,
                y,
                width as i32,
                height as i32,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
    }

    /// Update hit bounds matching the notch shape (skipping SetWindowPos when unchanged).
    pub fn set_pill_bounds(&self, x: i32, y: i32, width: u32, height: u32, _radius: f32) {
        let w = width as i32;
        let h = height as i32;
        let prev_x = self.shared.current_x.swap(x, Ordering::AcqRel);
        let prev_y = self.shared.current_y.swap(y, Ordering::AcqRel);
        let prev_w = self.shared.current_width.swap(w, Ordering::AcqRel);
        let prev_h = self.shared.current_height.swap(h, Ordering::AcqRel);

        self.shared.hit_x.store(0, Ordering::Release);
        self.shared.hit_y.store(0, Ordering::Release);
        self.shared.hit_width.store(w, Ordering::Release);
        self.shared.hit_height.store(h, Ordering::Release);

        if prev_x != x || prev_y != y || prev_w != w || prev_h != h {
            // SAFETY: SetWindowPos only runs when bounds changed, avoiding per-frame DWM/input contention.
            unsafe {
                let _ = SetWindowPos(
                    self.hwnd,
                    HWND(std::ptr::null_mut()),
                    x,
                    y,
                    w,
                    h,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
            }
        }
    }

    /// Non-blocking message pump for the notch window.
    pub fn pump_messages(&self) -> bool {
        let mut msg = MSG::default();
        // SAFETY: Standard non-blocking PeekMessageW loop.
        unsafe {
            while PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                    return false;
                }
                let _ = DispatchMessageW(&msg);
            }
        }
        true
    }
}

impl Drop for NotchWindow {
    fn drop(&mut self) {
        // SAFETY: Cleanly destroy window and retrieve the raw Arc pointer.
        unsafe {
            let ptr = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            if ptr != 0 {
                let _ = Arc::from_raw(ptr as *const WindowSharedState);
            }
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.hwnd);
        }
    }
}

// SAFETY: Window procedure executing on the UI thread for notch message handling.
unsafe extern "system" fn notch_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if ptr == 0 {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let shared = &*(ptr as *const WindowSharedState);

    match msg {
        WM_NCHITTEST => {
            // Use lparam (exact screen coords for this hit-test) instead of stale GetMessagePos
            let screen_x = (lparam.0 & 0xFFFF) as i16 as i32;
            let screen_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = ClientToScreen(hwnd, &mut pt);

            let local_x = screen_x - pt.x;
            let local_y = screen_y - pt.y;

            let hx = shared.hit_x.load(Ordering::Acquire);
            let hy = shared.hit_y.load(Ordering::Acquire);
            let hw = shared.hit_width.load(Ordering::Acquire);
            let hh = shared.hit_height.load(Ordering::Acquire);

            if local_x >= hx && local_x <= (hx + hw) && local_y >= hy && local_y <= (hy + hh) {
                LRESULT(HTCLIENT as isize)
            } else {
                LRESULT(HTTRANSPARENT as isize)
            }
        }
        WM_MOUSEACTIVATE => {
            // MA_NOACTIVATE (3): allow clicks inside notch without stealing focus from foreground apps
            LRESULT(3)
        }
        WM_MOUSEMOVE => {
            let lx = (lparam.0 & 0xFFFF) as i16 as i32;
            let ly = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            shared.hover_x.store(lx, Ordering::Release);
            shared.hover_y.store(ly, Ordering::Release);

            // Left-mouse drag (>=6px): emit OLE file drag-out trigger without hijacking normal clicks
            if shared.l_is_down.load(Ordering::Acquire) {
                let sx = shared.l_down_x.load(Ordering::Acquire);
                let sy = shared.l_down_y.load(Ordering::Acquire);
                if ((lx - sx).abs() >= 6 || (ly - sy).abs() >= 6)
                    && shared.l_is_down.swap(false, Ordering::AcqRel)
                {
                    let _ = shared
                        .event_tx
                        .try_send(WindowEvent::DragStart { x: sx, y: sy });
                }
            }

            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);

            // Only right-mouse drag (>4px threshold) initiates 2D move mode so left-clicks never swallow buttons
            if shared.r_is_down.load(Ordering::Acquire)
                && !shared.drag_active.load(Ordering::Acquire)
            {
                let sx = shared.r_down_x.load(Ordering::Acquire);
                let sy = shared.r_down_y.load(Ordering::Acquire);
                if (pt.x - sx).abs() >= 4 || (pt.y - sy).abs() >= 4 {
                    shared.drag_active.store(true, Ordering::Release);
                    let _ = SetCapture(hwnd);
                }
            }

            // Real-time 2D notch repositioning (DECIDED-007: X and Y translation)
            if shared.drag_active.load(Ordering::Acquire) {
                let delta_x = pt.x - shared.drag_start_x.load(Ordering::Acquire);
                let delta_y = pt.y - shared.drag_start_y.load(Ordering::Acquire);

                let cur_w = shared.current_width.load(Ordering::Acquire);
                let raw_win_x = shared.drag_start_win_x.load(Ordering::Acquire) + delta_x;
                let raw_win_y = shared.drag_start_win_y.load(Ordering::Acquire) + delta_y;

                let m = crate::monitor::get_primary_monitor_bounds();
                let mon_center_x = m.left + m.width() / 2;

                let min_y = m.top;
                let max_y = (m.top + m.height() - 40).max(min_y);
                let final_win_y = if (raw_win_y - m.top).abs() <= 12 {
                    m.top
                } else {
                    raw_win_y.clamp(min_y, max_y)
                };

                let raw_cx = raw_win_x + cur_w / 2;
                let min_cx = m.left + cur_w / 2;
                let max_cx = m.left + m.width() - cur_w / 2;
                let final_cx = if (raw_cx - mon_center_x).abs() <= 12 || min_cx > max_cx {
                    mon_center_x
                } else {
                    raw_cx.clamp(min_cx, max_cx)
                };
                let final_win_x = final_cx - cur_w / 2;

                shared.user_center_x.store(final_cx, Ordering::Release);
                shared
                    .user_offset_y
                    .store(final_win_y - m.top, Ordering::Release);
                shared.current_x.store(final_win_x, Ordering::Release);
                shared.current_y.store(final_win_y, Ordering::Release);

                let _ = SetWindowPos(
                    hwnd,
                    HWND(std::ptr::null_mut()),
                    final_win_x,
                    final_win_y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                let _ = shared.event_tx.try_send(WindowEvent::Moved {
                    x: final_win_x,
                    y: final_win_y,
                });
                return LRESULT(0);
            }

            if !shared.is_hovered.swap(true, Ordering::AcqRel) {
                let _ = shared.event_tx.try_send(WindowEvent::MouseEnter);

                let mut tme = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tme);
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            shared.hover_x.store(-1, Ordering::Release);
            shared.hover_y.store(-1, Ordering::Release);
            if !shared.drag_active.load(Ordering::Acquire) {
                shared.is_hovered.store(false, Ordering::Release);
                let _ = shared.event_tx.try_send(WindowEvent::MouseLeave);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let lx = (lparam.0 & 0xFFFF) as i16 as i32;
            let ly = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            shared.l_down_x.store(lx, Ordering::Release);
            shared.l_down_y.store(ly, Ordering::Release);
            shared.l_is_down.store(true, Ordering::Release);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if shared.l_is_down.swap(false, Ordering::AcqRel) {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let _ = shared.event_tx.try_send(WindowEvent::Click { x, y });
            }
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = shared.event_tx.try_send(WindowEvent::DoubleClick { x, y });
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);
            shared.r_down_x.store(pt.x, Ordering::Release);
            shared.r_down_y.store(pt.y, Ordering::Release);
            shared.drag_start_x.store(pt.x, Ordering::Release);
            shared.drag_start_y.store(pt.y, Ordering::Release);
            shared
                .drag_start_win_x
                .store(shared.current_x.load(Ordering::Acquire), Ordering::Release);
            shared
                .drag_start_win_y
                .store(shared.current_y.load(Ordering::Acquire), Ordering::Release);
            shared.r_is_down.store(true, Ordering::Release);
            let _ = SetCapture(hwnd);
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            let was_down = shared.r_is_down.swap(false, Ordering::AcqRel);
            let was_drag = shared.drag_active.swap(false, Ordering::AcqRel);
            if was_down || was_drag {
                let _ = ReleaseCapture();
            }
            if was_down && !was_drag {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let _ = shared.event_tx.try_send(WindowEvent::RightClick { x, y });
            }
            LRESULT(0)
        }
        WM_CAPTURECHANGED | WM_CANCELMODE => {
            shared.r_is_down.store(false, Ordering::Release);
            shared.drag_active.store(false, Ordering::Release);
            shared.l_is_down.store(false, Ordering::Release);
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if wparam.0 == 0x1B {
                let _ = shared.event_tx.try_send(WindowEvent::HotKey { id: 2 });
            }
            LRESULT(0)
        }
        WM_HOTKEY => {
            let id = wparam.0 as i32;
            let _ = shared.event_tx.try_send(WindowEvent::HotKey { id });
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let new_dpi = (wparam.0 & 0xFFFF) as u32;
            shared.current_dpi.store(new_dpi, Ordering::Release);
            let _ = shared
                .event_tx
                .try_send(WindowEvent::DpiChanged { new_dpi });
            let rect = &*(lparam.0 as *const RECT);
            let _ = SetWindowPos(
                hwnd,
                HWND(std::ptr::null_mut()),
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            let _ = shared.event_tx.try_send(WindowEvent::DisplayChanged);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::ValidateRect(hwnd, None);
            }
            LRESULT(0)
        }
        WM_DROPFILES_MSG => {
            let hdrop = HDROP(wparam.0 as *mut std::ffi::c_void);
            let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
            for i in 0..count {
                let len = DragQueryFileW(hdrop, i, None);
                if len > 0 {
                    let mut buf = vec![0u16; (len + 1) as usize];
                    let written = DragQueryFileW(hdrop, i, Some(&mut buf));
                    if written > 0 {
                        let path_str = String::from_utf16_lossy(&buf[..written as usize]);
                        let _ = shared
                            .event_tx
                            .try_send(WindowEvent::FileDropped(PathBuf::from(path_str)));
                    }
                }
            }
            DragFinish(hdrop);
            LRESULT(0)
        }
        m if m == crate::tray::WM_TRAY_CALLBACK => {
            let ev = lparam.0 as u32;
            if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP {
                let _ = shared.event_tx.try_send(WindowEvent::TrayClick);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
