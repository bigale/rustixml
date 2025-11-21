#!/usr/bin/env rust
//! rustixml CLI tool - parse iXML grammars and generate XML
//!
//! Usage: ixml_cli [OPTIONS] [GRAMMAR] INPUT
//!
//! Compatible with markup-blitz CLI interface

use rustixml::grammar_ast::parse_ixml_grammar;
use rustixml::native_parser::NativeParser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let mut indent = false;
    let mut fail_on_error = false;
    let mut timing = false;
    let mut verbose = false;

    let mut positional: Vec<String> = Vec::new();

    // Parse arguments
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--indent" => indent = true,
            "--fail-on-error" => fail_on_error = true,
            "--timing" => timing = true,
            "--verbose" => verbose = true,
            "--help" | "-h" => {
                print_usage(&args[0]);
                process::exit(0);
            }
            _ => positional.push(arg.clone()),
        }
    }

    // Need grammar and input
    if positional.len() < 2 {
        eprintln!("Error: Missing required arguments");
        print_usage(&args[0]);
        process::exit(1);
    }

    let grammar_text = read_arg(&positional[0]);
    let input_text = read_arg(&positional[1]);

    if verbose {
        eprintln!("Grammar: {} bytes", grammar_text.len());
        eprintln!("Input: {} bytes", input_text.len());
    }

    let start = std::time::Instant::now();

    // Parse grammar
    let grammar = match parse_ixml_grammar(&grammar_text) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Grammar parse error: {}", e);
            if fail_on_error {
                process::exit(1);
            } else {
                // Return error document (iXML spec behavior)
                println!("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
                println!("<error type=\"grammar\">{}</error>", escape_xml(&e));
                process::exit(0);
            }
        }
    };

    if timing {
        eprintln!("Grammar parsed in {:?}", start.elapsed());
    }

    // Create parser
    let parser = NativeParser::new(grammar);

    // Parse input
    let parse_start = std::time::Instant::now();
    let xml = match parser.parse(&input_text) {
        Ok(xml) => xml,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            if fail_on_error {
                process::exit(1);
            } else {
                // Return error document
                println!("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
                println!("<error type=\"parse\">{}</error>", escape_xml(&e));
                process::exit(0);
            }
        }
    };

    if timing {
        eprintln!("Input parsed in {:?}", parse_start.elapsed());
        eprintln!("Total time: {:?}", start.elapsed());
    }

    // Output XML
    if indent {
        // TODO: Implement indentation
        println!("<?xml version=\"1.0\" encoding=\"utf-8\"?>{}", xml);
    } else {
        println!("<?xml version=\"1.0\" encoding=\"utf-8\"?>{}", xml);
    }
}

fn read_arg(arg: &str) -> String {
    if arg.starts_with('!') {
        // Literal (preceded by !)
        arg[1..].to_string()
    } else {
        // File path or URL
        fs::read_to_string(arg).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", arg, e);
            process::exit(1);
        })
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} [<OPTION>...] [<GRAMMAR>] <INPUT>", program);
    eprintln!();
    eprintln!("  Compile an Invisible XML grammar, and parse input with the resulting parser.");
    eprintln!();
    eprintln!("  <GRAMMAR>          the grammar (literal, file name or URL), in ixml notation.");
    eprintln!("                     When omitted, the ixml grammar will be used.");
    eprintln!("  <INPUT>            the input (literal, file name or URL).");
    eprintln!();
    eprintln!("  <OPTION>:");
    eprintln!("    --indent         generate resulting xml with indentation.");
    eprintln!("    --fail-on-error  throw an exception instead of returning an error document.");
    eprintln!("    --timing         print timing information.");
    eprintln!("    --verbose        print intermediate results.");
    eprintln!("    --help, -h       show this help message.");
    eprintln!();
    eprintln!("  A literal grammar or input must be preceded by an exclamation point (!).");
    eprintln!("  All inputs must be presented in UTF-8 encoding, and output is written in");
    eprintln!("  UTF-8 as well. Resulting XML goes to standard output, all diagnostics go");
    eprintln!("  to standard error.");
}
