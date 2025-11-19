/// Test with first N lines from actual file
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_lines: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(38);
    
    // Load the actual grammar 
    let grammar_text = fs::read_to_string("ixml_tests/correct/unicode-classes.ixml")
        .expect("Failed to read grammar file");
    let full_input = fs::read_to_string("ixml_tests/correct/unicode-classes.inp")
        .expect("Failed to read input file");
    
    // Take only first N lines
    let input: String = full_input.lines().take(num_lines).collect::<Vec<_>>().join("\n") + "\n";
    
    println!("Testing with first {} lines", num_lines);
    println!("Input: {} chars, {} lines", input.chars().count(), input.lines().count());
    
    let ast = match parse_ixml_grammar(&grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed ({} rules)", ast.rules.len());
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let (builder, _transformed) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("✓ Converted to Earley");
            result
        }
        Err(e) => {
            println!("✗ Conversion failed: {}", e);
            std::process::exit(1);
        }
    };
    
    let start = &ast.rules[0].name;
    let grammar = match builder.into_grammar(start) {
        Ok(g) => {
            println!("✓ Grammar built");
            g
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("Tokens: {}", tokens.len());
    println!("First 15 tokens: {:?}", &tokens[..tokens.len().min(15)]);
    
    println!("\nAttempting parse...");
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("✓ PARSE SUCCESS with {} lines!", num_lines);
        }
        Err(e) => {
            println!("✗ Parse failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
