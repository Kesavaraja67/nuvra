// nuvra-mod-shelf/src/module.rs
// Shelf module — IDropTarget file shelf.
// Phase 0: Module trait stub. Full implementation in v1.
// R-12: Explicitly Phase 0 stub — PARTIAL.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::state::ShelfState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct ShelfModule {
    state: ShelfState,
    tx: Option<Sender<ModuleEvent>>,
}

impl ShelfModule {
    pub fn new() -> Self {
        Self {
            state: ShelfState::default(),
            tx: None,
        }
    }
}

impl Default for ShelfModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ShelfModule {
    fn id(&self) -> &'static str {
        "shelf"
    }

    fn capabilities(&self) -> &[Capability] {
        &[Capability::FileSystemRead]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        // v1: register IDropTarget on the notch HWND.
        Ok(())
    }

    fn on_event(&mut self, _event: OsEvent) -> Result<(), ModuleError> {
        // v1: handle drop events, persist to shelf DB table.
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Shelf(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        // v1: revoke IDropTarget.
        self.tx = None;
    }
}
