#!/usr/bin/env rust-script
//! Test repetitions
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;

fn test_case(name: &str, grammar: &str, input: &str, expected_contains: &[&str]) {
    println!("\n=== {} ===", name);
    let ast = parse_ixml_grammar(grammar).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(input) {
        Ok(xml) => {
            println!("✓ Parse succeeded");
            println!("Output: {}", xml);
            
            let normalized = xml.split_whitespace().collect::<Vec<_>>().join("");
            for expected in expected_contains {
                if !normalized.contains(expected) {
                    println!("❌ Missing: {}", expected);
                } else {
                    println!("✓ Contains: {}", expected);
                }
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }
}

fn main() {
    // Test zero-or-more
    test_case(
        "zero-or-more empty",
        "test: 'a'*.",
        "",
        &["<test/>"]
    );
    
    test_case(
        "zero-or-more one",
        "test: 'a'*.",
        "a",
        &["<test>a</test>"]
    );
    
    test_case(
        "zero-or-more many",
        "test: 'a'*.",
        "aaa",
        &["<test>", "a", "a", "a"]
    );
    
    // Test one-or-more
    test_case(
        "one-or-more one",
        "test: 'a'+.",
        "a",
        &["<test>a</test>"]
    );
    
    test_case(
        "one-or-more many",
        "test: 'a'+.",
        "aaa",
        &["<test>", "a", "a", "a"]
    );
    
    // Test optional
    test_case(
        "optional empty",
        "test: 'a'?.",
        "",
        &["<test/>"]
    );
    
    test_case(
        "optional present",
        "test: 'a'?.",
        "a",
        &["<test>a</test>"]
    );
    
    // Test character class repetition
    test_case(
        "digits",
        "test: ['0'-'9']+.",
        "123",
        &["<test>", "1", "2", "3"]
    );
}
