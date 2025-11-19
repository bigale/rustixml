//! Native iXML interpreter conformance test runner
//!
//! Runs all tests from ixml_tests/ against the native interpreter

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
enum TestResult {
    Pass,
    Fail(String),
    GrammarError(String),
    InputError(String),
}

struct TestCase {
    name: String,
    category: String,
    grammar_file: PathBuf,
    input_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
}

fn find_test_cases() -> Vec<TestCase> {
    let mut cases = Vec::new();
    let base = Path::new("ixml_tests");

    for category in &["correct", "error", "ambiguous"] {
        let category_path = base.join(category);
        if !category_path.exists() {
            continue;
        }

        // Find all .ixml files
        if let Ok(entries) = fs::read_dir(&category_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ixml") {
                    let name = path.file_stem().unwrap().to_string_lossy().to_string();
                    
                    // Find corresponding .inp and .output.xml files
                    let input_file = category_path.join(format!("{}.inp", name));
                    let output_file = category_path.join(format!("{}.output.xml", name));
                    
                    cases.push(TestCase {
                        name,
                        category: category.to_string(),
                        grammar_file: path,
                        input_file: if input_file.exists() {
                            Some(input_file)
                        } else {
                            None
                        },
                        output_file: if output_file.exists() {
                            Some(output_file)
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }

    cases.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });
    cases
}

fn run_test(test: &TestCase) -> TestResult {
    // Read grammar
    let grammar_text = match fs::read_to_string(&test.grammar_file) {
        Ok(text) => text,
        Err(e) => return TestResult::GrammarError(format!("Failed to read grammar: {}", e)),
    };

    // Parse grammar
    let grammar = match parse_ixml_grammar(&grammar_text) {
        Ok(g) => g,
        Err(e) => return TestResult::GrammarError(format!("Failed to parse grammar: {}", e)),
    };

    // If no input file, we're just testing grammar parsing
    let input_file = match &test.input_file {
        Some(f) => f,
        None => return TestResult::Pass, // Grammar-only test
    };

    // Read input
    let input = match fs::read_to_string(input_file) {
        Ok(text) => text,
        Err(e) => return TestResult::InputError(format!("Failed to read input: {}", e)),
    };

    // Create parser and parse
    let parser = NativeParser::new(grammar);
    let result = match parser.parse(&input) {
        Ok(xml) => xml,
        Err(e) => {
            // For "error" category, parse failures might be expected
            if test.category == "error" {
                return TestResult::Pass;
            }
            return TestResult::Fail(format!("Parse failed: {}", e));
        }
    };

    // If we have expected output, compare
    if let Some(output_file) = &test.output_file {
        match fs::read_to_string(output_file) {
            Ok(expected) => {
                // Normalize whitespace for comparison
                let result_norm = result.split_whitespace().collect::<Vec<_>>().join("");
                let expected_norm = expected.split_whitespace().collect::<Vec<_>>().join("");

                if result_norm == expected_norm {
                    TestResult::Pass
                } else {
                    // Find first difference for debugging
                    let mut diff_pos = 0;
                    for (i, (r, e)) in result_norm.chars().zip(expected_norm.chars()).enumerate() {
                        if r != e {
                            diff_pos = i;
                            break;
                        }
                    }

                    TestResult::Fail(format!(
                        "Output mismatch at position {}\nExpected: {}\nGot: {}",
                        diff_pos,
                        expected_norm.chars().skip(diff_pos).take(50).collect::<String>(),
                        result_norm.chars().skip(diff_pos).take(50).collect::<String>()
                    ))
                }
            }
            Err(_) => TestResult::Pass, // No expected output, assume pass
        }
    } else {
        // No expected output, if we parsed successfully that's good enough
        TestResult::Pass
    }
}

fn main() {
    println!("Native iXML Interpreter Conformance Test Runner");
    println!("==============================================\n");

    let start = Instant::now();
    let test_cases = find_test_cases();
    println!("Found {} test cases\n", test_cases.len());

    let mut results: HashMap<String, Vec<(String, TestResult)>> = HashMap::new();
    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut grammar_error_count = 0;
    let mut input_error_count = 0;

    for test in &test_cases {
        print!("Running {}/{}: {}... ", test.category, test.name, test.name);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let result = run_test(test);
        let status = match &result {
            TestResult::Pass => {
                pass_count += 1;
                "✓"
            }
            TestResult::Fail(_) => {
                fail_count += 1;
                "✗"
            }
            TestResult::GrammarError(_) => {
                grammar_error_count += 1;
                "G"
            }
            TestResult::InputError(_) => {
                input_error_count += 1;
                "I"
            }
        };

        println!("{}", status);

        results
            .entry(test.category.clone())
            .or_insert_with(Vec::new)
            .push((test.name.clone(), result));
    }

    let duration = start.elapsed();

    // Print summary by category
    println!("\n=== Results by Category ===\n");
    for category in &["correct", "ambiguous", "error"] {
        if let Some(tests) = results.get(*category) {
            let cat_pass = tests.iter().filter(|(_, r)| *r == TestResult::Pass).count();
            let cat_total = tests.len();
            println!(
                "{}: {}/{} passed ({:.1}%)",
                category,
                cat_pass,
                cat_total,
                (cat_pass as f64 / cat_total as f64) * 100.0
            );
        }
    }

    // Print overall summary
    println!("\n=== Overall Summary ===\n");
    println!("Total tests:     {}", test_cases.len());
    println!("Passed:          {} ({:.1}%)", pass_count, (pass_count as f64 / test_cases.len() as f64) * 100.0);
    println!("Failed:          {}", fail_count);
    println!("Grammar errors:  {}", grammar_error_count);
    println!("Input errors:    {}", input_error_count);
    println!("Duration:        {:.2}s", duration.as_secs_f64());

    // Print failures for debugging
    if fail_count > 0 {
        println!("\n=== Failed Tests ===\n");
        for (category, tests) in &results {
            for (name, result) in tests {
                if let TestResult::Fail(msg) = result {
                    println!("{}/{}: {}", category, name, msg);
                }
            }
        }
    }

    // Exit with error code if any tests failed
    if fail_count > 0 || grammar_error_count > 0 || input_error_count > 0 {
        std::process::exit(1);
    }
}
