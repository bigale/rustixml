use rustixml::testsuite_utils::read_simple_test;
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "expr";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("=== Debug expr symbols ===");
            println!("Grammar:\n{}\n", test.grammar);

            match parse_ixml_grammar(&test.grammar) {
                Ok(ast) => {
                    println!("Parsed AST with {} rules:", ast.rules.len());
                    for rule in &ast.rules {
                        println!("  Rule: {}", rule.name);
                    }
                    println!();

                    match ast_to_earlgrey(&ast) {
                        Ok(grammar_builder) => {
                            println!("Grammar builder created successfully");

                            // Try to build with better error handling
                            match grammar_builder.into_grammar("test") {
                                Ok(_grammar) => {
                                    println!("Grammar built successfully!");
                                }
                                Err(e) => {
                                    println!("Grammar build error: {}", e);
                                    println!("\nError type: This indicates a symbol was referenced but never declared.");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Conversion error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Parse error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading test: {}", e);
        }
    }
}
