// nuvra-mod-shelf/src/state.rs
// Internal mutable state for the shelf module.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::ShelfSnapshot;

#[derive(Default)]
pub struct ShelfState {
    pub item_count: usize,
}

impl ShelfState {
    pub fn snapshot(&self) -> ShelfSnapshot {
        ShelfSnapshot {
            item_count: self.item_count,
        }
    }
}
