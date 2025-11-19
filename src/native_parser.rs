//! Native iXML interpreter - direct implementation of iXML specification
//!
//! This module implements a recursive descent parser that directly interprets
//! iXML grammar ASTs without translation to an intermediate parser representation.
//! It handles insertion and suppression semantics natively.

use crate::ast::{Alternatives, BaseFactor, Factor, IxmlGrammar, Mark, Repetition, Rule, Sequence};
use crate::input_stream::InputStream;
use crate::parse_context::{ParseContext, ParseError, ParseResult};
use crate::runtime_parser::XmlNode;
use std::collections::HashMap;

/// Native iXML parser that interprets grammar ASTs directly
pub struct NativeParser {
    grammar: IxmlGrammar,
    rules: HashMap<String, Rule>,
}

impl NativeParser {
    /// Create a new native parser from an iXML grammar
    pub fn new(grammar: IxmlGrammar) -> Self {
        // Build rule lookup table for O(1) access
        let rules: HashMap<String, Rule> = grammar
            .rules
            .iter()
            .map(|rule| (rule.name.clone(), rule.clone()))
            .collect();

        NativeParser { grammar, rules }
    }

    /// Parse input text according to the grammar
    ///
    /// Returns XML string on success, or error message on failure
    pub fn parse(&self, input: &str) -> Result<String, String> {
        let mut stream = InputStream::new(input);
        let mut ctx = ParseContext::new();

        // Start with the first rule in the grammar
        let start_rule = self
            .grammar
            .rules
            .first()
            .ok_or_else(|| "Grammar has no rules".to_string())?;

        match self.parse_rule(&mut stream, start_rule, &mut ctx) {
            Ok(result) => {
                // Check if all input was consumed
                if !stream.is_eof() {
                    let remaining = stream.remaining();
                    return Err(format!(
                        "Parse succeeded but input remains: {:?}",
                        remaining.chars().take(20).collect::<String>()
                    ));
                }

                // Convert node to XML string
                if let Some(node) = result.node {
                    Ok(node.to_xml())
                } else {
                    Err("Parse succeeded but produced no output (fully suppressed)".to_string())
                }
            }
            Err(e) => Err(e.format_with_context(input)),
        }
    }

    /// Parse a complete rule
    fn parse_rule(
        &self,
        stream: &mut InputStream,
        rule: &Rule,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        // Check for left recursion
        if !ctx.enter_rule(&rule.name) {
            return Err(ParseError::LeftRecursion {
                rule: rule.name.clone(),
                position: stream.position(),
            });
        }

        let result = self.parse_alternatives(stream, &rule.alternatives, ctx);

        ctx.exit_rule(&rule.name);

        // Apply rule-level mark to result
        result.and_then(|res| Ok(self.apply_rule_mark(res, rule)))
    }

    /// Apply rule-level mark to parse result
    fn apply_rule_mark(&self, mut result: ParseResult, rule: &Rule) -> ParseResult {
        result.node = result.node.map(|node| match rule.mark {
            Mark::Hidden => return None,
            Mark::Attribute => {
                // Convert to attribute
                Some(XmlNode::Attribute {
                    name: rule.name.clone(),
                    value: node.text_content(),
                })
            }
            Mark::Promoted => {
                // Promote content (unwrap element)
                Some(node)
            }
            Mark::None => {
                // Wrap in element
                Some(XmlNode::Element {
                    name: rule.name.clone(),
                    attributes: vec![],
                    children: vec![node],
                })
            }
        }).flatten();

        result
    }

    /// Parse alternatives (choice)
    fn parse_alternatives(
        &self,
        stream: &mut InputStream,
        alts: &Alternatives,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();
        let mut attempts = 0;

        // Try each alternative in order (PEG-style: first match wins)
        for alt in &alts.alts {
            stream.set_position(start_pos); // Reset for each alternative
            attempts += 1;

            match self.parse_sequence(stream, alt, ctx) {
                Ok(result) => return Ok(result),
                Err(_) => continue, // Try next alternative
            }
        }

        // All alternatives failed
        Err(ParseError::NoAlternativeMatched {
            position: start_pos,
            rule: ctx.rule_name.clone(),
            attempts,
        })
    }

    /// Parse a sequence (concatenation)
    fn parse_sequence(
        &self,
        stream: &mut InputStream,
        seq: &Sequence,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();
        let mut children = Vec::new();
        let mut total_consumed = 0;

        // Parse each factor in sequence
        for factor in &seq.factors {
            match self.parse_factor(stream, factor, ctx) {
                Ok(result) => {
                    // Collect non-suppressed nodes
                    if let Some(node) = result.node {
                        children.push(node);
                    }
                    total_consumed += result.consumed;
                }
                Err(e) => {
                    // Sequence failed - backtrack
                    stream.set_position(start_pos);
                    return Err(e);
                }
            }
        }

        // Return sequence as children nodes
        let node = if children.is_empty() {
            None // All suppressed
        } else if children.len() == 1 {
            Some(children.into_iter().next().unwrap())
        } else {
            // Multiple children - wrap in a container element
            Some(XmlNode::Element {
                name: "_sequence".to_string(),
                attributes: vec![],
                children,
            })
        };

        Ok(ParseResult::new(node, total_consumed))
    }

    /// Parse a factor (base + repetition)
    fn parse_factor(
        &self,
        stream: &mut InputStream,
        factor: &Factor,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        // TODO: Implement in Phase 4 (repetitions)
        // For now, just parse the base factor
        match &factor.repetition {
            Repetition::None => self.parse_base_factor(stream, &factor.base, ctx),
            _ => {
                // Placeholder for repetitions
                Err(ParseError::Custom {
                    message: "Repetitions not yet implemented".to_string(),
                    position: stream.position(),
                })
            }
        }
    }

    /// Parse a base factor (terminal, nonterminal, charclass, group)
    fn parse_base_factor(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        match base {
            BaseFactor::Literal { value, insertion, mark } => {
                self.parse_terminal(stream, value, *mark, *insertion)
            }
            BaseFactor::Nonterminal { name, mark } => {
                self.parse_nonterminal(stream, name, *mark, ctx)
            }
            BaseFactor::CharClass { content, negated, mark } => {
                self.parse_charclass(stream, content, *negated, *mark)
            }
            BaseFactor::Group { alternatives } => {
                self.parse_alternatives(stream, alternatives, ctx)
            }
        }
    }

    /// Parse a terminal literal
    fn parse_terminal(
        &self,
        stream: &mut InputStream,
        _value: &str,
        _mark: Mark,
        _insertion: bool,
    ) -> Result<ParseResult, ParseError> {
        // TODO: Implement in Phase 2
        Err(ParseError::Custom {
            message: "Terminals not yet implemented".to_string(),
            position: stream.position(),
        })
    }

    /// Parse a character class
    fn parse_charclass(
        &self,
        stream: &mut InputStream,
        _content: &str,
        _negated: bool,
        _mark: Mark,
    ) -> Result<ParseResult, ParseError> {
        // TODO: Implement in Phase 2
        Err(ParseError::Custom {
            message: "Character classes not yet implemented".to_string(),
            position: stream.position(),
        })
    }

    /// Parse a nonterminal (rule reference)
    fn parse_nonterminal(
        &self,
        stream: &mut InputStream,
        _name: &str,
        _mark: Mark,
        _ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        // TODO: Implement in Phase 2
        Err(ParseError::Custom {
            message: "Nonterminals not yet implemented".to_string(),
            position: stream.position(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        use crate::grammar_ast::parse_ixml_grammar;

        let grammar_text = "test: 'hello'.";
        let grammar = parse_ixml_grammar(grammar_text).expect("Grammar should parse");
        let parser = NativeParser::new(grammar);

        assert_eq!(parser.rules.len(), 1);
        assert!(parser.rules.contains_key("test"));
    }

    #[test]
    fn test_empty_grammar() {
        let grammar = IxmlGrammar::new(vec![]);
        let parser = NativeParser::new(grammar);

        let result = parser.parse("anything");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no rules"));
    }
}
