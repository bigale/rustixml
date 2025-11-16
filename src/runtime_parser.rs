//! Runtime parser using Earlgrey
//!
//! This module provides runtime parsing capabilities by converting iXML AST
//! to Earlgrey grammars and using them to parse input.

use earlgrey::{EarleyParser, GrammarBuilder, EarleyForest};
use crate::ast::{IxmlGrammar, Rule, Alternatives, Sequence, Factor, BaseFactor, Repetition};

/// Convert an iXML AST to an Earlgrey grammar
///
/// This is the "translator" that takes our parsed iXML grammar and converts it
/// to Earlgrey's format so we can use it to parse input at runtime.
pub fn ast_to_earlgrey(grammar: &IxmlGrammar) -> Result<GrammarBuilder, String> {
    let mut builder = GrammarBuilder::default();

    // First pass: collect all unique characters from literals and define terminals
    let mut chars_seen = std::collections::HashSet::new();
    collect_literal_chars(grammar, &mut chars_seen);

    for ch in chars_seen {
        let term_name = char_terminal_name(ch);
        builder = builder.terminal(&term_name, move |s: &str| {
            s.len() == 1 && s.chars().next() == Some(ch)
        });
    }

    // Second pass: declare all nonterminals (including multi-char literal sequences)
    for rule in &grammar.rules {
        builder = builder.nonterm(&rule.name);
    }

    // Declare sequence nonterminals for multi-character literals
    declare_literal_sequences(grammar, &mut builder);

    // Third pass: add all the rules
    for rule in &grammar.rules {
        builder = convert_rule(builder, rule)?;
    }

    Ok(builder)
}

/// Collect all unique characters from literal strings in the grammar
fn collect_literal_chars(grammar: &IxmlGrammar, chars: &mut std::collections::HashSet<char>) {
    for rule in &grammar.rules {
        collect_chars_from_alternatives(&rule.alternatives, chars);
    }
}

fn collect_chars_from_alternatives(alts: &Alternatives, chars: &mut std::collections::HashSet<char>) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            collect_chars_from_factor(factor, chars);
        }
    }
}

fn collect_chars_from_factor(factor: &Factor, chars: &mut std::collections::HashSet<char>) {
    if let BaseFactor::Literal { value, .. } = &factor.base {
        for ch in value.chars() {
            chars.insert(ch);
        }
    }
}

/// Declare all sequence nonterminals for multi-character literals
fn declare_literal_sequences(grammar: &IxmlGrammar, builder: &mut GrammarBuilder) {
    let mut sequences_declared = std::collections::HashSet::new();

    for rule in &grammar.rules {
        declare_sequences_from_alternatives(&rule.alternatives, builder, &mut sequences_declared);
    }
}

fn declare_sequences_from_alternatives(alts: &Alternatives, builder: &mut GrammarBuilder, declared: &mut std::collections::HashSet<String>) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            declare_sequences_from_factor(factor, builder, declared);
        }
    }
}

fn declare_sequences_from_factor(factor: &Factor, builder: &mut GrammarBuilder, declared: &mut std::collections::HashSet<String>) {
    if let BaseFactor::Literal { value, .. } = &factor.base {
        if value.len() > 1 {
            let seq_name = format!("lit_seq_{}", value.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"));
            if !declared.contains(&seq_name) {
                let old_builder = std::mem::replace(builder, GrammarBuilder::default());
                *builder = old_builder.nonterm(&seq_name);
                declared.insert(seq_name);
            }
        }
    }
}

/// Convert a single iXML rule to Earlgrey format
fn convert_rule(builder: GrammarBuilder, rule: &Rule) -> Result<GrammarBuilder, String> {
    // For now, we'll ignore marks on the rule itself
    // We'll handle them later when generating XML

    convert_alternatives(builder, &rule.name, &rule.alternatives)
}

/// Convert alternatives (multiple options separated by |)
fn convert_alternatives(mut builder: GrammarBuilder, rule_name: &str, alts: &Alternatives) -> Result<GrammarBuilder, String> {
    for seq in &alts.alts {
        builder = convert_sequence(builder, rule_name, seq)?;
    }
    Ok(builder)
}

/// Create a consistent terminal name for a character
fn char_terminal_name(ch: char) -> String {
    match ch {
        ' ' => "char_SPACE".to_string(),
        '\t' => "char_TAB".to_string(),
        '\n' => "char_NEWLINE".to_string(),
        '\r' => "char_RETURN".to_string(),
        '"' => "char_QUOTE".to_string(),
        '\'' => "char_APOS".to_string(),
        '<' => "char_LT".to_string(),
        '>' => "char_GT".to_string(),
        '&' => "char_AMP".to_string(),
        _ if ch.is_alphanumeric() => format!("char_{}", ch),
        _ => format!("char_U{:04X}", ch as u32),
    }
}

/// Convert a sequence (multiple factors in a row)
fn convert_sequence(mut builder: GrammarBuilder, rule_name: &str, seq: &Sequence) -> Result<GrammarBuilder, String> {
    // Build a list of symbols (terminals and nonterminals) for this production
    let mut symbols = Vec::new();

    for factor in &seq.factors {
        let (new_builder, symbol_name) = convert_factor(builder, factor)?;
        builder = new_builder;
        symbols.push(symbol_name);
    }

    // Add the production rule: rule_name := symbols[0] symbols[1] ...
    builder = builder.rule(rule_name, &symbols.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    Ok(builder)
}

/// Convert a factor (a single grammar element, possibly with repetition)
fn convert_factor(mut builder: GrammarBuilder, factor: &Factor) -> Result<(GrammarBuilder, String), String> {
    // First get the base symbol name
    let (new_builder, base_name) = match &factor.base {
        BaseFactor::Literal { value, insertion: _ } => {
            // For character-level parsing, split literal into individual characters
            if value.len() == 1 {
                // Single character - terminal is already defined, just return the name
                let ch = value.chars().next().unwrap();
                let term_name = char_terminal_name(ch);
                (builder, term_name)
            } else {
                // Multi-character literal - create a sequence rule
                let seq_name = format!("lit_seq_{}", value.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"));

                // Collect character terminal names
                let char_symbols: Vec<String> = value.chars()
                    .map(|ch| char_terminal_name(ch))
                    .collect();

                // Create a rule: seq_name := char1 char2 char3 ...
                let b = builder.rule(&seq_name, &char_symbols.iter().map(|s| s.as_str()).collect::<Vec<_>>());

                (b, seq_name)
            }

            // TODO: Track insertion flag for XML generation
        }
        BaseFactor::Nonterminal { name, mark: _ } => {
            // Just reference the nonterminal by name
            // TODO: Track mark for XML generation
            (builder, name.clone())
        }
        _ => {
            return Err("Character classes and groups not yet supported".to_string());
        }
    };
    builder = new_builder;

    // Handle repetition by creating helper rules
    match factor.repetition {
        Repetition::None => Ok((builder, base_name)),
        Repetition::OneOrMore => {
            // Create a new rule: base_name_plus := base_name | base_name_plus base_name
            let plus_name = format!("{}_plus", base_name);
            builder = builder.nonterm(&plus_name);
            builder = builder.rule(&plus_name, &[&base_name]);
            builder = builder.rule(&plus_name, &[&plus_name, &base_name]);
            Ok((builder, plus_name))
        }
        Repetition::ZeroOrMore => {
            // Create a new rule: base_name_star := ε | base_name_star base_name
            let star_name = format!("{}_star", base_name);
            builder = builder.nonterm(&star_name);
            builder = builder.rule(&star_name, &[] as &[&str]); // epsilon production
            builder = builder.rule(&star_name, &[&star_name, &base_name]);
            Ok((builder, star_name))
        }
        Repetition::Optional => {
            // Create a new rule: base_name_opt := ε | base_name
            let opt_name = format!("{}_opt", base_name);
            builder = builder.nonterm(&opt_name);
            builder = builder.rule(&opt_name, &[] as &[&str]); // epsilon production
            builder = builder.rule(&opt_name, &[&base_name]);
            Ok((builder, opt_name))
        }
    }
}

/// Simple XML node representation
#[derive(Clone, Debug, PartialEq)]
pub enum XmlNode {
    Element { name: String, children: Vec<XmlNode> },
    Text(String),
}

impl XmlNode {
    pub fn to_xml(&self) -> String {
        match self {
            XmlNode::Element { name, children } => {
                if children.is_empty() {
                    format!("<{}/>", name)
                } else {
                    let children_xml: String = children.iter()
                        .map(|c| c.to_xml())
                        .collect();
                    format!("<{}>{}</{}>", name, children_xml, name)
                }
            }
            XmlNode::Text(s) => s.clone(),
        }
    }
}

/// Build an EarleyForest configured for XML generation
/// Returns the forest which can be used with forest.eval(&parse_trees)
pub fn build_xml_forest(grammar: &IxmlGrammar) -> EarleyForest<'static, XmlNode> {
    // Create an EarleyForest to walk the parse tree
    let mut forest = EarleyForest::new(|_symbol, token| {
        // For terminals (leaves), just return the token text
        XmlNode::Text(token.to_string())
    });

    // Register actions for all productions in the grammar
    // Unlike traditional semantic actions, Earlgrey requires actions per production,
    // not per nonterminal. The format is "nonterminal -> symbol1 symbol2 ..."
    for rule in &grammar.rules {
        register_rule_actions(&mut forest, &rule.name, &rule.alternatives);
    }

    // Also register actions for literal sequence nonterminals
    register_literal_sequence_actions(&mut forest, grammar);

    forest
}

/// Helper function to register actions for all productions of a rule
/// Production format: "nonterminal -> symbol1 symbol2 ..."
fn register_rule_actions(
    forest: &mut EarleyForest<'static, XmlNode>,
    rule_name: &str,
    alts: &Alternatives,
) {
    use std::collections::HashSet;
    let mut registered = HashSet::new();

    for seq in &alts.alts {
        // Build the list of symbols for this production
        let mut symbols = Vec::new();

        for factor in &seq.factors {
            let (_builder_placeholder, symbol_name) = get_factor_symbol(factor);
            symbols.push(symbol_name);
        }

        // Create the production string: "rule_name -> symbol1 symbol2 ..."
        let production = if symbols.is_empty() {
            rule_name.to_string()
        } else {
            format!("{} -> {}", rule_name, symbols.join(" "))
        };

        // Register the action for this production
        let rule_name_for_closure = rule_name.to_string();
        forest.action(&production, move |nodes: Vec<XmlNode>| {
            // Wrap all child nodes in an element with the rule name
            XmlNode::Element {
                name: rule_name_for_closure.clone(),
                children: nodes,
            }
        });

        // Also register actions for any helper rules we create for repetition
        for factor in &seq.factors {
            register_repetition_actions(forest, factor, &mut registered);
        }
    }
}

/// Register semantic actions for literal sequence nonterminals
/// These are the aux nonterminals created for multi-character literals like "hello"
fn register_literal_sequence_actions(forest: &mut EarleyForest<'static, XmlNode>, grammar: &IxmlGrammar) {
    let mut literals_seen = std::collections::HashSet::new();

    // Collect all unique multi-character literals
    for rule in &grammar.rules {
        collect_literals_from_alternatives(&rule.alternatives, &mut literals_seen);
    }

    // Register actions for each multi-character literal sequence
    for literal in literals_seen {
        if literal.len() > 1 {
            let seq_name = format!("lit_seq_{}", literal.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"));

            // Build the character symbol list
            let char_symbols: Vec<String> = literal.chars()
                .map(|ch| char_terminal_name(ch))
                .collect();

            // Production: lit_seq_hello -> char_h char_e char_l char_l char_o
            let production = format!("{} -> {}", seq_name, char_symbols.join(" "));

            // Action: concatenate all character texts
            forest.action(&production, |nodes: Vec<XmlNode>| {
                let mut text = String::new();
                for node in nodes {
                    if let XmlNode::Text(t) = node {
                        text.push_str(&t);
                    }
                }
                XmlNode::Text(text)
            });
        }
    }
}

fn collect_literals_from_alternatives(alts: &Alternatives, literals: &mut std::collections::HashSet<String>) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            if let BaseFactor::Literal { value, .. } = &factor.base {
                literals.insert(value.clone());
            }
        }
    }
}

/// Get the symbol name for a factor (matches the logic in ast_to_earlgrey)
fn get_factor_symbol(factor: &Factor) -> ((), String) {
    let base_name = match &factor.base {
        BaseFactor::Literal { value, insertion: _ } => {
            if value.len() == 1 {
                // Single character - use char terminal name
                let ch = value.chars().next().unwrap();
                char_terminal_name(ch)
            } else {
                // Multi-character literal - use sequence name
                format!("lit_seq_{}", value.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"))
            }
        }
        BaseFactor::Nonterminal { name, mark: _ } => name.clone(),
        _ => {
            // TODO: Handle character classes and groups
            "UNSUPPORTED".to_string()
        }
    };

    // Handle repetition by using the helper rule name
    match factor.repetition {
        Repetition::None => ((), base_name),
        Repetition::OneOrMore => ((), format!("{}_plus", base_name)),
        Repetition::ZeroOrMore => ((), format!("{}_star", base_name)),
        Repetition::Optional => ((), format!("{}_opt", base_name)),
    }
}

/// Register actions for repetition helper rules
fn register_repetition_actions(
    forest: &mut EarleyForest<'static, XmlNode>,
    factor: &Factor,
    registered: &mut std::collections::HashSet<String>,
) {
    let base_name = match &factor.base {
        BaseFactor::Literal { value, insertion: _ } => {
            format!("lit_{}", value.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"))
        }
        BaseFactor::Nonterminal { name, mark: _ } => name.clone(),
        _ => return,
    };

    match factor.repetition {
        Repetition::OneOrMore => {
            let plus_name = format!("{}_plus", base_name);
            if !registered.contains(&plus_name) {
                registered.insert(plus_name.clone());

                // Register actions for both productions: base and recursive
                forest.action(&format!("{} -> {}", plus_name, base_name), |nodes| {
                    XmlNode::Element { name: "repeat".to_string(), children: nodes }
                });
                forest.action(&format!("{} -> {} {}", plus_name, plus_name, base_name), |nodes| {
                    XmlNode::Element { name: "repeat".to_string(), children: nodes }
                });
            }
        }
        Repetition::ZeroOrMore => {
            let star_name = format!("{}_star", base_name);
            if !registered.contains(&star_name) {
                registered.insert(star_name.clone());

                // Register actions for epsilon and recursive productions
                forest.action(&star_name, |_nodes| {
                    XmlNode::Element { name: "repeat".to_string(), children: vec![] }
                });
                forest.action(&format!("{} -> {} {}", star_name, star_name, base_name), |nodes| {
                    XmlNode::Element { name: "repeat".to_string(), children: nodes }
                });
            }
        }
        Repetition::Optional => {
            let opt_name = format!("{}_opt", base_name);
            if !registered.contains(&opt_name) {
                registered.insert(opt_name.clone());

                // Register actions for epsilon and base productions
                forest.action(&opt_name, |_nodes| {
                    XmlNode::Element { name: "optional".to_string(), children: vec![] }
                });
                forest.action(&format!("{} -> {}", opt_name, base_name), |nodes| {
                    XmlNode::Element { name: "optional".to_string(), children: nodes }
                });
            }
        }
        Repetition::None => {}
    }
}

/// Simple test to verify Earlgrey works
pub fn test_earlgrey_basic() -> Result<(), String> {
    // Build a simple grammar: expr := "a" | "b"
    let grammar = GrammarBuilder::default()
        .nonterm("expr")
        .terminal("a", move |s| s == "a")
        .terminal("b", move |s| s == "b")
        .rule("expr", &["a"])
        .rule("expr", &["b"])
        .into_grammar("expr")
        .map_err(|e| format!("Grammar error: {:?}", e))?;

    // Create parser
    let parser = EarleyParser::new(grammar);

    // Parse "a"
    let input = vec!["a"];
    let result = parser.parse(input.into_iter());

    match result {
        Ok(_trees) => Ok(()),
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar_ast::parse_ixml_grammar;

    #[test]
    fn test_earlgrey_works() {
        let result = test_earlgrey_basic();
        println!("Earlgrey test result: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ast_to_earlgrey_simple() {
        // Parse a simple iXML grammar: choice: "a" | "b".
        let ixml = r#"choice: "a" | "b"."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        // Convert to Earlgrey
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("choice").expect("Failed to build grammar");

        // Create parser
        let parser = EarleyParser::new(grammar);

        // Parse "a"
        let input = vec!["a"];
        let result = parser.parse(input.into_iter());

        println!("Parse result for 'a': {:?}", result);
        assert!(result.is_ok(), "Failed to parse 'a'");

        // Parse "b"
        let input = vec!["b"];
        let result = parser.parse(input.into_iter());

        println!("Parse result for 'b': {:?}", result);
        assert!(result.is_ok(), "Failed to parse 'b'");
    }

    #[test]
    fn test_ast_to_earlgrey_sequence() {
        // Parse an iXML grammar with a sequence: greeting: "hello" "world".
        let ixml = r#"greeting: "hello" "world"."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        // Convert to Earlgrey
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("greeting").expect("Failed to build grammar");

        // Create parser
        let parser = EarleyParser::new(grammar);

        // Parse "helloworld" character-by-character
        let input_str = "helloworld";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        println!("Parse result for 'helloworld': {:?}", result);
        assert!(result.is_ok(), "Failed to parse 'helloworld'");
    }

    #[test]
    fn test_runtime_parse_simple_grammar() {
        // Now let's try end-to-end: parse an iXML grammar, then use it to parse input

        // Step 1: Define a simple iXML grammar
        let ixml = r#"
            word: letter+.
            letter: "a" | "b" | "c".
        "#;

        // Step 2: Parse the iXML grammar to AST
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        // Step 3: Convert AST to Earlgrey grammar
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("word").expect("Failed to build grammar");

        // Step 4: Create parser for the target language
        let parser = EarleyParser::new(grammar);

        // Step 5: Parse some input using the generated grammar
        let input = vec!["a", "b", "c"];
        let result = parser.parse(input.into_iter());

        println!("Parse result for 'abc': {:?}", result);
        assert!(result.is_ok(), "Failed to parse 'abc' with generated grammar");
    }

    #[test]
    fn test_explore_parse_tree_structure() {
        // Let's examine what Earlgrey's parse trees look like

        let ixml = r#"greeting: "hello"."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("greeting").expect("Failed to build grammar");
        let parser = EarleyParser::new(grammar);

        let input_str = "hello";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                println!("\n=== Parse Trees Structure ===");
                println!("Number of parse trees: {}", trees.0.len());
                for (i, tree) in trees.0.iter().enumerate() {
                    println!("\nTree {}: {:?}", i, tree);
                    println!("Tree {} Debug: {:#?}", i, tree);
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_build_xml_tree() {
        // Test that we can build XML from a parse tree

        let ixml = r#"greeting: "hello"."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("greeting").expect("Failed to build grammar");
        let parser = EarleyParser::new(grammar);

        let input_str = "hello";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                println!("\n=== Building XML Tree ===");

                // Build the forest with semantic actions
                let forest = build_xml_forest(&ast);

                // Evaluate the parse trees to get XML
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        println!("XML Tree: {:#?}", tree);
                        let xml_string = tree.to_xml();
                        println!("XML String: {}", xml_string);
                        assert_eq!(xml_string, "<greeting>hello</greeting>");
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }
}
