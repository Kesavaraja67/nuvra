// nuvra-platform-win/src/window.rs
// Topmost transparent Win32 notch window lifecycle with dynamic hit-testing (D-WIN-01..09).
// ONLY file in nuvra-platform-win using unsafe for Win32 window APIs.
// Every unsafe block carries a // SAFETY: comment.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessagePos, PeekMessageW,
    PostQuitMessage, RegisterClassExW, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWLP_USERDATA,
    HTCLIENT, HTTRANSPARENT, MSG, PM_REMOVE, SWP_NOACTIVATE, SWP_NOZORDER, SW_SHOWNA, WM_DESTROY,
    WM_DISPLAYCHANGE, WM_DPICHANGED, WM_ERASEBKGND, WM_HOTKEY, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_NCHITTEST, WM_PAINT, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

pub const WM_MOUSELEAVE: u32 = 0x02A3;

/// Interaction state communicated from the Win32 message pump to the Notch controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEvent {
    MouseEnter,
    MouseLeave,
    Click { x: i32, y: i32 },
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
        });

        // SAFETY: RegisterClassExW and CreateWindowExW with verified class names and styles.
        let hwnd = unsafe {
            let hinstance = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW: {e}"))?;
            let class_name = w!("NuvraNotchWindowClass");

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
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

    /// Position and size the physical Notch window on screen.
    pub fn set_geometry(&self, x: i32, y: i32, width: u32, height: u32) {
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
            shared.is_hovered.store(false, Ordering::Release);
            let _ = shared.event_tx.try_send(WindowEvent::MouseLeave);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => LRESULT(0),
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let _ = shared.event_tx.try_send(WindowEvent::Click { x, y });
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
