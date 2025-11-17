//! Test arith grammar with attribute marker

use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "arith";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("Grammar:\n{}\n", test.grammar);
            println!("Input: {}\n", test.input);
            println!("Expected:\n{:?}\n", test.expected_xml);

            match run_test(&test) {
                TestOutcome::Pass => println!("✅ PASS"),
                TestOutcome::Fail { expected, actual } => {
                    println!("❌ FAIL");
                    println!("Expected:\n{}", expected);
                    println!("\nActual:\n{}", actual);
                }
                TestOutcome::GrammarParseError(e) => println!("❌ Grammar error: {}", e),
                TestOutcome::InputParseError(e) => println!("❌ Input error: {}", e),
                TestOutcome::Skip(reason) => println!("⏭️ SKIP: {}", reason),
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
