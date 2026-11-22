// nuvra-mod-notifications/src/state.rs
// Internal mutable state for the notifications module.
// R-13: No unsafe.
// R-20: Notification content is never logged or persisted beyond the display window.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::NotificationSnapshot;

#[derive(Default)]
pub struct NotificationsState {
    /// Current banner being shown (if any).
    pub app_name: String,
    /// Pre-rendered, truncated summary — not raw notification body (R-20).
    pub summary: String,
    /// How many are queued behind the current banner.
    pub queued: u8,
}

impl NotificationsState {
    pub fn snapshot(&self) -> NotificationSnapshot {
        NotificationSnapshot {
            app_name: self.app_name.clone(),
            summary: self.summary.clone(),
            queued: self.queued,
        }
    }
}
