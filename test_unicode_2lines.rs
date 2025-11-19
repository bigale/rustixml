#!/usr/bin/env rust-script
//! Test first two lines of unicode-classes
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    println!("=== Testing first 2 lines ===\n");
    
    let grammar_text = fs::read_to_string("ixml_tests/correct/unicode-classes.ixml")
        .expect("Failed to read grammar");
    let full_input = fs::read_to_string("ixml_tests/correct/unicode-classes.inp")
        .expect("Failed to read input");
    
    // Take just first 2 lines
    let input_lines: Vec<&str> = full_input.lines().take(2).collect();
    let input = input_lines.join("\n") + "\n";
    
    println!("Input ({} lines):", input_lines.len());
    for (i, line) in input_lines.iter().enumerate() {
        println!("  {}: {:?}", i+1, line);
    }
    
    let ast = parse_ixml_grammar(&grammar_text).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(&input) {
        Ok(xml) => {
            println!("\n✓ Parse succeeded!");
            println!("Output:");
            println!("{}", xml);
        }
        Err(e) => {
            println!("\n❌ Parse failed: {}", e);
        }
    }
}
