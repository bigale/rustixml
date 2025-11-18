use rustixml::grammar_ast::parse_ixml_grammar;

fn main() {
    let grammar = r#"-char: letgit; ["!#$%&'*+-/=?^_`{|}~"]."#;

    match parse_ixml_grammar(grammar) {
        Ok(ast) => {
            println!("AST:");
            println!("{:#?}", ast);
        }
        Err(e) => {
            println!("Grammar parse error: {}", e);
        }
    }
}
