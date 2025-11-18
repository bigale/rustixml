use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Single character class
    test_grammar("Test 1: Single char class", "s: [\"a\"].", "a");

    // Test 2: Character range
    test_grammar("Test 2: Char range", "s: [\"a\"-\"z\"].", "b");

    // Test 3: One-or-more repetition
    test_grammar("Test 3: letter+", "s: letter+. -letter: [\"a\"-\"z\"].", "abc");

    // Test 4: Sequence (comma separated)
    test_grammar("Test 4: Sequence", "s: a, b. a: [\"a\"]. b: [\"b\"].", "ab");

    // Test 5: Alternatives (semicolon)
    test_grammar("Test 5: Alternatives", "s: a; b. a: [\"a\"]. b: [\"b\"].", "a");

    // Test 6: Nesting
    test_grammar("Test 6: Nesting", "s: expr. expr: letter+. -letter: [\"a\"-\"z\"].", "hi");

    // Test 7: Simplified expr
    test_grammar("Test 7: expr lite",
        "s: name. name: letter+. -letter: [\"a\"-\"z\"].",
        "pi");

    // Test 8: Digits
    test_grammar("Test 8: Digits",
        "s: digit+. -digit: [\"0\"-\"9\"].",
        "123");

    // Test 9: Hidden literal
    test_grammar("Test 9: Hidden literal",
        "s: a, -\"+\", b. a: [\"a\"]. b: [\"b\"].",
        "a+b");

    // Test 10: Two alternatives
    test_grammar("Test 10: Two alternatives",
        "s: a; b. a: [\"a\"]. b: [\"b\"].",
        "b");

    // Test 11: Nested alternatives
    test_grammar("Test 11: expr structure",
        "expr: term; sum. -term: factor. -factor: letter+. -letter: [\"a\"-\"z\"]. sum: expr, \"+\", term.",
        "a+b");

    // Test 12: Marked nonterminal (@)
    test_grammar("Test 12: Marked nonterm",
        "s: @name. name: letter+. -letter: [\"a\"-\"z\"].",
        "hi");

    // Test 13: Digits and letters
    test_grammar("Test 13: id and number",
        "s: id; number. id: letter+. number: digit+. -letter: [\"a\"-\"z\"]. -digit: [\"0\"-\"9\"].",
        "abc");

    // Test 14: Parentheses
    test_grammar("Test 14: Parens",
        "s: \"(\", expr, \")\". expr: letter+. -letter: [\"a\"-\"z\"].",
        "(ab)");

    // Test 15: Full expr grammar (simplified)
    test_grammar("Test 15: Full expr",
        "expression: expr. -expr: term; sum. sum: expr, \"+\", term. -term: factor. -factor: id; number; bracketed. bracketed: \"(\", expr, \")\". id: letter+. number: digit+. -letter: [\"a\"-\"z\"]. -digit: [\"0\"-\"9\"].",
        "a+b");

    // Test 16: Unicode multiplication (×)
    test_grammar("Test 16: Unicode ×",
        "s: a, \"×\", b. a: [\"a\"]. b: [\"b\"].",
        "a×b");

    // Test 17: With multiplication
    test_grammar("Test 17: With prod",
        "expression: expr. -expr: term; sum. sum: expr, \"+\", term. -term: factor; prod. prod: term, \"×\", factor. -factor: id; number. id: letter+. number: digit+. -letter: [\"a\"-\"z\"]. -digit: [\"0\"-\"9\"].",
        "a×b");

    // Test 18: Number in parens
    test_grammar("Test 18: Number in parens",
        "expression: expr. -expr: term. -term: factor. -factor: number; bracketed. bracketed: \"(\", expr, \")\". number: digit+. -digit: [\"0\"-\"9\"].",
        "(10)");

    // Test 19: Full addition and mult
    test_grammar("Test 19: a+b×c",
        "expression: expr. -expr: term; sum. sum: expr, \"+\", term. -term: factor; prod. prod: term, \"×\", factor. -factor: id. id: letter+. -letter: [\"a\"-\"z\"].",
        "a+b");

    // Test 20: Attribute mark on id
    test_grammar("Test 20: @name attribute",
        "s: id. id: @name. name: letter+. -letter: [\"a\"-\"z\"].",
        "hi");

    // Test 21: The actual expr grammar (exact copy)
    test_grammar("Test 21: ACTUAL expr grammar",
        "expression: expr. -expr: term; sum; diff. sum: expr, -\"+\", term. diff: expr, \"-\", term. -term: factor; prod; div. prod: term, -\"×\", factor. div: term, \"÷\", factor. -factor: id; number; bracketed. bracketed: -\"(\", expr, -\")\". id: @name. name: letter+. number: @value. value: digit+. -letter: [\"a\"-\"z\"]. -digit: [\"0\"-\"9\"].",
        "a");
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
                    match builder.into_grammar(start) {
                        Ok(grammar) => {
                            let parser = EarleyParser::new(grammar);
                            let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

                            match parser.parse(tokens.iter().map(|s| s.as_str())) {
                                Ok(_) => println!("PASS\n"),
                                Err(e) => println!("FAIL: {}\n", e),
                            }
                        }
                        Err(e) => println!("Grammar build error: {}\n", e),
                    }
                }
                Err(e) => println!("Conversion error: {}\n", e),
            }
        }
        Err(e) => println!("Parse error: {}\n", e),
    }
}
