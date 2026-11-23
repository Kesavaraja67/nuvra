// nuvra-mod-media/src/state.rs
// Internal mutable state for the media module.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::MediaSnapshot;

#[derive(Default)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub has_art: bool,
    pub art_key: Option<u64>,
    pub position_s: f32,
    pub duration_s: f32,
    pub volume_norm: f32,
}

impl MediaState {
    /// Produce a cheap clone for the renderer.
    pub fn snapshot(&self) -> MediaSnapshot {
        MediaSnapshot {
            title: self.title.clone(),
            artist: self.artist.clone(),
            is_playing: self.is_playing,
            has_art: self.has_art,
            art_key: self.art_key,
            position_s: self.position_s,
            duration_s: self.duration_s,
            volume_norm: self.volume_norm,
        }
    }
}
