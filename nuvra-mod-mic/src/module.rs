// nuvra-mod-mic/src/module.rs
// Mic module — MMDevice mute state + call detection.
// Phase 0: Module trait stub. Full implementation in v1.
// R-12: Explicitly Phase 0 stub — PARTIAL.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::state::MicState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct MicModule {
    state: MicState,
    tx: Option<Sender<ModuleEvent>>,
}

impl MicModule {
    pub fn new() -> Self {
        Self {
            state: MicState::default(),
            tx: None,
        }
    }
}

impl Default for MicModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for MicModule {
    fn id(&self) -> &'static str {
        "mic"
    }

    fn capabilities(&self) -> &[Capability] {
        &[Capability::MicrophoneAccess]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        // v1: register IMMNotificationClient for default device changes.
        Ok(())
    }

    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError> {
        match event {
            OsEvent::MicMuteChanged { muted } => {
                self.state.is_muted = muted;
                // v1: push updated ModuleEvent via self.tx.
            }
            OsEvent::ForegroundWindowChanged { .. } => {
                // v1: detect call apps (Teams, Zoom, Meet, Discord).
            }
            _ => {}
        }
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Mic(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        // v1: unregister IMMNotificationClient.
        self.tx = None;
    }
}
