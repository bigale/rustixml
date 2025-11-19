/// Test sequences (comma-separated factors)

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Two single chars with comma
    println!("{}", "=".repeat(60));
    println!("Test 1: Two single chars with comma");
    println!("{}", "=".repeat(60));
    
    let grammar1 = r#"test: "a", "b"."#;
    let input1 = "ab";
    
    test_parse("Test1", grammar1, input1);
    
    // Test 2: Single char + char class
    println!("\n{}", "=".repeat(60));
    println!("Test 2: Single char + char class");
    println!("{}", "=".repeat(60));
    
    let grammar2 = r#"test: "a", [xyz]."#;
    let input2 = "ax";
    
    test_parse("Test2", grammar2, input2);
    
    // Test 3: Char class + single char
    println!("\n{}", "=".repeat(60));
    println!("Test 3: Char class + single char");
    println!("{}", "=".repeat(60));
    
    let grammar3 = r#"test: [xyz], "a"."#;
    let input3 = "xa";
    
    test_parse("Test3", grammar3, input3);
    
    // Test 4: Multi-char literal + single char
    println!("\n{}", "=".repeat(60));
    println!("Test 4: Multi-char literal + single char");
    println!("{}", "=".repeat(60));
    
    let grammar4 = r#"test: "ab", "c"."#;
    let input4 = "abc";
    
    test_parse("Test4", grammar4, input4);
}

fn test_parse(name: &str, grammar_text: &str, input: &str) {
    println!("Grammar: {}", grammar_text);
    println!("Input: {:?}", input);
    
    // Parse grammar
    let ast = match parse_ixml_grammar(grammar_text) {
        Ok(ast) => {
            println!("✓ Grammar parsed");
            println!("  Rule '{}' has {} alternatives", ast.rules[0].name, ast.rules[0].alternatives.alts.len());
            println!("  First alternative has {} factors", ast.rules[0].alternatives.alts[0].factors.len());
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
            println!("✓ {} PASSED", name);
        }
        Err(e) => {
            println!("✗ {} FAILED: {:?}", name, e);
        }
    }
}
