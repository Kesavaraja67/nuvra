// nuvra-core-shared/src/i18n.rs
// Fluent-based i18n infrastructure — D-PRD-06, D-GLB-01, D-GLB-02.
// v1: English only. Infrastructure ready for all 18 languages in v3 (D-GLB-03).
// R-17: All user-facing strings go through Fluent keys — no hardcoded strings.
// R-13: No unsafe.
#![forbid(unsafe_code)]

pub const DEFAULT_LOCALE: &str = "en-US";

/// Fluent message keys — R-17 compliance.
/// Every user-facing string must have a key here.
pub mod keys {
    // General
    pub const APP_NAME: &str = "app-name";

    // Mic module
    pub const MIC_MUTED: &str = "mic-muted";
    pub const MIC_LIVE: &str = "mic-live";
    pub const CALL_ACTIVE: &str = "call-active";

    // Media module
    pub const MEDIA_NO_SOURCE: &str = "media-no-source";
    pub const MEDIA_PAUSED: &str = "media-paused";

    // Notifications
    pub const NOTIFICATIONS_PERMISSION_NEEDED: &str = "notifications-permission-needed";
    pub const NOTIFICATIONS_DENIED_HELP: &str = "notifications-denied-help";

    // Clipboard
    pub const CLIPBOARD_EMPTY: &str = "clipboard-empty";
    pub const CLIPBOARD_WIPED: &str = "clipboard-wiped";
    pub const CLIPBOARD_ITEM_MISSING: &str = "clipboard-item-missing";

    // Shelf
    pub const SHELF_EMPTY: &str = "shelf-empty";
    pub const SHELF_FILE_MISSING: &str = "shelf-file-missing";
    pub const SHELF_DROP_LIMIT: &str = "shelf-drop-limit";

    // Errors — per copy rules: say what happened and what user can do.
    pub const ERROR_WEBVIEW2_MISSING: &str = "error-webview2-missing";
    pub const ERROR_MIC_ACCESS_DENIED: &str = "error-mic-access-denied";
    pub const ERROR_MODULE_RESTARTED: &str = "error-module-restarted";
}
