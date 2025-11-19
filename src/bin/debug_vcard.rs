use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "vcard";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            match run_test(&test) {
                TestOutcome::Fail { expected, actual } => {
                    println!("FAIL\n\nExpected:\n{}\n\nActual:\n{}", expected, actual);
                }
                TestOutcome::Pass => println!("PASS"),
                TestOutcome::GrammarParseError(e) => {
                    println!("Grammar error: {}", e);
                    println!("\nGrammar was:\n{}", test.grammar);
                }
                TestOutcome::InputParseError(e) => println!("Input error: {}", e),
                TestOutcome::Skip(r) => println!("Skip: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
