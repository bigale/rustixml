use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";

    // Test the failing expr variants
    for test_name in &["expr1", "expr2", "expr4"] {
        match read_simple_test(test_dir, test_name) {
            Ok(test) => {
                println!("=== {} ===", test_name);
                println!("Grammar:\n{}", test.grammar);
                println!("\nInput: '{}'\n", test.input);

                match run_test(&test) {
                    TestOutcome::Fail { expected, actual } => {
                        println!("FAIL\n\nExpected:\n{}\n\nActual:\n{}\n", expected, actual);
                    }
                    TestOutcome::Pass => println!("PASS\n"),
                    TestOutcome::GrammarParseError(e) => {
                        println!("Grammar parse error: {}\n", e);
                    }
                    TestOutcome::InputParseError(e) => println!("Input parse error: {}\n", e),
                    TestOutcome::Skip(r) => println!("Skip: {}\n", r),
                }
                println!("{}", "=".repeat(60));
                println!();
            }
            Err(e) => eprintln!("Error loading test {}: {}", test_name, e),
        }
    }
}
