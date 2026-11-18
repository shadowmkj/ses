use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (a_content, b_content) = if args.len() >= 3 {
        let a = fs::read_to_string(&args[1])?;
        let b = fs::read_to_string(&args[2])?;
        (a, b)
    } else {
        (
            "banana\nmango\ncarrot\ngrapes\napple\n".to_string(),
            "apple\nbanana\nmango\ngrapes\ncarrot\n".to_string(),
        )
    };

    let a_lines: Vec<&str> = a_content.lines().collect();
    let b_lines: Vec<&str> = b_content.lines().collect();

    let diff = ses::diff_slices(&a_lines, &b_lines);
    for hunk in diff.script().hunks() {
        let old_r = hunk.old_range();
        let new_r = hunk.new_range();
        println!(
            "{},{}:{},{}",
            old_r.start, old_r.end, new_r.start, new_r.end
        );
    }

    Ok(())
}
