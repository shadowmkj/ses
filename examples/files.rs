use ses::{Change, DiffChunk};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (old_bytes, new_bytes) = if args.len() >= 3 {
        let old = fs::read(&args[1]).unwrap_or_else(|err| {
            eprintln!("Error reading '{}': {}", args[1], err);
            std::process::exit(1);
        });
        let new = fs::read(&args[2]).unwrap_or_else(|err| {
            eprintln!("Error reading '{}': {}", args[2], err);
            std::process::exit(1);
        });
        (old, new)
    } else {
        (
            b"banana\nmango\ncarrot\ngrapes\napple\n".to_vec(),
            b"apple\nbanana\nmango\ngrapes\ncarrot\n".to_vec(),
        )
    };

    // Split on newline while preserving the newline byte (exactly how diffutils does it)
    let old: Vec<&[u8]> = old_bytes.split_inclusive(|&b| b == b'\n').collect();
    let new: Vec<&[u8]> = new_bytes.split_inclusive(|&b| b == b'\n').collect();

    // T is &[u8] (zero-copy line diff over raw byte slices with token interning)
    let diff = ses::diff_lines(&old, &new);

    println!("--- Item-by-item changes (Change) ---");
    for change in diff.iter_changes() {
        match change {
            Change::Equal(old, _new) => print!("  {}", String::from_utf8_lossy(old)),
            Change::Delete(item) => print!("- {}", String::from_utf8_lossy(item)),
            Change::Insert(item) => print!("+ {}", String::from_utf8_lossy(item)),
        }
    }

    println!("\n--- Contiguous chunk slices (DiffChunk) ---");
    for chunk in diff.iter_chunks() {
        match chunk {
            DiffChunk::Equal(lines, _new_lines) => {
                for line in lines {
                    print!("  {}", String::from_utf8_lossy(line));
                }
            }
            DiffChunk::Delete(lines) => {
                for line in lines {
                    print!("- {}", String::from_utf8_lossy(line));
                }
            }
            DiffChunk::Insert(lines) => {
                for line in lines {
                    print!("+ {}", String::from_utf8_lossy(line));
                }
            }
        }
    }
}
