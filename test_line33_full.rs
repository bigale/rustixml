#!/usr/bin/env rust-script
//! Test just line 33 (Co line)
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;

fn main() {
    println!("=== Testing Co rule directly ===\n");
    
    let grammar = r#"
        Co: -"Co ", (-[Co], +".")*.
    "#;
    
    let input = "Co \u{E000}";
    
    println!("Input: {:?}", input);
    
    let ast = parse_ixml_grammar(grammar).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(input) {
        Ok(xml) => {
            println!("\n✓ Parse succeeded!");
            println!("Output: {}", xml);
        }
        Err(e) => {
            println!("\n❌ Parse failed: {}", e);
        }
    }
    
    println!("\n=== Now testing with full grammar and line wrapper ===\n");
    
    let grammar2 = r#"
        test: line.
        -line: Co, -#a.
        Co: -"Co ", (-[Co], +".")*.
    "#;
    
    let input2 = "Co \u{E000}\n";
    println!("Input: {:?}", input2);
    
    let ast2 = parse_ixml_grammar(grammar2).expect("Grammar should parse");
    let parser2 = NativeParser::new(ast2);
    
    match parser2.parse(input2) {
        Ok(xml) => {
            println!("\n✓ Parse succeeded!");
            println!("Output: {}", xml);
        }
        Err(e) => {
            println!("\n❌ Parse failed: {}", e);
        }
    }
}
