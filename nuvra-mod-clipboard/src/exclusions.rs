// nuvra-mod-clipboard/src/exclusions.rs
// Clipboard exclusion list — D-DAT-03.
// Prevents sensitive app windows from being captured (password managers, etc).
// R-13: No unsafe.
// R-20: We store only window class names, never clipboard content.
#![forbid(unsafe_code)]

/// Default exclusion list — window class names whose clipboard events are silently ignored.
/// D-DAT-03: User can extend this list via Settings.
pub const DEFAULT_EXCLUDED_CLASSES: &[&str] = &[
    "KeePass",     // KeePass 2.x
    "KeePassXC",   // KeePassXC
    "1Password",   // 1Password
    "Bitwarden",   // Bitwarden
    "LastPass",    // LastPass
    "pwsafe",      // Password Safe
    "Enpass",      // Enpass
    "RoboForm",    // RoboForm
    "NordPass",    // NordPass
    "DashlaneApp", // Dashlane
];

/// Returns true if the foreground window's class name matches the exclusion list.
/// Used in v1's ClipboardChanged handler before reading clipboard content.
pub fn is_excluded(window_class: &str) -> bool {
    DEFAULT_EXCLUDED_CLASSES
        .iter()
        .any(|&excluded| window_class.eq_ignore_ascii_case(excluded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_known_password_managers() {
        assert!(is_excluded("KeePass"));
        assert!(is_excluded("keepass")); // case-insensitive
        assert!(is_excluded("Bitwarden"));
    }

    #[test]
    fn allows_normal_apps() {
        assert!(!is_excluded("Notepad"));
        assert!(!is_excluded("chrome"));
        assert!(!is_excluded(""));
    }
}
