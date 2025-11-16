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

    // First pass: declare all nonterminals
    // (Earlgrey needs to know about them before we use them)
    for rule in &grammar.rules {
        builder = builder.nonterm(&rule.name);
    }

    // Second pass: add all the rules
    for rule in &grammar.rules {
        builder = convert_rule(builder, rule)?;
    }

    Ok(builder)
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
            // Create a unique terminal name for this literal
            let term_name = format!("lit_{}", value.replace(" ", "_SPACE_").replace("\"", "_QUOTE_"));

            // Define the terminal matcher
            let val = value.clone();
            let b = builder.terminal(&term_name, move |s: &str| s == val);

            // TODO: Track insertion flag for XML generation
            (b, term_name)
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

/// Build a simple XML tree from parse results
/// For now, just concatenates all terminals
/// We take the result of parser.parse() which returns Result<ParseTrees, String>
pub fn build_xml_tree<T>(
    grammar: &IxmlGrammar,
    parse_result: Result<T, String>,
) -> Result<XmlNode, String>
where
    T: AsRef<Vec<std::rc::Rc<earlgrey::EarleyParser>>>  // Placeholder - we'll use generic T and pass through
{
    let parse_trees = parse_result?;

    // Create an EarleyForest to walk the parse tree
    let mut forest = EarleyForest::new(|_symbol, token| {
        // For terminals (leaves), just return the token text
        XmlNode::Text(token.to_string())
    });

    // Register semantic actions for each rule in the grammar
    for rule in &grammar.rules {
        let rule_name = rule.name.clone();
        // Clone rule_name for the move closure
        let rule_name_for_closure = rule_name.clone();
        forest.action(&rule_name, move |nodes: Vec<XmlNode>| {
            // For now, just wrap all child nodes in an element with the rule name
            XmlNode::Element {
                name: rule_name_for_closure.clone(),
                children: nodes,
            }
        });
    }

    // Also register actions for the helper rules we generated for repetitions
    // These are the *_plus, *_star, *_opt rules
    // For now, just concatenate their children
    forest.action("greeting_star", |nodes| XmlNode::Element {
        name: "greeting_star".to_string(),
        children: nodes,
    });

    // Evaluate the parse tree
    forest.eval(&parse_trees)
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

        // Parse "hello" "world"
        let input = vec!["hello", "world"];
        let result = parser.parse(input.into_iter());

        println!("Parse result for 'hello world': {:?}", result);
        assert!(result.is_ok(), "Failed to parse 'hello world'");
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

        let input = vec!["hello"];
        let result = parser.parse(input.into_iter());

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

        let input = vec!["hello"];
        let result = parser.parse(input.into_iter());

        match result {
            Ok(trees) => {
                println!("\n=== Building XML Tree ===");
                let xml_tree = build_xml_tree(&ast, &trees);
                match xml_tree {
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
