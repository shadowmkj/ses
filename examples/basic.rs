use ses::Change;

fn main() {
    let old = ["banana", "mango", "carrot", "grapes", "apple"];
    let new = ["apple", "banana", "mango", "grapes", "carrot"];

    let diff = ses::diff_slices(&old, &new);

    println!("--- Item-by-item changes (Change) ---");
    for change in diff.iter_changes() {
        match change {
            Change::Equal(item) => println!("  {item}"),
            Change::Delete(item) => println!("- {item}"),
            Change::Insert(item) => println!("+ {item}"),
        }
    }

    println!("\n--- Contiguous chunk slices (DiffChunk) ---");
    for chunk in diff.iter_chunks() {
        println!("{chunk:?}");
    }
}
