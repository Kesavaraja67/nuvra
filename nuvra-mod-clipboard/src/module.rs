// nuvra-mod-clipboard/src/module.rs
// Clipboard module — history, pinning, AES-256-GCM storage.
// Phase 0: Module trait stub. Full implementation in v1.
// R-12: Explicitly Phase 0 stub — PARTIAL.
// R-13: No unsafe.
// R-20: Clipboard content never logged.
#![forbid(unsafe_code)]

use crate::state::ClipboardState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct ClipboardModule {
    state: ClipboardState,
    tx: Option<Sender<ModuleEvent>>,
}

impl ClipboardModule {
    pub fn new() -> Self {
        Self {
            state: ClipboardState::default(),
            tx: None,
        }
    }
}

impl Default for ClipboardModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ClipboardModule {
    fn id(&self) -> &'static str {
        "clipboard"
    }

    fn capabilities(&self) -> &[Capability] {
        &[Capability::ClipboardRead]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        // v1: AddClipboardFormatListener on HWND; open SQLCipher DB.
        Ok(())
    }

    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError> {
        if let OsEvent::ClipboardChanged = event {
            // v1: read clipboard text, check exclusions, encrypt+store.
            // R-20: never log the text.
            self.state.item_count += 1;
        }
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Clipboard(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        // v1: RemoveClipboardFormatListener; close DB.
        self.tx = None;
    }
}
