use rustixml::testsuite_utils::{TestCase, run_test, TestOutcome};

fn main() {
    // Minimal tests to isolate text interleaving bug

    // Test 1: Simple separated repetition with visible separator
    test_grammar(
        "Test 1: a++\"+\"",
        "s: a++\"+\". a: [\"0\"-\"9\"].",
        "1+2"
    );

    // Test 2: Named separator
    test_grammar(
        "Test 2: name++sep",
        "s: name++sep. name: [\"a\"-\"z\"]+. sep: \"+\".",
        "aa+bb"
    );

    // Test 3: Like expr2 - nested with promoted
    test_grammar(
        "Test 3: Nested",
        "s: term++\"+\". term: name. name: [\"a\"-\"z\"]+.",
        "a+b+c"
    );

    // Test 4: Very simple - one element then text
    test_grammar(
        "Test 4: a, \"+\"",
        "s: a, \"+\". a: [\"x\"].",
        "x+"
    );

    // Test 5: Multiple text pieces
    test_grammar(
        "Test 5: a, \"+\", a",
        "s: a, \"+\", a. a: [\"x\"].",
        "x+x"
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
        expected_xml: None, // We just want to see actual output
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
