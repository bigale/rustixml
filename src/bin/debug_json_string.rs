use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use earlgrey::EarleyParser;

fn main() {
    // Minimal JSON string parsing with escapes
    let grammar = r#"
        test: string.
        string: -'"', character*, -'"'.
        -character: ~['"\'; #0-#19];
                   "\", escape.
        escape: ['"\/bfnrt'];
               "u", hex, hex, hex, hex.
        hex: digit; ["A"-"F"; "a"-"f"].
        digit: ["0"-"9"].
    "#;

    // Test with escape sequence
    let input = r#""\uffff""#;

    println!("Grammar:\n{}", grammar);
    println!("\nInput: {}", input);
    println!("Input chars: {:?}", input.chars().collect::<Vec<_>>());

    match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            // Print character class info from AST
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

    // Also test simple string
    println!("\n\n=== Test 2: Simple string ===");
    let input2 = r#""hello""#;
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
}
