use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let tests = vec![
        ("ixml_tests/syntax", "elem1"),
        ("ixml_tests/ambiguous", "css"),
        ("ixml_tests/ambiguous", "else"),
        ("ixml_tests/ixml", "ixml"),
        ("ixml_tests/ixml", "ixml-spaces"),
        ("ixml_tests/ixml", "ixml3"),
    ];

    for (dir, test_name) in tests {
        println!("\n{}", "=".repeat(60));
        println!("Test: {}/{}", dir, test_name);
        println!("{}", "=".repeat(60));
        
        let full_path = format!("/home/bigale/repos/rustixml/{}", dir);
        match read_simple_test(&full_path, test_name) {
            Ok(test) => {
                println!("Grammar length: {} chars", test.grammar.len());
                println!("Input length: {} chars", test.input.len());
                
                match run_test(&test) {
                    TestOutcome::InputParseError(e) => {
                        println!("INPUT_ERROR: {}", &e[..e.len().min(300)]);
                    }
                    TestOutcome::GrammarParseError(e) => {
                        println!("GRAMMAR_ERROR: {}", &e[..e.len().min(300)]);
                    }
                    TestOutcome::Pass => println!("PASS"),
                    TestOutcome::Fail { expected, actual } => {
                        println!("FAIL");
                        println!("Expected: {}", &expected[..expected.len().min(100)]);
                        println!("Actual: {}", &actual[..actual.len().min(100)]);
                    }
                    TestOutcome::Skip(r) => println!("SKIP: {}", r),
                }
            }
            Err(e) => println!("Error loading: {}", e),
        }
    }
}
