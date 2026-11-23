// nuvra-platform-win — the ONLY crate permitted to use unsafe code.
// Every unsafe block carries a // SAFETY: comment stating the invariant.
// D-STK-02: Windows API via windows-rs.

pub mod audio;
pub mod camera;
pub mod dcomp;
pub mod dpapi;
pub mod dxgi;
pub mod hotkey;
pub mod monitor;
pub mod tray;
pub mod window;
