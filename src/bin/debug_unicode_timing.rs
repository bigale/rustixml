use rustixml::testsuite_utils::read_simple_test;
use std::time::Instant;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "unicode-classes";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("Grammar length: {}", test.grammar.len());
            println!("Input length: {}", test.input.len());
            
            // Time the grammar parsing
            let start = Instant::now();
            let ast = rustixml::grammar_ast::parse_ixml_grammar(&test.grammar);
            let parse_time = start.elapsed();
            println!("Grammar parsing: {:?}", parse_time);
            
            if let Ok(ast) = ast {
                // Time the Earley grammar construction
                let start = Instant::now();
                let builder_result = rustixml::runtime_parser::ast_to_earlgrey(&ast);
                let earley_time = start.elapsed();
                println!("Earley grammar building: {:?}", earley_time);
                
                if let Ok(builder) = builder_result {
                    let start_symbol = &ast.rules[0].name;
                    let start = Instant::now();
                    let grammar_result = builder.0.into_grammar(start_symbol);
                    let into_time = start.elapsed();
                    println!("Grammar finalization: {:?}", into_time);
                    
                    if grammar_result.is_ok() {
                        println!("Grammar built successfully!");
                        println!("Total grammar build time: {:?}", parse_time + earley_time + into_time);
                    } else {
                        println!("Grammar finalization failed: {:?}", grammar_result.err());
                    }
                } else if let Err(e) = builder_result {
                    println!("Earley grammar building failed: {}", e);
                }
            } else {
                println!("Grammar parsing failed: {}", ast.unwrap_err());
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
