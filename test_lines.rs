#!/usr/bin/env rust-script
//! Test line+ pattern
//!
//! ```cargo
//! [dependencies]
//! rustixml = { path = "." }
//! ```

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;

fn main() {
    let grammar = r#"
        lines: line+.
        line: 'a', nl.
        -nl: #a.
    "#;
    
    let input = "a\na\n";
    
    println!("Testing line+ pattern");
    let ast = parse_ixml_grammar(grammar).expect("Grammar should parse");
    let parser = NativeParser::new(ast);
    
    match parser.parse(input) {
        Ok(xml) => {
            println!("✓ Parse succeeded!");
            println!("Output: {}", xml);
        }
        Err(e) => {
            println!("❌ Parse failed: {}", e);
        }
    }
}
