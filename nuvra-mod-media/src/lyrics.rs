// nuvra-mod-media/src/lyrics.rs
// Synced LRC parser and local cache (Section 14.2, D-NET-01/02).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub timestamp_s: f32,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct SyncedLyrics {
    pub lines: Vec<LyricLine>,
}

impl SyncedLyrics {
    /// Parse standard LRC format string into ordered timestamps and lyrics.
    pub fn parse(lrc_content: &str) -> Self {
        let mut lines = Vec::new();

        for raw_line in lrc_content.lines() {
            let line = raw_line.trim();
            if !line.starts_with('[') {
                continue;
            }

            if let Some(close_bracket) = line.find(']') {
                let tag = &line[1..close_bracket];
                let text = line[close_bracket + 1..].trim().to_string();

                // Format: mm:ss.xx or mm:ss
                let parts: Vec<&str> = tag.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(minutes), Ok(seconds)) =
                        (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                    {
                        let timestamp_s = minutes * 60.0 + seconds;
                        lines.push(LyricLine { timestamp_s, text });
                    }
                }
            }
        }

        lines.sort_by(|a, b| {
            a.timestamp_s
                .partial_cmp(&b.timestamp_s)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { lines }
    }

    /// Return the active lyric line for the current playback position.
    pub fn current_line(&self, position_s: f32) -> Option<&LyricLine> {
        self.lines
            .iter()
            .rev()
            .find(|line| line.timestamp_s <= position_s)
    }
}

pub struct LyricsCache {
    cache: HashMap<String, SyncedLyrics>,
}

impl LyricsCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn make_key(title: &str, artist: &str) -> String {
        format!(
            "{}::{}",
            title.trim().to_lowercase(),
            artist.trim().to_lowercase()
        )
    }

    pub fn get(&self, title: &str, artist: &str) -> Option<&SyncedLyrics> {
        self.cache.get(&Self::make_key(title, artist))
    }

    pub fn insert(&mut self, title: &str, artist: &str, lyrics: SyncedLyrics) {
        self.cache.insert(Self::make_key(title, artist), lyrics);
    }
}

impl Default for LyricsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let lrc = r#"
[00:04.50] First line of the song
[00:08.20] Second line follows
[00:15.00] Chorus starts now
"#;
        let synced = SyncedLyrics::parse(lrc);
        assert_eq!(synced.lines.len(), 3);
        assert_eq!(synced.lines[0].text, "First line of the song");
        assert_eq!(synced.lines[0].timestamp_s, 4.5);

        assert_eq!(synced.current_line(2.0), None);
        assert_eq!(
            synced.current_line(5.0).unwrap().text,
            "First line of the song"
        );
        assert_eq!(
            synced.current_line(10.0).unwrap().text,
            "Second line follows"
        );
        assert_eq!(synced.current_line(20.0).unwrap().text, "Chorus starts now");
    }
}
