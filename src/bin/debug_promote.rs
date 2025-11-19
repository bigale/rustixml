use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Simple promote - factor promotes term
    test_grammar(
        "Test 1: ^term promotion",
        "s: ^term. term: a. a: [\"x\"].",
        "x"
    );

    // Test 2: Hidden element promoted - like expr2's ^expr
    test_grammar(
        "Test 2: ^(-hidden)",
        "s: ^term. -term: a. a: [\"x\"].",
        "x"
    );

    // Test 3: Nested hidden with promote
    test_grammar(
        "Test 3: parenthesized promoted",
        "s: \"(\", ^inner, \")\". -inner: a. a: [\"x\"].",
        "(x)"
    );

    // Test 4: Simplified expr2 pattern
    test_grammar(
        "Test 4: expr2 minimal",
        "s: -factor. -factor: name; \"(\", ^expr, \")\". -expr: sum. sum: name, \"+\", name. name: [\"a\"-\"z\"]+.",
        "(aa+bb)"
    );

    // Test 5: Without promote (should hide)
    test_grammar(
        "Test 5: without promote",
        "s: -factor. -factor: name; \"(\", expr, \")\". -expr: sum. sum: name, \"+\", name. name: [\"a\"-\"z\"]+.",
        "(aa+bb)"
    );

    // Test 6: Separated with text interleaving
    test_grammar(
        "Test 6: separated names",
        "s: sum. sum: name, \"+\", name++\"+\". name: [\"a\"-\"z\"]+.",
        "aa+bb+cc"
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
