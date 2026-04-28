use std::cell::RefCell;
use std::collections::HashMap;

/// One slot in the profile table.
/// `total_nanos` is `None` for pure-count entries (from `profile_count!`).
#[derive(Debug, Default)]
pub struct ProfileEntry {
    pub total_nanos: Option<u64>,
    pub call_count: u64,
}

thread_local! {
    static PROFILE_DATA: RefCell<HashMap<&'static str, ProfileEntry>> =
        RefCell::new(HashMap::new());
}

/// Called by code emitted from `profile!(expr)`.
#[inline]
pub fn record(name: &'static str, nanos: u64) {
    PROFILE_DATA.with(|data| {
        let mut map = data.borrow_mut();
        let entry = map.entry(name).or_default();
        *entry.total_nanos.get_or_insert(0) += nanos;
        entry.call_count += 1;
    });
}

/// Called by code emitted from `profile_count!(name)`.
#[inline]
pub fn increment(name: &'static str) {
    PROFILE_DATA.with(|data| {
        let mut map = data.borrow_mut();
        let entry = map.entry(name).or_default();
        entry.call_count += 1;
        // total_nanos stays None — this is a counter-only entry
    });
}

/// Called by code emitted from `summarise!()`.
pub fn print_summary() {
    PROFILE_DATA.with(|data| {
        let map = data.borrow();

        if map.is_empty() {
            println!("[profiler] No data recorded.");
            return;
        }

        // Timed entries sorted by total_nanos descending
        let mut timed: Vec<(&&'static str, &ProfileEntry)> = map
            .iter()
            .filter(|(_, e)| e.total_nanos.is_some())
            .collect();
        timed.sort_by(|a, b| b.1.total_nanos.cmp(&a.1.total_nanos));

        // Counter-only entries sorted by name
        let mut counters: Vec<(&&'static str, &ProfileEntry)> = map
            .iter()
            .filter(|(_, e)| e.total_nanos.is_none())
            .collect();
        counters.sort_by_key(|(name, _)| **name);

        let rows: Vec<(&&str, &ProfileEntry)> = timed.into_iter().chain(counters).collect();

        const W_NAME: usize = 34;
        const W_CALLS: usize = 10;
        const W_TOTAL: usize = 13;
        const W_AVG: usize = 13;

        let h_bar = |l: &str, jn: &str, r: &str| {
            format!(
                "{l}{n}{jn}{c}{jn}{t}{jn}{a}{r}",
                l = l,
                r = r,
                jn = jn,
                n = "─".repeat(W_NAME + 2),
                c = "─".repeat(W_CALLS + 2),
                t = "─".repeat(W_TOTAL + 2),
                a = "─".repeat(W_AVG + 2),
            )
        };

        println!("{}", h_bar("┌", "┬", "┐"));
        println!(
            "│ {n:<W_NAME$} │ {c:>W_CALLS$} │ {t:>W_TOTAL$} │ {a:>W_AVG$} │",
            n = "Name",
            c = "Calls",
            t = "Total (ms)",
            a = "Avg (µs)",
        );
        println!("{}", h_bar("├", "┼", "┤"));

        for (name, entry) in &rows {
            let display_name = if name.len() > W_NAME {
                format!("{}...", &name[..W_NAME - 3])
            } else {
                name.to_string()
            };

            match entry.total_nanos {
                Some(nanos) => {
                    let total_ms = nanos as f64 / 1_000_000.0;
                    let avg_us = if entry.call_count > 0 {
                        (nanos as f64 / entry.call_count as f64) / 1_000.0
                    } else {
                        0.0
                    };
                    println!(
                        "│ {n:<W_NAME$} │ {c:>W_CALLS$} │ {t:>W_TOTAL$.3} │ {a:>W_AVG$.3} │",
                        n = display_name,
                        c = entry.call_count,
                        t = total_ms,
                        a = avg_us,
                    );
                }
                None => {
                    println!(
                        "│ {n:<W_NAME$} │ {c:>W_CALLS$} │ {t:>W_TOTAL$} │ {a:>W_AVG$} │",
                        n = display_name,
                        c = entry.call_count,
                        t = "—",
                        a = "—",
                    );
                }
            }
        }

        println!("{}", h_bar("└", "┴", "┘"));
    });
}

// ─── CSV output ──────────────────────────────────────────────────────────────

use std::io::Write as _;

const CSV_HEADER: &str = "name,calls,total_nanos,avg_nanos";

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Writes the current thread's profile data as CSV data rows (no header) to `out`.
fn write_csv_rows<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    PROFILE_DATA.with(|data| {
        let map = data.borrow();
        let mut entries: Vec<(&&'static str, &ProfileEntry)> = map.iter().collect();
        entries.sort_by(|a, b| match (a.1.total_nanos, b.1.total_nanos) {
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => y.cmp(&x),
            (None, None) => a.0.cmp(b.0),
        });

        for (name, entry) in entries {
            let name_field = csv_escape(name);
            match entry.total_nanos {
                Some(nanos) => {
                    let avg = if entry.call_count > 0 {
                        nanos / entry.call_count
                    } else {
                        0
                    };
                    writeln!(out, "{},{},{},{}", name_field, entry.call_count, nanos, avg)?;
                }
                None => {
                    writeln!(out, "{},{},,", name_field, entry.call_count)?;
                }
            }
        }
        Ok(())
    })
}

/// Called by code emitted from `summarise_csv!()`.
/// Prints the CSV header followed by data rows to stdout.
pub fn print_csv() {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    // Errors writing to stdout are unrecoverable here; mirror println! semantics.
    let _ = writeln!(handle, "{}", CSV_HEADER);
    let _ = write_csv_rows(&mut handle);
}

// ─── append_file!(target) dispatch ───────────────────────────────────────────

/// Targets accepted by `append_file!`. Implemented for path-like types
/// (`&str`, `String`, `&Path`, `PathBuf`, …) and file handles
/// (`&File`, `&mut File`).
pub trait AppendTarget {
    fn append_profile(self) -> std::io::Result<()>;
}

fn append_to_path(path: &std::path::Path) -> std::io::Result<()> {
    use std::fs::OpenOptions;

    let needs_header = match std::fs::metadata(path) {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    };

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    if needs_header {
        writeln!(file, "{}", CSV_HEADER)?;
    }

    write_csv_rows(&mut file)
}

fn append_to_writer<W: std::io::Write>(
    out: &mut W,
    needs_header: bool,
) -> std::io::Result<()> {
    if needs_header {
        writeln!(out, "{}", CSV_HEADER)?;
    }
    write_csv_rows(out)
}

// String-like
impl AppendTarget for &str {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(std::path::Path::new(self))
    }
}
impl AppendTarget for &String {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(std::path::Path::new(self))
    }
}
impl AppendTarget for String {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(std::path::Path::new(&self))
    }
}

// Path-like
impl AppendTarget for &std::path::Path {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(self)
    }
}
impl AppendTarget for &std::path::PathBuf {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(self)
    }
}
impl AppendTarget for std::path::PathBuf {
    fn append_profile(self) -> std::io::Result<()> {
        append_to_path(&self)
    }
}

// File handles. `File: Write` and `&File: Write` both exist in std,
// so we accept owned, shared, and mut references.
impl AppendTarget for &std::fs::File {
    fn append_profile(self) -> std::io::Result<()> {
        let needs_header = self.metadata().map(|m| m.len() == 0).unwrap_or(false);
        let mut writer = self;
        append_to_writer(&mut writer, needs_header)
    }
}
impl AppendTarget for &mut std::fs::File {
    fn append_profile(self) -> std::io::Result<()> {
        let needs_header = self.metadata().map(|m| m.len() == 0).unwrap_or(false);
        append_to_writer(self, needs_header)
    }
}
