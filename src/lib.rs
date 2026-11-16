pub(crate) mod algorithm;
pub mod diff;
pub mod edit;
pub mod views;

pub use diff::Diff;
pub use edit::{EditScript, Hunk};
pub use views::change::{Change, ChangesIter};
pub use views::chunk::{ChunksIter, DiffChunk};

/// Computes the diff between two slices using custom element equality logic.
pub fn diff_slices_by<'a, T, F>(a: &'a [T], b: &'a [T], mut equals: F) -> Diff<'a, T>
where
    F: FnMut(&T, &T) -> bool,
{
    let script = algorithm::myers::myers(a.len(), b.len(), |i, j| equals(&a[i], &b[j]));

    Diff::from_parts(script, a, b)
}

/// Computes the diff between two slices of elements implementing `PartialEq`.
pub fn diff_slices<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Diff<'a, T> {
    diff_slices_by(a, b, |x, y| x == y)
}

/// Convenience type alias for line-by-line text diffs.
pub type LineDiff<'a> = Diff<'a, &'a str>;

/// Convenience type alias for raw byte-line diffs (for non-UTF8 binary files).
pub type ByteLineDiff<'a> = Diff<'a, &'a [u8]>;

/// Splits text into lines and computes a diff.
pub fn diff_lines<'a>(old: &'a [&'a str], new: &'a [&'a str]) -> LineDiff<'a> {
    diff_slices(old, new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_slices() {
        let a = ["banana", "mango", "carrot", "grapes", "apple"];
        let b = ["apple", "banana", "mango", "grapes", "carrot"];
        let diff = diff_slices(&a, &b);
        assert!(!diff.script().is_empty());
        assert_eq!(diff.old(), &a);
        assert_eq!(diff.new(), &b);
    }

    #[test]
    fn test_diff_slices_by_case_insensitive() {
        let a = ["hello", "WORLD"];
        let b = ["HELLO", "world"];
        let diff = diff_slices_by(&a, &b, |x, y| x.eq_ignore_ascii_case(y));
        assert!(diff.script().is_empty());
    }

    #[test]
    fn test_diff_lines() {
        let old = ["line 1", "line 2", "line 3"];
        let new = ["line 1", "line 2 modified", "line 3"];
        let diff = diff_lines(&old, &new);

        assert_eq!(diff.script().len(), 1);
        assert_eq!(diff.old(), &old);
        assert_eq!(diff.new(), &new);
    }
}
