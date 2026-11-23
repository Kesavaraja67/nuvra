// nuvra-mod-notifications/src/module.rs
// Notifications module — UserNotificationListener and ephemeral banner queue (Section 14.4).
// R-13: No unsafe.
// R-20: Content is never logged or persisted beyond the ephemeral queue.
#![forbid(unsafe_code)]

use crate::queue::NotificationQueue;
use crate::state::NotificationsState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct NotificationsModule {
    queue: NotificationQueue,
    state: NotificationsState,
    tx: Option<Sender<ModuleEvent>>,
}

impl NotificationsModule {
    pub fn new() -> Self {
        Self {
            queue: NotificationQueue::new(10),
            state: NotificationsState::default(),
            tx: None,
        }
    }

    pub fn queue_mut(&mut self) -> &mut NotificationQueue {
        &mut self.queue
    }

    /// Push an incoming notification toast.
    pub fn push_toast(&mut self, app_name: String, summary: String) -> bool {
        if self.queue.push(app_name, summary) {
            self.refresh_state();
            true
        } else {
            false
        }
    }

    /// Dismiss the currently visible notification banner.
    pub fn dismiss_current(&mut self) {
        let _ = self.queue.pop();
        self.refresh_state();
    }

    fn refresh_state(&mut self) {
        if let Some(item) = self.queue.peek() {
            self.state.app_name = item.app_name.clone();
            self.state.summary = item.summary.clone();
            self.state.queued = self.queue.len() as u8;
        } else {
            self.state.app_name.clear();
            self.state.summary.clear();
            self.state.queued = 0;
        }

        if let Some(tx) = &self.tx {
            let _ = tx.try_send(ModuleEvent {
                module_id: self.id(),
                state: ModuleState::Notification(self.state.snapshot()),
            });
        }
    }
}

impl Default for NotificationsModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for NotificationsModule {
    fn id(&self) -> &'static str {
        "notifications"
    }

    fn capabilities(&self) -> &[Capability] {
        &[Capability::NotificationAccess]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        self.refresh_state();
        Ok(())
    }

    fn on_event(&mut self, _event: OsEvent) -> Result<(), ModuleError> {
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Notification(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        self.tx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifications_module_lifecycle() {
        let mut module = NotificationsModule::new();
        assert_eq!(module.id(), "notifications");

        let (tx, rx) = flume::bounded(10);
        module.init(tx).expect("init failed");
        let _ = rx.try_recv(); // drain init

        assert!(module.push_toast("Outlook".into(), "New meeting invite".into()));
        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Notification(snap) = evt.state {
            assert_eq!(snap.app_name, "Outlook");
            assert_eq!(snap.summary, "New meeting invite");
            assert_eq!(snap.queued, 1);
        } else {
            panic!("Expected Notification state");
        }

        module.dismiss_current();
        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Notification(snap) = evt.state {
            assert_eq!(snap.queued, 0);
        }
    }
}
