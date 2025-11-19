/// Comprehensive test runner for all iXML test categories
/// 
/// This runs through all test categories in ixml_tests/ and reports progress:
/// - correct/ (49 tests) - Basic correctness tests
/// - syntax/ (52 tests) - Grammar syntax tests
/// - ambiguous/ (13 tests) - Ambiguous grammar tests
/// - ixml/ (8 tests) - Parsing ixml grammars
/// - chars/ (4 tests) - Character handling tests
/// - error/ (3 tests) - Error handling tests
/// - parse/ (3 tests) - Parse tests
/// - reference/ (1 test) - Reference test

use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug)]
struct TestResults {
    pass: usize,
    fail: usize,
    grammar_error: usize,
    input_error: usize,
    skip: usize,
    total: usize,
}

impl TestResults {
    fn new() -> Self {
        TestResults {
            pass: 0,
            fail: 0,
            grammar_error: 0,
            input_error: 0,
            skip: 0,
            total: 0,
        }
    }

    fn record(&mut self, outcome: &TestOutcome) {
        self.total += 1;
        match outcome {
            TestOutcome::Pass => self.pass += 1,
            TestOutcome::Fail { .. } => self.fail += 1,
            TestOutcome::GrammarParseError(_) => self.grammar_error += 1,
            TestOutcome::InputParseError(_) => self.input_error += 1,
            TestOutcome::Skip(_) => self.skip += 1,
        }
    }

    fn print_summary(&self, category: &str) {
        println!("\n{} Summary:", category);
        println!("  PASS:          {} ({:.1}%)", self.pass, 100.0 * self.pass as f64 / self.total as f64);
        println!("  FAIL:          {} ({:.1}%)", self.fail, 100.0 * self.fail as f64 / self.total as f64);
        println!("  GRAMMAR_ERROR: {} ({:.1}%)", self.grammar_error, 100.0 * self.grammar_error as f64 / self.total as f64);
        println!("  INPUT_ERROR:   {} ({:.1}%)", self.input_error, 100.0 * self.input_error as f64 / self.total as f64);
        println!("  SKIP:          {} ({:.1}%)", self.skip, 100.0 * self.skip as f64 / self.total as f64);
        println!("  TOTAL:         {}", self.total);
    }
}

fn run_test_category(base_path: &str, category: &str, verbose: bool) -> TestResults {
    let mut results = TestResults::new();
    let category_path = PathBuf::from(base_path).join(category);
    
    println!("\n{}", "=".repeat(60));
    println!("Testing: {}", category);
    println!("{}", "=".repeat(60));

    // Get all .ixml files in the category
    let entries = match fs::read_dir(&category_path) {
        Ok(e) => e,
        Err(_) => {
            println!("Category not found or empty");
            return results;
        }
    };

    let mut test_names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension()? == "ixml" {
                path.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    test_names.sort();

    for test_name in test_names {
        let test = match read_simple_test(&category_path.to_string_lossy(), &test_name) {
            Ok(t) => t,
            Err(e) => {
                if verbose {
                    println!("  {:<30} SKIP ({})", test_name, e);
                }
                results.skip += 1;
                results.total += 1;
                continue;
            }
        };

        let start = Instant::now();
        let outcome = run_test(&test);
        let duration = start.elapsed();

        results.record(&outcome);

        if verbose || !matches!(outcome, TestOutcome::Pass) {
            let status = match &outcome {
                TestOutcome::Pass => "PASS",
                TestOutcome::Fail { .. } => "FAIL",
                TestOutcome::GrammarParseError(_) => "GRAMMAR_ERROR",
                TestOutcome::InputParseError(_) => "INPUT_ERROR",
                TestOutcome::Skip(_) => "SKIP",
            };
            println!("  {:<30} {} ({:?})", test_name, status, duration);

            if verbose {
                match &outcome {
                    TestOutcome::Fail { expected, actual } => {
                        println!("    Expected: {}", &expected[..expected.len().min(100)]);
                        println!("    Actual:   {}", &actual[..actual.len().min(100)]);
                    }
                    TestOutcome::GrammarParseError(e) => {
                        println!("    Error: {}", &e[..e.len().min(200)]);
                    }
                    TestOutcome::InputParseError(e) => {
                        println!("    Error: {}", &e[..e.len().min(200)]);
                    }
                    TestOutcome::Skip(r) => {
                        println!("    Reason: {}", r);
                    }
                    _ => {}
                }
            }
        }
    }

    results.print_summary(category);
    results
}

fn main() {
    let base_path = "/home/bigale/repos/rustixml/ixml_tests";
    let verbose = std::env::args().any(|arg| arg == "-v" || arg == "--verbose");

    println!("\n{}", "=".repeat(60));
    println!("iXML Comprehensive Test Suite Runner");
    println!("{}", "=".repeat(60));

    let mut overall_results = TestResults::new();

    // Test categories in order of importance
    let categories = vec![
        "correct",
        "syntax",
        "ambiguous",
        "ixml",
        "chars",
        "error",
        "parse",
        "reference",
    ];

    for category in &categories {
        let results = run_test_category(base_path, category, verbose);
        overall_results.pass += results.pass;
        overall_results.fail += results.fail;
        overall_results.grammar_error += results.grammar_error;
        overall_results.input_error += results.input_error;
        overall_results.skip += results.skip;
        overall_results.total += results.total;
    }

    println!("\n{}", "=".repeat(60));
    println!("OVERALL RESULTS");
    println!("{}", "=".repeat(60));
    overall_results.print_summary("All Categories");
    
    println!("\nTest categories:");
    println!("  correct:   Basic correctness tests");
    println!("  syntax:    Grammar syntax tests");
    println!("  ambiguous: Ambiguous grammar handling");
    println!("  ixml:      Parsing ixml grammars themselves");
    println!("  chars:     Character handling");
    println!("  error:     Error handling");
    println!("  parse:     Parse tests");
    println!("  reference: Reference implementation tests");
}
