use rustixml::grammar_parser::parse_ixml_grammar;

fn main() {
    let grammar_text = r#"Sunday: "Sun"; "Sunday"."#;
    
    match parse_ixml_grammar(grammar_text) {
        Ok(grammar) => {
            println!("Grammar parsed successfully!");
            for rule in &grammar.rules {
                println!("Rule: {}", rule.name);
                println!("  Alternatives: {}", rule.alternatives.alts.len());
                for (i, alt) in rule.alternatives.alts.iter().enumerate() {
                    println!("    Alt {}: {} factors", i + 1, alt.factors.len());
                }
            }
        }
        Err(e) => {
            eprintln!("Grammar parse error: {}", e);
            std::process::exit(1);
        }
    }
}
