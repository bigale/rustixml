/// Test the ACTUAL unicode-classes grammar with ACTUAL input
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;
use std::fs;

fn main() {
    // Load the actual grammar and input files
    let grammar_text = fs::read_to_string("ixml_tests/correct/unicode-classes.ixml")
        .expect("Failed to read grammar file");
    let input = fs::read_to_string("ixml_tests/correct/unicode-classes.inp")
        .expect("Failed to read input file");
    
    println!("Testing with ACTUAL files");
    println!("Grammar: {} bytes, {} lines", grammar_text.len(), grammar_text.lines().count());
    println!("Input: {} bytes, {} lines", input.len(), input.lines().count());
    
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
            println!("✓ Grammar built (start: {})", start);
            g
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("Tokens: {} total", tokens.len());
    println!("First 20 tokens: {:?}", &tokens[..tokens.len().min(20)]);
    
    println!("\nAttempting parse...");
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_trees) => {
            println!("✓ PARSE SUCCESS!");
        }
        Err(e) => {
            println!("✗ Parse failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
