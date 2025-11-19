#!/usr/bin/env rust-script
//! Test full unicode-classes test
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    println!("=== Testing unicode-classes ===\n");
    
    let grammar_text = fs::read_to_string("ixml_tests/correct/unicode-classes.ixml")
        .expect("Failed to read grammar");
    let input = fs::read_to_string("ixml_tests/correct/unicode-classes.inp")
        .expect("Failed to read input");
    
    let ast = parse_ixml_grammar(&grammar_text).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(&input) {
        Ok(xml) => {
            println!("✓ Parse succeeded!");
            println!("\nFirst 500 chars of output:");
            println!("{}", xml.chars().take(500).collect::<String>());
            
            // Check if expected output exists
            if let Ok(expected) = fs::read_to_string("ixml_tests/correct/unicode-classes.output.xml") {
                let xml_norm = xml.split_whitespace().collect::<Vec<_>>().join("");
                let expected_norm = expected.split_whitespace().collect::<Vec<_>>().join("");
                
                if xml_norm == expected_norm {
                    println!("\n✅ Output matches expected!");
                } else {
                    println!("\n⚠ Output differs from expected");
                    println!("Expected length: {}", expected_norm.len());
                    println!("Got length:      {}", xml_norm.len());
                    
                    // Find first difference
                    for (i, (e, g)) in expected_norm.chars().zip(xml_norm.chars()).enumerate() {
                        if e != g {
                            println!("First difference at position {}:", i);
                            println!("Expected: {:?}", expected_norm.chars().skip(i).take(50).collect::<String>());
                            println!("Got:      {:?}", xml_norm.chars().skip(i).take(50).collect::<String>());
                            break;
                        }
                    }
                }
            } else {
                println!("\n⚠ No expected output file found");
            }
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }
}
