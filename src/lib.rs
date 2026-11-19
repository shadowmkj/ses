//! # `ses`
//!
//! A high-performance, zero-dependency, linear-space Myers diff engine in Rust.
//!
//! `ses` computes the Shortest Edit Script (SES) between two sequences using Eugene Myers'
//! 1986 algorithm (*"An O(ND) Difference Algorithm and Its Variations"*). It features:
//! - **Strictly `O(N + M)` Linear Space:** Bidirectional middle-snake divide-and-conquer
//!   reusing a single preallocated diagonal workspace buffer without call-stack recursion.
//! - **`O(N)` Boundary Pruning:** Strips identical prefixes and suffixes before graph
//!   searches, reducing problem dimensions.
//! - **Semantic Boundary Shifting:** Post-processes edits to slide change boundaries across
//!   identical delimiters, blank lines, or repeated elements—matching GNU `diffutils` hunk alignment.
//! - **Collision-Free Token Interning:** `diff_lines` interns text lines into `u32` tokens for
//!   fast, single-instruction equality checks.
//! - **Zero Dependencies:** Pure standard library, suitable for embedded, CLI, and systems programming.
//!
//! # Quick Start
//!
//! ```rust
//! use ses::{diff_slices, Change, DiffChunk};
//!
//! let old = ["apple", "banana", "cherry"];
//! let new = ["apple", "blueberry", "cherry"];
//!
//! let diff = diff_slices(&old, &new);
//!
//! // Inspect hunks
//! for hunk in diff.hunks() {
//!     println!("Replaced old[{:?}] with new[{:?}]", hunk.old_range(), hunk.new_range());
//! }
//!
//! // Stream element-by-element changes
//! for change in diff.iter_changes() {
//!     match change {
//!         Change::Equal(old, _new) => println!("  {old}"),
//!         Change::Delete(item) => println!("- {item}"),
//!         Change::Insert(item) => println!("+ {item}"),
//!     }
//! }
//!
//! // Stream contiguous chunk slices
//! for chunk in diff.iter_chunks() {
//!     match chunk {
//!         DiffChunk::Equal(slice) => println!("Keep: {slice:?}"),
//!         DiffChunk::Delete(slice) => println!("Del:  {slice:?}"),
//!         DiffChunk::Insert(slice) => println!("Ins:  {slice:?}"),
//!     }
//! }
//! ```

pub(crate) mod algorithm;
mod diff;
mod edit;
pub(crate) mod optimize;
mod views;

pub use diff::Diff;
pub use edit::{EditScript, Hunk};
pub use views::{Change, ChangesIter, ChunksIter, DiffChunk};

/// Computes the diff between two slices using custom element equality logic.
///
/// This function is ideal when element equality cannot be expressed via standard
/// [`PartialEq`], such as:
/// - Case-insensitive string comparisons (e.g. `str::eq_ignore_ascii_case`),
/// - Whitespace-normalized comparisons,
/// - Comparing domain models by key/ID while ignoring metadata fields.
///
/// # Arguments
///
/// * `a` - The original source slice (sequence A).
/// * `b` - The modified target slice (sequence B).
/// * `equals` - A closure or function `FnMut(&T, &T) -> bool` that returns `true` if two
///   elements are considered equal.
///
/// # Complexity
///
/// - **Time:** `O((N + M) D)` in the worst case, where `N = a.len()`, `M = b.len()`,
///   and `D` is the size of the minimal edit script. Near `O(N)` for sequences with long matching runs.
/// - **Space:** Strictly `O(N + M)` linear space.
///
/// # Examples
///
/// Case-insensitive ASCII comparison:
///
/// ```rust
/// use ses::diff_slices_by;
///
/// let old = ["HELLO", "world"];
/// let new = ["hello", "WORLD"];
///
/// let diff = diff_slices_by(&old, &new, |x, y| x.eq_ignore_ascii_case(y));
///
/// // Since elements match case-insensitively, no edits are emitted
/// assert!(diff.is_empty());
/// ```
///
/// Custom struct diffing by ID:
///
/// ```rust
/// use ses::diff_slices_by;
///
/// #[derive(Debug)]
/// struct User { id: u64, name: &'static str }
///
/// let old = [User { id: 1, name: "Alice" }, User { id: 2, name: "Bob" }];
/// let new = [User { id: 1, name: "Alice Updated" }, User { id: 3, name: "Charlie" }];
///
/// let diff = diff_slices_by(&old, &new, |u1, u2| u1.id == u2.id);
///
/// assert_eq!(diff.hunks().len(), 1);
/// assert_eq!(diff.hunks()[0].old_range(), 1..2); // User id 2 deleted
/// assert_eq!(diff.hunks()[0].new_range(), 1..2); // User id 3 inserted
/// ```
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

/// Computes the diff between two slices of elements implementing [`PartialEq`].
///
/// This is the primary general-purpose diff function in `ses`. It operates in three stages:
/// 1. **Prefix & Suffix Pruning:** Strips identical boundary runs in `O(N)` time.
/// 2. **Myers SES Algorithm:** Solves the remaining problem in `O((N + M) D)` time and `O(N + M)` space
///    using linear-space divide-and-conquer bisection.
/// 3. **Semantic Boundary Shifting:** Post-processes edit hunks to slide change boundaries across
///    matching tokens (closing delimiters, blank lines, repeated elements), merging adjacent
///    edits and aligning hunk boundaries with GNU `diffutils` convention.
///
/// # Arguments
///
/// * `a` - The original source slice (sequence A).
/// * `b` - The modified target slice (sequence B).
///
/// # Returns
///
/// A [`Diff<'a, T>`] container holding borrowed references to `a` and `b` alongside the computed
/// [`EditScript`]. The returned diff exposes:
/// - [`Diff::hunks`]: Slice of compact edit ranges ([`Hunk`]).
/// - [`Diff::iter_changes`]: Lazy zero-copy iterator over item-by-item changes ([`Change`]).
/// - [`Diff::iter_chunks`]: Lazy zero-copy iterator over contiguous change slices ([`DiffChunk`]).
///
/// # Complexity
///
/// - **Time:** `O((N + M) D)`, where `D` is the minimal edit distance. For identical slices, runs in `O(N)` time.
/// - **Space:** `O(N + M)` linear workspace memory.
///
/// # Examples
///
/// Inspecting hunks:
///
/// ```rust
/// use ses::diff_slices;
///
/// let old = [1, 2, 3, 4, 5];
/// let new = [1, 9, 3, 4, 5];
/// let diff = diff_slices(&old, &new);
///
/// assert_eq!(diff.len(), 1);
/// let hunk = &diff.hunks()[0];
/// assert_eq!(hunk.old_range(), 1..2); // replaced 2
/// assert_eq!(hunk.new_range(), 1..2); // with 9
/// ```
///
/// Iterating item-by-item changes:
///
/// ```rust
/// use ses::{diff_slices, Change};
///
/// let old = ["a", "b"];
/// let new = ["b", "c"];
/// let diff = diff_slices(&old, &new);
///
/// let changes: Vec<_> = diff.iter_changes().collect();
/// assert_eq!(
///     changes,
///     vec![
///         Change::Delete(&"a"),
///         Change::Equal(&"b", &"b"),
///         Change::Insert(&"c"),
///     ]
/// );
/// ```
///
/// Iterating contiguous slice chunks:
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
///         DiffChunk::Equal(slice) => println!("Unchanged: {slice:?}"),
///         DiffChunk::Delete(slice) => println!("Removed:   {slice:?}"),
///         DiffChunk::Insert(slice) => println!("Added:     {slice:?}"),
///     }
/// }
/// ```
pub fn diff_slices<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Diff<'a, T> {
    diff_slices_by(a, b, |x, y| x == y)
}

/// Convenience type alias for line-by-line text diffs.
pub type LineDiff<'a> = Diff<'a, &'a str>;

/// Convenience type alias for raw byte-line diffs (for non-UTF8 binary files).
pub type ByteLineDiff<'a> = Diff<'a, &'a [u8]>;

/// Computes a line-by-line diff across pre-split slices of string lines using token interning.
///
/// This function is optimized for text comparison. Before running Myers' algorithm, it interns
/// all unique string lines in `old` and `new` into `u32` token IDs using a collision-free hash map.
/// This transforms costly string equality checks during graph exploration into single-CPU-cycle
/// integer comparisons (`u32 == u32`).
///
/// Edits are automatically post-processed through boundary shifting to
/// produce intuitive hunk boundaries that match GNU diff output.
///
/// # Arguments
///
/// * `old` - A slice of line string slices from the original text (e.g. from `split_inclusive('\n')` or `lines()`).
/// * `new` - A slice of line string slices from the modified text.
///
/// # Returns
///
/// A [`LineDiff<'a>`] (alias for `Diff<'a, &'a str>`).
///
/// # Examples
///
/// ```rust
/// use ses::diff_lines;
///
/// let old_text = "fn foo() {\n    println!(\"hello\");\n}\n";
/// let new_text = "fn foo() {\n    println!(\"world\");\n}\n";
///
/// let old_lines: Vec<&str> = old_text.lines().collect();
/// let new_lines: Vec<&str> = new_text.lines().collect();
///
/// let diff = diff_lines(&old_lines, &new_lines);
/// assert_eq!(diff.len(), 1);
/// assert_eq!(diff.hunks()[0].old_range(), 1..2);
/// assert_eq!(diff.hunks()[0].new_range(), 1..2);
/// ```
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
