// nuvra-platform-win/src/network.rs
// Real Windows active network adapter detection (Wi-Fi vs Ethernet vs Offline).
// R-13: Unsafe confined here with // SAFETY: comments.

#![allow(unsafe_code)]

use windows::Win32::NetworkManagement::IpHelper::{
    GetBestInterface, GetIfEntry, GetIfTable, MIB_IFROW, MIB_IFTABLE,
};

const IF_TYPE_ETHERNET_CSMACD: u32 = 6;
const IF_TYPE_IEEE80211: u32 = 71;
const IF_TYPE_GIGABIT_ETHERNET: u32 = 117;
const IF_OPER_OPERATIONAL: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkKind {
    #[default]
    Offline,
    Wifi,
    Ethernet,
}

fn classify_if_type(dw_type: u32) -> Option<NetworkKind> {
    match dw_type {
        IF_TYPE_IEEE80211 => Some(NetworkKind::Wifi),
        IF_TYPE_ETHERNET_CSMACD | IF_TYPE_GIGABIT_ETHERNET => Some(NetworkKind::Ethernet),
        _ => None,
    }
}

/// Query the OS for the active default-route network transport (Wi-Fi, Ethernet, or Offline).
pub fn query_active_network() -> NetworkKind {
    let mut best_idx: u32 = 0;
    let probe_ip = u32::from_be_bytes([8, 8, 8, 8]);
    // SAFETY: Valid output u32 pointer passed to Win32 GetBestInterface routing lookup.
    let rc = unsafe { GetBestInterface(probe_ip, &mut best_idx) };
    if rc == 0 {
        let mut row = MIB_IFROW {
            dwIndex: best_idx,
            ..Default::default()
        };
        // SAFETY: Zeroed MIB_IFROW with valid dwIndex passed to GetIfEntry.
        if unsafe { GetIfEntry(&mut row) } == 0 {
            if let Some(kind) = classify_if_type(row.dwType) {
                return kind;
            }
        }
    }
    fallback_physical_adapter()
}

fn fallback_physical_adapter() -> NetworkKind {
    let mut size: u32 = 0;
    // SAFETY: Query required buffer size for GetIfTable.
    unsafe {
        let _ = GetIfTable(None, &mut size, false);
    }
    if size < std::mem::size_of::<u32>() as u32 {
        return NetworkKind::Offline;
    }
    let mut buf = vec![0u8; size as usize];
    let table_ptr = buf.as_mut_ptr().cast::<MIB_IFTABLE>();
    // SAFETY: Properly sized heap buffer passed to GetIfTable.
    if unsafe { GetIfTable(Some(table_ptr), &mut size, false) } != 0 {
        return NetworkKind::Offline;
    }
    // SAFETY: Valid MIB_IFTABLE populated by GetIfTable; clamp count to buffer bounds.
    let entries = unsafe {
        let num = (*table_ptr).dwNumEntries as usize;
        let first = (*table_ptr).table.as_ptr();
        std::slice::from_raw_parts(first, num)
    };
    let mut saw_wifi = false;
    for row in entries {
        if row.dwOperStatus.0 == IF_OPER_OPERATIONAL {
            match classify_if_type(row.dwType) {
                Some(NetworkKind::Ethernet) => return NetworkKind::Ethernet,
                Some(NetworkKind::Wifi) => saw_wifi = true,
                _ => {}
            }
        }
    }
    if saw_wifi { NetworkKind::Wifi } else { NetworkKind::Offline }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_query_returns_valid_variant() {
        let kind = query_active_network();
        assert!(matches!(kind, NetworkKind::Wifi | NetworkKind::Ethernet | NetworkKind::Offline));
    }
}
