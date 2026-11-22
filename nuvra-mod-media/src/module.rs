// nuvra-mod-media/src/module.rs
// Media module — GSMTC (GlobalSystemMediaTransportControls) integration.
// Phase 0: Module trait stub. Full implementation in v1.
// R-12: This is explicitly scoped as a Phase 0 stub — PARTIAL (see handoff).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::state::MediaState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct MediaModule {
    state: MediaState,
    tx: Option<Sender<ModuleEvent>>,
}

impl MediaModule {
    pub fn new() -> Self {
        Self {
            state: MediaState::default(),
            tx: None,
        }
    }
}

impl Default for MediaModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for MediaModule {
    fn id(&self) -> &'static str {
        "media"
    }

    fn capabilities(&self) -> &[Capability] {
        // No special OS capability required; GSMTC is accessible to any process.
        &[]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        // v1: register GSMTC session manager change handler here.
        Ok(())
    }

    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError> {
        if let OsEvent::MediaSessionChanged = event {
            // v1: query GSMTC, update self.state, push ModuleEvent via self.tx.
        }
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Media(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        // v1: unregister GSMTC callbacks.
        self.tx = None;
    }
}
