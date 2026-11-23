// nuvra-mod-shelf/src/module.rs
// Shelf module — IDropTarget file shelf (Section 14.5).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::state::ShelfState;
use crate::store::{ShelfError, ShelfStore};
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};
use std::path::PathBuf;

pub struct ShelfModule {
    store: ShelfStore,
    state: ShelfState,
    tx: Option<Sender<ModuleEvent>>,
}

impl ShelfModule {
    pub fn new() -> Self {
        Self {
            store: ShelfStore::new(),
            state: ShelfState::default(),
            tx: None,
        }
    }

    pub fn store(&self) -> &ShelfStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut ShelfStore {
        &mut self.store
    }

    /// Add a dropped file path to the shelf.
    pub fn add_file(&mut self, path: PathBuf) -> Result<(), ShelfError> {
        self.store.add_path(path)?;
        self.refresh_state();
        Ok(())
    }

    /// Remove a file at index.
    pub fn remove_file(&mut self, index: usize) {
        let _ = self.store.remove_at(index);
        self.refresh_state();
    }

    fn refresh_state(&mut self) {
        self.state.item_count = self.store.len();

        if let Some(tx) = &self.tx {
            let _ = tx.try_send(ModuleEvent {
                module_id: self.id(),
                state: ModuleState::Shelf(self.state.snapshot()),
            });
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
        self.refresh_state();
        Ok(())
    }

    fn on_event(&mut self, _event: OsEvent) -> Result<(), ModuleError> {
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Shelf(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        self.tx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_module_lifecycle() {
        let mut module = ShelfModule::new();
        assert_eq!(module.id(), "shelf");

        let (tx, rx) = flume::bounded(10);
        module.init(tx).expect("init failed");
        let _ = rx.try_recv(); // drain init

        let cargo_path = PathBuf::from("Cargo.toml");
        module.add_file(cargo_path).expect("add file");

        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Shelf(snap) = evt.state {
            assert_eq!(snap.item_count, 1);
        } else {
            panic!("Expected Shelf state");
        }

        module.remove_file(0);
        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Shelf(snap) = evt.state {
            assert_eq!(snap.item_count, 0);
        }
    }
}
