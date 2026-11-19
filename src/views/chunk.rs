use crate::edit::EditScript;

/// Represents a contiguous sub-slice of changes (grouping sequential tokens).
///
/// Unlike [`Change`](crate::views::change::Change), which yields elements one by one,
/// `DiffChunk` groups adjacent elements of the same kind into contiguous slices (`&'a [T]`).
/// This enables high-performance block-level rendering and formatted output.
///
/// # Examples
///
/// ```rust
/// use ses::{diff_slices, DiffChunk};
///
/// let old = ["a", "b", "c"];
/// let new = ["a", "x", "y", "c"];
/// let diff = diff_slices(&old, &new);
///
/// for chunk in diff.iter_chunks() {
///     match chunk {
///         DiffChunk::Equal(slice) => println!("  {slice:?}"),
///         DiffChunk::Delete(slice) => println!("- {slice:?}"),
///         DiffChunk::Insert(slice) => println!("+ {slice:?}"),
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffChunk<'a, T> {
    /// A contiguous slice of elements present and unchanged in both sequences.
    Equal(&'a [T]),
    /// A contiguous slice of elements removed from the original sequence.
    Delete(&'a [T]),
    /// A contiguous slice of elements added to the modified sequence.
    Insert(&'a [T]),
}

enum ChunkStage {
    BeforeHunk,
    DeleteHunk,
    InsertHunk,
    AfterHunks,
    Done,
}

/// A zero-copy lazy iterator yielding contiguous [`DiffChunk<'a, T>`] slices.
///
/// Constructed via [`Diff::iter_chunks`](crate::Diff::iter_chunks).
///
/// Uses an iterative loop state machine (never recursing) to prevent stack overflows
/// even on massive diffs.
///
/// # Examples
///
/// ```rust
/// use ses::{diff_slices, DiffChunk};
///
/// let diff = diff_slices(&["a", "b"], &["b", "c"]);
/// let mut iter = diff.iter_chunks();
///
/// assert_eq!(iter.next(), Some(DiffChunk::Delete(&["a"][..])));
/// assert_eq!(iter.next(), Some(DiffChunk::Equal(&["b"][..])));
/// assert_eq!(iter.next(), Some(DiffChunk::Insert(&["c"][..])));
/// assert_eq!(iter.next(), None);
/// ```
pub struct ChunksIter<'s, 'a, T> {
    old: &'a [T],
    new: &'a [T],
    script: &'s EditScript,
    hunk_idx: usize,
    old_cursor: usize,
    stage: ChunkStage,
}

impl<'s, 'a, T> ChunksIter<'s, 'a, T> {
    /// Creates a new `ChunksIter` over the provided script and sequences.
    pub fn new(script: &'s EditScript, old: &'a [T], new: &'a [T]) -> Self {
        Self {
            old,
            new,
            script,
            hunk_idx: 0,
            old_cursor: 0,
            stage: ChunkStage::BeforeHunk,
        }
    }
}

impl<'s, 'a, T> Iterator for ChunksIter<'s, 'a, T> {
    type Item = DiffChunk<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let hunks = self.script.hunks();

        loop {
            match self.stage {
                ChunkStage::BeforeHunk => {
                    if self.hunk_idx < hunks.len() {
                        let hunk = &hunks[self.hunk_idx];
                        let old_r = hunk.old_range();
                        if self.old_cursor < old_r.start {
                            let slice = &self.old[self.old_cursor..old_r.start];
                            self.old_cursor = old_r.start;
                            self.stage = ChunkStage::DeleteHunk;
                            return Some(DiffChunk::Equal(slice));
                        }
                        self.stage = ChunkStage::DeleteHunk;
                    } else {
                        self.stage = ChunkStage::AfterHunks;
                    }
                }
                ChunkStage::DeleteHunk => {
                    let hunk = &hunks[self.hunk_idx];
                    self.stage = ChunkStage::InsertHunk;
                    if !hunk.is_insertion() {
                        let slice = &self.old[hunk.old_range()];
                        self.old_cursor = hunk.old_range().end;
                        return Some(DiffChunk::Delete(slice));
                    }
                }
                ChunkStage::InsertHunk => {
                    let hunk = &hunks[self.hunk_idx];
                    self.hunk_idx += 1;
                    self.stage = ChunkStage::BeforeHunk;
                    if !hunk.is_deletion() {
                        let slice = &self.new[hunk.new_range()];
                        return Some(DiffChunk::Insert(slice));
                    }
                }
                ChunkStage::AfterHunks => {
                    self.stage = ChunkStage::Done;
                    if self.old_cursor < self.old.len() {
                        let slice = &self.old[self.old_cursor..];
                        self.old_cursor = self.old.len();
                        return Some(DiffChunk::Equal(slice));
                    }
                }
                ChunkStage::Done => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff_slices;

    #[test]
    fn test_iter_chunks() {
        let a = ["a", "b", "c", "d"];
        let b = ["a", "x", "y", "d"];
        let diff = diff_slices(&a, &b);

        let chunks: Vec<_> = diff.iter_chunks().collect();
        assert_eq!(
            chunks,
            vec![
                DiffChunk::Equal(&["a"][..]),
                DiffChunk::Delete(&["b", "c"][..]),
                DiffChunk::Insert(&["x", "y"][..]),
                DiffChunk::Equal(&["d"][..]),
            ]
        );
    }

    #[test]
    fn test_iter_chunks_empty() {
        let a: [&str; 0] = [];
        let b: [&str; 0] = [];
        let diff = diff_slices(&a, &b);
        let chunks: Vec<_> = diff.iter_chunks().collect();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_iter_chunks_pure_insertion() {
        let a: [&str; 0] = [];
        let b = ["x", "y"];
        let diff = diff_slices(&a, &b);
        let chunks: Vec<_> = diff.iter_chunks().collect();
        assert_eq!(chunks, vec![DiffChunk::Insert(&["x", "y"][..])]);
    }

    #[test]
    fn test_iter_chunks_pure_deletion() {
        let a = ["x", "y"];
        let b: [&str; 0] = [];
        let diff = diff_slices(&a, &b);
        let chunks: Vec<_> = diff.iter_chunks().collect();
        assert_eq!(chunks, vec![DiffChunk::Delete(&["x", "y"][..])]);
    }

    #[test]
    fn test_iter_chunks_identical() {
        let a = ["x", "y"];
        let b = ["x", "y"];
        let diff = diff_slices(&a, &b);
        let chunks: Vec<_> = diff.iter_chunks().collect();
        assert_eq!(chunks, vec![DiffChunk::Equal(&["x", "y"][..])]);
    }
}
