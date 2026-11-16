use ses::{Change, DiffChunk};
use std::fs;

fn main() {
    let old_bytes = fs::read("a.txt").unwrap();
    let new_bytes = fs::read("b.txt").unwrap();

    // Split on newline while preserving the newline byte (exactly how diffutils does it)
    let old: Vec<&[u8]> = old_bytes.split_inclusive(|&b| b == b'\n').collect();
    let new: Vec<&[u8]> = new_bytes.split_inclusive(|&b| b == b'\n').collect();

    // T is &[u8] (zero-copy line diff over raw byte slices)
    let diff = ses::diff_slices(&old, &new);

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
            DiffChunk::Equal(lines) => {
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
