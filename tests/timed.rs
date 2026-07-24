use profiler::AppendTarget;
use profiler_macros::timed;

#[timed]
fn recursive_sum(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    n + recursive_sum(n - 1)
}

#[timed]
fn early_return() -> &'static str {
    return "done";
}

#[test]
fn timed_attribute_records_each_function_invocation() {
    assert_eq!(recursive_sum(4), 10);
    assert_eq!(early_return(), "done");

    let path = std::env::temp_dir().join(format!(
        "rust-yudi-profiler-timed-{}.csv",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    path.clone().append_profile().unwrap();
    let csv = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(path);

    assert!(csv.lines().any(|line| line.starts_with("recursive_sum,5,")));
    assert!(csv.lines().any(|line| line.starts_with("early_return,1,")));
}
