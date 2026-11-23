// nuvra-render/src/tokens.rs
// Design system tokens — Section 6 of the master spec.
// R-27: All colors, sizes, radii, durations come from here. No magic numbers elsewhere.
// R-13: No unsafe.
#![forbid(unsafe_code)]

// ── Color tokens (Obsidian / dark default) ─────────────────────────────────
pub mod obsidian {
    pub const SURFACE: u32 = 0xFF060709;
    pub const SURFACE_RAISED: u32 = 0xFF0E1014;
    pub const SURFACE_HIGH: u32 = 0xFF161920;
    pub const EDGE: u32 = 0xFF232732;
    /// edge-lit: white @ 6% — stored as ARGB
    pub const EDGE_LIT: u32 = 0x0FFFFFFF;
    pub const TEXT_PRIMARY: u32 = 0xFFF3F4F6;
    pub const TEXT_SECONDARY: u32 = 0xFF9AA0AC;
    pub const TEXT_TERTIARY: u32 = 0xFF5E6470;
    pub const ACCENT: u32 = 0xFFFF6B3D;
    pub const DANGER: u32 = 0xFFFF4258;
    pub const OK: u32 = 0xFF46D39A;
    pub const WARNING: u32 = 0xFFFFC247;
}

pub mod paper {
    pub const SURFACE: u32 = 0xFFF6F5F2;
    pub const SURFACE_RAISED: u32 = 0xFFFFFFFF;
    pub const SURFACE_HIGH: u32 = 0xFFECEAE4;
    pub const EDGE: u32 = 0xFFDAD8D2;
    /// edge-lit: white @ 80%
    pub const EDGE_LIT: u32 = 0xCCFFFFFF;
    pub const TEXT_PRIMARY: u32 = 0xFF14151A;
    pub const TEXT_SECONDARY: u32 = 0xFF5A5E68;
    pub const TEXT_TERTIARY: u32 = 0xFF8A8D96;
    pub const ACCENT: u32 = 0xFFE8501F;
    pub const DANGER: u32 = 0xFFD42A40;
    pub const OK: u32 = 0xFF1E9E6A;
    pub const WARNING: u32 = 0xFFB87A00;
}

pub mod signal {
    pub const SURFACE: u32 = 0xFF000000;
    pub const SURFACE_RAISED: u32 = 0xFF0A0A0A;
    pub const SURFACE_HIGH: u32 = 0xFF141414;
    pub const EDGE: u32 = 0xFFFFFFFF;
    /// edge-lit: white @ 100%
    pub const EDGE_LIT: u32 = 0xFFFFFFFF;
    pub const TEXT_PRIMARY: u32 = 0xFFFFFFFF;
    pub const TEXT_SECONDARY: u32 = 0xFFE0E0E0;
    pub const TEXT_TERTIARY: u32 = 0xFFBDBDBD;
    pub const ACCENT: u32 = 0xFFFFD400;
    pub const DANGER: u32 = 0xFFFF6B6B;
    pub const OK: u32 = 0xFF7CFF9E;
    pub const WARNING: u32 = 0xFFFFD400;
}

// ── Spacing grid (px, logical) ─────────────────────────────────────────────
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

// ── Typography scale (px) ──────────────────────────────────────────────────
pub mod type_scale {
    pub const CAPTION: f32 = 11.0;
    pub const SMALL: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 15.0;
    pub const DISPLAY: f32 = 20.0;
}

// ── Radii (px, logical — applied to BOTTOM corners only; top = 0) ──────────
pub mod radius {
    pub const CONTROL: f32 = 8.0;
    pub const AMBIENT: f32 = 14.0;
    pub const GLANCE: f32 = 16.0;
    pub const EXPANDED: f32 = 18.0;
}

// ── Window sizes (logical px, DPI-scaled at runtime) ──────────────────────
pub mod window {
    pub const IDLE_WIDTH: f32 = 160.0;
    pub const IDLE_HEIGHT: f32 = 2.0;

    pub const AMBIENT_WIDTH: f32 = 290.0;
    pub const AMBIENT_HEIGHT: f32 = 34.0;

    pub const GLANCE_WIDTH: f32 = 380.0;
    pub const GLANCE_HEIGHT: f32 = 64.0;

    pub const EXPANDED_WIDTH: f32 = 720.0;
    pub const EXPANDED_DEFAULT_HEIGHT: f32 = 54.0;
    pub const EXPANDED_DRAWER_HEIGHT: f32 = 180.0;
    pub const EXPANDED_MAX_HEIGHT: f32 = 360.0;
}

// ── Timing tokens (ms) ────────────────────────────────────────────────────
pub mod timing {
    /// Hover intent delay default (ms). Configurable 0–400.
    pub const HOVER_INTENT_DEFAULT_MS: u64 = 90;
    /// Collapse delay after pointer leaves (ms).
    pub const COLLAPSE_DELAY_MS: u64 = 350;
    /// Notification banner display duration (ms).
    pub const BANNER_DURATION_MS: u64 = 4000;
    /// Transient confirmation duration (ms) — mic mute, screenshot.
    pub const CONFIRMATION_DURATION_MS: u64 = 1500;
    /// Content crossfade duration (ms).
    pub const CROSSFADE_MS: f32 = 120.0;
    /// Reduced-motion fade duration (ms).
    pub const REDUCED_MOTION_FADE_MS: f32 = 80.0;
}

// ── Spring parameters (DirectComposition animation) ───────────────────────
pub mod spring {
    pub struct SpringSpec {
        pub stiffness: f32,
        pub damping: f32,
        pub mass: f32,
    }

    pub const EXPAND: SpringSpec = SpringSpec {
        stiffness: 380.0,
        damping: 32.0,
        mass: 1.0,
    };
    pub const COLLAPSE: SpringSpec = SpringSpec {
        stiffness: 460.0,
        damping: 40.0,
        mass: 1.0,
    };
}
