const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Unpadded URL-safe base64; any character outside the alphabet (including `=`) refuses the text.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    // One leftover character cannot hold a whole byte.
    if bytes.len() % 4 == 1 {
        return None;
    }
    let mut decoded = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut accumulator: u32 = 0;
    let mut bit_count = 0;
    for &character in bytes {
        let value = ALPHABET.iter().position(|&candidate| candidate == character)? as u32;
        accumulator = (accumulator << 6) | value;
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            decoded.push((accumulator >> bit_count) as u8);
            accumulator &= (1 << bit_count) - 1;
        }
    }
    // Leftover bits must be zero, or two texts would decode to the same bytes.
    if accumulator != 0 {
        return None;
    }
    Some(decoded)
}

pub fn encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let block = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        let character_count = chunk.len() + 1;
        for index in 0..character_count {
            encoded.push(ALPHABET[((block >> (18 - 6 * index)) & 63) as usize] as char);
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_round_trips_every_length_from_0_to_5() {
        for length in 0..=5u8 {
            let bytes: Vec<u8> = (0..length).map(|index| 250u8.wrapping_add(index.wrapping_mul(3))).collect();
            let encoded = encode(&bytes);
            assert!(!encoded.contains('='));
            assert_eq!(decode(&encoded), Some(bytes));
        }
    }

    #[test]
    fn url_safe_characters_replace_plus_and_slash() {
        assert_eq!(encode(&[0xfb, 0xff]), "-_8");
        assert_eq!(decode("+/8"), None);
    }

    #[test]
    fn padding_and_an_impossible_length_are_refused() {
        assert_eq!(decode("YQ=="), None);
        assert_eq!(decode("Y"), None);
    }

    #[test]
    fn non_canonical_trailing_bits_are_refused() {
        assert_eq!(decode("YQ"), Some(b"a".to_vec()));
        assert_eq!(decode("YR"), None);
    }
}
