use rustixml::testsuite_utils::{TestCase, run_test, TestOutcome};

fn main() {
    // Simplified version of lf test to debug negated character class
    let test = TestCase {
        name: "test_negated".to_string(),
        grammar: r#"input: word.
word: ~[#a]*.
"#.to_string(),
        input: "hello".to_string(),
        expected_xml: Some("<input><word>hello</word></input>".to_string()),
        expect_failure: false,
    };

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
            println!("\nGrammar was:");
            println!("{}", test.grammar);
        }
        TestOutcome::InputParseError(e) => println!("Input error: {}", e),
        TestOutcome::Skip(r) => println!("Skip: {}", r),
    }
}
