// nuvra-mod-notifications/src/module.rs
// Notifications module — UserNotificationListener (WinRT).
// Phase 0: Module trait stub. Full implementation in v1.
// R-12: Explicitly Phase 0 stub — PARTIAL.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::state::NotificationsState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct NotificationsModule {
    state: NotificationsState,
    tx: Option<Sender<ModuleEvent>>,
}

impl NotificationsModule {
    pub fn new() -> Self {
        Self {
            state: NotificationsState::default(),
            tx: None,
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
        // v1: request UserNotificationListener access, register Changed handler.
        Ok(())
    }

    fn on_event(&mut self, _event: OsEvent) -> Result<(), ModuleError> {
        // v1: handle WinRT notification events.
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Notification(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        // v1: revoke WinRT handlers.
        self.tx = None;
    }
}
