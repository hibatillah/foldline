const TEST_KEY_ID: &str = "test-1";
const TEST_PUBLIC_KEY_HEX: &str = include_str!("samples/test-1.public.hex");

/// Production keys by id; none is issued yet.
const PRODUCTION_KEYS: &[(&str, [u8; 64])] = &[];

/// Raw 64-byte X||Y of the key with this id; the test key exists in debug builds only.
pub fn public_key(kid: &str) -> Option<[u8; 64]> {
    if let Some((_, key)) = PRODUCTION_KEYS.iter().find(|(id, _)| *id == kid) {
        return Some(*key);
    }
    if cfg!(debug_assertions) && kid == TEST_KEY_ID {
        return decode_hex_key(TEST_PUBLIC_KEY_HEX.trim());
    }
    None
}

/// Signatures are enforced in debug builds and once a production key ships.
pub fn is_enforced() -> bool {
    is_enforced_for(cfg!(debug_assertions), PRODUCTION_KEYS.len())
}

pub fn is_enforced_for(is_debug: bool, production_key_count: usize) -> bool {
    is_debug || production_key_count > 0
}

fn decode_hex_key(hex: &str) -> Option<[u8; 64]> {
    if hex.len() != 128 || !hex.is_ascii() {
        return None;
    }
    let mut key = [0u8; 64];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_without_a_production_key_is_not_enforced() {
        assert!(!is_enforced_for(false, 0));
        assert!(is_enforced_for(false, 1));
        assert!(is_enforced_for(true, 0));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn the_test_key_is_known_in_debug_builds_only_by_its_id() {
        assert!(public_key("test-1").is_some());
        assert!(public_key("other-key").is_none());
    }
}
