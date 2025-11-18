use rustixml::lexer::Lexer;
use rustixml::grammar_ast::parse_ixml_grammar;
use std::time::Instant;

fn main() {
    let grammar = r#"expression: expr.
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
-digit: ["0"-"9"].
"#;

    println!("Testing iXML grammar parsing...");
    println!("Grammar:");
    println!("{}", grammar);
    println!();

    println!("Step 1: Tokenizing...");
    let mut lexer = Lexer::new(grammar);
    let start = Instant::now();
    match lexer.tokenize() {
        Ok(tokens) => {
            let lex_time = start.elapsed();
            println!("✓ Tokenized in {:?} ({} tokens)", lex_time, tokens.len());

            println!("\nStep 2: Parsing tokens to AST with RustyLR...");
            println!("NOTE: This is where it might hang!");
            let start = Instant::now();
            match parse_ixml_grammar(grammar) {
                Ok(ast) => {
                    let parse_time = start.elapsed();
                    println!("✓ Parsed in {:?}", parse_time);
                    println!("AST has {} rules", ast.rules.len());
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
