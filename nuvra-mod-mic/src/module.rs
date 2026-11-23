// nuvra-mod-mic/src/module.rs
// Microphone mute control and call activity monitor (Section 14.3).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::call_detector::CallDetector;
use crate::state::MicState;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};
use nuvra_platform_win::audio::{get_microphone_mute, toggle_microphone_mute};
use nuvra_platform_win::camera::CameraMonitor;

pub struct MicModule {
    detector: CallDetector,
    camera: Option<CameraMonitor>,
    state: MicState,
    tx: Option<Sender<ModuleEvent>>,
}

impl MicModule {
    pub fn new() -> Self {
        let initial_mute = get_microphone_mute().unwrap_or(false);
        let camera = CameraMonitor::new();

        Self {
            detector: CallDetector::new(),
            camera,
            state: MicState {
                is_muted: initial_mute,
                call_active: false,
                app_name: None,
            },
            tx: None,
        }
    }

    pub fn detector(&self) -> &CallDetector {
        &self.detector
    }

    /// Toggle global microphone mute state via CoreAudio.
    pub fn toggle_mute(&mut self) -> bool {
        if let Ok(new_mute) = toggle_microphone_mute() {
            self.state.is_muted = new_mute;
            self.notify_change();
            new_mute
        } else {
            self.state.is_muted
        }
    }

    pub fn set_call_active(&mut self, active: bool, app_name: Option<String>) {
        self.state.call_active = active;
        self.state.app_name = app_name;
        self.notify_change();
    }

    fn notify_change(&self) {
        if let Some(tx) = &self.tx {
            let _ = tx.try_send(ModuleEvent {
                module_id: self.id(),
                state: ModuleState::Mic(self.state.snapshot()),
            });
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
        self.notify_change();
        Ok(())
    }

    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError> {
        match event {
            OsEvent::MicMuteChanged { muted } => {
                self.state.is_muted = muted;
                self.notify_change();
            }
            OsEvent::HotKey { id } => {
                // Hotkey ID 1: Ctrl+Alt+M global mute
                if id == 1 {
                    let _ = self.toggle_mute();
                }
            }
            OsEvent::TimerTick { .. } => {
                // Periodic sensor state check
                if let Some(cam) = &self.camera {
                    if cam.is_camera_active() && !self.state.call_active {
                        self.set_call_active(true, Some("Camera Active".into()));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Mic(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        self.tx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mic_module_init_and_state() {
        let mut module = MicModule::new();
        assert_eq!(module.id(), "mic");
        assert_eq!(module.capabilities(), &[Capability::MicrophoneAccess]);

        let (tx, _rx) = flume::bounded(10);
        module.init(tx).expect("init failed");

        let state = module.state();
        assert!(matches!(state, ModuleState::Mic(_)));
    }

    #[test]
    fn test_mic_module_call_state_update() {
        let mut module = MicModule::new();
        let (tx, rx) = flume::bounded(10);
        module.init(tx).expect("init failed");
        let _ = rx.try_recv(); // Drain initial event from init

        module.set_call_active(true, Some("Zoom".into()));
        let evt = rx.try_recv().expect("event expected");
        if let ModuleState::Mic(snap) = evt.state {
            assert!(snap.call_active);
            assert_eq!(snap.app_name, Some("Zoom".into()));
        } else {
            panic!("Expected Mic state");
        }
    }
}
