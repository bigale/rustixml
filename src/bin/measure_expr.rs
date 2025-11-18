use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};
use std::time::Instant;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "expr";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("=== Measuring expr test (NO TIMEOUT) ===");
            println!("Input: {}", test.input);
            println!("Input length: {} characters\n", test.input.len());
            println!("Starting test...");

            let start = Instant::now();
            let outcome = run_test(&test);
            let elapsed = start.elapsed();

            println!("\n=== RESULT ===");
            println!("Time elapsed: {:?}", elapsed);
            println!("Time in milliseconds: {}", elapsed.as_millis());
            println!("Time in seconds: {:.2}", elapsed.as_secs_f64());

            match outcome {
                TestOutcome::Pass => println!("Status: PASS"),
                TestOutcome::Fail { expected, actual } => {
                    println!("Status: FAIL");
                    println!("Expected:\n{}", expected);
                    println!("Actual:\n{}", actual);
                }
                TestOutcome::GrammarParseError(e) => println!("Status: GRAMMAR ERROR - {}", e),
                TestOutcome::InputParseError(e) => println!("Status: INPUT ERROR - {}", e),
                TestOutcome::Skip(r) => println!("Status: SKIP - {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
