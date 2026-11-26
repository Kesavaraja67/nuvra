// nuvra-render — DirectComposition + D3D11 + D2D/DWrite renderer
// ADR-0002: D-STK-03 — no wgpu, no Electron, no WebView for the notch window.
// R-13: No unsafe in this crate (platform calls go through nuvra-platform-win).
#![forbid(unsafe_code)]

pub mod bake;
pub mod choreo;
pub mod engine;
pub mod hover;
pub mod icons;
pub mod layout;
pub mod move_mode;
pub mod spike;
pub mod spring;
pub mod tokens;
