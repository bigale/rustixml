//! Minimal reproduction case for Earlgrey * operator with character class
//!
//! This tests the exact grammar pattern from star-test:
//! word: letter*.
//! letter: ["a"-"z"].

use earlgrey::{GrammarBuilder, EarleyParser};

fn main() {
    println!("=== Minimal Earlgrey Star Test ===\n");

    // Build the grammar
    let mut builder = GrammarBuilder::default();

    // 1. Define terminal: charclass_a_z matches any lowercase letter
    println!("Step 1: Defining terminal 'charclass_a_z' for [a-z]");
    builder = builder.terminal("charclass_a_z", move |c: &str| {
        let ch = c.chars().next().unwrap();
        ch >= 'a' && ch <= 'z'
    });

    // 2. Define nonterminals
    println!("Step 2: Declaring nonterminals: letter, letter_star, word");
    builder = builder.nonterm("letter");
    builder = builder.nonterm("letter_star");
    builder = builder.nonterm("word");

    // 3. Define rule: word -> letter_star (WORD FIRST, like runtime_parser)
    println!("Step 3: Adding rule: word -> letter_star");
    builder = builder.rule("word", &["letter_star"]);

    // 4. Define rules for letter_star (LEFT recursion with epsilon)
    println!("Step 4: Adding rules for letter_star:");
    println!("  letter_star -> ε (epsilon production)");
    builder = builder.rule("letter_star", &[] as &[&str]);
    println!("  letter_star -> letter_star letter (LEFT recursion)");
    builder = builder.rule("letter_star", &["letter_star", "letter"]);

    // 5. Define rule: letter -> charclass_a_z (LETTER LAST, like runtime_parser)
    println!("Step 5: Adding rule: letter -> charclass_a_z");
    builder = builder.rule("letter", &["charclass_a_z"]);

    // 6. Build the grammar
    println!("\nStep 6: Building grammar...");
    let grammar = builder.into_grammar("word");

    match grammar {
        Ok(g) => {
            println!("✅ Grammar built successfully!\n");

            // Test parsing
            let test_inputs = vec![
                "",       // empty input
                "a",      // single character
                "hello",  // multiple characters
            ];

            for input in test_inputs {
                println!("Testing input: {:?}", input);

                // Character-level tokenization
                let tokens: Vec<String> = input.chars().map(|c| c.to_string()).collect();

                println!("  Tokens: {:?}", tokens);

                let parser = EarleyParser::new(g.clone());
                match parser.parse(tokens.iter().map(|s| s.as_str())) {
                    Ok(state) => {
                        println!("  ✅ Parse succeeded!");
                        println!("  Parse state: {:?}\n", state);
                    }
                    Err(e) => {
                        println!("  ❌ Parse failed: {:?}\n", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Grammar build failed: {:?}", e);
        }
    }
}
