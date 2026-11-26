// nuvra-platform-win/src/shell_ops.rs
// Safe Windows Shell file actions: Reveal in Explorer and non-overwriting ZIP compression (Section 14.5).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Open Windows Explorer highlighting the specified file or folder.
pub fn reveal_in_explorer(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    Command::new("explorer.exe")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .is_ok()
}

fn pick_unique_zip_path(dir: &Path, stem: &str) -> PathBuf {
    let base = format!("{stem}.zip");
    let first = dir.join(&base);
    if !first.exists() {
        return first;
    }
    for n in 2..=999 {
        let cand = dir.join(format!("{stem} ({n}).zip"));
        if !cand.exists() {
            return cand;
        }
    }
    dir.join(format!("{stem}-archive.zip"))
}

/// Compress shelf paths into a new ZIP archive beside the first item without overwriting existing files.
pub fn compress_to_zip(paths: &[PathBuf]) -> Result<PathBuf, String> {
    let first = paths.first().ok_or_else(|| "No files to compress".to_string())?;
    if !first.exists() {
        return Err("Source file missing".into());
    }
    let parent = first.parent().unwrap_or_else(|| Path::new("."));
    let stem = first
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Nuvra-Archive");
    let zip_path = pick_unique_zip_path(parent, stem);

    let mut cmd = Command::new("tar.exe");
    cmd.arg("-a").arg("-c").arg("-f").arg(&zip_path).arg("-C").arg(parent);
    let mut added = 0usize;
    for p in paths {
        if p.exists() && p.parent() == Some(parent) {
            if let Some(name) = p.file_name() {
                cmd.arg(name);
                added += 1;
            }
        }
    }
    if added == 0 {
        return Err("Selected files must exist in the folder".into());
    }
    let status = cmd.status().map_err(|e| format!("Failed to invoke tar.exe: {e}"))?;
    if status.success() && zip_path.exists() {
        Ok(zip_path)
    } else {
        Err("ZIP compression failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_to_zip_creates_non_overwriting_archive() {
        let dir = std::env::temp_dir().join("nuvra_zip_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let sample = dir.join("note.txt");
        std::fs::write(&sample, b"hello nuvra").expect("write");

        let zip1 = compress_to_zip(std::slice::from_ref(&sample)).expect("zip1");
        assert!(zip1.exists());
        let zip2 = compress_to_zip(std::slice::from_ref(&sample)).expect("zip2");
        assert!(zip2.exists());
        assert_ne!(zip1, zip2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
