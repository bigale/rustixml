use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    // Read grammar
    let grammar_text = fs::read_to_string("ixml_tests/correct/vcard.ixml")
        .expect("Failed to read vcard.ixml");
    
    let grammar = match parse_ixml_grammar(&grammar_text) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Grammar parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!("Grammar parsed successfully");
    
    // Print eoln rule info
    for rule in &grammar.rules {
        if rule.name == "eoln" {
            println!("eoln rule has {} alternatives", rule.alternatives.alts.len());
        }
    }

    // Read input
    let input = fs::read_to_string("ixml_tests/correct/vcard.inp")
        .expect("Failed to read vcard.inp");
    
    println!("Input length: {} chars", input.len());
    println!("First 50 chars: {:?}", &input[..std::cmp::min(50, input.len())]);

    // Create parser and parse
    let parser = NativeParser::new(grammar);
    match parser.parse(&input) {
        Ok(xml) => {
            println!("\nParse succeeded!");
            println!("{}", xml);
        }
        Err(e) => {
            println!("\nParse failed: {}", e);
        }
    }
}
