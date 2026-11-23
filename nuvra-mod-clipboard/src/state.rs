// nuvra-mod-clipboard/src/state.rs
// Internal mutable state for the clipboard module.
// R-13: No unsafe.
// R-20: No clipboard content stored in memory beyond immediate processing.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::ClipboardSnapshot;

#[derive(Default)]
pub struct ClipboardState {
    pub item_count: usize,
    pub pinned_count: usize,
}

impl ClipboardState {
    pub fn snapshot(&self) -> ClipboardSnapshot {
        ClipboardSnapshot {
            item_count: self.item_count,
            pinned_count: self.pinned_count,
        }
    }
}
