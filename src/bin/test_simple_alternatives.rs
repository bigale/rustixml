use rustixml::lexer::Lexer;
use rustixml::grammar_ast::parse_ixml_grammar;
use std::time::Instant;

fn main() {
    // Test progressively complex grammars with alternatives

    let test_cases = vec![
        ("Simple (no alts)", r#"expr: term."#),
        ("One alt with ;", r#"expr: term; sum."#),
        ("Two alts with ;", r#"expr: term; sum; diff."#),
        ("Three alts with ;", r#"expr: term; sum; diff; prod."#),
        ("Marked with alts", r#"-expr: term; sum; diff."#),
    ];

    for (name, grammar) in test_cases {
        println!("\n=== Test: {} ===", name);
        println!("Grammar: {}", grammar);

        let mut lexer = Lexer::new(grammar);
        match lexer.tokenize() {
            Ok(tokens) => {
                println!("✓ Tokenized ({} tokens)", tokens.len());

                let start = Instant::now();
                match parse_ixml_grammar(grammar) {
                    Ok(ast) => {
                        let elapsed = start.elapsed();
                        println!("✓ Parsed in {:?} ({} rules)", elapsed, ast.rules.len());
                    }
                    Err(e) => {
                        println!("✗ Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("✗ Tokenize error: {}", e);
            }
        }
    }
}
