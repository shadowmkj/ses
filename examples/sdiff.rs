use std::env;
use std::fs;
use std::io;

/// Formats a side-by-side diff similar to GNU sdiff / diff -y.
pub struct SideBySide<'s, 'a> {
    diff: &'s ses::Diff<'a, &'a str>,
    width: usize,
    suppress_common: bool,
}

impl<'s, 'a> SideBySide<'s, 'a> {
    pub fn new(diff: &'s ses::Diff<'a, &'a str>) -> Self {
        Self {
            diff,
            width: 80,
            suppress_common: false,
        }
    }

    /// Sets the total width for both columns (including gutter). Default: 80.
    pub fn width(mut self, width: usize) -> Self {
        self.width = width.max(20);
        self
    }

    /// Suppress printing unchanged common lines (equivalent to `sdiff -s`).
    pub fn suppress_common(mut self, suppress: bool) -> Self {
        self.suppress_common = suppress;
        self
    }

    /// Prints the side-by-side diff to the given writer.
    pub fn print<W: io::Write>(&self, mut out: W) -> io::Result<()> {
        let col_width = (self.width.saturating_sub(4)) / 2;
        let old = self.diff.old();
        let new = self.diff.new();
        let hunks = self.diff.script().hunks();

        let mut old_cursor = 0;
        let mut new_cursor = 0;

        for hunk in hunks {
            // 1. Unchanged common lines preceding this hunk
            while old_cursor < hunk.old_range().start {
                if !self.suppress_common {
                    Self::print_row(
                        &mut out,
                        col_width,
                        Some(old[old_cursor]),
                        "   ",
                        Some(new[new_cursor]),
                    )?;
                }
                old_cursor += 1;
                new_cursor += 1;
            }

            // 2. Changed lines within this hunk
            let del_count = hunk.old_range().len();
            let ins_count = hunk.new_range().len();
            let common_count = del_count.min(ins_count);

            // Replaced lines: both sides changed -> gutter " | "
            for i in 0..common_count {
                Self::print_row(
                    &mut out,
                    col_width,
                    Some(old[old_cursor + i]),
                    " | ",
                    Some(new[new_cursor + i]),
                )?;
            }

            // Excess deletions on left only -> gutter " <"
            for i in common_count..del_count {
                Self::print_row(&mut out, col_width, Some(old[old_cursor + i]), " <", None)?;
            }

            // Excess insertions on right only -> gutter " > "
            for i in common_count..ins_count {
                Self::print_row(&mut out, col_width, None, " > ", Some(new[new_cursor + i]))?;
            }

            old_cursor = hunk.old_range().end;
            new_cursor = hunk.new_range().end;
        }

        // 3. Trailing unchanged lines after the last hunk
        while old_cursor < old.len() {
            if !self.suppress_common {
                Self::print_row(
                    &mut out,
                    col_width,
                    Some(old[old_cursor]),
                    "   ",
                    Some(new[new_cursor]),
                )?;
            }
            old_cursor += 1;
            new_cursor += 1;
        }

        Ok(())
    }

    fn print_row<W: io::Write>(
        out: &mut W,
        col_width: usize,
        left: Option<&str>,
        gutter: &str,
        right: Option<&str>,
    ) -> io::Result<()> {
        let left_str = left.unwrap_or("");
        let right_str = right.unwrap_or("");

        let left_padded = if left_str.len() > col_width {
            &left_str[..col_width]
        } else {
            left_str
        };

        if right.is_none() {
            writeln!(out, "{:<width$}{}", left_padded, gutter, width = col_width)
        } else {
            let right_padded = if right_str.len() > col_width {
                &right_str[..col_width]
            } else {
                right_str
            };
            writeln!(
                out,
                "{:<width$}{}{}",
                left_padded,
                gutter,
                right_padded,
                width = col_width
            )
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let (from_content, to_content) = if args.len() >= 3 {
        let from = fs::read_to_string(&args[1])?;
        let to = fs::read_to_string(&args[2])?;
        (from, to)
    } else if fs::metadata("a.txt").is_ok() && fs::metadata("b.txt").is_ok() {
        let from = fs::read_to_string("a.txt")?;
        let to = fs::read_to_string("b.txt")?;
        (from, to)
    } else {
        (
            "apple\nbanana\ncarrot\n".to_string(),
            "apple\nblueberry\ncarrot\n".to_string(),
        )
    };

    let old_lines: Vec<&str> = from_content.lines().collect();
    let new_lines: Vec<&str> = to_content.lines().collect();

    let diff = ses::diff_slices(&old_lines, &new_lines);

    println!("=== Side-by-side diff (sdiff standard format, width 60) ===");
    let sdiff = SideBySide::new(&diff).width(60);
    sdiff.print(io::stdout())?;

    println!("\n=== Side-by-side diff (sdiff -s, suppress common lines) ===");
    let sdiff_suppressed = SideBySide::new(&diff).width(60).suppress_common(true);
    sdiff_suppressed.print(io::stdout())?;

    Ok(())
}
