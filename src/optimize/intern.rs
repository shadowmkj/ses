use rustc_hash::{FxBuildHasher, FxHashMap};

/// Maps arbitrary byte sequences to unique `u32` token IDs.
///
/// Uses [`FxHashMap`] for fast, non-cryptographic hashing of short byte slices
/// (such as source code lines), avoiding the latency of standard cryptographic hashing
/// while guaranteeing sound equality checks without hash collisions.
pub(crate) struct TokenInterner<'a> {
    map: FxHashMap<&'a [u8], u32>,
}

impl<'a> TokenInterner<'a> {
    /// Creates a new `TokenInterner` with pre-allocated capacity for `capacity` unique tokens,
    /// eliminating dynamic table reallocations and rehashing.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
        }
    }

    /// Interns a slice of tokens implementing `AsRef<[u8]>` into a `Vec<u32>`, mapping identical
    /// tokens to matching incremental `u32` IDs.
    pub fn intern<T: AsRef<[u8]>>(&mut self, tokens: &'a [T]) -> Vec<u32> {
        let mut ids = Vec::with_capacity(tokens.len());
        for tok in tokens {
            let bytes: &'a [u8] = tok.as_ref();
            let next_id = self.map.len() as u32;
            let id = *self.map.entry(bytes).or_insert(next_id);
            ids.push(id);
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interner() {
        let tokens = ["apple", "banana", "apple", "cherry"];
        let mut interner = TokenInterner::with_capacity(tokens.len());
        let ids = interner.intern(&tokens);

        assert_eq!(ids.len(), 4);
        assert_eq!(ids[0], ids[2]); // "apple" == "apple"
        assert_ne!(ids[0], ids[1]); // "apple" != "banana"
        assert_ne!(ids[1], ids[3]); // "banana" != "cherry"
    }

    #[test]
    fn test_interner_raw_bytes() {
        let tokens: [&[u8]; 4] = [b"apple", b"banana", b"apple", b"cherry"];
        let mut interner = TokenInterner::with_capacity(tokens.len());
        let ids = interner.intern(&tokens);

        assert_eq!(ids.len(), 4);
        assert_eq!(ids[0], ids[2]); // b"apple" == b"apple"
        assert_ne!(ids[0], ids[1]); // b"apple" != b"banana"
        assert_ne!(ids[1], ids[3]); // b"banana" != b"cherry"
    }
}
