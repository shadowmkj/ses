use crate::edit::{EditScript, Hunk};

/// The universal container pairing an `EditScript` with original borrowed inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff<'a, T> {
    pub(crate) script: EditScript,
    pub(crate) old: &'a [T],
    pub(crate) new: &'a [T],
}

impl<'a, T> Diff<'a, T> {
    /// Constructs a `Diff` container wrapping an edit script and the original sequences.
    pub fn from_parts(script: EditScript, old: &'a [T], new: &'a [T]) -> Self {
        Self { script, old, new }
    }

    /// Borrows the computed edit script.
    #[inline]
    pub fn script(&self) -> &EditScript {
        &self.script
    }

    /// Borrows the original sequence A.
    #[inline]
    pub fn old(&self) -> &'a [T] {
        self.old
    }

    /// Borrows the target sequence B.
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    pub fn new(&self) -> &'a [T] {
        self.new
    }

    /// Alias for [`Diff::new`], borrowing the target sequence B.
    #[inline]
    pub fn new_slice(&self) -> &'a [T] {
        self.new
    }

    /// Borrows the slice of computed edit hunks.
    #[inline]
    pub fn hunks(&self) -> &[Hunk] {
        self.script.hunks()
    }

    /// Returns `true` if the diff contains no changes (both sequences are identical).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.script.is_empty()
    }

    /// Returns the number of edit hunks.
    #[inline]
    pub fn len(&self) -> usize {
        self.script.len()
    }

    /// Iterates lazily over individual token changes (Equal, Delete, Insert).
    pub fn iter_changes<'s>(&'s self) -> crate::views::change::ChangesIter<'s, 'a, T> {
        crate::views::change::ChangesIter::new(&self.script, self.old, self.new)
    }

    /// Iterates lazily over contiguous chunk slices (Equal, Delete, Insert).
    pub fn iter_chunks<'s>(&'s self) -> crate::views::chunk::ChunksIter<'s, 'a, T> {
        crate::views::chunk::ChunksIter::new(&self.script, self.old, self.new)
    }
}

impl<'s, 'a, T> IntoIterator for &'s Diff<'a, T> {
    type Item = &'s Hunk;
    type IntoIter = std::slice::Iter<'s, Hunk>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hunks().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_forwarding_methods() {
        let old = [1, 2, 3];
        let new = [1, 9, 3];
        let empty_diff = Diff::from_parts(EditScript::default(), &old, &old);
        assert!(empty_diff.is_empty());
        assert_eq!(empty_diff.len(), 0);
        assert_eq!(empty_diff.hunks(), &[]);

        let hunk = Hunk::new(1..2, 1..2);
        let script = EditScript::new(vec![hunk.clone()]);
        let diff = Diff::from_parts(script, &old, &new);

        assert!(!diff.is_empty());
        assert_eq!(diff.len(), 1);
        assert_eq!(diff.hunks(), &[hunk]);
    }

    #[test]
    fn test_diff_into_iterator() {
        let old = [1, 2, 3];
        let new = [1, 9, 3];
        let hunk = Hunk::new(1..2, 1..2);
        let script = EditScript::new(vec![hunk.clone()]);
        let diff = Diff::from_parts(script, &old, &new);

        // Test iterating over diff.script()
        let mut count = 0;
        for h in diff.script() {
            assert_eq!(h, &hunk);
            count += 1;
        }
        assert_eq!(count, 1);

        // Test iterating over &diff
        let mut count2 = 0;
        for h in &diff {
            assert_eq!(h, &hunk);
            count2 += 1;
        }
        assert_eq!(count2, 1);
    }
}
