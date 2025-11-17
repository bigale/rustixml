use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = std::env::var("TEST_DIR")
        .unwrap_or_else(|_| "/ixml_tests/correct".to_string());
    let test_name = "arith";

    match read_simple_test(&test_dir, test_name) {
        Ok(test) => {
            println!("=== GRAMMAR ===");
            println!("{}", test.grammar);
            println!("\n=== INPUT ===");
            println!("{:?}", test.input);
            println!();

            match run_test(&test) {
                TestOutcome::Fail { expected, actual } => {
                    println!("FAIL\n\n=== EXPECTED ===\n{}\n\n=== ACTUAL ===\n{}", expected, actual);
                }
                TestOutcome::Pass => println!("PASS"),
                TestOutcome::GrammarParseError(e) => {
                    println!("Grammar error: {}", e);
                }
                TestOutcome::InputParseError(e) => println!("Input error: {}", e),
                TestOutcome::Skip(r) => println!("Skip: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
