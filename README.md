# ses (Shortest Edit Script)

[![Crates.io](https://img.shields.io/crates/v/ses.svg)](https://crates.io/crates/ses)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> [!NOTE]
> **Notice**: This is a temporary crate built for testing and evaluating a modern linear-space Myers
> diff algorithm for [`uutils/diffutils`](https://github.com/uutils/diffutils). For tracking and
> context, see [uutils/diffutils#401](https://github.com/uutils/diffutils/issues/401).

A high-performance, zero-dependency, linear-space Myers diff engine written in idiomatic Rust
(Edition 2024).

`ses` computes the **Shortest Edit Script (SES)** between arbitrary sequences, based on Eugene
Myers' 1986 seminal paper
_["An O(ND) Difference Algorithm and Its Variations"](https://publications.mpi-cbg.de/Myers_1986_6330.pdf)_.
It is designed as a memory-safe, robust foundation for CLI tools (such as `uutils/diffutils`), code
editors, version control systems, and language servers.

---

## Why `ses`?

While the Rust ecosystem now offers several capable diff crates, `ses` is intentionally designed as
an **ultra-minimal, zero-dependency, and blazing-fast** Myers diff engine. It provides a lean,
robust foundation for systems tools (such as `uutils/diffutils`), CLI utilities, and embedded
environments that require maximum efficiency and zero external baggage:

| Feature                   | `ses`                                                                                            |
| :------------------------ | :----------------------------------------------------------------------------------------------- |
| **Dependencies**          | **Zero dependencies** (pure standard library, rapid compilation)                                 |
| **Performance & Speed**   | **Blazing-fast** with linear-space Myers diff, `O(N)` prefix/suffix pruning, and token interning |
| **Memory Complexity**     | **Strictly `O(N + M)` linear space** (Eugene Myers 1986, Section 4)                              |
| **Workspace Allocations** | **Preallocated workspace buffers** (zero allocations during middle-snake searches)               |
| **GNU Compatibility**     | **Semantic boundary shifting** matching GNU `diffutils` hunk alignment                           |
| **Result Representation** | **Lazy zero-copy streaming iterators** (`iter_changes`, `iter_chunks`) or compact range `Hunk`s  |
| **API Ergonomics**        | **Clean `Change::Equal(&old, &new)`** preserving references to both inputs                       |

---

## Installation

Add `ses` to your `Cargo.toml`:

```toml
[dependencies]
ses = "0.0.3"
```

---

## Usage & Examples

### 1. Element-by-Element Diffing (`Change`)

Use `iter_changes()` to lazily stream fine-grained changes. Unchanged items yield
`Change::Equal(&old, &new)`, allowing access to both original and modified references:

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

For bulk I/O and formatters, `iter_chunks()` groups consecutive items into sub-slices (`&'a [T]`),
avoiding element-by-element overhead:

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

Compare elements with custom matching logic (e.g., case-insensitive comparisons or whitespace
normalization). `Change::Equal` preserves both the original and modified references:

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

### 5. Inspecting Coordinate Ranges (`Hunk` & `Diff`)

For tool developers building custom formatters (like unified or side-by-side diffs), `Diff` and
`EditScript` implement `IntoIterator` for borrowed references, and `Diff` directly exposes
convenient helper methods:

```rust
use ses::diff_slices;

fn main() {
    let old = [10, 20, 30, 40];
    let new = [10, 25, 40];

    let diff = diff_slices(&old, &new);

    // Direct iteration over &diff yields &Hunk
    for hunk in &diff {
        println!(
            "Delete old[{}..{}], Insert new[{}..{}]",
            hunk.old_range().start, hunk.old_range().end,
            hunk.new_range().start, hunk.new_range().end,
        );
    }

    // Direct slice and summary methods on Diff
    println!("Total hunks: {}", diff.len());
    println!("Has changes: {}", !diff.is_empty());
    println!("Hunks slice: {:?}", diff.hunks());
}
```

---

## Architecture

- **Linear-Space Diagonal Workspace**: Reuses preallocated forward and backward diagonal buffers
  (`Workspace`) throughout the middle-snake search, keeping total memory strictly bounded to `O(N +
  M)` with zero reallocations.
- **Divide-and-Conquer (`EditBox`)**: Divides the search box around optimal middle snakes, in-place
  trimming common prefixes and suffixes before each split.
- **Semantic Boundary Shifting**: Post-processes Myers edits to slide change boundaries across
  duplicate tokens (closing delimiters, blank lines, repeated words), cleanly coalescing adjacent
  hunks and matching canonical GNU `diffutils` formatting.
- **Prefix & Suffix Pruning**: Strips identical boundaries in `O(N)$ time before running graph
  searches, drastically reducing sub-problem dimensions.
- **Token Interning**: In `diff_lines`, interns text lines into collision-free `u32` tokens for
  fast, single-instruction equality comparisons.

---

## Examples

Runnable examples are provided in the [`examples/`](examples) directory:

- **`basic`**: Demonstrates streaming `Change` and `DiffChunk` views:
  ```bash
  cargo run --example basic
  ```
- **`hunks`**: Demonstrates accessing raw coordinate ranges (`Range<usize>`) and `Hunk` metadata:
  ```bash
  cargo run --example hunks
  ```
- **`files`**: Demonstrates zero-copy raw byte line diffing over files:
  ```bash
  cargo run --example files
  ```
- **`unified`**: Demonstrates building a custom unified diff formatter (`diff -u`) with context
  lines from `ses::Diff`:
  ```bash
  cargo run --example unified -- file_a.txt file_b.txt
  ```
- **`sdiff`**: Demonstrates side-by-side diffing (`sdiff` / `diff -y`) with column alignment and
  suppression options:
  ```bash
  cargo run --example sdiff -- file_a.txt file_b.txt
  ```

---

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org)


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
