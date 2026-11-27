// nuvra-platform-win/src/clipboard.rs
// Win32 Clipboard read/write, OLE initialization, and native file drag-out (CF_HDROP / SHDoDragDrop).
// Every unsafe block carries a // SAFETY: comment.

use std::path::PathBuf;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GlobalFree, BOOL, HANDLE, HGLOBAL, HWND, POINT};
use windows::Win32::System::Com::{CoInitializeEx, IDataObject, COINIT_MULTITHREADED};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::{
    IDropSource, OleInitialize, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE,
};
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    ILFree, IShellFolder, SHBindToParent, SHDoDragDrop, SHParseDisplayName, DROPFILES,
};

const CF_UNICODEXT_FMT: u32 = 13;
const CF_HDROP_FMT: u32 = 15;

/// Initialize OLE + COM in apartment-threaded mode (required for CoreAudio and SHDoDragDrop).
pub fn init_com_sta() {
    // SAFETY: Standard Win32 OLE/COM STA initialization on the UI thread.
    unsafe {
        let _ = OleInitialize(None);
    }
}

/// Initialize COM in multi-threaded mode on worker threads for non-blocking WinRT GSMTC queries.
pub fn init_com_mta() {
    // SAFETY: Standard Win32 COM MTA initialization on a background worker thread.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// Monotonic Windows clipboard sequence counter for zero-allocation change detection.
pub fn get_clipboard_sequence() -> u32 {
    // SAFETY: Thread-safe Win32 query returning current clipboard sequence number.
    unsafe { GetClipboardSequenceNumber() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardReadResult {
    Text(String),
    NonText,
    OpenFailed,
}

struct ClipboardOpenGuard;

impl Drop for ClipboardOpenGuard {
    fn drop(&mut self) {
        // SAFETY: Always balances a successful OpenClipboard call.
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

struct HGlobalGuard(Option<HGLOBAL>);

impl HGlobalGuard {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for HGlobalGuard {
    fn drop(&mut self) {
        if let Some(hmem) = self.0.take() {
            // SAFETY: Frees uncommitted global memory not yet owned by Windows clipboard.
            unsafe {
                let _ = GlobalFree(hmem);
            }
        }
    }
}

/// Read at most `byte_size` bytes of UTF-16LE from `wide_ptr`, stopping cleanly at NUL or bound.
///
/// # Safety
/// `wide_ptr` must be valid for reads up to `byte_size` bytes when non-null.
pub(crate) unsafe fn extract_utf16_bounded(wide_ptr: *const u16, byte_size: usize) -> String {
    let max_units = (byte_size / std::mem::size_of::<u16>()).min(65_536);
    if wide_ptr.is_null() || max_units == 0 {
        return String::new();
    }
    let mut len = 0usize;
    while len < max_units && *wide_ptr.add(len) != 0 {
        len += 1;
    }
    if len == 0 {
        return String::new();
    }
    let slice = std::slice::from_raw_parts(wide_ptr, len);
    String::from_utf16_lossy(slice)
}

/// Read UTF-16 Unicode text from the Windows clipboard, distinguishing transient open failure from non-text data.
pub fn try_read_clipboard_text() -> ClipboardReadResult {
    // SAFETY: OpenClipboard -> GlobalSize-bounded GlobalLock/Unlock -> CloseClipboard via guard.
    unsafe {
        if OpenClipboard(HWND(ptr::null_mut())).is_err() {
            return ClipboardReadResult::OpenFailed;
        }
        let _clip_guard = ClipboardOpenGuard;

        let Ok(handle) = GetClipboardData(CF_UNICODEXT_FMT) else {
            return ClipboardReadResult::NonText;
        };
        let hglobal = HGLOBAL(handle.0);
        let byte_size = GlobalSize(hglobal);
        if byte_size < std::mem::size_of::<u16>() {
            return ClipboardReadResult::NonText;
        }
        let ptr_void = GlobalLock(hglobal);
        if ptr_void.is_null() {
            return ClipboardReadResult::NonText;
        }
        let text = extract_utf16_bounded(ptr_void as *const u16, byte_size);
        let _ = GlobalUnlock(hglobal);
        ClipboardReadResult::Text(text)
    }
}

/// Read UTF-16 Unicode text from the Windows clipboard if available.
pub fn read_clipboard_text() -> Option<String> {
    match try_read_clipboard_text() {
        ClipboardReadResult::Text(s) => Some(s),
        ClipboardReadResult::NonText | ClipboardReadResult::OpenFailed => None,
    }
}

/// Write UTF-16 Unicode text onto the Windows clipboard owned by `owner`.
pub fn write_clipboard_text(owner: HWND, text: &str) -> bool {
    if owner.0.is_null() {
        return false;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let byte_len = wide.len() * std::mem::size_of::<u16>();

    // SAFETY: Allocates movable HGLOBAL, checks lock/open/EmptyClipboard, and frees allocation on any failure.
    unsafe {
        let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, byte_len) else {
            return false;
        };
        let mut mem_guard = HGlobalGuard(Some(hmem));

        let ptr_void = GlobalLock(hmem);
        if ptr_void.is_null() {
            return false;
        }
        ptr::copy_nonoverlapping(wide.as_ptr(), ptr_void as *mut u16, wide.len());
        let _ = GlobalUnlock(hmem);

        if OpenClipboard(owner).is_err() {
            return false;
        }
        let _clip_guard = ClipboardOpenGuard;

        if EmptyClipboard().is_err() {
            return false;
        }
        if SetClipboardData(CF_UNICODEXT_FMT, HANDLE(hmem.0)).is_ok() {
            mem_guard.disarm();
            true
        } else {
            false
        }
    }
}

/// Copy one or more staged file paths to the Windows clipboard as CF_HDROP (pastable in Explorer/apps).
pub fn write_clipboard_files(owner: HWND, paths: &[PathBuf]) -> bool {
    if owner.0.is_null() || paths.is_empty() {
        return false;
    }
    let mut wide_list: Vec<u16> = Vec::new();
    for p in paths {
        let s = p.to_string_lossy();
        wide_list.extend(s.encode_utf16());
        wide_list.push(0);
    }
    wide_list.push(0);

    let hdr_size = std::mem::size_of::<DROPFILES>();
    let total_bytes = hdr_size + wide_list.len() * std::mem::size_of::<u16>();

    // SAFETY: Allocates movable HGLOBAL containing DROPFILES, verifies EmptyClipboard, and frees allocation on failure.
    unsafe {
        let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, total_bytes) else {
            return false;
        };
        let mut mem_guard = HGlobalGuard(Some(hmem));

        let ptr_void = GlobalLock(hmem);
        if ptr_void.is_null() {
            return false;
        }
        let df = DROPFILES {
            pFiles: hdr_size as u32,
            pt: POINT { x: 0, y: 0 },
            fNC: BOOL(0),
            fWide: BOOL(1),
        };
        ptr::write_unaligned(ptr_void as *mut DROPFILES, df);
        let dst = (ptr_void as *mut u8).add(hdr_size) as *mut u16;
        ptr::copy_nonoverlapping(wide_list.as_ptr(), dst, wide_list.len());
        let _ = GlobalUnlock(hmem);

        if OpenClipboard(owner).is_err() {
            return false;
        }
        let _clip_guard = ClipboardOpenGuard;

        if EmptyClipboard().is_err() {
            return false;
        }
        if SetClipboardData(CF_HDROP_FMT, HANDLE(hmem.0)).is_ok() {
            mem_guard.disarm();
            true
        } else {
            false
        }
    }
}

/// Initiate native Windows OLE file drag-out (`SHDoDragDrop`) from the shelf onto Explorer/Desktop/apps.
pub fn start_file_drag_drop(hwnd: HWND, path: &std::path::Path) -> bool {
    let mut wide: Vec<u16> = path.to_string_lossy().encode_utf16().collect();
    wide.push(0);

    // SAFETY: Parses display name to PIDL, retrieves Shell IDataObject via SHBindToParent+GetUIObjectOf, runs SHDoDragDrop, frees PIDL.
    unsafe {
        let mut full_pidl: *mut ITEMIDLIST = ptr::null_mut();
        if SHParseDisplayName(PCWSTR(wide.as_ptr()), None, &mut full_pidl, 0, None).is_err()
            || full_pidl.is_null()
        {
            return false;
        }

        let mut child_pidl: *mut ITEMIDLIST = ptr::null_mut();
        let folder_res: windows::core::Result<IShellFolder> =
            SHBindToParent(full_pidl, Some(&mut child_pidl));
        let data_obj: Option<IDataObject> = folder_res.ok().and_then(|folder| {
            if child_pidl.is_null() {
                None
            } else {
                let children = [child_pidl as *const ITEMIDLIST];
                folder.GetUIObjectOf(hwnd, &children, None).ok()
            }
        });

        ILFree(Some(full_pidl as *const ITEMIDLIST));

        if let Some(obj) = data_obj {
            let allowed = DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK;
            let _ = SHDoDragDrop(hwnd, &obj, None::<&IDropSource>, allowed);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_utf16_bounded_handles_missing_terminator() {
        let units = [b'H' as u16, b'i' as u16, b'!' as u16];
        // Odd byte size or missing NUL terminator never reads out of bounds
        let text = unsafe { extract_utf16_bounded(units.as_ptr(), 7) };
        assert_eq!(text, "Hi!");

        let empty = unsafe { extract_utf16_bounded(ptr::null(), 1) };
        assert!(empty.is_empty());
    }

    #[test]
    fn test_clipboard_writers_reject_null_owner_hwnd() {
        let null_hwnd = HWND(ptr::null_mut());
        assert!(!write_clipboard_text(null_hwnd, "hello"));
        assert!(!write_clipboard_files(
            null_hwnd,
            &[PathBuf::from("Cargo.toml")]
        ));
    }
}
