# rust-yudi-profiler

A lightweight Rust proc-macro profiler. Three macros, zero external runtime dependencies.

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

## Usage

Add both crates to your `Cargo.toml`:

```toml
[dependencies]
profiler        = { path = "profiler" }
profiler-macros = { path = "profiler-macros" }
```

Then import and use:

```rust
use profiler_macros::{count, summarise, timed};

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
