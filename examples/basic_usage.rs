//! Basic usage example of rustixml parser

use rustixml::{parse_ixml_grammar, NativeParser};

fn main() -> Result<(), String> {
    println!("=== rustixml Basic Usage Example ===\n");

    // Example 1: Simple greeting
    println!("Example 1: Greeting Parser");
    let grammar1 = r#"
        greeting: "Hello, ", name, "!".
        name: letter+.
        letter: ["A"-"Z"; "a"-"z"].
    "#;

    let ast1 = parse_ixml_grammar(grammar1)?;
    let parser1 = NativeParser::new(ast1);
    
    let xml1 = parser1.parse("Hello, World!")?;
    println!("Input:  'Hello, World!'");
    println!("Output: {}\n", xml1);

    // Example 2: Date parser
    println!("Example 2: Date Parser");
    let grammar2 = r#"
        date: year, "-", month, "-", day.
        year: digit, digit, digit, digit.
        month: digit, digit.
        day: digit, digit.
        -digit: ["0"-"9"].
    "#;

    let ast2 = parse_ixml_grammar(grammar2)?;
    let parser2 = NativeParser::new(ast2);
    
    let xml2 = parser2.parse("2024-11-20")?;
    println!("Input:  '2024-11-20'");
    println!("Output: {}\n", xml2);

    // Example 3: CSV parser
    println!("Example 3: CSV Parser (single row)");
    let grammar3 = r#"
        csv: row.
        row: field, separator, field, separator, field.
        field: char*.
        separator: ",".
        -char: ~[","].
    "#;

    let ast3 = parse_ixml_grammar(grammar3)?;
    let parser3 = NativeParser::new(ast3);
    
    let xml3 = parser3.parse("Alice,30,NYC")?;
    println!("Input:  'Alice,30,NYC'");
    println!("Output: {}\n", xml3);

    // Example 4: Suppression with -
    println!("Example 4: Suppression with - mark");
    let grammar4 = r#"
        sentence: word, space, word, space, word, ".".
        -space: " ".
        word: letter+.
        letter: ["a"-"z"; "A"-"Z"].
    "#;

    let ast4 = parse_ixml_grammar(grammar4)?;
    let parser4 = NativeParser::new(ast4);
    
    let xml4 = parser4.parse("The quick brown.")?;
    println!("Input:  'The quick brown.'");
    println!("Output: {}\n", xml4);

    // Example 5: Attributes with @
    println!("Example 5: Attributes with @ mark");
    let grammar5 = r#"
        element: "<", @name, ">", content, "</", name, ">".
        name: letter+.
        content: letter*.
        letter: ["a"-"z"; "A"-"Z"].
    "#;

    let ast5 = parse_ixml_grammar(grammar5)?;
    let parser5 = NativeParser::new(ast5);
    
    let xml5 = parser5.parse("<div>Hello</div>")?;
    println!("Input:  '<div>Hello</div>'");
    println!("Output: {}\n", xml5);

    println!("=== All examples completed successfully! ===");
    
    Ok(())
}
