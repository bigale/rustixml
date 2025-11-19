/// Test the suppressed+inserted sequence pattern
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;

fn main() {
    // Simpler test: just the problematic pattern
    let grammar1 = r#"
test: "Co ", item*.
-item: -[Co], +".".
"#;
    
    let private_use_char = '\u{E000}';  // Co character
    let input1 = format!("Co {}", private_use_char);
    
    println!("=== Test 1: Simple sequence with -[Co], +\".\" ===");
    test(grammar1, &input1);
    
    // Test without the insertion
    let grammar2 = r#"
test: "Co ", [Co]*.
"#;
    
    println!("\n=== Test 2: Just [Co]* without insertion ===");
    test(grammar2, &input1);
    
    // Test with inline sequence
    let grammar3 = r#"
test: "Co ", (-[Co], +".")*.
"#;
    
    println!("\n=== Test 3: Inline sequence (-[Co], +\".\")* ===");
    test(grammar3, &input1);
}

fn test(grammar: &str, input: &str) {
    println!("  Input: {:?}", input);
    let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();
    println!("  Tokens: {:?}", &tokens);
    
    let ast = match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            println!("  ✓ Grammar parsed");
            ast
        }
        Err(e) => {
            println!("  ✗ Grammar parse failed: {:?}", e);
            return;
        }
    };
    
    let (builder, _transformed) = match ast_to_earlgrey(&ast) {
        Ok(result) => {
            println!("  ✓ Converted to Earley");
            result
        }
        Err(e) => {
            println!("  ✗ Conversion failed: {}", e);
            return;
        }
    };
    
    let start = &ast.rules[0].name;
    let grammar = match builder.into_grammar(start) {
        Ok(g) => {
            println!("  ✓ Grammar built");
            g
        }
        Err(e) => {
            println!("  ✗ Grammar build failed: {:?}", e);
            return;
        }
    };
    
    let parser = EarleyParser::new(grammar);
    
    match parser.parse(tokens.iter().map(|s| s.as_str())) {
        Ok(_) => {
            println!("  ✓ PARSE SUCCESS!");
        }
        Err(e) => {
            println!("  ✗ Parse failed: {:?}", e);
        }
    }
}
