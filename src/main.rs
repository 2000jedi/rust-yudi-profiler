use profiler_macros::{count, summarise, timed};

fn compute_fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => compute_fib(n - 1) + compute_fib(n - 2),
    }
}

fn slow_string_work() -> String {
    (0..1_000).map(|i| i.to_string()).collect::<Vec<_>>().join(", ")
}

fn main() {
    for _ in 0..10 {
        let _fib = timed!(compute_fib(20));
    }

    let _s = timed!(slow_string_work());

    for i in 0..50_u32 {
        if i % 2 == 0 {
            count!("even_iteration");
        } else {
            count!(odd_iteration);
        }
    }

    summarise!();
}
