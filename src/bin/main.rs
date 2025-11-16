//! rustixml CLI tool
//!
//! Command-line interface for parsing and testing iXML grammars

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: rustixml <grammar_file>");
        eprintln!("       rustixml test");
        process::exit(1);
    }

    match args[1].as_str() {
        "test" => {
            println!("Running rustixml tests...");
            println!("Phase 2 implementation in progress.");
        }
        filename => {
            match fs::read_to_string(filename) {
                Ok(content) => {
                    println!("Parsing grammar from: {}", filename);
                    println!("Content length: {} bytes", content.len());
                    println!("\nPhase 2: Full parser implementation coming soon.");
                }
                Err(e) => {
                    eprintln!("Error reading file {}: {}", filename, e);
                    process::exit(1);
                }
            }
        }
    }
}
