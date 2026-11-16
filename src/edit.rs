use std::ops::Range;

/// Represents a single replacement edit (modification, deletion, or insertion)
/// between sequence A (original) and sequence B (target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    old_range: Range<usize>,
    new_range: Range<usize>,
}

impl Hunk {
    /// Internal constructor so external callers cannot fabricate arbitrary/invalid hunks.
    pub(crate) fn new(old_range: Range<usize>, new_range: Range<usize>) -> Self {
        Self {
            old_range,
            new_range,
        }
    }

    /// The half-open range `start..end` of indices removed from sequence A.
    #[inline]
    pub fn old_range(&self) -> Range<usize> {
        self.old_range.clone()
    }

    /// The half-open range `start..end` of indices inserted from sequence B.
    #[inline]
    pub fn new_range(&self) -> Range<usize> {
        self.new_range.clone()
    }

    /// Number of elements removed from sequence A.
    #[inline]
    pub fn old_len(&self) -> usize {
        self.old_range.len()
    }

    /// Number of elements inserted from sequence B.
    #[inline]
    pub fn new_len(&self) -> usize {
        self.new_range.len()
    }

    /// Returns `true` if this hunk is a pure insertion (no elements deleted from A).
    #[inline]
    pub fn is_insertion(&self) -> bool {
        self.old_range.is_empty()
    }

    /// Returns `true` if this hunk is a pure deletion (no elements inserted from B).
    #[inline]
    pub fn is_deletion(&self) -> bool {
        self.new_range.is_empty()
    }
}

/// An encapsulated sequence of non-overlapping, contiguous edit `Hunk`s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EditScript {
    hunks: Vec<Hunk>,
}

impl EditScript {
    /// Constructs an `EditScript` wrapping the provided list of hunks.
    pub(crate) fn new(hunks: Vec<Hunk>) -> Self {
        Self { hunks }
    }

    /// Borrows the slice of hunks.
    pub fn hunks(&self) -> &[Hunk] {
        &self.hunks
    }

    /// Iterates over hunks by reference.
    pub fn iter(&self) -> std::slice::Iter<'_, Hunk> {
        self.hunks.iter()
    }

    /// Returns `true` if there are no hunks (i.e. sequences are identical).
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }

    /// Returns the number of edit hunks.
    pub fn len(&self) -> usize {
        self.hunks.len()
    }
}

impl IntoIterator for EditScript {
    type Item = Hunk;
    type IntoIter = std::vec::IntoIter<Hunk>;

    fn into_iter(self) -> Self::IntoIter {
        self.hunks.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_edit_script() {
        let script = EditScript::new(vec![]);
        assert!(script.is_empty());
        assert_eq!(script.len(), 0);
        assert_eq!(script.hunks().len(), 0);
    }

    #[test]
    fn test_hunk_ranges_and_accessors() {
        let hunk = Hunk::new(0..2, 0..1);
        assert_eq!(hunk.old_range(), 0..2);
        assert_eq!(hunk.new_range(), 0..1);
        assert_eq!(hunk.old_len(), 2);
        assert_eq!(hunk.new_len(), 1);
        assert!(!hunk.is_insertion());
        assert!(!hunk.is_deletion());

        let ins = Hunk::new(2..2, 2..4);
        assert!(ins.is_insertion());
        assert!(!ins.is_deletion());

        let del = Hunk::new(5..7, 4..4);
        assert!(!del.is_insertion());
        assert!(del.is_deletion());

        let script = EditScript::new(vec![hunk.clone()]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0], hunk);

        // Test IntoIterator
        let collected: Vec<_> = script.into_iter().collect();
        assert_eq!(collected.len(), 1);
    }
}
