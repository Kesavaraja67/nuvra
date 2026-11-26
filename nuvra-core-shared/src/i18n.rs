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

use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

const EN_US_FTL: &str = include_str!("../../locales/en-US/main.ftl");

/// In-memory Fluent catalog initialized from locales/en-US/main.ftl (R-17).
pub struct FluentCatalog {
    bundle: FluentBundle<FluentResource>,
}

impl FluentCatalog {
    pub fn en_us() -> Self {
        let lang: LanguageIdentifier = DEFAULT_LOCALE.parse().unwrap_or_default();
        let mut bundle = FluentBundle::new(vec![lang]);
        if let Ok(res) = FluentResource::try_new(EN_US_FTL.to_string()) {
            let _ = bundle.add_resource(res);
        }
        Self { bundle }
    }

    /// Resolve a Fluent message key without arguments, returning the key on miss.
    pub fn get(&self, key: &str) -> String {
        let Some(msg) = self.bundle.get_message(key) else { return key.to_string() };
        let Some(pattern) = msg.value() else { return key.to_string() };
        let mut errors = Vec::new();
        self.bundle.format_pattern(pattern, None, &mut errors).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluent_catalog_resolves_v1_keys() {
        let cat = FluentCatalog::en_us();
        assert_eq!(cat.get(keys::APP_NAME), "Nuvra");
        assert_eq!(cat.get(keys::MIC_MUTED), "Mic muted");
        assert_eq!(cat.get(keys::MEDIA_NO_SOURCE), "Nothing playing");
    }
}
