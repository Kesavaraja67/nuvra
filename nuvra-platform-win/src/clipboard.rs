// nuvra-platform-win/src/clipboard.rs
// Win32 Clipboard read/write, OLE initialization, and native file drag-out (CF_HDROP / SHDoDragDrop).
// Every unsafe block carries a // SAFETY: comment.

use std::path::PathBuf;
use std::ptr;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, HANDLE, HGLOBAL, HWND, POINT};
use windows::Win32::System::Com::{CoInitializeEx, IDataObject, COINIT_MULTITHREADED};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
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

/// Read UTF-16 Unicode text from the Windows clipboard if available.
pub fn read_clipboard_text() -> Option<String> {
    // SAFETY: OpenClipboard -> GetClipboardData -> GlobalLock/Unlock -> CloseClipboard.
    unsafe {
        if OpenClipboard(HWND(ptr::null_mut())).is_err() {
            return None;
        }
        let result = (|| {
            let handle = GetClipboardData(CF_UNICODEXT_FMT).ok()?;
            let hglobal = HGLOBAL(handle.0);
            let ptr_void = GlobalLock(hglobal);
            if ptr_void.is_null() {
                return None;
            }
            let wide_ptr = ptr_void as *const u16;
            let mut len = 0usize;
            while *wide_ptr.add(len) != 0 && len < 65_536 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(wide_ptr, len);
            let text = String::from_utf16_lossy(slice);
            let _ = GlobalUnlock(hglobal);
            Some(text)
        })();
        let _ = CloseClipboard();
        result
    }
}

/// Write UTF-16 Unicode text onto the Windows clipboard.
pub fn write_clipboard_text(text: &str) -> bool {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    let byte_len = wide.len() * std::mem::size_of::<u16>();

    // SAFETY: Allocates movable global buffer, copies null-terminated UTF-16 string, transfers ownership to clipboard.
    unsafe {
        let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, byte_len) else {
            return false;
        };
        let ptr_void = GlobalLock(hmem);
        if ptr_void.is_null() {
            return false;
        }
        ptr::copy_nonoverlapping(wide.as_ptr(), ptr_void as *mut u16, wide.len());
        let _ = GlobalUnlock(hmem);

        if OpenClipboard(HWND(ptr::null_mut())).is_err() {
            return false;
        }
        let _ = EmptyClipboard();
        let set_ok = SetClipboardData(CF_UNICODEXT_FMT, HANDLE(hmem.0)).is_ok();
        let _ = CloseClipboard();
        set_ok
    }
}

/// Copy one or more staged file paths to the Windows clipboard as CF_HDROP (pastable in Explorer/apps).
pub fn write_clipboard_files(paths: &[PathBuf]) -> bool {
    if paths.is_empty() {
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

    // SAFETY: Allocates movable HGLOBAL containing DROPFILES header + double-null UTF-16 paths and passes to SetClipboardData.
    unsafe {
        let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, total_bytes) else {
            return false;
        };
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

        if OpenClipboard(HWND(ptr::null_mut())).is_err() {
            return false;
        }
        let _ = EmptyClipboard();
        let ok = SetClipboardData(CF_HDROP_FMT, HANDLE(hmem.0)).is_ok();
        let _ = CloseClipboard();
        ok
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
