/// Debug character class terminal creation

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;

fn main() {
    let grammar = r#"test: [xyz]."#;
    
    println!("Grammar: {}", grammar);
    
    let ast = parse_ixml_grammar(grammar).expect("Parse failed");
    println!("Parsed AST");
    
    let (builder, _transformed_ast) = ast_to_earlgrey(&ast).expect("Conversion failed");
    println!("Converted to Earlgrey");
    
    // Try to build the grammar
    let start = &ast.rules[0].name;
    println!("Start symbol: {}", start);
    
    match builder.into_grammar(start) {
        Ok(_g) => {
            println!("✓ Grammar built successfully");
        }
        Err(e) => {
            println!("✗ Grammar build failed: {:?}", e);
        }
    }
}
