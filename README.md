# rust-yudi-profiler

A lightweight Rust proc-macro profiler. Five macros, zero external runtime dependencies.

## Macros

### `timed!(expr)`
Wraps any expression, recording elapsed time and call count into thread-local storage.

```rust
let result = timed!(compute_fib(40));
let s = timed!(my_vec.sort());
```

The name key is derived automatically from the expression:
- `foo::bar(args)` → `"bar"`
- `obj.method(args)` → `"method"`
- anything else → stringified tokens (capped at 32 chars)

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
│ compute_fib                        │         10 │         0.521 │        52.110 │
│ slow_string_work                   │          1 │         0.183 │       182.574 │
│ even_iteration                     │         25 │             — │             — │
│ odd_iteration                      │         25 │             — │             — │
└────────────────────────────────────┴────────────┴───────────────┴───────────────┘
```

### `summarise_csv!()`
Prints CSV (header + rows) to stdout. Same sort order as `summarise!()`. Count-only entries leave the timing columns empty.

```csv
name,calls,total_nanos,avg_nanos
compute_fib,10,705036,70503
slow_string_work,1,276671,276671
even_iteration,25,,
odd_iteration,25,,
```

### `append_file!(path)`
Appends profile data to a CSV file at `path`. Writes a header row if the file does not exist or is empty; otherwise just appends data rows so repeated runs accumulate. Returns `std::io::Result<()>`.

```rust
append_file!("profile.csv").unwrap();
```

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

fn main() {
    for _ in 0..10 {
        let _fib = timed!(compute_fib(20));
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
