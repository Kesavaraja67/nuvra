// nuvra-mod-mic/src/state.rs
// Internal mutable state for the mic module.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::MicSnapshot;

#[derive(Default)]
pub struct MicState {
    pub is_muted: bool,
    pub call_active: bool,
    pub app_name: Option<String>,
}

impl MicState {
    pub fn snapshot(&self) -> MicSnapshot {
        MicSnapshot {
            is_muted: self.is_muted,
            call_active: self.call_active,
            app_name: self.app_name.clone(),
        }
    }
}
