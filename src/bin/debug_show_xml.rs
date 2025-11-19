use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use earlgrey::EarleyParser;

fn main() {
    // Test the expr4 pattern that leaks group/repeat_container
    test_grammar(
        "Test: number like expr4",
        "s: number. number: digit+, (\".\", digit+)?. -digit: [\"0\"-\"9\"].",
        "3.14"
    );
}

fn test_grammar(name: &str, grammar_src: &str, input: &str) {
    println!("=== {} ===", name);
    println!("Grammar: {}", grammar_src);
    println!("Input: '{}'", input);

    match parse_ixml_grammar(grammar_src) {
        Ok(ast) => {
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
                                            println!("Output:\n{}\n", xml);
                                        }
                                        Err(e) => println!("Eval error: {}\n", e),
                                    }
                                }
                                Err(e) => println!("Parse error: {}\n", e),
                            }
                        }
                        Err(e) => println!("Grammar build error: {}\n", e),
                    }
                }
                Err(e) => println!("Conversion error: {}\n", e),
            }
        }
        Err(e) => println!("Grammar parse error: {}\n", e),
    }
}
