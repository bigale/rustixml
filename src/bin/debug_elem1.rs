/// Debug tool for elem1 test

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    let grammar_text = r#"a: "a", b, c.
b: "b", c, d.
c: "c", []. {it should block here, since nothing matches}
d: "d"."#;

    let input = "abcd";

    println!("Grammar:\n{}\n", grammar_text);
    println!("Input: {}\n", input);

    // Step 1: Parse the iXML grammar
    println!("Step 1: Parsing iXML grammar...");
    let ast = match parse_ixml_grammar(grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed successfully");
            println!("Rules: {}", ast.rules.len());
            for rule in &ast.rules {
                println!("  - {}: {} alternatives", rule.name, rule.alternatives.alts.len());
            }
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse error: {}", e);
            return;
        }
    };

    // Step 2: Convert to Earlgrey
    println!("\nStep 2: Converting to Earlgrey grammar...");
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
    println!("\nStep 3: Building Earlgrey grammar...");
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
    println!("\nStep 4: Parsing input...");
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_trees) => {
            println!("✓ Parse succeeded - grammar accepts the input");
        }
        Err(e) => {
            println!("✗ Parse error: {:?}", e);
        }
    }
}
