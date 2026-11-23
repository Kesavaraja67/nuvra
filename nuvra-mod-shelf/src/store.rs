// nuvra-mod-shelf/src/store.rs
// File shelf item reference store and operations (Section 14.5, D-DAT-04).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use thiserror::Error;

pub const MAX_SHELF_ITEMS: usize = 20;

#[derive(Debug, Error)]
pub enum ShelfError {
    #[error("Shelf capacity exceeded (max {0} items)")]
    CapacityExceeded(usize),
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfFileItem {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub is_directory: bool,
}

pub struct ShelfStore {
    items: Vec<ShelfFileItem>,
    max_items: usize,
}

impl ShelfStore {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: MAX_SHELF_ITEMS,
        }
    }

    /// Add a file or folder path dropped onto the shelf.
    pub fn add_path(&mut self, path: PathBuf) -> Result<(), ShelfError> {
        if self.items.len() >= self.max_items {
            return Err(ShelfError::CapacityExceeded(self.max_items));
        }

        // Avoid duplicate paths
        if self.items.iter().any(|item| item.path == path) {
            return Ok(());
        }

        let metadata =
            std::fs::metadata(&path).map_err(|_| ShelfError::FileNotFound(path.clone()))?;
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_directory = metadata.is_dir();
        let size_bytes = if is_directory { 0 } else { metadata.len() };

        self.items.push(ShelfFileItem {
            path,
            name,
            size_bytes,
            is_directory,
        });

        Ok(())
    }

    pub fn remove_at(&mut self, index: usize) -> Option<ShelfFileItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn items(&self) -> &[ShelfFileItem] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for ShelfStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_store_add_and_remove() {
        let mut store = ShelfStore::new();
        // Use an existing file in the repo (Cargo.toml)
        let cargo_path = PathBuf::from("Cargo.toml");
        store.add_path(cargo_path.clone()).expect("add Cargo.toml");
        assert_eq!(store.len(), 1);
        assert_eq!(store.items()[0].name, "Cargo.toml");

        // Duplicate add is a no-op
        store.add_path(cargo_path).expect("re-add Cargo.toml");
        assert_eq!(store.len(), 1);

        let removed = store.remove_at(0).expect("remove");
        assert_eq!(removed.name, "Cargo.toml");
        assert!(store.is_empty());
    }

    #[test]
    fn test_shelf_store_nonexistent() {
        let mut store = ShelfStore::new();
        let fake = PathBuf::from("nonexistent_file_path_12345.txt");
        assert!(matches!(
            store.add_path(fake),
            Err(ShelfError::FileNotFound(_))
        ));
    }
}
