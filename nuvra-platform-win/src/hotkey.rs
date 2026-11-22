// nuvra-platform-win/src/hotkey.rs
// RegisterHotKey / UnregisterHotKey wrapper (D-ARCH-04, D-KBD-01).
// ONLY file in the project permitted unsafe. Every block carries a SAFETY comment.
//
// UNVERIFIED — requires owner to run on a real Windows machine.
// Manual test: docs/owner-test-scripts/phase-0-test-script.md § HotKey

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, VIRTUAL_KEY,
};

/// A registered global hot key.
/// Dropping it unregisters the hot key automatically.
pub struct HotKeyRegistration {
    hwnd: HWND,
    id: i32,
}

impl HotKeyRegistration {
    /// Register a global hot key with Windows.
    ///
    /// `hwnd`: the window that will receive WM_HOTKEY messages.
    /// `id`:   application-unique integer identifier (1–0xBFFF per MSDN).
    /// `modifiers`: MOD_ALT | MOD_CONTROL etc.
    /// `vk`:   virtual-key code.
    pub fn register(
        hwnd: HWND,
        id: i32,
        modifiers: HOT_KEY_MODIFIERS,
        vk: VIRTUAL_KEY,
    ) -> Result<Self, HotKeyError> {
        if !(1..=0xBFFF).contains(&id) {
            return Err(HotKeyError::InvalidId(id));
        }

        // SAFETY: RegisterHotKey is a pure Win32 call. `hwnd` is a window
        // handle owned by the caller that outlives this registration. `id`
        // is in range (checked above). The kernel validates modifiers and vk.
        let res = unsafe { RegisterHotKey(hwnd, id, modifiers, u32::from(vk.0)) };

        if res.is_ok() {
            Ok(Self { hwnd, id })
        } else {
            Err(HotKeyError::Win32(unsafe {
                windows::Win32::Foundation::GetLastError().0
            }))
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

impl Drop for HotKeyRegistration {
    fn drop(&mut self) {
        // SAFETY: We call UnregisterHotKey only once (in drop).
        // `hwnd` is the same window that registered it. `id` is valid.
        // Failure here is non-fatal (window may already be destroyed).
        let _ = unsafe { UnregisterHotKey(self.hwnd, self.id) };
    }
}

#[derive(Debug)]
pub enum HotKeyError {
    InvalidId(i32),
    /// Win32 error code (e.g. ERROR_HOTKEY_ALREADY_REGISTERED = 1409).
    Win32(u32),
}

impl std::fmt::Display for HotKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotKeyError::InvalidId(id) => write!(f, "hot key id {id} out of range 1..0xBFFF"),
            HotKeyError::Win32(code) => write!(f, "RegisterHotKey Win32 error: 0x{:08X}", code),
        }
    }
}

impl std::error::Error for HotKeyError {}
