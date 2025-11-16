//! Quick test runner for iXML conformance tests

use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_base = "/tmp/ixml-tests";

    // List of test cases to try
    let test_cases = vec![
        "simple",
        "charclass-simple",
        "group-simple",
        "comma-test",
    ];

    println!("Running iXML Conformance Tests\n");
    println!("================================\n");

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    for test_name in test_cases {
        print!("Testing {}: ", test_name);

        match read_simple_test(test_base, test_name) {
            Ok(test) => {
                match run_test(&test) {
                    TestOutcome::Pass => {
                        println!("✅ PASS");
                        passed += 1;
                    }
                    TestOutcome::Fail { expected, actual } => {
                        println!("❌ FAIL");
                        println!("  Expected: {}", expected.chars().take(100).collect::<String>());
                        println!("  Actual:   {}", actual.chars().take(100).collect::<String>());
                        failed += 1;
                    }
                    TestOutcome::GrammarParseError(e) => {
                        println!("❌ Grammar Parse Error: {}", e);
                        errors += 1;
                    }
                    TestOutcome::InputParseError(e) => {
                        println!("❌ Input Parse Error: {}", e);
                        errors += 1;
                    }
                    TestOutcome::Skip(reason) => {
                        println!("⏭️  SKIP: {}", reason);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error reading test: {}", e);
                errors += 1;
            }
        }
    }

    println!("\n================================");
    println!("Results: {} passed, {} failed, {} errors", passed, failed, errors);
}
