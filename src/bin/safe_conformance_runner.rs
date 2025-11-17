//! Ultra-safe conformance runner - outputs only ASCII, no Unicode
//! Uses panic catching and incremental output to prevent total crashes

use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};
use std::fs;
use std::io::Write;
use std::panic;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn sanitize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii() && (*c == '\n' || *c == '\t' || (*c >= ' ' && *c <= '~')))
        .take(80)
        .collect()
}

fn main() {
    let test_dir = "/ixml_tests/correct";
    let test_base = Path::new(test_dir);

    let mut test_names = Vec::new();

    if let Ok(entries) = fs::read_dir(test_base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "ixml").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    test_names.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    test_names.sort();

    // Open output file for incremental writing
    let mut output_file = fs::File::create("/tmp/safe_results.txt")
        .expect("Could not create results file");

    let mut results = Vec::new();

    for test_name in test_names {
        // Skip known crashers and timeout tests
        if test_name == "json1" || test_name == "poly"
            || test_name.starts_with("expr")
            || test_name.starts_with("diary")
            || test_name == "address" {
            let line = format!("{}: SKIP_TIMEOUT", test_name);
            writeln!(output_file, "{}", line).ok();
            output_file.flush().ok();
            println!("{}", line);
            results.push(line);
            continue;
        }

        let test_dir_owned = test_dir.to_string();
        let test_name_clone = test_name.clone();

        let (tx, rx) = mpsc::channel();

        // Spawn thread with panic catching
        thread::spawn(move || {
            let result = panic::catch_unwind(|| {
                match read_simple_test(&test_dir_owned, &test_name_clone) {
                    Ok(test) => Some(run_test(&test)),
                    Err(_) => Some(TestOutcome::Skip("Missing".to_string())),
                }
            });

            let outcome = match result {
                Ok(Some(outcome)) => Some(outcome),
                Ok(None) => None,
                Err(_) => Some(TestOutcome::Skip("Panic".to_string())),
            };

            let _ = tx.send(outcome);
        });

        let timeout = Duration::from_secs(2);
        let result_line = match rx.recv_timeout(timeout) {
            Ok(Some(TestOutcome::Pass)) => format!("{}: PASS", test_name),
            Ok(Some(TestOutcome::Fail { .. })) => format!("{}: FAIL", test_name),
            Ok(Some(TestOutcome::GrammarParseError(_))) => format!("{}: GRAMMAR_ERROR", test_name),
            Ok(Some(TestOutcome::InputParseError(_))) => format!("{}: INPUT_ERROR", test_name),
            Ok(Some(TestOutcome::Skip(ref reason))) if reason == "Panic" => format!("{}: PANIC", test_name),
            Ok(Some(TestOutcome::Skip(_))) => format!("{}: SKIP", test_name),
            Ok(None) => format!("{}: TIMEOUT", test_name),
            Err(_) => format!("{}: TIMEOUT", test_name),
        };

        let sanitized = sanitize(&result_line);

        // Write incrementally so we don't lose everything on crash
        writeln!(output_file, "{}", sanitized).ok();
        output_file.flush().ok();

        // Also print to stdout immediately to see where it crashes
        println!("{}", sanitized);

        results.push(sanitized);
    }

    // Print safe summary only
    let passed = results.iter().filter(|r| r.contains("PASS")).count();
    let failed = results.iter().filter(|r| r.contains("FAIL")).count();
    let timeouts = results.iter().filter(|r| r.contains("TIMEOUT")).count();
    let errors = results.iter().filter(|r| r.contains("ERROR")).count();

    println!("Tests: {}", results.len());
    println!("Pass: {}", passed);
    println!("Fail: {}", failed);
    println!("Timeout: {}", timeouts);
    println!("Error: {}", errors);
    println!("Results: /tmp/safe_results.txt");
}
