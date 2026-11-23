// nuvra-core-shared/src/config.rs
// Typed, schema-versioned configuration — D-STK-05.
// File: %APPDATA%\Nuvra\config.toml
// R-13: No unsafe.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Current config schema version. Increment on breaking changes; migrate on load.
pub const CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NuvraConfig {
    pub schema_version: u32,
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    pub hotkeys: HotkeyConfig,
    pub modules: ModulesConfig,
    pub privacy: PrivacyConfig,
}

impl Default for NuvraConfig {
    fn default() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            appearance: AppearanceConfig::default(),
            behavior: BehaviorConfig::default(),
            hotkeys: HotkeyConfig::default(),
            modules: ModulesConfig::default(),
            privacy: PrivacyConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// "obsidian" | "paper" | "signal" | "graphite-sand" | "ink-blue"
    pub theme: String,
    /// Follow Windows color mode if true.
    pub follow_system: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "obsidian".into(),
            follow_system: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Hover intent delay in ms (0–400). Default 90.
    pub hover_intent_ms: u64,
    /// Launch at startup.
    pub launch_at_startup: bool,
    /// Monitor preference: "primary" | "all" | "active-window"
    pub monitor: String,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            hover_intent_ms: 90,
            launch_at_startup: true,
            monitor: "primary".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Virtual key string for mic mute. Default "Ctrl+Alt+M".
    pub mic_mute: String,
    pub toggle_expanded: String,
    pub clipboard_panel: String,
    pub launcher: String,
    pub color_picker: String,
    pub screenshot: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            mic_mute: "Ctrl+Alt+M".into(),
            toggle_expanded: "Ctrl+Alt+N".into(),
            clipboard_panel: "Ctrl+Alt+V".into(),
            launcher: "Ctrl+Alt+Space".into(),
            color_picker: "Ctrl+Alt+C".into(),
            screenshot: "Ctrl+Alt+S".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    pub media: bool,
    pub mic: bool,
    pub notifications: bool,
    pub shelf: bool,
    pub clipboard: bool,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            media: true,
            mic: true,
            notifications: true,
            shelf: true,
            clipboard: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Opt-in to lyrics lookup (D-NET-01-a).
    pub lyrics_enabled: bool,
    /// Process denylist for clipboard exclusion (D-DAT-03).
    pub clipboard_denylist: Vec<String>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            lyrics_enabled: false,
            clipboard_denylist: vec![
                "1Password".into(),
                "Bitwarden".into(),
                "KeePass".into(),
                "KeePassXC".into(),
                "Dashlane".into(),
                "LastPass".into(),
            ],
        }
    }
}
