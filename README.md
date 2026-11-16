# ses (Shortest Edit Script)

[![Crates.io](https://img.shields.io/crates/v/ses.svg)](https://crates.io/crates/ses)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> [!NOTE]
> **Notice**: This is a temporary crate built for testing and evaluating a modern linear-space Myers diff algorithm for [`uutils/diffutils`](https://github.com/uutils/diffutils). For tracking and context, see [uutils/diffutils#401](https://github.com/uutils/diffutils/issues/401).

A high-performance, zero-dependency, linear-space Myers diff engine written in idiomatic Rust (Edition 2024).

`ses` computes the **Shortest Edit Script (SES)** between arbitrary sequences, based on Eugene Myers' 1986 seminal paper *["An O(ND) Difference Algorithm and Its Variations"](https://publications.mpi-cbg.de/Myers_1986_6330.pdf)*. It is designed as a memory-safe, robust foundation for CLI tools (such as `uutils/diffutils`), code editors, version control systems, and language servers.

---

## Why `ses`?

Traditional diff crates often suffer from high memory consumption, recursive stack overflows, or leaky API abstractions:

| Feature | Legacy `diff` Crate | `ses` |
| :--- | :--- | :--- |
| **Memory Complexity** | Unbounded allocations | **Strictly $O(N + M)$ linear space** |
| **Workspace Allocations** | Allocates on every recursive call | **Exactly 1 heap allocation** for the entire algorithm |
| **Stack Safety** | Recursive (overflows on large files) | **100% Iterative** (zero recursion, stack-safe) |
| **Result Representation** | Eager `Vec<Result>` (allocates millions of enum instances) | **Lazy zero-copy streaming iterators** (`iter_changes`, `iter_chunks`) |
| **Equal Elements** | Collides with `std::result::Result` | **Clean `Change::Equal(&old, &new)`** preserving both sides |
| **Pruning Optimizations** | None | **$O(N)$ prefix & suffix pruning** + token interning |
| **Dependencies** | Zero | **Zero dependencies** (standard library only) |

---

## Installation

Add `ses` to your `Cargo.toml`:

```toml
[dependencies]
ses = "0.0.1"
```

---

## Usage & Examples

### 1. Element-by-Element Diffing (`Change`)

Use `iter_changes()` to lazily stream fine-grained changes. Unchanged items yield `Change::Equal(&old, &new)`, allowing access to both original and modified references:

```rust
use ses::{diff_slices, Change};

fn main() {
    let old = ["banana", "mango", "carrot", "grapes", "apple"];
    let new = ["apple", "banana", "mango", "grapes", "carrot"];

    let diff = diff_slices(&old, &new);

    for change in diff.iter_changes() {
        match change {
            Change::Equal(old, _new) => println!("  {old}"),
            Change::Delete(item) => println!("- {item}"),
            Change::Insert(item) => println!("+ {item}"),
        }
    }
}
```

---

### 2. Contiguous Block Diffing (`DiffChunk`)

For bulk I/O and formatters, `iter_chunks()` groups consecutive items into sub-slices (`&'a [T]`), avoiding element-by-element overhead:

```rust
use ses::{diff_slices, DiffChunk};

fn main() {
    let old = ["header", "item1", "item2", "footer"];
    let new = ["header", "item2 modified", "footer"];

    let diff = diff_slices(&old, &new);

    for chunk in diff.iter_chunks() {
        match chunk {
            DiffChunk::Equal(slice) => println!("Unchanged: {:?}", slice),
            DiffChunk::Delete(slice) => println!("Deleted: {:?}", slice),
            DiffChunk::Insert(slice) => println!("Inserted: {:?}", slice),
        }
    }
}
```

---

### 3. Fast Line-by-Line Text Diffing

`diff_lines` uses collision-free token interning to compare lines at CPU register speeds:

```rust
use ses::{diff_lines, Change};

fn main() {
    let old_text = "line 1\nline 2\nline 3";
    let new_text = "line 1\nline 2 (edited)\nline 3";

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let diff = diff_lines(&old_lines, &new_lines);

    for change in diff.iter_changes() {
        match change {
            Change::Equal(l, _) => println!(" {l}"),
            Change::Delete(l) => println!("-{l}"),
            Change::Insert(r) => println!("+{r}"),
        }
    }
}
```

---

### 4. Custom Equality Predicates (`diff_slices_by`)

Compare elements with custom matching logic (e.g., case-insensitive comparisons or whitespace normalization). `Change::Equal` preserves both the original and modified references:

```rust
use ses::{diff_slices_by, Change};

fn main() {
    let old = ["HELLO", "world"];
    let new = ["hello", "WORLD"];

    let diff = diff_slices_by(&old, &new, |a, b| a.eq_ignore_ascii_case(b));

    for change in diff.iter_changes() {
        if let Change::Equal(left, right) = change {
            // "HELLO" != "hello" byte-wise, but they matched under our custom predicate!
            println!("Matched: left='{left}', right='{right}'");
        }
    }
}
```

---

### 5. Inspecting Raw Coordinate Ranges (`EditScript` / `Hunk`)

For tool developers (such as building custom unified diff or side-by-side formatters), `diff.script()` gives direct access to half-open coordinate index ranges (`old_range` and `new_range`):

```rust
use ses::diff_slices;

fn main() {
    let old = [10, 20, 30, 40];
    let new = [10, 25, 40];

    let diff = diff_slices(&old, &new);

    for hunk in diff.script().hunks() {
        println!(
            "Delete old[{}..{}], Insert new[{}..{}]",
            hunk.old_range().start, hunk.old_range().end,
            hunk.new_range().start, hunk.new_range().end,
        );
    }
}
```

---

## Architecture

- **`Workspace`**: Pre-allocates forward (`vf`) and backward (`vb`) diagonal vectors in a single contiguous buffer (`2 * (N + M + 1)` slots), partitioned using `split_at_mut`. Reused throughout the entire divide-and-conquer process.
- **`EditBox` Worklist**: Replaces recursive function calls with an iterative stack, pushing right partitions before left partitions to guarantee hunks are emitted in strictly ascending coordinate order without sorting.
- **Prefix & Suffix Trimming**: Strips identical boundaries in $O(N)$ time before running the graph search, drastically reducing edit box dimensions.
- **Zero-Breaking-Change Encapsulation**: Struct fields are private with clean accessor methods following Rust API guidelines (C-STRUCT-PRIVATE).

---

## Examples

Runnable examples are provided in the [`examples/`](examples) directory:

- **`basic`**: Demonstrates streaming `Change` and `DiffChunk` views:
  ```bash
  cargo run --example basic
  ```
- **`files`**: Demonstrates zero-copy raw byte line diffing over files:
  ```bash
  cargo run --example files
  ```
- **`unified`**: Demonstrates building a custom unified diff formatter (`diff -u`) with context lines from `ses::Diff`:
  ```bash
  cargo run --example unified -- file_a.txt file_b.txt
  ```
- **`sdiff`**: Demonstrates side-by-side diffing (`sdiff` / `diff -y`) with column alignment and suppression options:
  ```bash
  cargo run --example sdiff -- file_a.txt file_b.txt
  ```

---

## License

This project is licensed under the [MIT License](LICENSE).
