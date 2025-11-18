use rustixml::lexer::Lexer;
use rustixml::grammar_ast::parse_ixml_grammar;
use std::time::Instant;

fn main() {
    // Build up the expr grammar incrementally to find the breaking point

    let test_cases = vec![
        ("1 rule", r#"expression: expr."#),

        ("2 rules", r#"expression: expr.
-expr: term."#),

        ("3 rules (with alt)", r#"expression: expr.
-expr: term; sum.
sum: expr."#),

        ("4 rules (2 alts)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr.
diff: expr."#),

        ("6 rules (full expr+term)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod."#),

        ("8 rules (add prod/div)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor."#),
    ];

    for (name, grammar) in test_cases {
        println!("\n=== Test: {} ===", name);
        println!("Grammar ({} lines)", grammar.lines().count());

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
