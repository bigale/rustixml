//! Debug star-test to understand why it fails

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    let ixml = r#"word: letter*.
letter: ["a"-"z"]."#;

    println!("=== Parsing iXML Grammar ===\n");
    let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

    println!("Grammar AST:");
    println!("{:#?}\n", ast);

    println!("=== Converting to Earlgrey ===\n");
    let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");

    println!("\n=== Building Grammar ===\n");
    let grammar = builder.into_grammar("word").expect("Failed to build grammar");

    println!("✅ Grammar built successfully!\n");

    // Debug: Print grammar rules
    println!("=== Grammar Debug Info ===");
    println!("Grammar: {:#?}\n", grammar);

    println!("=== Testing Parse ===\n");
    let parser = EarleyParser::new(grammar);

    let test_inputs = vec![
        "",
        "a",
        "hello",
    ];

    for input in test_inputs {
        println!("Input: {:?}", input);
        let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
        println!("Tokens: {:?}", tokens);

        match parser.parse(tokens.iter().map(|s| s.as_str())) {
            Ok(trees) => {
                println!("  ✅ Parse succeeded!");
                println!("  Trees: {:?}\n", trees);
            }
            Err(e) => {
                println!("  ❌ Parse failed: {:?}\n", e);
            }
        }
    }
}
