use rustixml::testsuite_utils::read_simple_test;
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "expr";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("=== iXML Grammar ===");
            println!("{}\n", test.grammar);

            println!("=== Parsing iXML grammar to AST ===");
            match parse_ixml_grammar(&test.grammar) {
                Ok(ast) => {
                    println!("Grammar parsed successfully!");
                    println!("Number of rules: {}", ast.rules.len());
                    println!();

                    println!("=== Rules ===");
                    for (i, rule) in ast.rules.iter().enumerate() {
                        println!("{}. {} (mark: {:?})", i+1, rule.name, rule.mark);
                        for (j, alt) in rule.alternatives.alts.iter().enumerate() {
                            print!("   Alt {}: ", j+1);
                            for (k, factor) in alt.factors.iter().enumerate() {
                                if k > 0 { print!(", "); }
                                print!("{:?}", factor.base);
                            }
                            println!();
                        }
                    }
                    println!();

                    println!("=== Converting to Earley grammar ===");
                    match ast_to_earlgrey(&ast) {
                        Ok(builder) => {
                            println!("Conversion successful!");
                            println!("NOTE: Cannot inspect GrammarBuilder internals");
                            println!("      but conversion completed without errors");
                        }
                        Err(e) => {
                            println!("Conversion error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Grammar parse error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading test: {}", e);
        }
    }
}
