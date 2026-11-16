use std::env;
use std::fmt;
use std::fs;

/// Formatter generating standard unified diff output with configurable context lines.
pub struct UnifiedDiff<'s, 'a, T> {
    script: &'s ses::EditScript,
    old: &'a [T],
    new: &'a [T],
    context_lines: usize,
}

impl<'s, 'a, T> UnifiedDiff<'s, 'a, T> {
    pub fn new(diff: &'s ses::Diff<'a, T>) -> Self {
        Self {
            script: diff.script(),
            old: diff.old(),
            new: diff.new(),
            context_lines: 3,
        }
    }

    pub fn context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }
}

impl<'s, 'a, T: fmt::Display> fmt::Display for UnifiedDiff<'s, 'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hunks = self.script.hunks();
        if hunks.is_empty() {
            return Ok(());
        }

        // Group nearby hunks whose context ranges overlap into combined blocks
        let mut group_start = 0;
        while group_start < hunks.len() {
            let mut group_end = group_start;

            while group_end + 1 < hunks.len() {
                let curr_end = hunks[group_end].old_range().end;
                let next_start = hunks[group_end + 1].old_range().start;
                if next_start <= curr_end + (2 * self.context_lines) {
                    group_end += 1;
                } else {
                    break;
                }
            }

            // Calculate context boundaries
            let first_hunk = &hunks[group_start];
            let last_hunk = &hunks[group_end];

            let old_ctx_start = first_hunk
                .old_range()
                .start
                .saturating_sub(self.context_lines);
            let old_ctx_end = (last_hunk.old_range().end + self.context_lines).min(self.old.len());

            let new_ctx_start = first_hunk
                .new_range()
                .start
                .saturating_sub(self.context_lines);
            let new_ctx_end = (last_hunk.new_range().end + self.context_lines).min(self.new.len());

            let old_count = old_ctx_end - old_ctx_start;
            let new_count = new_ctx_end - new_ctx_start;

            // Header: @@ -old_start,old_count +new_start,new_count @@
            writeln!(
                f,
                "@@ -{},{} +{},{} @@",
                old_ctx_start + 1,
                old_count,
                new_ctx_start + 1,
                new_count
            )?;

            let mut old_cursor = old_ctx_start;
            for h in &hunks[group_start..=group_end] {
                // Preceding context
                while old_cursor < h.old_range().start {
                    writeln!(f, " {}", self.old[old_cursor])?;
                    old_cursor += 1;
                }
                // Deletions
                for i in h.old_range() {
                    writeln!(f, "-{}", self.old[i])?;
                    old_cursor += 1;
                }
                // Insertions
                for i in h.new_range() {
                    writeln!(f, "+{}", self.new[i])?;
                }
            }
            // Trailing context
            while old_cursor < old_ctx_end {
                writeln!(f, " {}", self.old[old_cursor])?;
                old_cursor += 1;
            }

            group_start = group_end + 1;
        }

        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (from_content, to_content) = if args.len() >= 3 {
        let from = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
            eprintln!("Error reading '{}': {}", args[1], err);
            std::process::exit(1);
        });
        let to = fs::read_to_string(&args[2]).unwrap_or_else(|err| {
            eprintln!("Error reading '{}': {}", args[2], err);
            std::process::exit(1);
        });
        (from, to)
    } else {
        (
            "banana\nmango\ncarrot\ngrapes\napple\n".to_string(),
            "apple\nbanana\nmango\ngrapes\ncarrot\n".to_string(),
        )
    };

    let from_lines: Vec<&str> = from_content.lines().collect();
    let to_lines: Vec<&str> = to_content.lines().collect();

    let diff = ses::diff_lines(&from_lines, &to_lines);
    print!("{}", UnifiedDiff::new(&diff).context_lines(2));
}
