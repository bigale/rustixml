/// Test hex character matching

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Test 1: Match newline with #a
    println!("{}", "=".repeat(60));
    println!("Test 1: Hex char #a (newline)");
    println!("{}", "=".repeat(60));
    
    let grammar1 = r#"test: #a."#;
    let input1 = "\n";
    
    test_parse("Test1", grammar1, input1);
    
    // Test 2: Match newline with alternatives
    println!("\n{}", "=".repeat(60));
    println!("Test 2: Newline alternatives (#a ; #d)");
    println!("{}", "=".repeat(60));
    
    let grammar2 = r#"test: #a ; #d."#;
    let input2 = "\n";
    
    test_parse("Test2", grammar2, input2);
    
    // Test 3: One or more newlines
    println!("\n{}", "=".repeat(60));
    println!("Test 3: Newlines plus");
    println!("{}", "=".repeat(60));
    
    let grammar3 = r#"test: (#a ; #d)+."#;
    let input3 = "\n";
    
    test_parse("Test3", grammar3, input3);
}

fn test_parse(name: &str, grammar_text: &str, input: &str) {
    println!("Grammar: {}", grammar_text);
    println!("Input: {:?}", input);
    
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
