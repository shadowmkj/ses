use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ses::{diff_lines, diff_slices};

/// Generates synthetic source code lines resembling structured Rust code.
fn generate_synthetic_code(num_functions: usize) -> (Vec<String>, Vec<String>) {
    let mut old_lines = Vec::with_capacity(num_functions * 6);
    let mut new_lines = Vec::with_capacity(num_functions * 6);

    for i in 0..num_functions {
        let header = format!("pub fn process_data_{i}(input: &[u8]) -> usize {{");
        old_lines.push(header.clone());
        new_lines.push(header);

        if i % 10 == 0 {
            // Modification (delete + insert)
            old_lines.push(format!("    let mut total = {i} * 2;"));
            old_lines.push("    total += input.len();".to_string());
            new_lines.push(format!("    let mut total = {i} * 3 + 1;"));
            new_lines
                .push("    total ^= input.iter().map(|&x| x as usize).sum::<usize>();".to_string());
        } else if i % 15 == 0 {
            // Insertion in new
            old_lines.push("    let total = input.len();".to_string());
            new_lines.push("    let total = input.len();".to_string());
            new_lines.push(format!("    log_metrics(\"metric_{i}\", total);"));
        } else if i % 20 == 0 {
            // Deletion from old
            old_lines.push(format!("    validate_input_{i}(input);"));
            old_lines.push("    let total = input.len();".to_string());
            new_lines.push("    let total = input.len();".to_string());
        } else {
            // Identical
            old_lines.push(format!("    let total = input.len() + {i};"));
            new_lines.push(format!("    let total = input.len() + {i};"));
        }

        let footer = "    total\n}";
        old_lines.push(footer.to_string());
        new_lines.push(footer.to_string());

        old_lines.push(String::new());
        new_lines.push(String::new());
    }

    (old_lines, new_lines)
}

/// Generates a massive package lockfile (e.g. Cargo.lock or package-lock.json)
/// with 4 lines per package block and scattered version bumps.
fn generate_lockfile(num_packages: usize) -> (Vec<String>, Vec<String>) {
    let mut old_lines = Vec::with_capacity(num_packages * 5);
    let mut new_lines = Vec::with_capacity(num_packages * 5);

    for i in 0..num_packages {
        old_lines.push("[[package]]".to_string());
        new_lines.push("[[package]]".to_string());

        let name = format!("name = \"internal-crate-{i}\"");
        old_lines.push(name.clone());
        new_lines.push(name);

        if i % 25 == 0 {
            // Upgraded package
            old_lines.push(format!("version = \"1.2.{i}\""));
            old_lines.push(format!("checksum = \"hash_old_abc123_{i}\""));
            new_lines.push(format!("version = \"1.3.{i}\""));
            new_lines.push(format!("checksum = \"hash_new_xyz789_{i}\""));
        } else {
            // Unchanged package
            let ver = format!("version = \"1.2.{i}\"");
            let sum = format!("checksum = \"hash_fixed_abc123_{i}\"");
            old_lines.push(ver.clone());
            old_lines.push(sum.clone());
            new_lines.push(ver);
            new_lines.push(sum);
        }

        old_lines.push(String::new());
        new_lines.push(String::new());
    }

    (old_lines, new_lines)
}

fn bench_diff_slices(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_slices");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Identical slices - Small (10k) & Large (50k)
    for &size in &[10_000, 50_000] {
        let identical: Vec<usize> = (0..size).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("identical", size), |b| {
            b.iter(|| diff_slices(black_box(&identical), black_box(&identical)))
        });
    }

    // Scattered real-world edits - 5,000 and 50,000 items (~1% edits)
    for &size in &[5_000, 50_000] {
        let mut seq_a: Vec<usize> = (0..size).collect();
        let mut seq_b = seq_a.clone();
        for i in (0..size).step_by(100) {
            seq_b[i] = 999_999 + i;
        }
        seq_b.insert(size / 3, 777_777);
        seq_a.insert(size * 2 / 3, 888_888);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("scattered_edits_1pct", size), |bencher| {
            bencher.iter(|| diff_slices(black_box(&seq_a), black_box(&seq_b)))
        });
    }

    // Completely disjoint slices (Full bisection across maximum D)
    for &size in &[1_000, 3_000] {
        let disjoint_a: Vec<usize> = (0..size).collect();
        let disjoint_b: Vec<usize> = (size..size * 2).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("completely_disjoint", size), |b| {
            b.iter(|| diff_slices(black_box(&disjoint_a), black_box(&disjoint_b)))
        });
    }

    group.finish();
}

fn bench_diff_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_lines");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Identical lines (5,000 and 50,000 lines)
    for &size in &[5_000, 50_000] {
        let raw_identical: Vec<String> = (0..size).map(|i| format!("line content {i}")).collect();
        let id_lines: Vec<&str> = raw_identical.iter().map(|s| s.as_str()).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("identical_lines", size), |b| {
            b.iter(|| diff_lines(black_box(&id_lines), black_box(&id_lines)))
        });
    }

    // Structured source code diffs across sizes (200, 2,000, 10,000, and 50,000 lines)
    for &funcs in &[40, 400, 2_000, 10_000] {
        let (old_code, new_code) = generate_synthetic_code(funcs);
        let old_ref: Vec<&str> = old_code.iter().map(|s| s.as_str()).collect();
        let new_ref: Vec<&str> = new_code.iter().map(|s| s.as_str()).collect();
        let line_count = old_ref.len();

        group.throughput(Throughput::Elements(line_count as u64));
        group.bench_function(BenchmarkId::new("synthetic_code", line_count), |b| {
            b.iter(|| diff_lines(black_box(&old_ref), black_box(&new_ref)))
        });
    }

    // Massive package lockfile diff (100,000 lines, 20,000 packages with 1% upgrades)
    let (lock_old, lock_new) = generate_lockfile(20_000);
    let lock_old_ref: Vec<&str> = lock_old.iter().map(|s| s.as_str()).collect();
    let lock_new_ref: Vec<&str> = lock_new.iter().map(|s| s.as_str()).collect();
    group.throughput(Throughput::Elements(lock_old_ref.len() as u64));
    group.bench_function(
        BenchmarkId::new("massive_lockfile", lock_old_ref.len()),
        |b| b.iter(|| diff_lines(black_box(&lock_old_ref), black_box(&lock_new_ref))),
    );

    // Heavy boundary shifting stress test (2,000 and 10,000 lines of delimiters & blocks)
    for &num_blocks in &[500, 2_500] {
        let mut shift_old = Vec::with_capacity(num_blocks * 4);
        let mut shift_new = Vec::with_capacity(num_blocks * 4);
        for i in 0..num_blocks {
            shift_old.push("{\n");
            shift_old.push("    data\n");
            shift_old.push("}\n");
            shift_old.push("\n");

            shift_new.push("{\n");
            if i % 5 == 0 {
                shift_new.push("    modified data\n");
                shift_new.push("    extra line\n");
            } else {
                shift_new.push("    data\n");
            }
            shift_new.push("}\n");
            shift_new.push("\n");
        }
        let s_old_ref: Vec<&str> = shift_old.to_vec();
        let s_new_ref: Vec<&str> = shift_new.to_vec();
        let total_lines = s_old_ref.len();

        group.throughput(Throughput::Elements(total_lines as u64));
        group.bench_function(
            BenchmarkId::new("boundary_shifting_stress", total_lines),
            |b| b.iter(|| diff_lines(black_box(&s_old_ref), black_box(&s_new_ref))),
        );
    }

    group.finish();
}

fn bench_views_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("views_iteration");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Test iteration throughput on a 10,000-line diff
    let (code_old, code_new) = generate_synthetic_code(2_000);
    let old_ref: Vec<&str> = code_old.iter().map(|s| s.as_str()).collect();
    let new_ref: Vec<&str> = code_new.iter().map(|s| s.as_str()).collect();
    let diff = diff_lines(&old_ref, &new_ref);

    group.throughput(Throughput::Elements(old_ref.len() as u64));

    group.bench_function(BenchmarkId::new("iter_changes", old_ref.len()), |b| {
        b.iter(|| {
            let mut count = 0usize;
            for change in diff.iter_changes() {
                black_box(change);
                count += 1;
            }
            count
        })
    });

    group.bench_function(BenchmarkId::new("iter_chunks", old_ref.len()), |b| {
        b.iter(|| {
            let mut count = 0usize;
            for chunk in diff.iter_chunks() {
                black_box(chunk);
                count += 1;
            }
            count
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_diff_slices,
    bench_diff_lines,
    bench_views_iteration
);
criterion_main!(benches);
