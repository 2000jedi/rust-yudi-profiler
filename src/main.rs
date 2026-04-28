use profiler_macros::{append_file, count, summarise, summarise_csv, timed};

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
    println!();
    summarise_csv!();

    // String literal (&'static str)
    append_file!("profile.csv").expect("string literal");

    // PathBuf
    let path: std::path::PathBuf = "profile.csv".into();
    append_file!(&path).expect("path");

    // Existing file handle
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("profile.csv")
        .expect("open");
    append_file!(&mut f).expect("file handle");
}
