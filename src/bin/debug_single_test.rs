use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::fs;

fn main() {
    let test_name = std::env::args().nth(1).unwrap_or_else(|| "expr".to_string());
    
    let grammar_path = format!("ixml_tests/correct/{}.ixml", test_name);
    let input_path = format!("ixml_tests/correct/{}.inp", test_name);
    
    println!("=== Testing: {} ===\n", test_name);
    
    // Read grammar
    let grammar_text = fs::read_to_string(&grammar_path)
        .expect("Failed to read grammar file");
    println!("Grammar:\n{}\n", grammar_text);
    
    // Parse grammar
    let grammar = match parse_ixml_grammar(&grammar_text) {
        Ok(g) => g,
        Err(e) => {
            println!("GRAMMAR PARSE ERROR: {:?}", e);
            return;
        }
    };
    println!("Grammar parsed successfully.\n");
    println!("Rules: {:?}\n", grammar.rules.iter().map(|r| &r.name).collect::<Vec<_>>());
    
    // Read input
    let input = fs::read_to_string(&input_path)
        .expect("Failed to read input file")
        .trim()
        .to_string();
    println!("Input: {:?} (len={})\n", input, input.len());
    
    // Create parser
    let parser = NativeParser::new(grammar);
    
    // Parse
    println!("Starting parse...\n");
    match parser.parse(&input) {
        Ok(result) => {
            println!("Parse succeeded!");
            println!("Result:\n{}\n", result);
        }
        Err(e) => {
            println!("PARSE ERROR: {}", e);
        }
    }
}
