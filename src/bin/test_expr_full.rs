use rustixml::lexer::Lexer;
use rustixml::grammar_ast::parse_ixml_grammar;
use std::time::Instant;

fn main() {
    // Test progressively adding the missing pieces from expr.ixml

    let test_cases = vec![
        ("8 rules (baseline)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor."#),

        ("9 rules (add factor stub)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: number."#),

        ("10 rules (factor with 2 alts)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: number; id."#),

        ("11 rules (factor with 3 alts)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed."#),

        ("12 rules (add bracketed self-ref)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")"."#),

        ("13 rules (add id/name)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")".
id: @name.
name: letter+."#),

        ("14 rules (add number/value)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")".
id: @name.
name: letter+.
number: @value.
value: digit+."#),

        ("15 rules (add letter charclass)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")".
id: @name.
name: letter+.
number: @value.
value: digit+.
-letter: ["a"-"z"]."#),

        ("FULL expr.ixml (16 rules)", r#"expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")".
id: @name.
name: letter+.
number: @value.
value: digit+.
-letter: ["a"-"z"].
-digit: ["0"-"9"]."#),
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

                        // Stop if parsing took > 1 second (approaching danger zone)
                        if elapsed.as_secs() > 1 {
                            println!("\n⚠ WARNING: Parsing took > 1 second!");
                            println!("   Stopping before next test to avoid hang.");
                            break;
                        }
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
