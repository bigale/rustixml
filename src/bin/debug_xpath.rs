use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "xpath";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("Grammar:");
            println!("{}", test.grammar);
            println!("\n---\n");

            match run_test(&test) {
                TestOutcome::Fail { expected, actual } => {
                    println!("FAIL\nExpected:\n{}\nActual:\n{}", expected, actual);
                }
                TestOutcome::Pass => println!("PASS"),
                TestOutcome::GrammarParseError(e) => {
                    println!("Grammar parse error: {}", e);
                }
                TestOutcome::InputParseError(e) => println!("Input parse error: {}", e),
                TestOutcome::Skip(r) => println!("Skip: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
