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

fn pick_unique_zip_path_with_limit(
    dir: &Path,
    stem: &str,
    max_numbered: usize,
) -> Result<PathBuf, String> {
    let base = format!("{stem}.zip");
    let first = dir.join(&base);
    if !first.exists() {
        return Ok(first);
    }
    for n in 2..=max_numbered {
        let cand = dir.join(format!("{stem} ({n}).zip"));
        if !cand.exists() {
            return Ok(cand);
        }
    }
    let fallback = dir.join(format!("{stem}-archive.zip"));
    if !fallback.exists() {
        return Ok(fallback);
    }
    for n in 2..=max_numbered {
        let cand = dir.join(format!("{stem}-archive ({n}).zip"));
        if !cand.exists() {
            return Ok(cand);
        }
    }
    Err("No unused destination archive filename remains".into())
}

fn common_parent_dir(paths: &[PathBuf]) -> Result<PathBuf, String> {
    let mut parents = Vec::with_capacity(paths.len());
    for p in paths {
        if !p.exists() {
            return Err(format!("Selected file does not exist: {}", p.display()));
        }
        let abs = std::path::absolute(p).map_err(|e| format!("Invalid file path: {e}"))?;
        let parent = abs
            .parent()
            .ok_or_else(|| "Selected path has no parent directory".to_string())?
            .to_path_buf();
        parents.push((abs, parent));
    }
    let first_parent = &parents[0].1;
    let mut common_comps: Vec<std::path::Component<'_>> = first_parent.components().collect();
    for (_, parent) in &parents[1..] {
        let comps: Vec<_> = parent.components().collect();
        let shared = common_comps
            .iter()
            .zip(comps.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common_comps.truncate(shared);
    }
    if common_comps.is_empty() {
        return Err("Selected files must share a common directory or drive".into());
    }
    let common: PathBuf = common_comps.iter().collect();
    // Require Prefix + RootDir if path uses a drive prefix so `tar -C` receives a real directory
    if common_comps.len() == 1 && matches!(common_comps[0], std::path::Component::Prefix(_)) {
        return Err("Selected files must share a common drive or folder".into());
    }
    Ok(common)
}

fn literal_tar_operand(abs_path: &Path, root: &Path) -> Result<String, String> {
    let rel = abs_path
        .strip_prefix(root)
        .map_err(|_| "Selected file cannot be made relative to archive root".to_string())?;
    let s = rel.to_string_lossy();
    if s.is_empty() {
        return Err("Unsupported empty path operand".into());
    }
    Ok(format!("./{s}"))
}

/// Compress all selected shelf paths into a new non-overwriting ZIP archive beside the first item.
/// Rejects the entire selection before creating output if any path is missing or unsupported.
pub fn compress_to_zip(paths: &[PathBuf]) -> Result<PathBuf, String> {
    if paths.is_empty() {
        return Err("No files to compress".into());
    }
    let common_root = common_parent_dir(paths)?;

    let mut abs_paths = Vec::with_capacity(paths.len());
    for p in paths {
        let abs = std::path::absolute(p).map_err(|e| format!("Invalid file path: {e}"))?;
        let _ = literal_tar_operand(&abs, &common_root)?;
        abs_paths.push(abs);
    }

    let first_abs = &abs_paths[0];
    let dest_dir = first_abs
        .parent()
        .ok_or_else(|| "Source file has no parent directory".to_string())?;
    let stem = first_abs
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Nuvra-Archive");
    let zip_path = pick_unique_zip_path_with_limit(dest_dir, stem, 999)?;

    let mut cmd = Command::new("tar.exe");
    cmd.arg("-a")
        .arg("-c")
        .arg("-f")
        .arg(&zip_path)
        .arg("-C")
        .arg(&common_root);
    for abs in &abs_paths {
        cmd.arg(literal_tar_operand(abs, &common_root)?);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to invoke tar.exe: {e}"))?;
    if status.success() && zip_path.exists() {
        Ok(zip_path)
    } else {
        let _ = std::fs::remove_file(&zip_path);
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

    #[test]
    fn test_compress_to_zip_rejects_partial_missing_selection() {
        let dir = std::env::temp_dir().join("nuvra_zip_partial_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let valid = dir.join("valid.txt");
        std::fs::write(&valid, b"ok").expect("write");
        let missing = dir.join("missing.txt");

        let err = compress_to_zip(&[valid, missing]).expect_err("must reject subset");
        assert!(err.contains("does not exist"));
        assert!(!dir.join("valid.zip").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_compress_to_zip_handles_cross_folder_and_special_names() {
        let dir = std::env::temp_dir().join("nuvra_zip_special_test");
        let _ = std::fs::remove_dir_all(&dir);
        let sub_a = dir.join("sub_a");
        let sub_b = dir.join("sub_b");
        std::fs::create_dir_all(&sub_a).expect("mkdir a");
        std::fs::create_dir_all(&sub_b).expect("mkdir b");
        let at_file = sub_a.join("@list.txt");
        let dash_file = sub_b.join("-flag.txt");
        std::fs::write(&at_file, b"at-body").expect("write @");
        std::fs::write(&dash_file, b"dash-body").expect("write -");

        let zip = compress_to_zip(&[at_file, dash_file]).expect("compress cross-folder");
        assert!(zip.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pick_unique_zip_path_never_overwrites_existing_fallback() {
        let dir = std::env::temp_dir().join("nuvra_zip_fallback_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("item.zip"), b"1").expect("w");
        std::fs::write(dir.join("item (2).zip"), b"2").expect("w");
        std::fs::write(dir.join("item-archive.zip"), b"fb").expect("w");

        let next = pick_unique_zip_path_with_limit(&dir, "item", 2).expect("next archive");
        assert_eq!(next, dir.join("item-archive (2).zip"));

        std::fs::write(dir.join("item-archive (2).zip"), b"fb2").expect("w");
        assert!(pick_unique_zip_path_with_limit(&dir, "item", 2).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
