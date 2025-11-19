/// Test line structure similar to unicode-classes

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Simple choice + newline
    println!("{}", "=".repeat(60));
    println!("Test 1: Choice + newline");
    println!("{}", "=".repeat(60));
    
    let grammar1 = r#"
line: (a; b), newline.
a: "A".
b: "B".
newline: #a.
"#;
    let input1 = "A\n";
    
    test_parse("Test1", grammar1, input1);
    
    // Test 2: Like Lm rule
    println!("\n{}", "=".repeat(60));
    println!("Test 2: Lm-like rule");
    println!("{}", "=".repeat(60));
    
    let grammar2 = r#"
line: lm, newline.
lm: -"Lm ", [Lm]*.
newline: #a.
"#;
    let input2 = "Lm ʰ\n";
    
    test_parse("Test2", grammar2, input2);
    
    // Test 3: With line+ like classes rule
    println!("\n{}", "=".repeat(60));
    println!("Test 3: lines+ (like classes rule)");
    println!("{}", "=".repeat(60));
    
    let grammar3 = r#"
classes: line+.
line: lm, newline.
lm: -"Lm ", [Lm]*.
newline: #a.
"#;
    let input3 = "Lm ʰ\n";
    
    test_parse("Test3", grammar3, input3);
}

fn test_parse(name: &str, grammar_text: &str, input: &str) {
    println!("Grammar:\n{}", grammar_text);
    println!("Input: {:?}", input);
    
    let ast = match parse_ixml_grammar(grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed ({} rules)", ast.rules.len());
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse failed: {}", e);
            return;
        }
    };
    
    let (builder, _transformed_ast) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("✓ Grammar converted");
            result
        }
        Err(e) => {
            println!("✗ Conversion failed: {}", e);
            return;
        }
    };
    
    let start = &ast.rules[0].name;
    let grammar = match builder.into_grammar(start) {
        Ok(g) => {
            println!("✓ Grammar built");
            g
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
            return;
        }
    };
    
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    
    println!("Tokens ({} total): {:?}", tokens.len(), tokens);
    
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("✓ {} PASSED\n", name);
        }
        Err(e) => {
            println!("✗ {} FAILED: {:?}\n", name, e);
        }
    }
}
