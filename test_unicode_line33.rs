#!/usr/bin/env rust-script
//! Test unicode-classes line 33 (the Earley blocker)
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;

fn main() {
    let grammar = r#"
        Co: -"Co ", (-[Co], +".")*.
    "#;
    
    // Line 33: "Co " followed by U+E000 (private use character)
    let input = "Co \u{E000}";
    
    println!("Testing unicode-classes line 33");
    println!("Input: {:?}", input);
    println!("Input bytes: {:?}", input.as_bytes());
    
    let ast = parse_ixml_grammar(grammar).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(input) {
        Ok(xml) => {
            println!("\n✓ Parse succeeded!");
            println!("Output XML: {}", xml);
            
            let normalized = xml.split_whitespace().collect::<Vec<_>>().join("");
            println!("Normalized: {}", normalized);
            
            if normalized == "<Co>.</Co>" {
                println!("\n✅ SUCCESS! Output matches expected: <Co>.</Co>");
            } else {
                println!("\n❌ Output mismatch");
                println!("Expected: <Co>.</Co>");
                println!("Got:      {}", normalized);
            }
        }
        Err(e) => {
            println!("\n❌ Parse failed: {}", e);
        }
    }
}
