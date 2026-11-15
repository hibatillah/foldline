use windows::Win32::Security::Cryptography::{BCRYPT_SHA256_ALG_HANDLE, BCryptHash};

/// SHA-256 of `bytes`; `None` only when Windows CNG fails.
pub fn sha256(bytes: &[u8]) -> Option<[u8; 32]> {
    let mut digest = [0u8; 32];
    // SAFETY: both buffers are live slices for the duration of the call.
    unsafe { BCryptHash(BCRYPT_SHA256_ALG_HANDLE, None, bytes, &mut digest).ok().ok()? };
    Some(digest)
}

/// Lowercase hexadecimal text of `bytes`.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_abc_matches_the_standard_vector() {
        let digest = sha256(b"abc").expect("cng works");
        assert_eq!(to_hex(&digest), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
