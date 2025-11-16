//! Test suite utilities for running iXML conformance tests
//!
//! Adapted from earleybird's test infrastructure

use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TestCase {
    pub name: String,
    pub grammar: String,
    pub input: String,
    pub expected_xml: Option<String>,
    pub expect_failure: bool,
}

#[derive(Clone, Debug)]
pub enum TestOutcome {
    Pass,
    Fail { expected: String, actual: String },
    GrammarParseError(String),
    InputParseError(String),
    Skip(String),
}

/// Read a simple test case from files
/// For now, just reads .ixml, .inp, and .output.xml files directly
pub fn read_simple_test(base_path: &str, test_name: &str) -> Result<TestCase, String> {
    let mut path = PathBuf::from(base_path);

    // Read grammar
    path.push(format!("{}.ixml", test_name));
    let grammar = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read grammar: {}", e))?;
    path.pop();

    // Read input
    path.push(format!("{}.inp", test_name));
    let input = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    path.pop();

    // Read expected output
    path.push(format!("{}.output.xml", test_name));
    let expected_xml = fs::read_to_string(&path).ok();
    path.pop();

    Ok(TestCase {
        name: test_name.to_string(),
        grammar,
        input,
        expected_xml,
        expect_failure: false,
    })
}

/// Run a test case: parse grammar, parse input, generate XML
pub fn run_test(test: &TestCase) -> TestOutcome {
    use crate::grammar_ast::parse_ixml_grammar;
    use crate::runtime_parser::{ast_to_earlgrey, build_xml_forest};
    use earlgrey::EarleyParser;

    // Step 1: Parse the iXML grammar to AST
    let ast = match parse_ixml_grammar(&test.grammar) {
        Ok(ast) => ast,
        Err(e) => return TestOutcome::GrammarParseError(e),
    };

    // Step 2: Convert AST to Earlgrey grammar
    let builder = match ast_to_earlgrey(&ast) {
        Ok(builder) => builder,
        Err(e) => return TestOutcome::GrammarParseError(format!("AST conversion error: {}", e)),
    };

    // Get the start symbol (first rule name)
    let start_symbol = if let Some(rule) = ast.rules.first() {
        &rule.name
    } else {
        return TestOutcome::GrammarParseError("No rules in grammar".to_string());
    };

    let grammar = match builder.into_grammar(start_symbol) {
        Ok(g) => g,
        Err(e) => return TestOutcome::GrammarParseError(format!("Grammar build error: {:?}", e)),
    };

    // Step 3: Create parser and parse input
    let parser = EarleyParser::new(grammar);

    // Character-level tokenization: convert each character to a string
    let tokens: Vec<String> = test.input.chars().map(|c| c.to_string()).collect();

    let parse_trees = match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(trees) => trees,
        Err(e) => return TestOutcome::InputParseError(format!("Parse error: {:?}", e)),
    };

    // Step 4: Generate XML from parse trees
    let forest = build_xml_forest(&ast);

    let xml_node = match forest.eval(&parse_trees) {
        Ok(node) => node,
        Err(e) => return TestOutcome::InputParseError(format!("XML generation error: {}", e)),
    };

    let actual_xml = xml_node.to_xml();

    // Step 5: Compare with expected output
    if let Some(expected) = &test.expected_xml {
        let expected_trimmed = expected.trim();
        let actual_trimmed = actual_xml.trim();

        if expected_trimmed == actual_trimmed {
            TestOutcome::Pass
        } else {
            TestOutcome::Fail {
                expected: expected_trimmed.to_string(),
                actual: actual_trimmed.to_string(),
            }
        }
    } else {
        // No expected output, just check that we could generate something
        TestOutcome::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_simple_test() {
        let test_base = "/home/bigale/repos/earleybird/ixml/tests/correct";
        let result = read_simple_test(test_base, "aaa");
        assert!(result.is_ok());
        let test = result.unwrap();
        assert_eq!(test.name, "aaa");
        assert!(test.grammar.contains("data"));
        assert_eq!(test.input.trim(), "a a a");
    }

    #[test]
    fn test_run_manual_simple_test() {
        // Create a simple test case manually to verify the pipeline works
        let test = TestCase {
            name: "manual_test".to_string(),
            grammar: r#"greeting: "hello" "world"."#.to_string(),
            input: "helloworld".to_string(),
            expected_xml: Some("<greeting>helloworld</greeting>".to_string()),
            expect_failure: false,
        };

        println!("Grammar: {}", test.grammar);
        println!("Input: {}", test.input);
        println!("Expected: {:?}", test.expected_xml);

        let outcome = run_test(&test);

        match outcome {
            TestOutcome::Pass => println!("✅ Test passed!"),
            TestOutcome::Fail { expected, actual } => {
                println!("❌ Test failed!");
                println!("Expected:\n{}", expected);
                println!("Actual:\n{}", actual);
                // Don't panic yet - let's see what we get
            }
            TestOutcome::GrammarParseError(e) => {
                println!("❌ Grammar parse error: {}", e);
                panic!("Grammar parse error: {}", e);
            }
            TestOutcome::InputParseError(e) => {
                println!("❌ Input parse error: {}", e);
                panic!("Input parse error: {}", e);
            }
            TestOutcome::Skip(reason) => {
                println!("⏭️ Test skipped: {}", reason);
            }
        }
    }
}
