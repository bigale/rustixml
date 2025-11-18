use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::{ast_to_earlgrey, build_xml_forest};
use std::time::Instant;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "expr";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("=== Profiling expr test ===");
            println!("Grammar:");
            println!("{}", test.grammar);
            println!("\nInput: {}", test.input);
            println!("\n--- Phase 1: Parse iXML grammar ---");

            let start = Instant::now();
            match parse_ixml_grammar(&test.grammar) {
                Ok(ast) => {
                    let grammar_parse_time = start.elapsed();
                    println!("✓ Grammar parsed in {:?}", grammar_parse_time);
                    println!("AST has {} rules", ast.rules.len());

                    println!("\n--- Phase 2: Convert to Earley grammar ---");
                    let start = Instant::now();
                    match ast_to_earlgrey(&ast) {
                        Ok(grammar_builder) => {
                            let conversion_time = start.elapsed();
                            println!("✓ Converted to Earley in {:?}", conversion_time);

                            println!("\n--- Phase 3: Build grammar ---");
                            let start = Instant::now();
                            let grammar = grammar_builder.into_grammar(&test.grammar);
                            let build_time = start.elapsed();
                            println!("✓ Grammar built in {:?}", build_time);

                            println!("\n--- Phase 4: Parse input (THIS IS THE SLOW PART) ---");
                            println!("Input length: {} characters", test.input.len());
                            println!("Starting parse...");
                            let start = Instant::now();

                            match grammar.parse(&test.input) {
                                Ok(trees) => {
                                    let parse_time = start.elapsed();
                                    println!("✓ Input parsed in {:?}", parse_time);
                                    println!("Generated {} parse trees", trees.len());

                                    if parse_time > std::time::Duration::from_millis(100) {
                                        println!("⚠ SLOW: Parse took > 100ms for input of {} chars", test.input.len());
                                    }
                                }
                                Err(e) => {
                                    let parse_time = start.elapsed();
                                    println!("✗ Parse failed after {:?}: {}", parse_time, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Conversion failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Grammar parse error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading test: {}", e);
        }
    }
}
