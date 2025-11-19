use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Simple grammar that uses Unicode categories
    let grammar_text = r#"test: [L]+."#;

    println!("Grammar: {}", grammar_text);

    match parse_ixml_grammar(grammar_text) {
        Ok(grammar) => {
            println!("Grammar parsed OK");
            match ast_to_earlgrey(&grammar) {
                Ok(builder) => {
                    println!("Earley grammar built OK");
                    
                    let start_symbol = &grammar.rules[0].name;
                    let earley_grammar = builder.0.into_grammar(start_symbol).expect("Grammar creation failed");
                    
                    // Try to parse some input
                    let input = "hello";
                    println!("\nParsing input: {:?}", input);
                    
                    let parser = EarleyParser::new(earley_grammar);
                    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
                    
                    match parser.parse(tokens.iter().map(|s| s.as_str())) {
                        Ok(_trees) => {
                            println!("Parse succeeded!");
                        }
                        Err(e) => {
                            println!("Parse failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to build Earley grammar: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to parse grammar: {}", e);
        }
    }
}
