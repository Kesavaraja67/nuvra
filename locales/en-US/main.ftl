# Fluent Catalog — en-US
# nuvra/locales/en-US/main.ftl
#
# R-17: Every user-facing string must have a key here.
# R-20: Strings must never contain clipboard text, notification content, file paths,
#       phone data, tokens, or keys.
# D-GLB-01/02: English catalog. All 18 locale catalogs use the same key set (v3 scope).
# Copy rules (Section 6.6): short, specific, human. No "Oops". No "!". No emoji.

# ── App ─────────────────────────────────────────────────────────────────────────
app-name = Nuvra

# ── Mic module ──────────────────────────────────────────────────────────────────
mic-muted = Mic muted
mic-live = Mic live
call-active = Call active

# ── Media module ────────────────────────────────────────────────────────────────
media-no-source = Nothing playing
media-paused = Paused

# ── Notifications ───────────────────────────────────────────────────────────────
notifications-permission-needed = Notification access is off. Turn it on in Windows Settings.
notifications-denied-help = Go to Settings › System › Notifications to allow Nuvra access.

# ── Clipboard module ─────────────────────────────────────────────────────────────
clipboard-empty = Clipboard is empty
clipboard-wiped = Clipboard cleared
clipboard-item-missing = That item is no longer available

# ── Shelf module ─────────────────────────────────────────────────────────────────
shelf-empty = Drop files here to pin them
shelf-file-missing = File not found — it may have been moved or deleted
shelf-drop-limit = Shelf is full. Remove an item first.

# ── Errors ───────────────────────────────────────────────────────────────────────
error-webview2-missing =
    WebView2 runtime is not installed.
    Download it from microsoft.com/edge/webview2 and restart Nuvra.
error-mic-access-denied =
    Nuvra needs microphone access to show call activity.
    Allow it in Windows Settings › Privacy › Microphone.
error-module-restarted = { $module } stopped unexpectedly and was restarted.

# ── Settings sections ─────────────────────────────────────────────────────────────
settings-title = Nuvra Settings
settings-section-appearance = Appearance
settings-section-modules = Modules
settings-section-clipboard = Clipboard
settings-section-shelf = Shelf
settings-section-hotkeys = Keyboard shortcuts
settings-section-privacy = Privacy
settings-section-about = About

# ── Appearance ────────────────────────────────────────────────────────────────────
appearance-theme = Theme
appearance-theme-obsidian = Obsidian
appearance-theme-paper = Paper
appearance-theme-signal = Signal (High Contrast)
appearance-hover-delay = Hover intent delay
appearance-hover-delay-hint = How long you must hover before the notch opens (0 – 400 ms)

# ── Modules ──────────────────────────────────────────────────────────────────────
module-media = Media
module-mic = Microphone
module-notifications = Notifications
module-clipboard = Clipboard
module-shelf = Shelf
module-enabled = Enabled
module-disabled = Disabled

# ── Clipboard settings ────────────────────────────────────────────────────────────
clipboard-history-limit = History limit
clipboard-excluded-apps = Excluded apps
clipboard-excluded-apps-hint =
    Clipboard entries from these apps are never recorded.
    Password managers are excluded by default.
clipboard-wipe-on-lock = Clear history when screen locks

# ── Hotkeys ───────────────────────────────────────────────────────────────────────
hotkey-open-shelf = Open shelf
hotkey-open-clipboard = Open clipboard history
hotkey-not-set = Not set

# ── About ─────────────────────────────────────────────────────────────────────────
about-version = Version { $version }
about-license = Proprietary license
about-privacy-policy = Privacy policy
about-third-party = Third-party licenses
about-support = Get support

# ── Confirm dialogs ───────────────────────────────────────────────────────────────
confirm-clear-clipboard = Clear all clipboard history?
confirm-clear-clipboard-detail = Pinned items will also be removed. This cannot be undone.
confirm-yes = Clear
confirm-cancel = Cancel

# ── Generic ───────────────────────────────────────────────────────────────────────
action-save = Save
action-close = Close
action-remove = Remove
action-add = Add
state-on = On
state-off = Off
