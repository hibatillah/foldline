use windows::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};

/// Random bytes from the system generator; `None` only when Windows CNG fails.
pub fn random_bytes<const LENGTH: usize>() -> Option<[u8; LENGTH]> {
    let mut bytes = [0u8; LENGTH];
    // SAFETY: `bytes` is a live buffer and the system-preferred generator needs no algorithm handle.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG).ok().ok()? };
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_draws_of_32_bytes_differ() {
        let first = random_bytes::<32>().expect("cng works");
        let second = random_bytes::<32>().expect("cng works");
        assert_ne!(first, second);
    }
}
