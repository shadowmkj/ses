use crate::edit::EditScript;

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

    /// Iterates lazily over individual token changes (Equal, Delete, Insert).
    pub fn iter_changes<'s>(&'s self) -> crate::views::change::ChangesIter<'s, 'a, T> {
        crate::views::change::ChangesIter::new(&self.script, self.old, self.new)
    }

    /// Iterates lazily over contiguous chunk slices (Equal, Delete, Insert).
    pub fn iter_chunks<'s>(&'s self) -> crate::views::chunk::ChunksIter<'s, 'a, T> {
        crate::views::chunk::ChunksIter::new(&self.script, self.old, self.new)
    }
}
