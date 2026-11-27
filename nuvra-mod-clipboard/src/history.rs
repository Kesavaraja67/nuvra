// nuvra-mod-clipboard/src/history.rs
// In-memory indexed clipboard history with fuzzy search, pinning, and capacity bounds (Section 14.6, D-DAT-01..03).
// R-13: No unsafe.
// R-20: Content is never logged.
#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_MAX_ITEMS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardContentType {
    Text,
    Url,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub is_pinned: bool,
    pub timestamp: u64,
    pub content_type: ClipboardContentType,
}

pub struct ClipboardHistoryStore {
    entries: Vec<ClipboardEntry>,
    max_items: usize,
    next_id: u64,
}

impl ClipboardHistoryStore {
    pub fn new(max_items: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_items,
            next_id: 1,
        }
    }

    /// Add a captured text snippet to history.
    pub fn add(&mut self, text: String) -> u64 {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return 0;
        }

        // Avoid exact duplicate at the top
        if let Some(first) = self.entries.first() {
            if first.content == text {
                return first.id;
            }
        }

        // Detect URL content type
        let content_type = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            ClipboardContentType::Url
        } else {
            ClipboardContentType::Text
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let entry = ClipboardEntry {
            id,
            content: text,
            is_pinned: false,
            timestamp: now,
            content_type,
        };

        self.entries.insert(0, entry);

        // Trim unpinned items exceeding capacity
        if self.entries.len() > self.max_items {
            if let Some(pos) = self.entries.iter().rposition(|e| !e.is_pinned) {
                self.entries.remove(pos);
            }
        }

        id
    }

    /// Instant in-memory search across clipboard snippets.
    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.entries.iter().collect();
        }

        self.entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&q))
            .collect()
    }

    /// Pin or unpin an item. Pinned items are exempt from automatic FIFO eviction.
    pub fn set_pinned(&mut self, id: u64, pinned: bool) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.is_pinned = pinned;
            true
        } else {
            false
        }
    }

    /// Delete an individual item by ID.
    pub fn delete(&mut self, id: u64) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    /// Wipe all unpinned history items (D-DAT-01).
    pub fn wipe_unpinned(&mut self) {
        self.entries.retain(|e| e.is_pinned);
    }

    /// Complete wipe of all history including pinned items.
    pub fn wipe_all(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn pinned_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_pinned).count()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ClipboardHistoryStore {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ITEMS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_add_and_search() {
        let mut store = ClipboardHistoryStore::new(10);
        let id1 = store.add("https://rust-lang.org".into());
        let _id2 = store.add("Hello World from Rust".into());
        let _id3 = store.add("Random text".into());

        assert_eq!(store.len(), 3);
        assert_eq!(store.entries[2].content_type, ClipboardContentType::Url);

        let results = store.search("rust");
        assert_eq!(results.len(), 2);

        let empty_search = store.search("");
        assert_eq!(empty_search.len(), 3);

        assert!(store.delete(id1));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_clipboard_pinning_and_wipe() {
        let mut store = ClipboardHistoryStore::new(5);
        let id1 = store.add("Keep this forever".into());
        let _id2 = store.add("Ephemeral snippet".into());

        assert!(store.set_pinned(id1, true));
        assert_eq!(store.pinned_count(), 1);

        store.wipe_unpinned();
        assert_eq!(store.len(), 1);
        assert_eq!(store.entries[0].id, id1);

        store.wipe_all();
        assert!(store.is_empty());
    }
}
