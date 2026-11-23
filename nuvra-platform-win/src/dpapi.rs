// nuvra-platform-win/src/dpapi.rs
// DPAPI-based key sealing / unsealing for SQLCipher key (D-DAT-02, ADR-0004).
// ONLY file in the project permitted unsafe. Every block carries a SAFETY comment.
//
// UNVERIFIED — requires owner to run on a real Windows machine.
// Manual test: docs/owner-test-scripts/phase-0-test-script.md § DPAPI

use std::ptr;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};
use zeroize::Zeroize;

/// Seal `plaintext` using DPAPI (current user scope).
/// Returns the encrypted blob. Caller-scoped; no key material lingers in this crate.
/// R-20: `plaintext` must never be a log entry; caller is responsible.
pub fn seal_key(plaintext: &[u8]) -> Result<Vec<u8>, SealError> {
    if plaintext.is_empty() {
        return Err(SealError::EmptyInput);
    }

    // We pass no optional entropy, description, or prompt struct (D-DAT-02 specifies
    // the simplest secure configuration: current-user scope only).
    let data_in = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(plaintext.len()).map_err(|_| SealError::TooLarge)?,
        // SAFETY: `plaintext` lives for the duration of this call and is at least 1 byte.
        pbData: plaintext.as_ptr() as *mut u8,
    };
    let mut data_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    // SAFETY: CryptProtectData reads `data_in.pbData[0..cbData]` and writes a
    // heap-allocated blob into `data_out`. We immediately copy that blob into a
    // Vec and release it via LocalFree. All pointers are valid for the duration
    // of the call. The function is documented as thread-safe.
    let res = unsafe {
        CryptProtectData(
            &data_in,
            None, // description
            None, // optional entropy
            None, // reserved
            None, // prompt struct
            0,    // 0 = current user
            &mut data_out,
        )
    };

    if res.is_err() || data_out.pbData.is_null() {
        return Err(SealError::Win32(unsafe {
            windows::Win32::Foundation::GetLastError().0
        }));
    }

    // SAFETY: `data_out.pbData` was allocated by CryptProtectData via LocalAlloc.
    // cbData accurately reflects the allocation size. We copy before freeing.
    let ciphertext =
        unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec() };

    // SAFETY: `data_out.pbData` is a valid HLOCAL from LocalAlloc.
    unsafe {
        let _ = LocalFree(HLOCAL(data_out.pbData as *mut _));
    }

    Ok(ciphertext)
}

/// Unseal a blob previously produced by `seal_key`.
/// Recovers plaintext only if called in the same user session on the same machine.
pub fn unseal_key(ciphertext: &[u8]) -> Result<Vec<u8>, SealError> {
    if ciphertext.is_empty() {
        return Err(SealError::EmptyInput);
    }

    let data_in = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(ciphertext.len()).map_err(|_| SealError::TooLarge)?,
        // SAFETY: `ciphertext` lives for the duration of this call.
        pbData: ciphertext.as_ptr() as *mut u8,
    };
    let mut data_out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };

    // SAFETY: Same contract as CryptProtectData above.
    let res = unsafe {
        CryptUnprotectData(
            &data_in,
            None, // description out
            None, // optional entropy
            None, // reserved
            None, // prompt struct
            0,    // flags
            &mut data_out,
        )
    };

    if res.is_err() || data_out.pbData.is_null() {
        return Err(SealError::Win32(unsafe {
            windows::Win32::Foundation::GetLastError().0
        }));
    }

    // SAFETY: same as seal_key — copy then free.
    let plaintext =
        unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec() };

    // SAFETY: Zeroize decrypted memory buffer using Zeroize trait before freeing to protect plaintext key.
    // LocalFree on DPAPI-allocated pointer.
    unsafe {
        let slice = std::slice::from_raw_parts_mut(data_out.pbData, data_out.cbData as usize);
        slice.zeroize();
        let _ = LocalFree(HLOCAL(data_out.pbData as *mut _));
    }

    Ok(plaintext)
}

#[derive(Debug)]
pub enum SealError {
    EmptyInput,
    TooLarge,
    /// Win32 error code from GetLastError.
    Win32(u32),
}

impl std::fmt::Display for SealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SealError::EmptyInput => write!(f, "seal_key: empty input"),
            SealError::TooLarge => write!(f, "seal_key: input exceeds u32::MAX bytes"),
            SealError::Win32(code) => write!(f, "DPAPI Win32 error: 0x{:08X}", code),
        }
    }
}

impl std::error::Error for SealError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpapi_roundtrip() {
        let input = b"nuvra-dpapi-test-payload-32bytes";
        let sealed = seal_key(input).expect("seal_key failed");
        assert_ne!(sealed, input);
        let unsealed = unseal_key(&sealed).expect("unseal_key failed");
        assert_eq!(unsealed, input);
    }

    #[test]
    fn test_dpapi_empty() {
        assert!(matches!(seal_key(b""), Err(SealError::EmptyInput)));
        assert!(matches!(unseal_key(b""), Err(SealError::EmptyInput)));
    }
}
