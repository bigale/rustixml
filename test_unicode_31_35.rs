#!/usr/bin/env rust-script
//! Test lines 31-35 of unicode-classes
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    println!("=== Testing lines 31-35 of unicode-classes ===\n");
    
    let grammar_text = fs::read_to_string("ixml_tests/correct/unicode-classes.ixml")
        .expect("Failed to read grammar");
    let full_input = fs::read_to_string("ixml_tests/correct/unicode-classes.inp")
        .expect("Failed to read input");
    
    // Get lines 31-35 (0-indexed: 30-34)
    let lines: Vec<&str> = full_input.lines().collect();
    let test_lines: Vec<&str> = lines[30..35].iter().copied().collect();
    let input = test_lines.join("\n") + "\n";
    
    println!("Lines 31-35:");
    for (i, line) in test_lines.iter().enumerate() {
        println!("  {}: {:?}", i+31, line);
    }
    println!();
    
    let ast = parse_ixml_grammar(&grammar_text).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(&input) {
        Ok(xml) => {
            println!("✓ Parse succeeded!");
            println!("\nOutput:");
            println!("{}", xml);
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }
}
