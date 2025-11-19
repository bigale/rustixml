/// Debug tool for unicode-classes test

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;
use std::fs;

fn main() {
    let grammar_path = "ixml_tests/correct/unicode-classes.ixml";
    let input_path = "ixml_tests/correct/unicode-classes.inp";

    println!("Loading unicode-classes test...\n");

    let grammar_text = fs::read_to_string(grammar_path)
        .expect("Failed to read grammar file");
    let input = fs::read_to_string(input_path)
        .expect("Failed to read input file");

    println!("Grammar length: {} chars", grammar_text.len());
    println!("Input length: {} chars", input.len());

    println!("\nFirst 300 chars of grammar:");
    let grammar_preview: String = grammar_text.chars().take(300).collect();
    println!("{}", grammar_preview);
    
    println!("\nFirst 300 chars of input:");
    let input_preview: String = input.chars().take(300).collect();
    println!("{}", input_preview);

    // Step 1: Parse the iXML grammar
    println!("\n{}", "=".repeat(60));
    println!("Step 1: Parsing iXML grammar...");
    println!("{}", "=".repeat(60));
    let ast = match parse_ixml_grammar(&grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed successfully");
            println!("Rules: {}", ast.rules.len());
            for (i, rule) in ast.rules.iter().enumerate() {
                println!("  {}: {} ({} alternatives)", i, rule.name, rule.alternatives.alts.len());
            }
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse error: {}", e);
            return;
        }
    };

    // Step 2: Convert to Earlgrey
    println!("\n{}", "=".repeat(60));
    println!("Step 2: Converting to Earlgrey grammar...");
    println!("{}", "=".repeat(60));
    let (builder, _transformed_ast) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("✓ Conversion successful");
            result
        }
        Err(e) => {
            println!("✗ Conversion error: {}", e);
            println!("Error details: {}", e);
            return;
        }
    };

    // Get start symbol
    let start_symbol = &ast.rules[0].name;
    println!("Start symbol: {}", start_symbol);

    // Build grammar
    println!("\n{}", "=".repeat(60));
    println!("Step 3: Building Earlgrey grammar...");
    println!("{}", "=".repeat(60));
    let grammar = match builder.into_grammar(start_symbol) {
        Ok(g) => {
            println!("✓ Grammar built successfully");
            g
        }
        Err(e) => {
            println!("✗ Grammar build error: {:?}", e);
            return;
        }
    };

    // Step 4: Parse input
    println!("\n{}", "=".repeat(60));
    println!("Step 4: Parsing input...");
    println!("{}", "=".repeat(60));
    let parser = EarleyParser::new(grammar);
    
    // Try parsing just the first line to see if it works
    let first_line = input.lines().next().unwrap_or("");
    println!("First line: {:?}", first_line);
    let tokens_first: Vec<String> = first_line.chars().map(|c| c.to_string()).collect();
    println!("First line tokens: {} tokens", tokens_first.len());

    println!("\nTrying to parse first line only...");
    match parser.parse(tokens_first.iter().map(|s| s.as_str())) {
        Ok(_trees) => {
            println!("✓ First line parsed successfully!");
        }
        Err(e) => {
            println!("✗ First line parse error: {:?}", e);
        }
    }

    // Try full input
    println!("\nTrying to parse full input...");
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("Full input: {} tokens", tokens.len());

    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_trees) => {
            println!("✓ Full input parsed successfully!");
        }
        Err(e) => {
            println!("✗ Full input parse error: {:?}", e);
        }
    }
}
