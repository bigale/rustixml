use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let dir = "/home/bigale/repos/rustixml/ixml_tests/ambiguous";
    let test_name = "css";

    match read_simple_test(dir, test_name) {
        Ok(test) => {
            println!("Grammar: {} chars", test.grammar.len());
            println!("Input: {} chars", test.input.len());
            
            match run_test(&test) {
                TestOutcome::Pass => println!("\nPASS"),
                TestOutcome::Fail { expected, actual } => {
                    println!("\nFAIL");
                    println!("Expected:\n{}", &expected[..expected.len().min(500)]);
                    println!("\nActual:\n{}", &actual[..actual.len().min(500)]);
                }
                TestOutcome::InputParseError(e) => {
                    println!("\nINPUT_ERROR: {}", &e[..e.len().min(300)]);
                }
                TestOutcome::GrammarParseError(e) => {
                    println!("\nGRAMMAR_ERROR: {}", e);
                }
                TestOutcome::Skip(r) => println!("\nSKIP: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading: {}", e),
    }
}
