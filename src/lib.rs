pub(crate) mod algorithm;
pub mod diff;
pub mod edit;
pub(crate) mod optimize;
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
    let trimmed = optimize::trim::trim_slices(a, b, &mut equals);

    if trimmed.slice_a.is_empty() && trimmed.slice_b.is_empty() {
        return Diff::from_parts(EditScript::default(), a, b);
    }

    let script = algorithm::myers::myers(trimmed.slice_a.len(), trimmed.slice_b.len(), |i, j| {
        equals(&trimmed.slice_a[i], &trimmed.slice_b[j])
    });

    let shifted = algorithm::shift::shift_boundaries(
        trimmed.slice_a,
        trimmed.slice_b,
        script.hunks(),
        &mut equals,
    );

    let mut final_hunks = Vec::with_capacity(shifted.len());
    for hunk in shifted.into_iter() {
        let old_r = hunk.old_range();
        let new_r = hunk.new_range();
        final_hunks.push(Hunk::new(
            old_r.start + trimmed.prefix_len..old_r.end + trimmed.prefix_len,
            new_r.start + trimmed.prefix_len..new_r.end + trimmed.prefix_len,
        ));
    }

    Diff::from_parts(EditScript::new(final_hunks), a, b)
}

/// Computes the diff between two slices of elements implementing `PartialEq`.
pub fn diff_slices<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Diff<'a, T> {
    diff_slices_by(a, b, |x, y| x == y)
}

/// Convenience type alias for line-by-line text diffs.
pub type LineDiff<'a> = Diff<'a, &'a str>;

/// Convenience type alias for raw byte-line diffs (for non-UTF8 binary files).
pub type ByteLineDiff<'a> = Diff<'a, &'a [u8]>;

/// Splits text into lines and computes a diff using collision-free token interning.
pub fn diff_lines<'a>(old: &'a [&'a str], new: &'a [&'a str]) -> LineDiff<'a> {
    let trimmed = optimize::trim::trim_slices(old, new, |x, y| x == y);

    if trimmed.slice_a.is_empty() && trimmed.slice_b.is_empty() {
        return Diff::from_parts(EditScript::default(), old, new);
    }

    let mut interner = optimize::intern::TokenInterner::new();
    let old_tokens = interner.intern(trimmed.slice_a);
    let new_tokens = interner.intern(trimmed.slice_b);

    let script = algorithm::myers::myers(old_tokens.len(), new_tokens.len(), |i, j| {
        old_tokens[i] == new_tokens[j]
    });

    let shifted =
        algorithm::shift::shift_boundaries(&old_tokens, &new_tokens, script.hunks(), |x, y| x == y);

    let mut final_hunks = Vec::with_capacity(shifted.len());
    for hunk in shifted.into_iter() {
        let old_r = hunk.old_range();
        let new_r = hunk.new_range();
        final_hunks.push(Hunk::new(
            old_r.start + trimmed.prefix_len..old_r.end + trimmed.prefix_len,
            new_r.start + trimmed.prefix_len..new_r.end + trimmed.prefix_len,
        ));
    }

    Diff::from_parts(EditScript::new(final_hunks), old, new)
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

    #[test]
    fn test_diff_lines_boundary_shifting() {
        let old = ["x", "a", "a", "a", "y"];
        let new = ["x", "a", "a", "y"];
        let diff_l = diff_lines(&old, &new);
        let diff_s = diff_slices(&old, &new);

        // diff_lines and diff_slices must produce identical shifted hunk boundaries
        assert_eq!(diff_l.hunks(), diff_s.hunks());
        assert_eq!(diff_l.hunks().len(), 1);
        assert_eq!(diff_l.hunks()[0].old_range(), 3..4);
    }
}
