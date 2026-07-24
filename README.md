# rust-yudi-profiler

A lightweight Rust proc-macro profiler. Five macros, zero external runtime dependencies.

## Macros

### `#[timed]`
Instruments a function, recording elapsed time and call count for every invocation. Recursive calls are recorded automatically.

```rust
#[timed]
fn compute_fib(n: u64) -> u64 {
    // function body
    n
}

let result = compute_fib(40);
```

The function name is used as the profile key. The attribute takes no arguments.

### `count!(name)`
Increments a named counter with no timing overhead. Accepts a string literal or a bare identifier.

```rust
count!("cache_hit");
count!(cache_miss);   // equivalent
```

### `summarise!()`
Prints a formatted summary table for the current thread. Timed entries are sorted by total time descending; counter-only entries follow sorted by name.

```
┌────────────────────────────────────┬────────────┬───────────────┬───────────────┐
│ Name                               │      Calls │    Total (ms) │      Avg (µs) │
├────────────────────────────────────┼────────────┼───────────────┼───────────────┤
│ compute_fib                        │     218910 │         0.521 │         0.002 │
│ slow_string_work                   │          1 │         0.183 │       182.574 │
│ even_iteration                     │         25 │             — │             — │
│ odd_iteration                      │         25 │             — │             — │
└────────────────────────────────────┴────────────┴───────────────┴───────────────┘
```

### `summarise_csv!()`
Prints CSV (header + rows) to stdout. Same sort order as `summarise!()`. Count-only entries leave the timing columns empty.

```csv
name,calls,total_nanos,avg_nanos
compute_fib,218910,705036,3
slow_string_work,1,276671,276671
even_iteration,25,,
odd_iteration,25,,
```

### `append_file!(target)`
Appends profile data as CSV to a target. Writes a header row if the target file is missing or empty; otherwise just appends data rows so repeated runs accumulate. Returns `std::io::Result<()>`.

`target` can be any value implementing `profiler::AppendTarget`:

| Input | Example |
|---|---|
| `&str` (incl. literal) | `append_file!("profile.csv")` |
| `String` / `&String` | `append_file!(my_string)` |
| `&Path` / `PathBuf` / `&PathBuf` | `append_file!(&path_buf)` |
| `&File` / `&mut File` | `append_file!(&mut file)` |

## Usage

Add both crates to your `Cargo.toml`:

```toml
[dependencies]
profiler        = { path = "profiler" }
profiler-macros = { path = "profiler-macros" }
```

Then import and use:

```rust
use profiler_macros::{append_file, count, summarise, summarise_csv, timed};

#[timed]
fn compute_fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => compute_fib(n - 1) + compute_fib(n - 2),
    }
}

fn main() {
    for _ in 0..10 {
        let _fib = compute_fib(20);
    }
    count!("startup");
    summarise!();
}
```

## Workspace Layout

```
rust-yudi-profiler/
├── src/main.rs             # example binary
├── profiler/               # runtime crate (thread_local storage, no deps)
└── profiler-macros/        # proc-macro crate (syn + quote)
```
