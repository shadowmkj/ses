use ses::diff_slices;
use std::process::Command;

/// Converts a GNU diff normal-format hunk string to (old_range, new_range)
fn parse_gnu_diff_normal(output: &str) -> Vec<((usize, usize), (usize, usize))> {
    let mut hunks = Vec::new();
    for line in output.lines() {
        // Hunk headers start with a digit: e.g. 0a1 or 1,2c3,4 or 3d2
        if !line.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }

        let bytes = line.as_bytes();
        let mut op_idx = None;
        let mut op_char = ' ';
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'a' || b == b'd' || b == b'c' {
                op_idx = Some(i);
                op_char = b as char;
                break;
            }
        }
        let Some(op_pos) = op_idx else { continue };

        let left = &line[..op_pos];
        let right = &line[op_pos + 1..];

        let parse_range = |s: &str| -> Option<(usize, usize)> {
            if let Some((start, end)) = s.split_once(',') {
                let s = start.parse::<usize>().ok()?;
                let e = end.parse::<usize>().ok()?;
                Some((s, e))
            } else {
                let n = s.parse::<usize>().ok()?;
                Some((n, n))
            }
        };

        let (Some((s1, e1)), Some((s2, e2))) = (parse_range(left), parse_range(right)) else {
            continue;
        };

        let (old_r, new_r) = match op_char {
            'a' => ((s1, s1), (s2 - 1, e2)),
            'd' => ((s1 - 1, e1), (s2, s2)),
            'c' => ((s1 - 1, e1), (s2 - 1, e2)),
            _ => continue,
        };

        hunks.push((old_r, new_r));
    }
    hunks
}

/// Helper that verifies `ses` matches GNU diff `-d` (minimal) if GNU diff is installed
fn assert_matches_gnu_diff(a: &[&str], b: &[&str]) {
    let diff = diff_slices(a, b);
    let ses_hunks: Vec<((usize, usize), (usize, usize))> = diff
        .script()
        .hunks()
        .iter()
        .map(|h| {
            (
                (h.old_range().start, h.old_range().end),
                (h.new_range().start, h.new_range().end),
            )
        })
        .collect();

    // Check if GNU diff is installed
    let gnu_diff_paths = ["/opt/homebrew/opt/diffutils/bin/diff", "gdiff", "diff"];
    let mut gnu_bin = None;
    for path in gnu_diff_paths {
        if let Ok(output) = Command::new(path).arg("--version").output() {
            let ver = String::from_utf8_lossy(&output.stdout);
            if ver.contains("GNU diffutils") {
                gnu_bin = Some(path);
                break;
            }
        }
    }

    if let Some(bin) = gnu_bin {
        use std::io::Write;
        let mut file_a = tempfile_named("ses_test_a");
        let mut file_b = tempfile_named("ses_test_b");

        for line in a {
            writeln!(file_a.1, "{}", line).unwrap();
        }
        for line in b {
            writeln!(file_b.1, "{}", line).unwrap();
        }

        let output = Command::new(bin)
            .arg("-d")
            .arg(&file_a.0)
            .arg(&file_b.0)
            .output()
            .expect("failed to execute GNU diff");

        let gnu_out = String::from_utf8_lossy(&output.stdout);
        let gnu_hunks = parse_gnu_diff_normal(&gnu_out);

        assert_eq!(
            ses_hunks, gnu_hunks,
            "SES hunks did not match GNU diff -d!\nInput A: {:?}\nInput B: {:?}\nGNU output:\n{}",
            a, b, gnu_out
        );

        let _ = std::fs::remove_file(&file_a.0);
        let _ = std::fs::remove_file(&file_b.0);
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn tempfile_named(prefix: &str) -> (String, std::fs::File) {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/{}_{}_{}.txt", prefix, std::process::id(), count);
    let f = std::fs::File::create(&path).unwrap();
    (path, f)
}

#[test]
fn test_gnu_diff_paper_1986() {
    let a = ["C", "B", "A", "B", "A", "C"];
    let b = ["A", "B", "C", "A", "B", "B", "A"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_fruits() {
    let a = ["banana", "mango", "carrot", "grapes", "apple"];
    let b = ["apple", "banana", "mango", "grapes", "carrot"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_boundary_shift_repeated_lines() {
    // Adding a second function sharing braces and blank lines
    let a = ["fn foo() {", "    body();", "}", ""];
    let b = [
        "fn foo() {",
        "    body();",
        "}",
        "",
        "fn bar() {",
        "    body();",
        "}",
        "",
    ];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_boundary_shift_duplicate_prefix() {
    // Duplicate lines at the start of a hunk
    let a = ["header", "item", "item", "footer"];
    let b = ["header", "item", "footer"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_boundary_shift_closing_delimiters() {
    let a = ["if cond {", "    a();", "}", "else {", "    b();", "}"];
    let b = ["if cond {", "    a();", "}"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_coalescing_adjacent_edits() {
    let a = ["x", "a", "c", "d", "d", "a", "d", "c", "d", "c", "y"];
    let b = ["x", "b", "b", "b", "a", "a", "d", "a", "y"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_alternating_periodic() {
    let a = ["a", "b", "a", "b", "a", "b"];
    let b = ["b", "a", "b", "a"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_reverse_elements() {
    let a = ["a", "b", "c", "d", "e"];
    let b = ["e", "d", "c", "b", "a"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_case_8_suffix_interaction() {
    let a = ["d", "c", "d"];
    let b = ["c", "d", "b", "a", "e", "d"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_completely_disjoint() {
    let a = ["one", "two", "three"];
    let b = ["four", "five", "six"];
    assert_matches_gnu_diff(&a, &b);
}

#[test]
fn test_gnu_diff_empty_and_single() {
    assert_matches_gnu_diff(&[], &["only"]);
    assert_matches_gnu_diff(&["only"], &[]);
    assert_matches_gnu_diff(&["same"], &["same"]);
}

#[test]
fn test_gnu_diff_random_differential_fuzz() {
    struct SimpleRng(u64);
    impl SimpleRng {
        fn next_u32(&mut self) -> u32 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            (self.0 >> 32) as u32
        }
        fn gen_range(&mut self, min: usize, max: usize) -> usize {
            min + (self.next_u32() as usize % (max - min + 1))
        }
    }

    let mut rng = SimpleRng(123456789);
    let tokens = ["a", "b", "c", "d", "e", "f"];

    for _ in 0..100 {
        let n = rng.gen_range(1, 20);
        let m = rng.gen_range(1, 20);
        let a: Vec<&str> = (0..n)
            .map(|_| tokens[rng.gen_range(0, tokens.len() - 1)])
            .collect();
        let b: Vec<&str> = (0..m)
            .map(|_| tokens[rng.gen_range(0, tokens.len() - 1)])
            .collect();
        assert_matches_gnu_diff(&a, &b);
    }
}
