use rustc_hash::{FxBuildHasher, FxHashMap};

/// Maps arbitrary string slices to unique `u32` token IDs.
///
/// Uses [`FxHashMap`] for fast, non-cryptographic hashing of short string slices
/// (such as source code lines), avoiding the latency of standard cryptographic hashing
/// while guaranteeing sound equality checks without hash collisions.
pub(crate) struct TokenInterner<'a> {
    map: FxHashMap<&'a str, u32>,
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

    /// Interns a slice of string tokens into a `Vec<u32>`, mapping identical tokens
    /// to matching incremental `u32` IDs.
    pub fn intern(&mut self, tokens: &[&'a str]) -> Vec<u32> {
        tokens
            .iter()
            .map(|&tok| {
                let next_id = self.map.len() as u32;
                *self.map.entry(tok).or_insert(next_id)
            })
            .collect()
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
}
