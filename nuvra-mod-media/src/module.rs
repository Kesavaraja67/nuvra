// nuvra-mod-media/src/module.rs
// Media module — GSMTC session manager and audio visualizer (Section 14.2).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::gsmtc::GsmtcController;
use crate::lyrics::LyricsCache;
use crate::state::MediaState;
use crate::visualizer::AudioVisualizer;
use flume::Sender;
use nuvra_core_shared::{
    event::OsEvent,
    module::{Capability, Module, ModuleError, ModuleEvent, ModuleState},
};

pub struct MediaModule {
    gsmtc: GsmtcController,
    visualizer: AudioVisualizer,
    lyrics: LyricsCache,
    state: MediaState,
    tx: Option<Sender<ModuleEvent>>,
}

impl MediaModule {
    pub fn new() -> Self {
        Self {
            gsmtc: GsmtcController::new(),
            visualizer: AudioVisualizer::new(),
            lyrics: LyricsCache::new(),
            state: MediaState::default(),
            tx: None,
        }
    }

    pub fn visualizer_mut(&mut self) -> &mut AudioVisualizer {
        &mut self.visualizer
    }

    pub fn lyrics_mut(&mut self) -> &mut LyricsCache {
        &mut self.lyrics
    }

    /// Update media snapshot from the OS GSMTC session.
    pub fn refresh_session(&mut self) {
        if let Some(snapshot) = self.gsmtc.query_current_session() {
            self.state.title = snapshot.title;
            self.state.artist = snapshot.artist;
            self.state.is_playing = snapshot.is_playing;
            self.state.position_s = snapshot.position_s;
            self.state.duration_s = snapshot.duration_s;
            self.state.has_art = snapshot.has_art;
            self.state.art_key = snapshot.art_key;

            if let Some(tx) = &self.tx {
                let _ = tx.try_send(ModuleEvent {
                    module_id: self.id(),
                    state: ModuleState::Media(self.state.snapshot()),
                });
            }
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
        &[Capability::AudioLoopback]
    }

    fn init(&mut self, tx: Sender<ModuleEvent>) -> Result<(), ModuleError> {
        self.tx = Some(tx);
        self.refresh_session();
        Ok(())
    }

    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError> {
        match event {
            OsEvent::MediaSessionChanged => {
                self.refresh_session();
            }
            OsEvent::TimerTick { .. } if self.state.is_playing => {
                // Periodic update while playing
                self.refresh_session();
            }
            _ => {}
        }
        Ok(())
    }

    fn state(&self) -> ModuleState {
        ModuleState::Media(self.state.snapshot())
    }

    fn shutdown(&mut self) {
        self.tx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_module_init_and_state() {
        let mut module = MediaModule::new();
        assert_eq!(module.id(), "media");
        assert_eq!(module.capabilities(), &[Capability::AudioLoopback]);

        let (tx, _rx) = flume::bounded(10);
        module.init(tx).expect("init failed");

        let state = module.state();
        assert!(matches!(state, ModuleState::Media(_)));
    }
}
