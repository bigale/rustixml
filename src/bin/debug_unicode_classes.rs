use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "unicode-classes";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("Grammar:\n{}", test.grammar);
            println!("\nInput: {:?}", test.input);
            println!("Input chars: {:?}", test.input.chars().collect::<Vec<_>>());

            match run_test(&test) {
                TestOutcome::Fail { expected, actual } => {
                    println!("\nFAIL\n\nExpected:\n{}\n\nActual:\n{}", expected, actual);
                }
                TestOutcome::Pass => println!("\nPASS"),
                TestOutcome::GrammarParseError(e) => {
                    println!("\nGrammar error: {}", e);
                }
                TestOutcome::InputParseError(e) => {
                    println!("\nInput error: {}", e);
                }
                TestOutcome::Skip(r) => println!("\nSkip: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
