use std::collections::HashMap;

/// Maps arbitrary string slices to unique `u32` token IDs.
/// Guarantees 100% sound equality checks with zero hash collisions.
pub(crate) struct TokenInterner<'a> {
    map: HashMap<&'a str, u32>,
}

impl<'a> TokenInterner<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Interns a slice of string tokens into a `Vec<u32>`.
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
        let mut interner = TokenInterner::new();
        let tokens = ["apple", "banana", "apple", "cherry"];
        let ids = interner.intern(&tokens);

        assert_eq!(ids.len(), 4);
        assert_eq!(ids[0], ids[2]); // "apple" == "apple"
        assert_ne!(ids[0], ids[1]); // "apple" != "banana"
        assert_ne!(ids[1], ids[3]); // "banana" != "cherry"
    }
}
