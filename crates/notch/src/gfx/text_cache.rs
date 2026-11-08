use super::fonts::StyleKey;

/// What makes two text results interchangeable besides the text itself.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct TextKey {
    pub style: StyleKey,
    pub tabular: bool,
    pub width: u32,
    pub height: u32,
}

/// Remembers the last results per (style, text, size) and overwrites the oldest entry when full. A lookup allocates
/// nothing; an insert reuses the evicted entry's string buffer.
pub(super) struct TextCache<V> {
    entries: Vec<(TextKey, String, V)>,
    next_evicted: usize,
}

impl<V: Clone> TextCache<V> {
    /// Enough for every tabular text of a busy page (a month of day numbers) while keeping memory flat.
    pub const CAPACITY: usize = 64;

    pub fn new() -> Self {
        Self { entries: Vec::with_capacity(Self::CAPACITY), next_evicted: 0 }
    }

    pub fn get(&self, key: &TextKey, text: &str) -> Option<V> {
        self.entries
            .iter()
            .find(|(entry_key, entry_text, _)| entry_key == key && entry_text == text)
            .map(|entry| entry.2.clone())
    }

    pub fn insert(&mut self, key: TextKey, text: &str, value: V) {
        if self.entries.len() < Self::CAPACITY {
            self.entries.push((key, text.to_owned(), value));
            return;
        }
        let entry = &mut self.entries[self.next_evicted];
        entry.0 = key;
        entry.1.clear();
        entry.1.push_str(text);
        entry.2 = value;
        self.next_evicted = (self.next_evicted + 1) % Self::CAPACITY;
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::{Align, Weight};

    fn key(width: u32) -> TextKey {
        TextKey {
            style: ("Geist", 14.0_f32.to_bits(), Weight::Regular, Align::Leading, false),
            tabular: false,
            width,
            height: 0,
        }
    }

    #[test]
    fn a_stored_result_is_found_by_text_and_size_only() {
        let mut cache = TextCache::new();
        cache.insert(key(10), "12:30", 1.5_f32);
        assert_eq!(cache.get(&key(10), "12:30"), Some(1.5));
        assert_eq!(cache.get(&key(11), "12:30"), None);
        assert_eq!(cache.get(&key(10), "12:31"), None);
    }

    #[test]
    fn the_cache_never_grows_past_its_capacity_and_drops_the_oldest_first() {
        let mut cache = TextCache::new();
        for index in 0..TextCache::<f32>::CAPACITY + 5 {
            cache.insert(key(0), &index.to_string(), index as f32);
        }
        assert_eq!(cache.len(), TextCache::<f32>::CAPACITY);
        assert_eq!(cache.get(&key(0), "0"), None);
        assert_eq!(cache.get(&key(0), "4"), None);
        assert_eq!(cache.get(&key(0), "5"), Some(5.0));
        assert_eq!(cache.get(&key(0), "68"), Some(68.0));
    }
}
