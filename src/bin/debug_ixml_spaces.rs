/// Debug tool for ixml-spaces test - tests parsing ixml grammar with spaces

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;
use std::fs;

fn main() {
    let grammar_path = "ixml_tests/ixml/ixml-spaces.ixml";
    let input_path = "ixml_tests/ixml/ixml-spaces.inp";

    println!("Loading ixml-spaces test...\n");

    let grammar_text = fs::read_to_string(grammar_path)
        .expect("Failed to read grammar file");
    let input = fs::read_to_string(input_path)
        .expect("Failed to read input file");

    println!("Grammar length: {} chars", grammar_text.len());
    println!("Input length: {} chars", input.len());
    println!("First 200 chars of grammar:");
    println!("{}", &grammar_text[..200.min(grammar_text.len())]);
    println!("\nFirst 200 chars of input:");
    println!("{}", &input[..200.min(input.len())]);

    // Step 1: Parse the iXML grammar
    println!("\n{}", "=".repeat(60));
    println!("Step 1: Parsing iXML grammar...");
    println!("{}", "=".repeat(60));
    let ast = match parse_ixml_grammar(&grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed successfully");
            println!("Rules: {}", ast.rules.len());
            for (i, rule) in ast.rules.iter().take(5).enumerate() {
                println!("  {}: {} ({}  alternatives)", i, rule.name, rule.alternatives.alts.len());
            }
            if ast.rules.len() > 5 {
                println!("  ... and {} more rules", ast.rules.len() - 5);
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
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

    println!("Tokenized input: {} tokens", tokens.len());

    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_trees) => {
            println!("✓ Parse succeeded - grammar accepts the input");
        }
        Err(e) => {
            println!("✗ Parse error: {:?}", e);
        }
    }
}
