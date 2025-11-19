/// Simple test to verify multi-character literal parsing works

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Simple multi-char literal
    println!("{}", "=".repeat(60));
    println!("Test 1: Simple multi-char literal");
    println!("{}", "=".repeat(60));
    
    let grammar1 = r#"test: "ab"."#;
    let input1 = "ab";
    
    test_parse("Test1", grammar1, input1);
    
    // Test 2: Hidden multi-char literal
    println!("\n{}", "=".repeat(60));
    println!("Test 2: Hidden multi-char literal");
    println!("{}", "=".repeat(60));
    
    let grammar2 = r#"test: -"ab"."#;
    let input2 = "ab";
    
    test_parse("Test2", grammar2, input2);
    
    // Test 3: Multi-char literal followed by char class
    println!("\n{}", "=".repeat(60));
    println!("Test 3: Multi-char literal + char class");
    println!("{}", "=".repeat(60));
    
    let grammar3 = r#"test: "ab", [xyz]."#;
    let input3 = "abx";
    
    test_parse("Test3", grammar3, input3);
    
    // Test 4: Hidden multi-char literal followed by char class (like unicode-classes Lm rule)
    println!("\n{}", "=".repeat(60));
    println!("Test 4: Hidden literal + char class (Lm pattern)");
    println!("{}", "=".repeat(60));
    
    let grammar4 = r#"test: -"Lm ", [Lm]*."#;
    let input4 = "Lm ";
    
    test_parse("Test4", grammar4, input4);
    
    // Test 5: With actual Lm character
    println!("\n{}", "=".repeat(60));
    println!("Test 5: Hidden literal + Lm char");
    println!("{}", "=".repeat(60));
    
    let grammar5 = r#"test: -"Lm ", [Lm]*."#;
    let input5 = "Lm ʰ";  // ʰ is Lm (modifier letter)
    
    test_parse("Test5", grammar5, input5);
}

fn test_parse(name: &str, grammar_text: &str, input: &str) {
    println!("Grammar: {}", grammar_text);
    println!("Input: {:?}", input);
    
    // Parse grammar
    let ast = match parse_ixml_grammar(grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed");
            ast
        }
        Err(e) => {
            println!("✗ Grammar parse failed: {}", e);
            return;
        }
    };
    
    // Convert to Earlgrey
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
    
    // Build grammar
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
    
    // Parse input
    let parser = EarleyParser::new(grammar);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    
    println!("Tokens: {:?}", tokens);
    
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("✓ {} PASSED - Input parsed successfully", name);
        }
        Err(e) => {
            println!("✗ {} FAILED - Parse error: {:?}", name, e);
        }
    }
}
