// nuvra-platform-win — the ONLY crate permitted to use unsafe code.
// Every unsafe block carries a // SAFETY: comment stating the invariant.
// D-STK-02: Windows API via windows-rs.

pub mod audio;
pub mod audio_loopback;
pub mod camera;
pub mod clipboard;
pub mod dcomp;
pub mod dpapi;
pub mod draw;
pub mod dxgi;
pub mod foreground;
pub mod hotkey;
pub mod monitor;
pub mod network;
pub mod notifications;
pub mod power;
pub mod shell_ops;
pub mod theme_sync;
pub mod tray;
pub mod window;
