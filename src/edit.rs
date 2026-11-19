use std::ops::Range;

/// Represents a single replacement edit (modification, deletion, or insertion)
/// between sequence A (original) and sequence B (target).
///
/// A `Hunk` is represented as a pair of half-open index ranges:
/// - [`old_range`](Hunk::old_range): The indices `start..end` removed from sequence A.
/// - [`new_range`](Hunk::new_range): The indices `start..end` inserted from sequence B.
///
/// # Examples
///
/// ```rust
/// use ses::diff_slices;
///
/// let old = ["apple", "banana", "cherry"];
/// let new = ["apple", "blueberry", "cherry"];
/// let diff = diff_slices(&old, &new);
///
/// let hunk = &diff.hunks()[0];
/// assert_eq!(hunk.old_range(), 1..2); // "banana" was removed
/// assert_eq!(hunk.new_range(), 1..2); // "blueberry" was added
/// assert_eq!(hunk.old_len(), 1);
/// assert_eq!(hunk.new_len(), 1);
/// assert!(!hunk.is_insertion());
/// assert!(!hunk.is_deletion());
/// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a", "b"], &["a", "c"]);
    /// assert_eq!(diff.hunks()[0].old_range(), 1..2);
    /// ```
    #[inline]
    pub fn old_range(&self) -> Range<usize> {
        self.old_range.clone()
    }

    /// The half-open range `start..end` of indices inserted from sequence B.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a", "b"], &["a", "c"]);
    /// assert_eq!(diff.hunks()[0].new_range(), 1..2);
    /// ```
    #[inline]
    pub fn new_range(&self) -> Range<usize> {
        self.new_range.clone()
    }

    /// Number of elements removed from sequence A.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a", "b", "c"], &["a"]);
    /// assert_eq!(diff.hunks()[0].old_len(), 2);
    /// ```
    #[inline]
    pub fn old_len(&self) -> usize {
        self.old_range.len()
    }

    /// Number of elements inserted from sequence B.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a"], &["a", "b", "c"]);
    /// assert_eq!(diff.hunks()[0].new_len(), 2);
    /// ```
    #[inline]
    pub fn new_len(&self) -> usize {
        self.new_range.len()
    }

    /// Returns `true` if this hunk is a pure insertion (no elements deleted from A).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a", "c"], &["a", "b", "c"]);
    /// assert!(diff.hunks()[0].is_insertion());
    /// ```
    #[inline]
    pub fn is_insertion(&self) -> bool {
        self.old_range.is_empty()
    }

    /// Returns `true` if this hunk is a pure deletion (no elements inserted from B).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&["a", "b", "c"], &["a", "c"]);
    /// assert!(diff.hunks()[0].is_deletion());
    /// ```
    #[inline]
    pub fn is_deletion(&self) -> bool {
        self.new_range.is_empty()
    }
}

/// An encapsulated sequence of non-overlapping, contiguous edit `Hunk`s.
///
/// An `EditScript` represents the minimal sequence of edits needed to transform
/// sequence A into sequence B. It is created internally by diff functions such
/// as [`diff_slices`](crate::diff_slices).
///
/// # Examples
///
/// ```rust
/// use ses::diff_slices;
///
/// let diff = diff_slices(&[1, 2, 3], &[1, 9, 3]);
/// let script = diff.script();
///
/// assert_eq!(script.len(), 1);
/// assert!(!script.is_empty());
///
/// for hunk in script {
///     println!("Hunk: {:?}", hunk);
/// }
/// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 3]);
    /// assert_eq!(diff.script().hunks().len(), 1);
    /// ```
    pub fn hunks(&self) -> &[Hunk] {
        &self.hunks
    }

    /// Iterates over hunks by reference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 3]);
    /// for hunk in diff.script().iter() {
    ///     assert_eq!(hunk.old_range(), 1..2);
    /// }
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, Hunk> {
        self.hunks.iter()
    }

    /// Returns `true` if there are no hunks (i.e. sequences are identical).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 2]);
    /// assert!(diff.script().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }

    /// Returns the number of edit hunks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 3]);
    /// assert_eq!(diff.script().len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.hunks.len()
    }
}

impl IntoIterator for EditScript {
    type Item = Hunk;
    type IntoIter = std::vec::IntoIter<Hunk>;

    /// Consumes the `EditScript`, yielding `Hunk`s by value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 3]);
    /// let script = diff.script().clone();
    /// for hunk in script {
    ///     assert_eq!(hunk.old_range(), 1..2);
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.hunks.into_iter()
    }
}

impl<'a> IntoIterator for &'a EditScript {
    type Item = &'a Hunk;
    type IntoIter = std::slice::Iter<'a, Hunk>;

    /// Iterates over `EditScript` hunks by reference.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ses::diff_slices;
    ///
    /// let diff = diff_slices(&[1, 2], &[1, 3]);
    /// for hunk in diff.script() {
    ///     assert_eq!(hunk.old_range(), 1..2);
    /// }
    /// ```
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hunks.iter()
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

        // Test IntoIterator by reference and by value
        let ref_collected: Vec<_> = (&script).into_iter().collect();
        assert_eq!(ref_collected.len(), 1);
        assert_eq!(ref_collected[0], &hunk);

        let collected: Vec<_> = script.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], hunk);
    }
}
