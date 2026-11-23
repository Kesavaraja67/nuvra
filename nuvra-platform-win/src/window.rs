// nuvra-platform-win/src/window.rs
// Topmost transparent Win32 notch window lifecycle with dynamic hit-testing (D-WIN-01..09).
// ONLY file in nuvra-platform-win using unsafe for Win32 window APIs.
// Every unsafe block carries a // SAFETY: comment.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_SYSTEMBACKDROP_TYPE,
    DWMWINDOWATTRIBUTE,
};
use windows::Win32::Graphics::Gdi::{ClientToScreen, CreateRoundRectRgn, SetWindowRgn};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetMessagePos, PeekMessageW,
    PostQuitMessage, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CS_DBLCLKS,
    GWLP_USERDATA, HTCLIENT, HTTRANSPARENT, MSG, PM_REMOVE, SWP_NOACTIVATE, SWP_NOSIZE,
    SWP_NOZORDER, SW_SHOWNA, WM_DESTROY, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_ERASEBKGND, WM_HOTKEY,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCHITTEST, WM_PAINT,
    WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WNDCLASSEXW, WS_EX_NOACTIVATE,
    WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

pub const WM_MOUSELEAVE: u32 = 0x02A3;

/// Interaction state communicated from the Win32 message pump to the Notch controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEvent {
    MouseEnter,
    MouseLeave,
    Click { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
    Moved { x: i32, y: i32 },
    HotKey { id: i32 },
    DpiChanged { new_dpi: u32 },
    DisplayChanged,
}

/// Shared state inspected by the window procedure during hit-testing and event dispatch.
struct WindowSharedState {
    hit_x: AtomicI32,
    hit_y: AtomicI32,
    hit_width: AtomicI32,
    hit_height: AtomicI32,
    is_hovered: AtomicBool,
    event_tx: flume::Sender<WindowEvent>,
    current_dpi: AtomicU32,
    // Notch dragging and positioning
    drag_active: AtomicBool,
    drag_start_x: AtomicI32,
    drag_start_win_x: AtomicI32,
    l_down_x: AtomicI32,
    l_down_y: AtomicI32,
    l_is_down: AtomicBool,
    user_center_x: AtomicI32,
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
            is_hovered: AtomicBool::new(false),
            event_tx: tx,
            current_dpi: AtomicU32::new(96),
            drag_active: AtomicBool::new(false),
            drag_start_x: AtomicI32::new(0),
            drag_start_win_x: AtomicI32::new(0),
            l_down_x: AtomicI32::new(0),
            l_down_y: AtomicI32::new(0),
            l_is_down: AtomicBool::new(false),
            user_center_x: AtomicI32::new(0),
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

            // Register class; if already registered by earlier instance, ignore error
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

            // Store shared state pointer in GWLP_USERDATA
            let ptr = Arc::into_raw(Arc::clone(&shared)) as isize;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr);

            // Enable real physical DWM Acrylic backdrop blur behind window
            enable_acrylic_blur(hwnd);

            // Fetch initial DPI
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

    /// Update the interactive hit-test region in window client coordinates.
    pub fn update_hit_bounds(&self, x: i32, y: i32, width: u32, height: u32) {
        self.shared.hit_x.store(x, Ordering::Release);
        self.shared.hit_y.store(y, Ordering::Release);
        self.shared.hit_width.store(width as i32, Ordering::Release);
        self.shared
            .hit_height
            .store(height as i32, Ordering::Release);
    }

    /// Read user-set horizontal center of the notch on the monitor.
    pub fn user_center_x(&self) -> i32 {
        self.shared.user_center_x.load(Ordering::Acquire)
    }

    /// Set user horizontal center of the notch on the monitor.
    pub fn set_user_center_x(&self, cx: i32) {
        self.shared.user_center_x.store(cx, Ordering::Release);
    }

    /// Whether the user is actively dragging the notch across the screen.
    pub fn is_dragging(&self) -> bool {
        self.shared.drag_active.load(Ordering::Acquire)
    }

    /// Position and size the physical Notch window on screen.
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

    /// Update the physical notch window geometry and shape region matching the exact pill shape.
    /// This strictly confines DWM Acrylic blur to the pill boundary with zero square background bleed.
    pub fn set_pill_bounds(&self, x: i32, y: i32, width: u32, height: u32, radius: f32) {
        self.shared.current_x.store(x, Ordering::Release);
        self.shared.current_y.store(y, Ordering::Release);
        self.shared
            .current_width
            .store(width as i32, Ordering::Release);
        self.shared
            .current_height
            .store(height as i32, Ordering::Release);
        self.shared.hit_x.store(0, Ordering::Release);
        self.shared.hit_y.store(0, Ordering::Release);
        self.shared.hit_width.store(width as i32, Ordering::Release);
        self.shared
            .hit_height
            .store(height as i32, Ordering::Release);

        // SAFETY: SetWindowPos and SetWindowRgn with GDI CreateRoundRectRgn
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

            let hrgn = CreateRoundRectRgn(
                0,
                0,
                width as i32 + 1,
                height as i32 + 1,
                (radius * 2.0) as i32,
                (radius * 2.0) as i32,
            );
            let _ = SetWindowRgn(self.hwnd, hrgn, true);
        }
    }

    /// Non-blocking message pump for the notch window.
    /// Returns false if WM_QUIT was encountered.
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
            // Screen coordinates from GetMessagePos
            let pos = GetMessagePos();
            let screen_x = (pos & 0xFFFF) as i16 as i32;
            let screen_y = ((pos >> 16) & 0xFFFF) as i16 as i32;

            // Map window origin to screen
            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = ClientToScreen(hwnd, &mut pt);

            let local_x = screen_x - pt.x;
            let local_y = screen_y - pt.y;

            let h_x = shared.hit_x.load(Ordering::Acquire);
            let h_y = shared.hit_y.load(Ordering::Acquire);
            let h_w = shared.hit_width.load(Ordering::Acquire);
            let h_h = shared.hit_height.load(Ordering::Acquire);

            // Dynamic hit test: inside pill -> HTCLIENT; outside -> HTTRANSPARENT (click-through)
            if local_x >= h_x && local_x <= (h_x + h_w) && local_y >= h_y && local_y <= (h_y + h_h)
            {
                LRESULT(HTCLIENT as isize)
            } else {
                LRESULT(HTTRANSPARENT as isize)
            }
        }
        WM_MOUSEMOVE => {
            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);

            // Left-click drag threshold detection (>= 4 pixels movement activates drag)
            if shared.l_is_down.load(Ordering::Acquire)
                && !shared.drag_active.load(Ordering::Acquire)
            {
                let sx = shared.l_down_x.load(Ordering::Acquire);
                let sy = shared.l_down_y.load(Ordering::Acquire);
                if (pt.x - sx).abs() >= 4 || (pt.y - sy).abs() >= 4 {
                    shared.drag_active.store(true, Ordering::Release);
                    let _ = SetCapture(hwnd);
                }
            }

            // Real-time notch repositioning (Left or Right click drag)
            if shared.drag_active.load(Ordering::Acquire) {
                let delta_x = pt.x - shared.drag_start_x.load(Ordering::Acquire);
                let cur_w = shared.current_width.load(Ordering::Acquire);
                let new_win_x = shared.drag_start_win_x.load(Ordering::Acquire) + delta_x;
                let new_cx = new_win_x + cur_w / 2;

                let m = crate::monitor::get_primary_monitor_bounds();
                let clamped_cx = new_cx.clamp(m.left + 80, m.left + m.width() - 80);
                let final_win_x = clamped_cx - cur_w / 2;
                let cur_y = shared.current_y.load(Ordering::Acquire);

                shared.user_center_x.store(clamped_cx, Ordering::Release);
                shared.current_x.store(final_win_x, Ordering::Release);

                let _ = SetWindowPos(
                    hwnd,
                    HWND(std::ptr::null_mut()),
                    final_win_x,
                    cur_y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
                let _ = shared.event_tx.try_send(WindowEvent::Moved {
                    x: final_win_x,
                    y: cur_y,
                });
                return LRESULT(0);
            }

            if !shared.is_hovered.swap(true, Ordering::AcqRel) {
                let _ = shared.event_tx.try_send(WindowEvent::MouseEnter);

                // Request WM_MOUSELEAVE notification
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
            if !shared.drag_active.load(Ordering::Acquire) {
                shared.is_hovered.store(false, Ordering::Release);
                let _ = shared.event_tx.try_send(WindowEvent::MouseLeave);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);
            shared.l_down_x.store(pt.x, Ordering::Release);
            shared.l_down_y.store(pt.y, Ordering::Release);
            shared.drag_start_x.store(pt.x, Ordering::Release);
            let cur_x = shared.current_x.load(Ordering::Acquire);
            shared.drag_start_win_x.store(cur_x, Ordering::Release);
            shared.l_is_down.store(true, Ordering::Release);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let was_down = shared.l_is_down.swap(false, Ordering::AcqRel);
            if shared.drag_active.swap(false, Ordering::AcqRel) {
                let _ = ReleaseCapture();
            } else if was_down {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let _ = shared.event_tx.try_send(WindowEvent::Click { x, y });
            }
            LRESULT(0)
        }
        WM_LBUTTONDBLCLK => {
            // Double left-click (double-tap): snap notch back to exact monitor center
            let m = crate::monitor::get_primary_monitor_bounds();
            let center_x = m.left + m.width() / 2;
            let cur_w = shared.current_width.load(Ordering::Acquire);
            let final_win_x = center_x - cur_w / 2;
            let cur_y = shared.current_y.load(Ordering::Acquire);

            shared.user_center_x.store(center_x, Ordering::Release);
            shared.current_x.store(final_win_x, Ordering::Release);

            let _ = SetWindowPos(
                hwnd,
                HWND(std::ptr::null_mut()),
                final_win_x,
                cur_y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            let _ = shared.event_tx.try_send(WindowEvent::Moved {
                x: final_win_x,
                y: cur_y,
            });
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let mut pt = windows::Win32::Foundation::POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);
            shared.drag_start_x.store(pt.x, Ordering::Release);
            let cur_x = shared.current_x.load(Ordering::Acquire);
            shared.drag_start_win_x.store(cur_x, Ordering::Release);
            shared.drag_active.store(true, Ordering::Release);
            let _ = SetCapture(hwnd);
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            if shared.drag_active.swap(false, Ordering::AcqRel) {
                let _ = ReleaseCapture();
            }
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = shared.event_tx.try_send(WindowEvent::RightClick { x, y });
            LRESULT(0)
        }
        WM_RBUTTONDBLCLK => {
            // Double right-click (double-tap): snap notch back to exact monitor center
            let m = crate::monitor::get_primary_monitor_bounds();
            let center_x = m.left + m.width() / 2;
            let cur_w = shared.current_width.load(Ordering::Acquire);
            let final_win_x = center_x - cur_w / 2;
            let cur_y = shared.current_y.load(Ordering::Acquire);

            shared.user_center_x.store(center_x, Ordering::Release);
            shared.current_x.store(final_win_x, Ordering::Release);

            let _ = SetWindowPos(
                hwnd,
                HWND(std::ptr::null_mut()),
                final_win_x,
                cur_y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            let _ = shared.event_tx.try_send(WindowEvent::Moved {
                x: final_win_x,
                y: cur_y,
            });
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

            // Suggested window rect from lparam
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
        WM_ERASEBKGND => {
            // Tell Windows background is handled to prevent opaque background fills
            LRESULT(1)
        }
        WM_PAINT => {
            // Validate the client area so PeekMessage does not loop continuously on WM_PAINT
            unsafe {
                let _ = windows::Win32::Graphics::Gdi::ValidateRect(hwnd, None);
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AccentPolicy {
    accent_state: u32,
    accent_flags: u32,
    gradient_color: u32,
    animation_id: u32,
}

#[repr(C)]
struct WindowCompositionAttribData {
    attrib: u32,
    pv_data: *mut std::ffi::c_void,
    cb_data: usize,
}

type SetWindowCompositionAttributeFn =
    unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

// SAFETY: Calls DWM and user32 APIs to activate hardware blur on underlying desktop wallpaper.
unsafe fn enable_acrylic_blur(hwnd: HWND) {
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    // 1. Windows 10/11 SetWindowCompositionAttribute (ACCENT_ENABLE_ACRYLICBLURBEHIND = 4)
    if let Ok(user32) = LoadLibraryW(w!("user32.dll")) {
        let proc = GetProcAddress(user32, windows::core::s!("SetWindowCompositionAttribute"));
        if let Some(func) = proc {
            let set_window_comp: SetWindowCompositionAttributeFn = std::mem::transmute(func);

            let mut accent = AccentPolicy {
                accent_state: 4, // ACCENT_ENABLE_ACRYLICBLURBEHIND
                accent_flags: 2,
                gradient_color: 0x01101826, // Subtle translucent dark indigo tint
                animation_id: 0,
            };

            let mut data = WindowCompositionAttribData {
                attrib: 19, // WCA_ACCENT_POLICY
                pv_data: &mut accent as *mut _ as *mut std::ffi::c_void,
                cb_data: std::mem::size_of::<AccentPolicy>(),
            };

            let _ = set_window_comp(hwnd, &mut data);
        }
    }

    // 2. Windows 11 22H2+ System Backdrop Type (DWMSBT_TRANSIENTWINDOW = 3 Acrylic)
    let backdrop_type: u32 = 3;
    let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &backdrop_type as *const _ as *const _,
        std::mem::size_of::<u32>() as u32,
    );

    // 3. Extend Frame into client area so glass encompasses entire surface
    let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
    };
    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

    // 4. Windows 11 rounded window corners (DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_ROUND = 2)
    let corner_preference: u32 = 2; // DWMWCP_ROUND
    let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWINDOWATTRIBUTE(33),
        &corner_preference as *const _ as *const _,
        std::mem::size_of::<u32>() as u32,
    );
}
