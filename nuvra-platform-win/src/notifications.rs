// nuvra-platform-win/src/notifications.rs
// Windows UserNotificationListener bridge (Section 14.4).
// Requires MSIX package identity for Allowed status; gracefully reports status when unpackaged.
// R-20: Notification text is never logged.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifPermission {
    Unknown,
    Allowed,
    NeedsPackagedIdentityOrPermission,
}

pub struct OsNotificationReader {
    perm: Arc<Mutex<NotifPermission>>,
    pending: Arc<Mutex<Vec<(String, String)>>>,
    stop: Arc<AtomicBool>,
}

fn resolve_permission_on_ui_thread() -> NotifPermission {
    let Ok(listener) = UserNotificationListener::Current() else {
        return NotifPermission::NeedsPackagedIdentityOrPermission;
    };
    let initial = listener.GetAccessStatus();
    if should_request_access(&initial) {
        let requested = listener
            .RequestAccessAsync()
            .ok()
            .and_then(|op| op.get().ok());
        classify_requested_access(requested)
    } else {
        classify_initial_access(initial.ok())
    }
}

fn should_request_access(
    status: &windows::core::Result<UserNotificationListenerAccessStatus>,
) -> bool {
    matches!(
        status,
        Ok(UserNotificationListenerAccessStatus::Unspecified)
    )
}

fn classify_initial_access(
    status: Option<UserNotificationListenerAccessStatus>,
) -> NotifPermission {
    if status == Some(UserNotificationListenerAccessStatus::Allowed) {
        NotifPermission::Allowed
    } else {
        NotifPermission::NeedsPackagedIdentityOrPermission
    }
}

fn classify_requested_access(
    status: Option<UserNotificationListenerAccessStatus>,
) -> NotifPermission {
    if status == Some(UserNotificationListenerAccessStatus::Allowed) {
        NotifPermission::Allowed
    } else {
        NotifPermission::NeedsPackagedIdentityOrPermission
    }
}

impl OsNotificationReader {
    /// Must be constructed on the UI thread so any first-run Unspecified access prompt is legal.
    pub fn new() -> Self {
        let initial_perm = resolve_permission_on_ui_thread();
        let perm = Arc::new(Mutex::new(initial_perm));
        let pending = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        if initial_perm == NotifPermission::Allowed {
            let (p_ref, q_ref, s_ref) =
                (Arc::clone(&perm), Arc::clone(&pending), Arc::clone(&stop));
            let _ = std::thread::Builder::new()
                .name("nuvra-notif".into())
                .spawn(move || listener_worker(p_ref, q_ref, s_ref));
        }
        Self {
            perm,
            pending,
            stop,
        }
    }

    pub fn permission_status(&self) -> NotifPermission {
        self.perm
            .lock()
            .map(|g| *g)
            .unwrap_or(NotifPermission::Unknown)
    }

    pub fn drain_new(&self) -> Vec<(String, String)> {
        self.pending
            .lock()
            .map(|mut g| std::mem::take(&mut *g))
            .unwrap_or_default()
    }
}

impl Default for OsNotificationReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OsNotificationReader {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn listener_worker(
    perm_ref: Arc<Mutex<NotifPermission>>,
    pending: Arc<Mutex<Vec<(String, String)>>>,
    stop: Arc<AtomicBool>,
) {
    crate::clipboard::init_com_mta();
    let Ok(listener) = UserNotificationListener::Current() else {
        if let Ok(mut p) = perm_ref.lock() {
            *p = NotifPermission::NeedsPackagedIdentityOrPermission;
        }
        return;
    };
    if listener.GetAccessStatus().ok() != Some(UserNotificationListenerAccessStatus::Allowed) {
        if let Ok(mut p) = perm_ref.lock() {
            *p = NotifPermission::NeedsPackagedIdentityOrPermission;
        }
        return;
    }
    let mut seen_ids: HashSet<u32> = HashSet::new();
    let mut primed = false;
    while !stop.load(Ordering::Acquire) {
        if let Ok(op) = listener.GetNotificationsAsync(NotificationKinds::Toast) {
            if let Ok(list) = op.get() {
                collect_new(&list, &mut seen_ids, primed, &pending);
                primed = true;
            }
        }
        std::thread::sleep(Duration::from_millis(1200));
    }
}

fn collect_new(
    list: &windows::Foundation::Collections::IVectorView<
        windows::UI::Notifications::UserNotification,
    >,
    seen: &mut HashSet<u32>,
    emit: bool,
    pending: &Arc<Mutex<Vec<(String, String)>>>,
) {
    let count = list.Size().unwrap_or(0);
    let binding_key = KnownNotificationBindings::ToastGeneric().ok();
    for i in 0..count {
        let Ok(item) = list.GetAt(i) else { continue };
        let Ok(id) = item.Id() else { continue };
        if !seen.insert(id) || !emit {
            continue;
        }
        let app = item
            .AppInfo()
            .ok()
            .and_then(|a| a.DisplayInfo().ok())
            .and_then(|d| d.DisplayName().ok())
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| "Windows".into());
        let mut summary = String::new();
        if let (Some(key), Ok(notif)) = (&binding_key, item.Notification()) {
            if let Ok(vis) = notif.Visual() {
                if let Ok(binding) = vis.GetBinding(key) {
                    if let Ok(texts) = binding.GetTextElements() {
                        let tcount = texts.Size().unwrap_or(0).min(2);
                        let mut parts = Vec::new();
                        for ti in 0..tcount {
                            if let Ok(el) = texts.GetAt(ti) {
                                if let Ok(txt) = el.Text() {
                                    let s = txt.to_string_lossy().trim().to_string();
                                    if !s.is_empty() {
                                        parts.push(s);
                                    }
                                }
                            }
                        }
                        summary = parts.join(" — ");
                    }
                }
            }
        }
        if !summary.is_empty() {
            if let Ok(mut q) = pending.lock() {
                q.push((app, summary));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_unspecified_triggers_request_and_classification() {
        assert!(should_request_access(&Ok(
            UserNotificationListenerAccessStatus::Unspecified
        )));
        assert!(!should_request_access(&Ok(
            UserNotificationListenerAccessStatus::Allowed
        )));
        assert!(!should_request_access(&Ok(
            UserNotificationListenerAccessStatus::Denied
        )));
        assert_eq!(
            classify_requested_access(Some(UserNotificationListenerAccessStatus::Allowed)),
            NotifPermission::Allowed
        );
        assert_eq!(
            classify_requested_access(Some(UserNotificationListenerAccessStatus::Denied)),
            NotifPermission::NeedsPackagedIdentityOrPermission
        );
    }
}
