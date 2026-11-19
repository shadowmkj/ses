use crate::edit::{EditScript, Hunk};

/// The universal container pairing an `EditScript` with original borrowed inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff<'a, T> {
    pub(crate) script: EditScript,
    pub(crate) old: &'a [T],
    pub(crate) new: &'a [T],
}

impl<'a, T> Diff<'a, T> {
    /// Internal constructor so external callers cannot fabricate arbitrary/invalid diff containers.
    #[inline]
    pub(crate) fn from_parts(script: EditScript, old: &'a [T], new: &'a [T]) -> Self {
        debug_assert!(
            Self::is_valid(&script, old.len(), new.len()),
            "invalid edit script: hunks do not match slice dimensions or are non-monotonic"
        );
        Self { script, old, new }
    }

    /// Verifies whether an `EditScript` is mathematically consistent with `old_len` and `new_len`.
    pub(crate) fn is_valid(script: &EditScript, old_len: usize, new_len: usize) -> bool {
        let hunks = script.hunks();

        if hunks.is_empty() {
            return old_len == new_len;
        }

        let mut prev_old_end = 0;
        let mut prev_new_end = 0;

        for hunk in hunks {
            let old_r = hunk.old_range();
            let new_r = hunk.new_range();

            // Valid range bounds within slice dimensions
            if old_r.start > old_r.end || old_r.end > old_len {
                return false;
            }
            if new_r.start > new_r.end || new_r.end > new_len {
                return false;
            }

            // A hunk cannot be empty on both sides
            if old_r.is_empty() && new_r.is_empty() {
                return false;
            }

            // Monotonic non-overlapping ordering
            if old_r.start < prev_old_end || new_r.start < prev_new_end {
                return false;
            }

            // Equal number of unchanged elements between hunks
            if (old_r.start - prev_old_end) != (new_r.start - prev_new_end) {
                return false;
            }

            prev_old_end = old_r.end;
            prev_new_end = new_r.end;
        }

        // After the last hunk: unchanged suffix lengths must match.
        (old_len - prev_old_end) == (new_len - prev_new_end)
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

    #[test]
    fn test_diff_validation_valid_cases() {
        // Empty slices with empty script
        assert!(Diff::<i32>::is_valid(&EditScript::default(), 0, 0));

        // Identical slices with empty script
        assert!(Diff::<i32>::is_valid(&EditScript::default(), 5, 5));

        // Pure insertion
        let ins = Hunk::new(1..1, 1..2);
        let script = EditScript::new(vec![ins]);
        assert!(Diff::<i32>::is_valid(&script, 2, 3));

        // Pure deletion
        let del = Hunk::new(0..1, 0..0);
        let script = EditScript::new(vec![del]);
        assert!(Diff::<i32>::is_valid(&script, 2, 1));

        // Replacement and multiple hunks
        let h1 = Hunk::new(1..2, 1..1); // del at 1
        let h2 = Hunk::new(3..3, 2..3); // ins at 2
        let script = EditScript::new(vec![h1, h2]);
        assert!(Diff::<i32>::is_valid(&script, 4, 4));
    }

    #[test]
    fn test_diff_validation_invalid_cases() {
        // Mismatched slice lengths with empty script
        assert!(!Diff::<i32>::is_valid(&EditScript::default(), 5, 2));

        // Out of bounds hunk
        let out_of_bounds = Hunk::new(0..5, 0..1);
        let script = EditScript::new(vec![out_of_bounds]);
        assert!(!Diff::<i32>::is_valid(&script, 3, 3));

        // Overlapping hunks
        let h1 = Hunk::new(0..2, 0..1);
        let h2 = Hunk::new(1..3, 1..2);
        let script = EditScript::new(vec![h1, h2]);
        assert!(!Diff::<i32>::is_valid(&script, 4, 4));

        // Non-monotonic gap (unequal unchanged elements between hunks)
        let h1 = Hunk::new(0..1, 0..1);
        let h2 = Hunk::new(3..4, 2..3);
        let script = EditScript::new(vec![h1, h2]);
        assert!(!Diff::<i32>::is_valid(&script, 4, 4));

        // Trailing unchanged mismatch
        let h = Hunk::new(0..1, 0..1);
        let script = EditScript::new(vec![h]);
        assert!(!Diff::<i32>::is_valid(&script, 3, 2));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "invalid edit script")]
    fn test_diff_from_parts_panics_on_mismatched_empty_script() {
        let old = [1, 2, 3];
        let new = [1];
        let _ = Diff::from_parts(EditScript::default(), &old, &new);
    }
}
