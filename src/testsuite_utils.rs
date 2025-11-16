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

/// Placeholder for running a test
/// TODO: Implement actual parsing and XML generation
pub fn run_test(test: &TestCase) -> TestOutcome {
    // For now, just try to parse the grammar
    use crate::grammar_ast::parse_ixml_grammar;

    match parse_ixml_grammar(&test.grammar) {
        Ok(_grammar) => {
            // TODO: Parse input with grammar
            // TODO: Generate XML from parse tree
            // TODO: Compare with expected output
            TestOutcome::Skip("XML generation not yet implemented".to_string())
        }
        Err(e) => {
            TestOutcome::GrammarParseError(e)
        }
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
}
