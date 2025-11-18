use rustixml::testsuite_utils::read_simple_test;
use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::runtime_parser::ast_to_earlgrey;
use earlgrey::EarleyParser;
use std::time::Instant;

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "expr";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            println!("=== Profiling expr test ===");
            println!("Grammar:");
            println!("{}", test.grammar);
            println!("\nInput: '{}' ({} chars)", test.input, test.input.len());
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
                            // In iXML, the first rule is the start symbol
                            let start_symbol = &ast.rules[0].name;
                            println!("Start symbol: {}", start_symbol);
                            let grammar = match grammar_builder.into_grammar(start_symbol) {
                                Ok(g) => g,
                                Err(e) => {
                                    println!("Grammar build error: {}", e);
                                    return;
                                }
                            };
                            let build_time = start.elapsed();
                            println!("✓ Grammar built in {:?}", build_time);

                            println!("\n--- Phase 4: Create parser ---");
                            let start = Instant::now();
                            let parser = EarleyParser::new(grammar);
                            let parser_time = start.elapsed();
                            println!("✓ Parser created in {:?}", parser_time);

                            println!("\n--- Phase 5: Parse input (THIS IS THE SLOW PART) ---");
                            println!("Input: '{}' ({} chars)", test.input, test.input.len());

                            // Tokenize
                            let tokens: Vec<String> = test.input.chars().map(|c| c.to_string()).collect();
                            println!("Tokenized into {} tokens", tokens.len());

                            println!("Starting parse...");
                            let start = Instant::now();

                            match parser.parse(tokens.iter().map(|s| s.as_str())) {
                                Ok(_parse_trees) => {
                                    let parse_time = start.elapsed();
                                    println!("✓ Input parsed in {:?}", parse_time);

                                    if parse_time > std::time::Duration::from_millis(100) {
                                        println!("\n⚠ SLOW: Parse took > 100ms for input of {} chars", test.input.len());
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
