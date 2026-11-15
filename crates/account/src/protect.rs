use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData, CryptUnprotectData};
use windows::core::PCWSTR;

/// Encrypts for the current Windows user; `None` when DPAPI refuses.
pub fn protect(plain: &[u8]) -> Option<Vec<u8>> {
    let input = blob_over(plain)?;
    let mut output = CRYPT_INTEGER_BLOB::default();
    // SAFETY: `input` points into `plain`, which outlives the call; `output` receives a LocalAlloc buffer.
    unsafe { CryptProtectData(&input, PCWSTR::null(), None, None, None, 0, &mut output).ok()? };
    Some(take_output(output))
}

/// Decrypts what `protect` made for this user; `None` for anything else.
pub fn unprotect(protected: &[u8]) -> Option<Vec<u8>> {
    let input = blob_over(protected)?;
    let mut output = CRYPT_INTEGER_BLOB::default();
    // SAFETY: `input` points into `protected`, which outlives the call; `output` receives a LocalAlloc buffer.
    unsafe { CryptUnprotectData(&input, None, None, None, None, 0, &mut output).ok()? };
    Some(take_output(output))
}

fn blob_over(bytes: &[u8]) -> Option<CRYPT_INTEGER_BLOB> {
    Some(CRYPT_INTEGER_BLOB { cbData: u32::try_from(bytes.len()).ok()?, pbData: bytes.as_ptr() as *mut u8 })
}

/// Copies the DPAPI output and frees it.
fn take_output(output: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    if output.pbData.is_null() {
        return Vec::new();
    }
    // SAFETY: DPAPI returned `cbData` valid bytes at `pbData`.
    let copied = unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    // SAFETY: the buffer was allocated by DPAPI with LocalAlloc and is freed once.
    unsafe { LocalFree(Some(HLOCAL(output.pbData.cast()))) };
    copied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_bytes_round_trip_and_differ_from_the_plain_bytes() {
        let protected = protect(b"secret value").expect("dpapi works");
        assert!(!protected.windows(12).any(|window| window == b"secret value"));
        assert_eq!(unprotect(&protected).as_deref(), Some(&b"secret value"[..]));
    }

    #[test]
    fn garbage_does_not_unprotect() {
        assert_eq!(unprotect(b"not a dpapi blob"), None);
    }
}
