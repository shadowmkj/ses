use crate::edit::EditScript;

/// Represents an edit operation on an individual element between two sequences.
///
/// Each item yielded during a diff inspection represents one of three states:
/// - [`Change::Equal`]: The element is present and unchanged in both sequences.
/// - [`Change::Delete`]: The element was present in the original sequence but removed.
/// - [`Change::Insert`]: The element was added to the modified sequence.
///
/// Unlike [`DiffChunk`](crate::views::chunk::DiffChunk), which groups contiguous elements
/// into slices (`&'a [T]`), `Change<'a, T>` yields elements one by one.
///
/// # Examples
///
/// ```
/// use ses::{diff_slices, Change};
///
/// let old = ["apple", "banana"];
/// let new = ["banana", "cherry"];
/// let diff = diff_slices(&old, &new);
///
/// for change in diff.iter_changes() {
///     match change {
///         Change::Equal(item) => println!("  {item}"),
///         Change::Delete(item) => println!("- {item}"),
///         Change::Insert(item) => println!("+ {item}"),
///     }
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum Change<'a, T> {
    /// The element is present and unchanged in both sequences.
    Equal(&'a T),

    /// The element was present in the original sequence and was deleted.
    Delete(&'a T),

    /// The element was added to the modified sequence.
    Insert(&'a T),
}

impl<'a, T> Clone for Change<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Change<'a, T> {}

/// A zero-copy lazy iterator yielding `Change<&T>` items on-the-fly.
/// Uses an iterative loop state machine (never recursing) to prevent stack overflows.
pub struct ChangesIter<'s, 'a, T> {
    old: &'a [T],
    new: &'a [T],
    script: &'s EditScript,
    hunk_idx: usize,
    old_idx: usize,
    new_idx: usize,
    insert_idx: usize,
}

impl<'s, 'a, T> ChangesIter<'s, 'a, T> {
    pub fn new(script: &'s EditScript, old: &'a [T], new: &'a [T]) -> Self {
        Self {
            old,
            new,
            script,
            hunk_idx: 0,
            old_idx: 0,
            new_idx: 0,
            insert_idx: 0,
        }
    }
}

impl<'s, 'a, T> Iterator for ChangesIter<'s, 'a, T> {
    type Item = Change<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let hunks = self.script.hunks();

        loop {
            if self.hunk_idx < hunks.len() {
                let hunk = &hunks[self.hunk_idx];
                let old_r = hunk.old_range();
                let new_r = hunk.new_range();

                // Unchanged items before the current hunk
                if self.old_idx < old_r.start {
                    let item = &self.old[self.old_idx];
                    self.old_idx += 1;
                    self.new_idx += 1;
                    return Some(Change::Equal(item));
                }

                // Deletions in current hunk
                if self.old_idx < old_r.end {
                    let item = &self.old[self.old_idx];
                    self.old_idx += 1;
                    return Some(Change::Delete(item));
                }

                // Insertions in current hunk
                let ins_pos = new_r.start + self.insert_idx;
                if ins_pos < new_r.end {
                    let item = &self.new[ins_pos];
                    self.insert_idx += 1;
                    self.new_idx += 1;
                    return Some(Change::Insert(item));
                }

                // Current hunk exhausted: loop to next hunk without recursing
                self.hunk_idx += 1;
                self.insert_idx = 0;
                continue;
            }

            // Trailing unchanged items after the last hunk
            if self.old_idx < self.old.len() {
                let item = &self.old[self.old_idx];
                self.old_idx += 1;
                self.new_idx += 1;
                return Some(Change::Equal(item));
            }

            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff_slices;

    #[test]
    fn test_iter_changes() {
        let a = ["a", "b"];
        let b = ["a", "c"];
        let diff = diff_slices(&a, &b);

        let changes: Vec<_> = diff.iter_changes().collect();
        assert_eq!(
            changes,
            vec![
                Change::Equal(&"a"),
                Change::Delete(&"b"),
                Change::Insert(&"c"),
            ]
        );
    }
}
