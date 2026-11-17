use crate::edit::{EditScript, Hunk};

/// Reusable workspace allocating forward and backward diagonal buffers to ensure
/// a single allocation for the entirety of the algorithm
pub(crate) struct Workspace {
    buffer: Vec<usize>,
    max: usize,
}

impl Workspace {
    pub fn new(max: usize) -> Self {
        // Number of diagonal slots from -max to +max inclusive:
        // max negative diagonals + 1 zero diagonal (k = 0) + max positive diagonals
        let stride = 2 * max + 1;
        Self {
            buffer: vec![0; stride * 2],
            max,
        }
    }

    /// The number of diagonal slots needed to store all diagonals from `-max` to `+max` inclusive.
    #[inline]
    pub fn stride(&self) -> usize {
        2 * self.max + 1
    }

    /// Slices the buffer into forward (`vf`) and backward (`vb`) mutable diagonal arrays.
    ///
    /// The underlying buffer is partitioned into two disjoint slices of equal size `stride`.
    /// Each slice is indexed by diagonal `k` offset by `max` (using `Workspace::offset`).
    /// - The first slice represents the forward search vector (`vf`).
    /// - The second slice represents the backward search vector (`vb`).
    #[inline]
    pub fn split_diagonals(&mut self) -> (&mut [usize], &mut [usize]) {
        let stride = self.stride();
        self.buffer.split_at_mut(stride)
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.max
    }
}

/// A "snake" in Myers' algorithm: a maximal diagonal run of identical elements (`A[x..u] == B[y..v]`).
///
/// In linear-space divide-and-conquer, the middle snake bisects an edit box into two smaller
/// partitions: `(0..x, 0..y)` on the left, and `(u..n, v..m)` on the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Snake {
    /// Start index in sequence A
    pub x: usize,
    /// Start index in sequence B
    pub y: usize,
    /// End index in sequence A
    pub u: usize,
    /// End index in sequence B
    pub v: usize,
}

/// A rectangular 2D slice of the edit graph defined by two sub-sequences:
/// `A[x_off..x_off + n]` along the X-axis and `B[y_off..y_off + m]` along the Y-axis.
///
/// Divide-and-conquer iteratively subdivides this box around middle snakes
/// until reaching base cases (identical matches or pure insertions/deletions).
#[derive(Debug, Clone, Copy)]
struct EditBox {
    /// Starting X-coordinate (offset into sequence A) where this box begins.
    x_off: usize,
    /// Starting Y-coordinate (offset into sequence B) where this box begins.
    y_off: usize,
    /// Number of elements from sequence A (width along X-axis).
    n: usize,
    /// Number of elements from sequence B (height along Y-axis).
    m: usize,
}

/// Computes the Shortest Edit Script (SES) in $O(N + M)$ space and $O(N \cdot D)$ time,
/// based on Eugene Myers' 1986 paper *"An O(ND) Difference Algorithm and Its Variations"*.
/// https://publications.mpi-cbg.de/Myers_1986_6330.pdf
///
/// Myers models diffing as finding the shortest path across a 2D edit graph where deletions
/// and insertions cost 1, while identical diagonal matches cost 0. By exploring along
/// diagonals $k = x - y$, it greedily tracks the furthest-reaching paths for each edit step.
///
/// This implementation maps Myers' linear-space refinement into an iterative architecture:
/// a single contiguous `Workspace` hosts both forward and backward diagonals via `split_at_mut`,
/// an explicit `EditBox` worklist replaces call-stack recursion to guarantee overflow safety,
/// and bidirectional middle snakes bisect ranges until reaching base cases that coalesce into hunks.
///
/// # Arguments
/// * `len_a` - Number of elements in sequence A.
/// * `len_b` - Number of elements in sequence B.
/// * `equals` - An index-based equality predicate `equals(i, j)` returning `true` if element `i`
///   in sequence A equals element `j` in sequence B.
pub(crate) fn myers<F>(len_a: usize, len_b: usize, mut equals: F) -> EditScript
where
    F: FnMut(usize, usize) -> bool,
{
    let max = len_a + len_b;
    let mut ws = Workspace::new(max);
    let mut hunks: Vec<Hunk> = Vec::new();

    // stack for iterative approach (avoid stack overflow)
    let mut stack = Vec::with_capacity(32);
    stack.push(EditBox {
        x_off: 0,
        y_off: 0,
        n: len_a,
        m: len_b,
    });

    while let Some(edit_box) = stack.pop() {
        let EditBox {
            mut x_off,
            mut y_off,
            mut n,
            mut m,
        } = edit_box;

        while n > 0 && m > 0 && equals(x_off, y_off) {
            x_off += 1;
            y_off += 1;
            n -= 1;
            m -= 1;
        }

        while n > 0 && m > 0 && equals(x_off + n - 1, y_off + m - 1) {
            n -= 1;
            m -= 1;
        }

        if n == 0 && m == 0 {
            continue;
        }

        if n == 0 {
            push_or_coalesce(&mut hunks, Hunk::new(x_off..x_off, y_off..y_off + m));
            continue;
        }

        if m == 0 {
            push_or_coalesce(&mut hunks, Hunk::new(x_off..x_off + n, y_off..y_off));
            continue;
        }

        // Find the middle snake: the maximal run of matching elements where forward and backward
        // searches meet, bisecting this edit box into two smaller independent partitions.
        if let Some(snake) = middle_snake(x_off, y_off, n, m, &mut equals, &mut ws) {
            // Push RIGHT box first so that LEFT box is popped first,
            // This guarantees hunks are emitted in strictly ascending index order so that there is
            // no need to sort after collection
            stack.push(EditBox {
                x_off: x_off + snake.u,
                y_off: y_off + snake.v,
                n: n - snake.u,
                m: m - snake.v,
            });

            stack.push(EditBox {
                x_off,
                y_off,
                n: snake.x,
                m: snake.y,
            });
        }
    }

    EditScript::new(hunks)
}

/// Appends a `Hunk` to the list, coalescing it with the preceding hunk if they are adjacent or overlapping.
fn push_or_coalesce(hunks: &mut Vec<Hunk>, hunk: Hunk) {
    if let Some(last) = hunks.last_mut() {
        let old_r = last.old_range();
        let new_r = last.new_range();
        let h_old = hunk.old_range();
        let h_new = hunk.new_range();

        if old_r.end >= h_old.start && new_r.end >= h_new.start {
            *last = Hunk::new(
                old_r.start..old_r.end.max(h_old.end),
                new_r.start..new_r.end.max(h_new.end),
            );
            return;
        }
    }
    hunks.push(hunk);
}

/// Finds the middle snake in O(ND) time and O(N + M) space.
/// Reuses the forward and backward slices from `ws` without allocating.
fn middle_snake<F>(
    x_off: usize,
    y_off: usize,
    n: usize,
    m: usize,
    equals: &mut F,
    ws: &mut Workspace,
) -> Option<Snake>
where
    F: FnMut(usize, usize) -> bool,
{
    let max = n + m;
    let delta = n as isize - m as isize;
    let odd = (delta & 1) != 0;
    let offset = ws.offset();

    let (vf, vb) = ws.split_diagonals();

    let k0_idx = offset;
    vf[k0_idx] = 0;

    let k_delta_idx = (delta + offset as isize) as usize;
    vb[k_delta_idx] = n;

    let d_max = max.div_ceil(2);
    let min_k = -(m as isize);
    let max_k = n as isize;

    for d in 1..=d_max {
        let d_isize = d as isize;

        for k in (-d_isize..=d_isize).rev().step_by(2) {
            if k < min_k || k > max_k {
                continue;
            }

            let k_idx = (k + offset as isize) as usize;
            let mut x = if k == -d_isize
                || k == min_k
                || (k != d_isize && k != max_k && vf[k_idx - 1] < vf[k_idx + 1])
            {
                vf[k_idx + 1]
            } else {
                vf[k_idx - 1] + 1
            };
            let y_isize = x as isize - k;
            let x0 = x;
            let y0 = if y_isize >= 0 { y_isize as usize } else { 0 };

            if y_isize >= 0 {
                let mut y = y_isize as usize;
                while x < n && y < m && equals(x_off + x, y_off + y) {
                    x += 1;
                    y += 1;
                }
            }
            vf[k_idx] = x;

            // If delta is odd, check if forward search overlaps backward search
            if odd
                && k >= delta - (d_isize - 1)
                && k <= delta + (d_isize - 1)
                && vf[k_idx] >= vb[k_idx]
            {
                let y = (x as isize - k).max(0) as usize;
                return Some(Snake {
                    x: x0,
                    y: y0,
                    u: x,
                    v: y,
                });
            }
        }

        for k in ((delta - d_isize)..=(delta + d_isize)).rev().step_by(2) {
            if k < min_k || k > max_k {
                continue;
            }

            let k_idx = (k + offset as isize) as usize;

            let mut u = if k == delta + d_isize || k == max_k {
                vb[k_idx - 1]
            } else if k == delta - d_isize || k == min_k {
                vb[k_idx + 1].saturating_sub(1)
            } else if vb[k_idx - 1] < vb[k_idx + 1].saturating_sub(1) {
                vb[k_idx - 1]
            } else {
                vb[k_idx + 1].saturating_sub(1)
            };

            let v_isize = u as isize - k;
            let u0 = u;
            let v0 = if v_isize >= 0 { v_isize as usize } else { 0 };

            if v_isize >= 0 {
                let mut v = v_isize as usize;
                while u > 0 && v > 0 && equals(x_off + u - 1, y_off + v - 1) {
                    u -= 1;
                    v -= 1;
                }
            }
            vb[k_idx] = u;

            // If delta is even, check if backward search overlaps forward search
            if !odd && k >= -d_isize && k <= d_isize && vf[k_idx] >= vb[k_idx] {
                let v = (u as isize - k).max(0) as usize;
                return Some(Snake {
                    x: u,
                    y: v,
                    u: u0,
                    v: v0,
                });
            }
        }
    }

    // Fallback: full substitution if search limit reached
    Some(Snake {
        x: 0,
        y: 0,
        u: n,
        v: m,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myers_identical() {
        let a = ["a", "b", "c"];
        let b = ["a", "b", "c"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert!(script.is_empty());
    }

    #[test]
    fn test_myers_delete_middle() {
        let a = ["a", "b", "c"];
        let b = ["a", "c"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_range(), 1..2);
        assert_eq!(script.hunks()[0].new_range(), 1..1);
    }

    #[test]
    fn test_myers_insert_middle() {
        let a = ["a", "c"];
        let b = ["a", "b", "c"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_range(), 1..1);
        assert_eq!(script.hunks()[0].new_range(), 1..2);
    }

    #[test]
    fn test_myers_complex() {
        let a = ["A", "B", "C", "A", "B", "B", "A"];
        let b = ["C", "B", "A", "B", "A", "C"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert!(!script.is_empty());
    }

    #[test]
    fn test_myers_both_empty() {
        let a: [&str; 0] = [];
        let b: [&str; 0] = [];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert!(script.is_empty());
    }

    #[test]
    fn test_myers_empty_to_nonempty() {
        let a: [&str; 0] = [];
        let b = ["x", "y"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_range(), 0..0);
        assert_eq!(script.hunks()[0].new_range(), 0..2);
    }

    #[test]
    fn test_myers_nonempty_to_empty() {
        let a = ["x", "y"];
        let b: [&str; 0] = [];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_range(), 0..2);
        assert_eq!(script.hunks()[0].new_range(), 0..0);
    }

    #[test]
    fn test_myers_completely_disjoint() {
        let a = ["1", "2", "3"];
        let b = ["4", "5", "6"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_range(), 0..3);
        assert_eq!(script.hunks()[0].new_range(), 0..3);
    }

    #[test]
    fn test_myers_repeated_elements() {
        let a = ["a", "a", "a"];
        let b = ["a", "a"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        assert_eq!(script.len(), 1);
        assert_eq!(script.hunks()[0].old_len(), 1);
        assert_eq!(script.hunks()[0].new_len(), 0);
    }

    #[test]
    fn test_push_or_coalesce() {
        let mut hunks = Vec::new();
        // Adjacent deletion and insertion at the same position should merge into a replacement
        push_or_coalesce(&mut hunks, Hunk::new(1..2, 1..1)); // Delete index 1
        push_or_coalesce(&mut hunks, Hunk::new(2..2, 1..2)); // Insert at index 1
        push_or_coalesce(&mut hunks, Hunk::new(5..7, 4..4)); // Separate disjoint deletion

        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].old_range(), 1..2);
        assert_eq!(hunks[0].new_range(), 1..2);
        assert_eq!(hunks[1].old_range(), 5..7);
        assert_eq!(hunks[1].new_range(), 4..4);
    }

    #[test]
    fn test_myers_paper_1986() {
        // The example in Eugene W Myers paper
        let a = ["C", "B", "A", "B", "A", "C"];
        let b = ["A", "B", "C", "A", "B", "B", "A"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        // Optimal D = 5: matches B, A, B, A (length 4)
        assert_eq!(script.len(), 4);
        assert_eq!(script.hunks()[0].old_range(), 0..1); // Delete C
        assert_eq!(script.hunks()[0].new_range(), 0..1); // Replace with A
        assert_eq!(script.hunks()[1].old_range(), 2..2); // Insert C
        assert_eq!(script.hunks()[1].new_range(), 2..3);
        assert_eq!(script.hunks()[2].old_range(), 4..4); // Insert B
        assert_eq!(script.hunks()[2].new_range(), 5..6);
        assert_eq!(script.hunks()[3].old_range(), 5..6); // Delete C
        assert_eq!(script.hunks()[3].new_range(), 7..7);
    }

    #[test]
    fn test_myers_gnu_diff_alignment_fruits() {
        let a = ["banana", "mango", "carrot", "grapes", "apple"];
        let b = ["apple", "banana", "mango", "grapes", "carrot"];
        let script = myers(a.len(), b.len(), |i, j| a[i] == b[j]);
        // Matches banana, mango, grapes (LCS length 3, D = 4)
        assert_eq!(script.len(), 3);
        assert_eq!(script.hunks()[0].old_range(), 0..0); // Insert apple
        assert_eq!(script.hunks()[0].new_range(), 0..1);
        assert_eq!(script.hunks()[1].old_range(), 2..3); // Delete carrot
        assert_eq!(script.hunks()[1].new_range(), 3..3);
        assert_eq!(script.hunks()[2].old_range(), 4..5); // Replace apple with carrot
        assert_eq!(script.hunks()[2].new_range(), 4..5);
    }
}
