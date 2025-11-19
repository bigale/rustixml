use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use earlgrey::EarleyParser;

fn main() {
    // Simplified json string parsing
    let grammar = r#"
        test: string.
        string: -'"', character*, -'"'.
        -character: ~['"\'; #0-#19];
                   "\", escape.
        escape: ['"\/bfnrt'].
    "#;

    // Test with simple string
    let input = r#""hello""#;

    println!("Grammar:\n{}", grammar);
    println!("\nInput: {}", input);

    match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            // Print character class info from AST
            println!("\nAST Rules:");
            for rule in &ast.rules {
                println!("  Rule: {} (mark: {:?})", rule.name, rule.mark);
            }

            match ast_to_earlgrey(&ast) {
                Ok(builder) => {
                    let start = &ast.rules[0].name;
                    match builder.0.into_grammar(start) {
                        Ok(grammar) => {
                            let parser = EarleyParser::new(grammar);
                            let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

                            match parser.parse(tokens.iter().map(|s| s.as_str())) {
                                Ok(state) => {
                                    let forest = build_xml_forest(&ast);

                                    match forest.eval(&state) {
                                        Ok(tree) => {
                                            let xml = tree.to_xml();
                                            println!("\nOutput:\n{}", xml);
                                        }
                                        Err(e) => println!("Eval error: {}", e),
                                    }
                                }
                                Err(e) => println!("Parse error: {}", e),
                            }
                        }
                        Err(e) => println!("Grammar build error: {}", e),
                    }
                }
                Err(e) => println!("Conversion error: {}", e),
            }
        }
        Err(e) => println!("Grammar parse error: {}", e),
    }

    // Test 2: String with escape
    println!("\n\n=== Test 2: String with escape ===");
    let input2 = r#""\n""#;
    println!("Input: {}", input2);

    match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            match ast_to_earlgrey(&ast) {
                Ok(builder) => {
                    let start = &ast.rules[0].name;
                    match builder.0.into_grammar(start) {
                        Ok(grammar) => {
                            let parser = EarleyParser::new(grammar);
                            let tokens: Vec<String> = input2.chars().map(|c| c.to_string()).collect();

                            match parser.parse(tokens.iter().map(|s| s.as_str())) {
                                Ok(state) => {
                                    let forest = build_xml_forest(&ast);

                                    match forest.eval(&state) {
                                        Ok(tree) => {
                                            let xml = tree.to_xml();
                                            println!("Output:\n{}", xml);
                                        }
                                        Err(e) => println!("Eval error: {}", e),
                                    }
                                }
                                Err(e) => println!("Parse error: {}", e),
                            }
                        }
                        Err(e) => println!("Grammar build error: {}", e),
                    }
                }
                Err(e) => println!("Conversion error: {}", e),
            }
        }
        Err(e) => println!("Grammar parse error: {}", e),
    }

    // Test 3: String that should stop at quote
    println!("\n\n=== Test 3: Two strings ===");
    let input3 = r#""a", "b""#;
    println!("Input: {}", input3);
    // This should only parse the first string "a"
}
