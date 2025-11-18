use rustixml::testsuite_utils::{TestCase, run_test, TestOutcome};

fn main() {
    // Test 1: Simple optional
    test_grammar(
        "Test 1: digit?",
        "s: digit+. -digit: [\"0\"-\"9\"].",
        "123"
    );

    // Test 2: Optional group
    test_grammar(
        "Test 2: (digit)?",
        "s: (digit)?. -digit: [\"0\"-\"9\"].",
        "5"
    );

    // Test 3: Sequence with optional part (like number pattern)
    test_grammar(
        "Test 3: digit+, (dot, digit+)?",
        "s: digit+, (\".\", digit+)?. -digit: [\"0\"-\"9\"].",
        "3"
    );

    // Test 4: Same but WITH optional
    test_grammar(
        "Test 4: digit+, (dot, digit+)? - with decimal",
        "s: digit+, (\".\", digit+)?. -digit: [\"0\"-\"9\"].",
        "3.14"
    );

    // Test 5: Named number like expr4
    test_grammar(
        "Test 5: number like expr4",
        "s: number. number: digit+, (\".\", digit+)?. -digit: [\"0\"-\"9\"].",
        "3.14"
    );

    // Test 6: Element + group text
    test_grammar(
        "Test 6: id with optional suffix",
        "s: id, (\"_\", digit+)?. id: [\"a\"-\"z\"]+. -digit: [\"0\"-\"9\"].",
        "foo_123"
    );
}

fn test_grammar(name: &str, grammar_src: &str, input: &str) {
    println!("=== {} ===", name);
    println!("Grammar: {}", grammar_src);
    println!("Input: '{}'", input);

    let test = TestCase {
        name: name.to_string(),
        grammar: grammar_src.to_string(),
        input: input.to_string(),
        expected_xml: None,
        expect_failure: false,
    };

    match run_test(&test) {
        TestOutcome::Fail { expected: _, actual } => {
            println!("Output:\n{}\n", actual);
        }
        TestOutcome::Pass => println!("PASS (unexpected - no expected output)\n"),
        TestOutcome::GrammarParseError(e) => println!("Grammar error: {}\n", e),
        TestOutcome::InputParseError(e) => println!("Parse error: {}\n", e),
        TestOutcome::Skip(r) => println!("Skip: {}\n", r),
    }
}
