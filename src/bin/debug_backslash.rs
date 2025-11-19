use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use earlgrey::EarleyParser;

fn main() {
    // Simplest test: just match a backslash
    let grammar = r#"
        test: char.
        char: "\".
    "#;

    let input = r#"\"#;

    println!("Grammar:\n{}", grammar);
    println!("\nInput: {:?}", input);
    println!("Input chars: {:?}", input.chars().collect::<Vec<_>>());

    match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            println!("\nAST Rules:");
            for rule in &ast.rules {
                println!("  Rule: {} (mark: {:?})", rule.name, rule.mark);
                for (i, alt) in rule.alternatives.alts.iter().enumerate() {
                    println!("    Alt {}: {:?}", i, alt);
                }
            }

            match ast_to_earlgrey(&ast) {
                Ok(builder) => {
                    let start = &ast.rules[0].name;
                    match builder.0.into_grammar(start) {
                        Ok(grammar) => {
                            let parser = EarleyParser::new(grammar);
                            let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

                            println!("\nParsing tokens: {:?}", tokens);

                            match parser.parse(tokens.iter().map(|s| s.as_str())) {
                                Ok(state) => {
                                    println!("Parse succeeded!");
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

    // Test 2: backslash OR other char
    println!("\n\n=== Test 2: Backslash OR 'a' ===");
    let grammar2 = r#"
        test: char.
        char: "\"; "a".
    "#;

    let input2 = "a";
    println!("Input: {:?}", input2);

    match parse_ixml_grammar(grammar2) {
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

    // Test 3: Now try backslash
    println!("\n=== Test 3: Now backslash ===");
    match parse_ixml_grammar(grammar2) {
        Ok(ast) => {
            match ast_to_earlgrey(&ast) {
                Ok(builder) => {
                    let start = &ast.rules[0].name;
                    match builder.0.into_grammar(start) {
                        Ok(grammar) => {
                            let parser = EarleyParser::new(grammar);
                            let input3 = r#"\"#;
                            let tokens: Vec<String> = input3.chars().map(|c| c.to_string()).collect();
                            println!("Input: {:?}", input3);
                            println!("Tokens: {:?}", tokens);

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
}
