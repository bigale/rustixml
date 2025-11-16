//! Quick test runner for iXML conformance tests

use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_base = "/tmp/ixml-tests";

    // List of test cases to try
    let test_cases = vec![
        "simple",
        "charclass-simple",
        "charclass",
        "group-simple",
        "group",
        "comma-test",
        "plus-simple",      // Test + with simple literal (for comparison)
        "test_star_empty",  // Test * with empty input
        "star-one",         // Test * with one character
        "star-two",         // Test * with two characters
        "star-simple",      // Test * with three characters
        "optional-empty",   // Test ? with empty input
        "optional-simple",  // Test ? with one character
        "optional-test",    // Test ? with space (comma-separated sequence)
        "star-test",        // Test * with character class ["a"-"z"] - FIXED!
        "mixed-quotes-single",  // Test charclass with single quotes ['a'-'z']
        "mixed-quotes-double",  // Test charclass with double quotes ["a"-"z"]
        "mixed-quotes-mixed",   // Test charclass with mixed quotes ['a"-"z']
        "escaped-quote-double", // Test escaped double quotes: ""Hello""
        "escaped-quote-single", // Test escaped single quotes: 'Don''t'
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
