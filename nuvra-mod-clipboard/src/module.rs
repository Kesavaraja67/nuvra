// nuvra-mod-clipboard/src/module.rs
// Clipboard module — history, search, pinning, and exclusion filtering (Section 14.6).
// R-13: No unsafe.
// R-20: Clipboard content never logged.
#![forbid(unsafe_code)]

use crate::exclusions::is_excluded;
use crate::history::ClipboardHistoryStore;
use crate::state::ClipboardState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct ClipboardModule {
    store: ClipboardHistoryStore,
    state: ClipboardState,
    tx: Option<Sender<ModuleEvent>>,
}

impl ClipboardModule {
    pub fn new() -> Self {
        Self {
            store: ClipboardHistoryStore::default(),
            state: ClipboardState::default(),
            tx: None,
        }
    }

    pub fn store(&self) -> &ClipboardHistoryStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut ClipboardHistoryStore {
        &mut self.store
    }

    /// Process a new clipboard text capture with exclusion check.
    pub fn process_capture(&mut self, text: String, source_window_class: Option<&str>) -> bool {
        if let Some(class) = source_window_class {
            if is_excluded(class) {
                // Silently drop excluded sensitive content per D-DAT-03
                return false;
            }
        }

        let id = self.store.add(text);
        if id > 0 {
            self.refresh_state();
            true
        } else {
            false
        }
    }

    fn refresh_state(&mut self) {
        self.state.item_count = self.store.len();
        self.state.pinned_count = self.store.pinned_count();

        if let Some(tx) = &self.tx {
            let _ = tx.try_send(ModuleEvent {
                module_id: self.id(),
                state: ModuleState::Clipboard(self.state.snapshot()),
            });
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
        self.refresh_state();
        Ok(())
    }

    fn on_event(&mut self, _event: OsEvent) -> Result<(), ModuleError> {
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Clipboard(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        self.tx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_module_capture_and_exclusion() {
        let mut module = ClipboardModule::new();
        assert_eq!(module.id(), "clipboard");

        let (tx, rx) = flume::bounded(10);
        module.init(tx).expect("init failed");
        let _ = rx.try_recv(); // drain init

        // Normal capture
        assert!(module.process_capture("Hello from Notepad".into(), Some("Notepad")));
        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Clipboard(snap) = evt.state {
            assert_eq!(snap.item_count, 1);
        } else {
            panic!("Expected Clipboard state");
        }

        // Excluded password manager capture (D-DAT-03)
        assert!(!module.process_capture("SecretPassword123".into(), Some("1Password")));
        assert_eq!(module.store().len(), 1); // unchanged
    }
}
