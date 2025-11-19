#!/usr/bin/env rust-script
//! Test native interpreter with basic test cases
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    let test_cases = vec![
        ("test", "ixml_tests/correct"),
        ("aaa", "ixml_tests/correct"),
        ("hex", "ixml_tests/correct"),
        ("hex1", "ixml_tests/correct"),
        ("range", "ixml_tests/correct"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, dir) in test_cases {
        println!("\n=== Test: {} ===", name);
        
        // Read grammar
        let grammar_path = format!("{}/{}.ixml", dir, name);
        let grammar_text = match fs::read_to_string(&grammar_path) {
            Ok(text) => text,
            Err(e) => {
                println!("❌ Failed to read grammar: {}", e);
                failed += 1;
                continue;
            }
        };

        // Parse grammar
        let grammar = match parse_ixml_grammar(&grammar_text) {
            Ok(g) => g,
            Err(e) => {
                println!("❌ Failed to parse grammar: {}", e);
                failed += 1;
                continue;
            }
        };

        // Create parser
        let parser = NativeParser::new(grammar);

        // Read input
        let input_path = format!("{}/{}.inp", dir, name);
        let input = match fs::read_to_string(&input_path) {
            Ok(text) => text,
            Err(e) => {
                println!("❌ Failed to read input: {}", e);
                failed += 1;
                continue;
            }
        };

        // Parse input
        let result = parser.parse(&input);
        match result {
            Ok(xml) => {
                println!("✓ Parse succeeded");
                println!("Output XML:");
                println!("{}", xml);
                
                // Read expected output
                let expected_path = format!("{}/{}.output.xml", dir, name);
                if let Ok(expected) = fs::read_to_string(&expected_path) {
                    let xml_norm = xml.split_whitespace().collect::<Vec<_>>().join("");
                    let expected_norm = expected.split_whitespace().collect::<Vec<_>>().join("");
                    
                    if xml_norm == expected_norm {
                        println!("✓ Output matches expected");
                        passed += 1;
                    } else {
                        println!("❌ Output mismatch");
                        println!("Expected (normalized):");
                        println!("{}", expected_norm);
                        println!("Got (normalized):");
                        println!("{}", xml_norm);
                        failed += 1;
                    }
                } else {
                    println!("⚠ No expected output file, assuming pass");
                    passed += 1;
                }
            }
            Err(e) => {
                println!("❌ Parse failed: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
}
