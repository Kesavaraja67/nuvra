// nuvra-render/src/tokens.rs
// Design system tokens — Document UI-1 Sections 5 & 6.
// R-27: All colors, sizes, radii, durations come from here. No magic numbers elsewhere.
// R-13: No unsafe.
#![forbid(unsafe_code)]

// ── Color tokens (Smoked Quartz Frosted Glass default) ─────────────────────
pub mod obsidian {
    /// Neutral smoked-quartz translucent glass base (~74% opacity)
    pub const SURFACE: u32 = 0xBD0A0B0E;
    pub const SURFACE_RAISED: u32 = 0x9E121419;
    pub const SURFACE_HIGH: u32 = 0x16FFFFFF;
    pub const EDGE: u32 = 0x2EFFFFFF;
    /// Rest button plate: translucent white glass @ 7%
    pub const PLATE_REST: u32 = 0x12FFFFFF;
    /// Hover button plate: frosted white glass @ 16%
    pub const PLATE_HOVER: u32 = 0x29FFFFFF;
    /// Active/pressed plate: crisp frosted platinum @ 24%
    pub const PLATE_ACTIVE: u32 = 0x3DFFFFFF;
    /// Divider line: hairline white @ 14%
    pub const DIVIDER: u32 = 0x24FFFFFF;
    pub const TEXT_PRIMARY: u32 = 0xFFF4F5F7;
    pub const TEXT_SECONDARY: u32 = 0xFFA2A8B4;
    pub const TEXT_TERTIARY: u32 = 0xFF666C78;
    /// Restrained platinum-silver focus accent (replaces generic purple AI glow)
    pub const ACCENT: u32 = 0xFFE6EAF2;
    pub const ACCENT_HI: u32 = 0xFFFFFFFF;
    /// Crisp signal coral for alerts & muted mic
    pub const DANGER: u32 = 0xFFFF5247;
    /// Crisp signal emerald for live active telemetry
    pub const OK: u32 = 0xFF3DD68C;
    pub const WARNING: u32 = 0xFFFFC247;
    pub const LOGO_A: u32 = 0xFFE6EAF2;
    pub const LOGO_B: u32 = 0xFF3DD68C;
}

pub mod paper {
    /// Light frosted glass surface (~82% white glass)
    pub const SURFACE: u32 = 0xD1F4F5F9;
    pub const SURFACE_RAISED: u32 = 0xB8FFFFFF;
    pub const SURFACE_HIGH: u32 = 0x66FFFFFF;
    pub const EDGE: u32 = 0x80FFFFFF;
    pub const PLATE_REST: u32 = 0x12000000;
    pub const PLATE_HOVER: u32 = 0x22000000;
    pub const PLATE_ACTIVE: u32 = 0x2E5B2EE0;
    pub const DIVIDER: u32 = 0x38000000;
    pub const TEXT_PRIMARY: u32 = 0xFF14151A;
    pub const TEXT_SECONDARY: u32 = 0xFF4A4E59;
    pub const TEXT_TERTIARY: u32 = 0xFF757985;
    pub const ACCENT: u32 = 0xFF5B2EE0;
    pub const ACCENT_HI: u32 = 0xFF754DF0;
    pub const DANGER: u32 = 0xFFD42A40;
    pub const OK: u32 = 0xFF1E9E6A;
    pub const WARNING: u32 = 0xFFB87A00;
    pub const LOGO_A: u32 = 0xFF5B2EE0;
    pub const LOGO_B: u32 = 0xFF754DF0;
}

pub mod signal {
    pub const SURFACE: u32 = 0xFF000000;
    pub const SURFACE_RAISED: u32 = 0xFF0A0A0A;
    pub const SURFACE_HIGH: u32 = 0xFF141414;
    pub const EDGE: u32 = 0xFFFFFFFF;
    pub const PLATE_REST: u32 = 0x00000000;
    pub const PLATE_HOVER: u32 = 0x00000000;
    pub const PLATE_ACTIVE: u32 = 0x00000000;
    pub const DIVIDER: u32 = 0xFFFFFFFF;
    pub const TEXT_PRIMARY: u32 = 0xFFFFFFFF;
    pub const TEXT_SECONDARY: u32 = 0xFFE0E0E0;
    pub const TEXT_TERTIARY: u32 = 0xFFBDBDBD;
    pub const ACCENT: u32 = 0xFFFFD400;
    pub const ACCENT_HI: u32 = 0xFFFFD400;
    pub const DANGER: u32 = 0xFFFF6B6B;
    pub const OK: u32 = 0xFF7CFF9E;
    pub const WARNING: u32 = 0xFFFFD400;
    pub const LOGO_A: u32 = 0xFFFFD400;
    pub const LOGO_B: u32 = 0xFFFFD400;
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
    pub const BADGE: f32 = 11.0;
    pub const CAPTION: f32 = 11.0;
    pub const SMALL: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const TITLE: f32 = 15.0;
    pub const DISPLAY: f32 = 20.0;
}

// ── Radii (px, logical — applied to BOTTOM corners only; top = 0) ──────────
pub mod radius {
    pub const CONTROL: f32 = 8.0;
    pub const BOTTOM: f32 = 20.0;
    pub const FILLET: f32 = 10.0;
}

// ── Window and Notch sizes (logical px, DPI-scaled at runtime) ────────────
pub mod window {
    pub const HOTZONE_WIDTH: f32 = 160.0;
    pub const HOTZONE_HEIGHT: f32 = 2.0;

    pub const COLLAPSED_WIDTH: f32 = 380.0;
    pub const COLLAPSED_HEIGHT: f32 = 38.0;

    pub const EXPANDED_WIDTH: f32 = 864.0;
    pub const EXPANDED_NO_MEDIA_WIDTH: f32 = 600.0;
    pub const EXPANDED_HEIGHT: f32 = 216.0;
    pub const EXPANDED_TOOLBAR_HEIGHT: f32 = 48.0;

    pub const BANNER_WIDTH: f32 = 480.0;
    pub const BANNER_HEIGHT: f32 = 56.0;

    pub const DRAWER_MIN_HEIGHT: f32 = 120.0;
    pub const DRAWER_MAX_HEIGHT: f32 = 300.0;
    pub const DRAWER_MAX_TOTAL_HEIGHT: f32 = 356.0;

    pub const FIXED_WINDOW_WIDTH: f32 = 900.0;
    pub const FIXED_WINDOW_HEIGHT: f32 = 368.0;
}

// ── Timing tokens (ms) — Section 6.10 ─────────────────────────────────────
pub mod timing {
    pub const HOVER_INTENT_DEFAULT_MS: u64 = 45;
    pub const COLLAPSE_DELAY_MS: u64 = 300;
    pub const HOVER_WAKE_MS: f32 = 80.0;
    pub const COLLAPSED_FADE_OUT_MS: f32 = 70.0;
    pub const TOOLBAR_FADE_IN_MS: f32 = 140.0;
    pub const TOOLBAR_STAGGER_CENTER_MS: f32 = 60.0;
    pub const TOOLBAR_STAGGER_OUTER_MS: f32 = 100.0;
    pub const TOOLBAR_RISE_PX: f32 = 4.0;
    pub const TOOLBAR_FADE_OUT_MS: f32 = 80.0;
    pub const COLLAPSED_FADE_IN_DELAY_MS: f32 = 120.0;
    pub const COLLAPSED_FADE_IN_MS: f32 = 120.0;
    pub const CROSSFADE_MS: f32 = 120.0;
    pub const TOOLTIP_DELAY_MS: u64 = 500;
    pub const TOOLTIP_FADE_MS: f32 = 100.0;
    pub const PRESS_MS: f32 = 60.0;
    pub const TOUCH_IDLE_COLLAPSE_MS: u64 = 5000;
    pub const BANNER_DURATION_MS: u64 = 4000;
    pub const CONFIRMATION_DURATION_MS: u64 = 1500;
    pub const MOVE_RIM_IN_MS: f32 = 100.0;
    pub const MOVE_RIM_OUT_MS: f32 = 140.0;
    pub const MOVE_IDLE_DROP_MS: u64 = 6000;
    pub const SNAP_FLASH_MS: f32 = 80.0;
    pub const SNAP_RADIUS_PX: f32 = 12.0;
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
        stiffness: 260.0,
        damping: 26.0,
        mass: 1.0,
    };
    pub const COLLAPSE: SpringSpec = SpringSpec {
        stiffness: 300.0,
        damping: 30.0,
        mass: 1.0,
    };
}
