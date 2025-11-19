use rustixml::testsuite_utils::{TestCase, run_test, TestOutcome};

fn main() {
    let test = TestCase {
        name: "attr-multipart".to_string(),
        grammar: r#"date: month, -',', -' '*, year . 
@month: 'Feb', 'ruary' .
year: ['0'-'9']+ ."#.to_string(),
        input: "February, 2022".to_string(),
        expected_xml: Some(r#"<date month="February"><year>2022</year></date>"#.to_string()),
        expect_failure: false,
    };

    println!("Grammar:\n{}", test.grammar);
    println!("\nInput: {:?}", test.input);
    println!("Expected: {:?}", test.expected_xml);

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
