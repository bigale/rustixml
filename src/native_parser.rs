//! Native iXML interpreter - direct implementation of iXML specification
//!
//! This module implements a recursive descent parser that directly interprets
//! iXML grammar ASTs without translation to an intermediate parser representation.
//! It handles insertion and suppression semantics natively.

use crate::ast::{Alternatives, BaseFactor, Factor, IxmlGrammar, Mark, Repetition, Rule, Sequence};
use crate::input_stream::InputStream;
use crate::parse_context::{ParseContext, ParseError, ParseResult};
use crate::charclass::charclass_to_rangeset;
use crate::xml_node::XmlNode;
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

    /// Get the number of rules in the grammar
    pub fn rule_count(&self) -> usize {
        self.rules.len()
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
        let start_pos = stream.position();
        
        // Check for left recursion at this position
        if !ctx.enter_rule(&rule.name, start_pos) {
            return Err(ParseError::LeftRecursion {
                rule: rule.name.clone(),
                position: start_pos,
            });
        }

        let result = self.parse_alternatives(stream, &rule.alternatives, ctx);

        ctx.exit_rule(&rule.name, start_pos);

        // Apply rule-level mark to result
        result.and_then(|res| Ok(self.apply_rule_mark(res, rule)))
    }

    /// Apply rule-level mark to parse result
    fn apply_rule_mark(&self, mut result: ParseResult, rule: &Rule) -> ParseResult {
        match rule.mark {
            Mark::Hidden => {
                // Don't wrap in element - pass through content as-is
                // This is different from factor-level hiding which suppresses output
                // Rule-level hiding just means "don't create wrapper element"
                // Content is already in result.node, so just return it
            }
            Mark::Attribute => {
                // Convert to attribute
                let text = result.node.map(|n| n.text_content()).unwrap_or_default();
                result.node = Some(XmlNode::Attribute {
                    name: rule.name.clone(),
                    value: text,
                });
            }
            Mark::Promoted => {
                // Keep node as-is (promoted)
                // Node is already unwrapped
            }
            Mark::None => {
                // Wrap in element
                // If the node is a _sequence wrapper, unwrap it and use its children
                let mut children = match result.node {
                    Some(XmlNode::Element { name, children, .. }) if name == "_sequence" => {
                        // Unwrap sequence and use its children directly
                        children
                    }
                    Some(node) => vec![node],
                    None => vec![], // Empty element
                };
                
                // Recursively flatten any nested _sequence elements
                children = self.flatten_sequences(children);
                
                // Extract attributes from children
                let (attributes, non_attrs): (Vec<_>, Vec<_>) = 
                    children.into_iter().partition(|node| {
                        matches!(node, XmlNode::Attribute { .. })
                    });
                
                // Convert attribute nodes to (name, value) tuples
                let attrs: Vec<(String, String)> = attributes
                    .into_iter()
                    .filter_map(|node| {
                        if let XmlNode::Attribute { name, value } = node {
                            Some((name, value))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                children = non_attrs;
                
                result.node = Some(XmlNode::Element {
                    name: rule.name.clone(),
                    attributes: attrs,
                    children,
                });
            }
        }

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
        let mut best_result: Option<(ParseResult, usize)> = None; // (result, end_position)
        let mut attempts = 0;

        // Try each alternative and keep the longest match
        for (_, alt) in alts.alts.iter().enumerate() {
            stream.set_position(start_pos); // Reset for each alternative
            attempts += 1;

            match self.parse_sequence(stream, alt, ctx) {
                Ok(result) => {
                    let end_pos = stream.position();
                    
                    // Keep this result if it's the longest match so far
                    match &best_result {
                        None => {
                            best_result = Some((result, end_pos));
                        }
                        Some((_, best_end)) => {
                            if end_pos > *best_end {
                                best_result = Some((result, end_pos));
                            }
                        }
                    }
                }
                Err(_) => {
                    continue; // Try next alternative
                }
            }
        }

        // Return the longest match, or error if all failed
        match best_result {
            Some((result, end_pos)) => {
                stream.set_position(end_pos); // Commit to longest match
                Ok(result)
            }
            None => Err(ParseError::NoAlternativeMatched {
                position: start_pos,
                rule: ctx.rule_name.clone(),
                attempts,
            }),
        }
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
        match &factor.repetition {
            Repetition::None => self.parse_base_factor(stream, &factor.base, ctx),
            Repetition::ZeroOrMore => self.parse_zero_or_more(stream, &factor.base, ctx),
            Repetition::OneOrMore => self.parse_one_or_more(stream, &factor.base, ctx),
            Repetition::Optional => self.parse_optional(stream, &factor.base, ctx),
            Repetition::SeparatedZeroOrMore(sep) => {
                self.parse_separated_zero_or_more(stream, &factor.base, sep, ctx)
            }
            Repetition::SeparatedOneOrMore(sep) => {
                self.parse_separated_one_or_more(stream, &factor.base, sep, ctx)
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
        value: &str,
        mark: Mark,
        insertion: bool,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();
        
        // Handle insertion: always succeeds, consumes no input
        if insertion {
            let node = match mark {
                Mark::Hidden => None,
                _ => Some(XmlNode::Text(value.to_string())),
            };
            return Ok(ParseResult::new(node, 0));
        }

        // Match literal string character by character
        let value_chars: Vec<char> = value.chars().collect();
        for expected_ch in &value_chars {
            match stream.current() {
                Some(actual_ch) if actual_ch == *expected_ch => {
                    stream.advance();
                }
                Some(actual_ch) => {
                    // Mismatch - restore position and fail
                    stream.set_position(start_pos);
                    return Err(ParseError::TerminalMismatch {
                        expected: value.to_string(),
                        actual: actual_ch.to_string(),
                        position: start_pos,
                    });
                }
                None => {
                    // Unexpected EOF
                    stream.set_position(start_pos);
                    return Err(ParseError::UnexpectedEof {
                        expected: value.to_string(),
                        position: start_pos,
                    });
                }
            }
        }

        // Success - create node based on mark
        let consumed = value_chars.len();
        let node = match mark {
            Mark::Hidden => None,
            _ => Some(XmlNode::Text(value.to_string())),
        };

        Ok(ParseResult::new(node, consumed))
    }

    /// Parse a character class
    fn parse_charclass(
        &self,
        stream: &mut InputStream,
        content: &str,
        negated: bool,
        mark: Mark,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();

        // Get current character
        let ch = match stream.current() {
            Some(c) => c,
            None => {
                return Err(ParseError::UnexpectedEof {
                    expected: format!("character matching class [{}{}]", if negated { "^" } else { "" }, content),
                    position: start_pos,
                });
            }
        };

        // Convert character class to RangeSet and check if character matches
        let rangeset = charclass_to_rangeset(content);
        let matches = rangeset.contains(ch);
        let actual_match = if negated { !matches } else { matches };

        if !actual_match {
            return Err(ParseError::CharClassMismatch {
                charclass: content.to_string(),
                negated,
                actual: ch,
                position: start_pos,
            });
        }

        // Success - consume character and create node
        stream.advance();
        let node = match mark {
            Mark::Hidden => None,
            _ => Some(XmlNode::Text(ch.to_string())),
        };

        Ok(ParseResult::new(node, 1))
    }

    /// Parse a nonterminal (rule reference)
    fn parse_nonterminal(
        &self,
        stream: &mut InputStream,
        name: &str,
        mark: Mark,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();

        // Look up the rule
        let rule = self.rules.get(name).ok_or_else(|| ParseError::Custom {
            message: format!("Undefined rule: {}", name),
            position: start_pos,
        })?;

        // Parse the rule
        let result = self.parse_rule(stream, rule, ctx)?;

        // Apply factor-level mark to the result
        let node = result.node.map(|n| match mark {
            Mark::Hidden => {
                // Factor-level hiding: unwrap element and pass through children + attributes
                // If the result is an Element, extract its children and attributes
                match n {
                    XmlNode::Element { children, attributes, .. } => {
                        // Pass through both children and attributes
                        // Convert attributes back to Attribute nodes
                        let mut all_nodes = Vec::new();
                        
                        // Add attributes as Attribute nodes
                        for (name, value) in attributes {
                            all_nodes.push(XmlNode::Attribute { name, value });
                        }
                        
                        // Add children
                        all_nodes.extend(children);
                        
                        if all_nodes.is_empty() {
                            return None;
                        } else if all_nodes.len() == 1 {
                            return Some(all_nodes.into_iter().next().unwrap());
                        } else {
                            // Multiple items - wrap in _sequence for now
                            return Some(XmlNode::Element {
                                name: "_sequence".to_string(),
                                attributes: vec![],
                                children: all_nodes,
                            });
                        }
                    }
                    // For non-Element nodes (Text, Attribute), keep them
                    other => return Some(other),
                }
            }
            Mark::Attribute => {
                // Convert to attribute
                Some(XmlNode::Attribute {
                    name: name.to_string(),
                    value: n.text_content(),
                })
            }
            Mark::Promoted => {
                // Promote content: Override any rule-level mark and wrap in element
                // If the result is NOT already wrapped in its rule name, wrap it
                match n {
                    XmlNode::Element { ref name, .. } if name == &rule.name => {
                        // Already wrapped in rule element, keep as-is
                        Some(n)
                    }
                    _ => {
                        // Not wrapped or wrapped in different element - wrap it
                        // First unwrap if it's a _sequence
                        let children = match n {
                            XmlNode::Element { name, children, .. } if name == "_sequence" => children,
                            other => vec![other],
                        };
                        
                        // Wrap in rule element
                        Some(XmlNode::Element {
                            name: rule.name.clone(),
                            attributes: vec![],
                            children,
                        })
                    }
                }
            }
            Mark::None => {
                // Keep as-is (already wrapped by rule-level mark)
                Some(n)
            }
        }).flatten();

        Ok(ParseResult::new(node, result.consumed))
    }

    /// Recursively flatten nested _sequence elements
    fn flatten_sequences(&self, children: Vec<XmlNode>) -> Vec<XmlNode> {
        let mut flattened = Vec::new();
        
        for node in children {
            match node {
                XmlNode::Element { name, children, .. } if name == "_sequence" => {
                    // Recursively flatten and add children
                    flattened.extend(self.flatten_sequences(children));
                }
                other => {
                    flattened.push(other);
                }
            }
        }
        
        flattened
    }

    /// Merge consecutive Text nodes and return an appropriate node
    fn merge_nodes(&self, children: Vec<XmlNode>) -> Option<XmlNode> {
        if children.is_empty() {
            return None;
        }
        
        // Merge consecutive Text nodes
        let mut merged = Vec::new();
        let mut text_buffer = String::new();
        
        for node in children {
            match node {
                XmlNode::Text(s) => {
                    text_buffer.push_str(&s);
                }
                other => {
                    // Flush text buffer if not empty
                    if !text_buffer.is_empty() {
                        merged.push(XmlNode::Text(text_buffer.clone()));
                        text_buffer.clear();
                    }
                    merged.push(other);
                }
            }
        }
        
        // Flush remaining text
        if !text_buffer.is_empty() {
            merged.push(XmlNode::Text(text_buffer));
        }
        
        // Return result
        if merged.is_empty() {
            None
        } else if merged.len() == 1 {
            Some(merged.into_iter().next().unwrap())
        } else {
            // Multiple non-text nodes - wrap in sequence
            Some(XmlNode::Element {
                name: "_sequence".to_string(),
                attributes: vec![],
                children: merged,
            })
        }
    }

    /// Parse zero or more repetitions (*)
    fn parse_zero_or_more(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();
        let mut children = Vec::new();
        let mut total_consumed = 0;

        // Keep matching until we fail
        loop {
            let loop_start = stream.position();
            
            // Try to match the base factor
            match self.parse_base_factor(stream, base, ctx) {
                Ok(result) => {
                    // Epsilon-match detection: prevent infinite loops
                    if result.consumed == 0 {
                        // If we matched but consumed nothing, we'd loop forever
                        // Break here (but keep the match if it produced a node)
                        if let Some(node) = result.node {
                            children.push(node);
                        }
                        break;
                    }
                    
                    // Collect non-suppressed nodes
                    if let Some(node) = result.node {
                        children.push(node);
                    }
                    total_consumed += result.consumed;
                }
                Err(_) => {
                    // Failed to match - that's OK for zero-or-more
                    stream.set_position(loop_start); // Backtrack this attempt
                    break;
                }
            }
        }

        // Return collected nodes (merged if they're all text)
        Ok(ParseResult::new(self.merge_nodes(children), total_consumed))
    }

    /// Parse one or more repetitions (+)
    fn parse_one_or_more(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();

        // Must match at least once
        let first_result = self.parse_base_factor(stream, base, ctx)?;
        let mut children = Vec::new();
        let mut total_consumed = first_result.consumed;

        if let Some(node) = first_result.node {
            children.push(node);
        }

        // Epsilon-match check: if first match consumed nothing, don't loop
        if first_result.consumed == 0 {
            let node = if children.is_empty() {
                None
            } else {
                Some(children.into_iter().next().unwrap())
            };
            return Ok(ParseResult::new(node, total_consumed));
        }

        // Try to match more
        loop {
            let loop_start = stream.position();
            
            match self.parse_base_factor(stream, base, ctx) {
                Ok(result) => {
                    // Epsilon-match detection
                    if result.consumed == 0 {
                        if let Some(node) = result.node {
                            children.push(node);
                        }
                        break;
                    }
                    
                    if let Some(node) = result.node {
                        children.push(node);
                    }
                    total_consumed += result.consumed;
                }
                Err(_) => {
                    stream.set_position(loop_start);
                    break;
                }
            }
        }

        // Return collected nodes (merged if they're all text)
        Ok(ParseResult::new(self.merge_nodes(children), total_consumed))
    }

    /// Parse optional (?)
    fn parse_optional(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();

        // Try to match once
        match self.parse_base_factor(stream, base, ctx) {
            Ok(result) => Ok(result),
            Err(_) => {
                // Failed - that's OK for optional
                stream.set_position(start_pos);
                Ok(ParseResult::new(None, 0))
            }
        }
    }

    /// Parse zero or more with separator (**)
    fn parse_separated_zero_or_more(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        separator: &Sequence,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();
        let mut children = Vec::new();
        let mut total_consumed = 0;

        // Try to match first element
        let first_pos = stream.position();
        match self.parse_base_factor(stream, base, ctx) {
            Ok(result) => {
                if let Some(node) = result.node {
                    children.push(node);
                }
                total_consumed += result.consumed;

                // Epsilon-match check
                if result.consumed == 0 {
                    return Ok(ParseResult::new(
                        if children.is_empty() { None } else { Some(children.into_iter().next().unwrap()) },
                        total_consumed
                    ));
                }
            }
            Err(_) => {
                // No elements - that's OK for zero-or-more
                stream.set_position(first_pos);
                return Ok(ParseResult::new(None, 0));
            }
        }

        // Try to match more: (separator element)*
        loop {
            let loop_start = stream.position();

            // Try to match separator
            match self.parse_sequence(stream, separator, ctx) {
                Ok(sep_result) => {
                    // Collect separator node (may be attribute)
                    if let Some(node) = sep_result.node {
                        children.push(node);
                    }
                    
                    // Separator matched, now try element
                    match self.parse_base_factor(stream, base, ctx) {
                        Ok(elem_result) => {
                            // Both matched - keep going
                            if let Some(node) = elem_result.node {
                                children.push(node);
                            }
                            total_consumed += sep_result.consumed + elem_result.consumed;

                            // Epsilon-match check
                            if elem_result.consumed == 0 {
                                break;
                            }
                        }
                        Err(_) => {
                            // Element failed after separator - backtrack separator too
                            stream.set_position(loop_start);
                            break;
                        }
                    }
                }
                Err(_) => {
                    // Separator failed - we're done
                    stream.set_position(loop_start);
                    break;
                }
            }
        }

        // Return collected nodes (merged if they're all text)
        Ok(ParseResult::new(self.merge_nodes(children), total_consumed))
    }

    /// Parse one or more with separator (++)
    fn parse_separated_one_or_more(
        &self,
        stream: &mut InputStream,
        base: &BaseFactor,
        separator: &Sequence,
        ctx: &mut ParseContext,
    ) -> Result<ParseResult, ParseError> {
        let start_pos = stream.position();

        // Must match at least one element
        let first_result = self.parse_base_factor(stream, base, ctx)?;
        let mut children = Vec::new();
        let mut total_consumed = first_result.consumed;

        if let Some(node) = first_result.node {
            children.push(node);
        }

        // Epsilon-match check
        if first_result.consumed == 0 {
            return Ok(ParseResult::new(
                if children.is_empty() { None } else { Some(children.into_iter().next().unwrap()) },
                total_consumed
            ));
        }

        // Try to match more: (separator element)*
        loop {
            let loop_start = stream.position();

            // Try to match separator
            match self.parse_sequence(stream, separator, ctx) {
                Ok(sep_result) => {
                    // Collect separator node (may be attribute)
                    if let Some(node) = sep_result.node {
                        children.push(node);
                    }
                    
                    // Separator matched, now try element
                    match self.parse_base_factor(stream, base, ctx) {
                        Ok(elem_result) => {
                            // Both matched
                            if let Some(node) = elem_result.node {
                                children.push(node);
                            }
                            total_consumed += sep_result.consumed + elem_result.consumed;

                            // Epsilon-match check
                            if elem_result.consumed == 0 {
                                break;
                            }
                        }
                        Err(_) => {
                            // Element failed after separator - backtrack
                            stream.set_position(loop_start);
                            break;
                        }
                    }
                }
                Err(_) => {
                    // Separator failed - we're done
                    stream.set_position(loop_start);
                    break;
                }
            }
        }

        // Return collected nodes (merged if they're all text)
        Ok(ParseResult::new(self.merge_nodes(children), total_consumed))
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

    #[test]
    fn test_simple_terminal() {
        use crate::grammar_ast::parse_ixml_grammar;

        let grammar_text = "test: 'hello'.";
        let grammar = parse_ixml_grammar(grammar_text).expect("Grammar should parse");
        let parser = NativeParser::new(grammar);

        // Should match "hello"
        let result = parser.parse("hello");
        assert!(result.is_ok(), "Parse should succeed: {:?}", result);
        let xml = result.unwrap();
        println!("XML output: {}", xml);
        assert!(xml.contains("<test>"));
        assert!(xml.contains("hello"));
    }

    #[test]
    fn test_terminal_mismatch() {
        use crate::grammar_ast::parse_ixml_grammar;

        let grammar_text = "test: 'hello'.";
        let grammar = parse_ixml_grammar(grammar_text).expect("Grammar should parse");
        let parser = NativeParser::new(grammar);

        // Should fail on "world"
        let result = parser.parse("world");
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error: {}", err);
        assert!(err.contains("No alternative matched") || err.contains("expected") || err.contains("hello"));
    }

    #[test]
    fn test_simple_charclass() {
        use crate::grammar_ast::parse_ixml_grammar;

        let grammar_text = "digit: ['0'-'9'].";
        let grammar = parse_ixml_grammar(grammar_text).expect("Grammar should parse");
        let parser = NativeParser::new(grammar);

        // Should match any digit
        for digit in '0'..='9' {
            let input = digit.to_string();
            let result = parser.parse(&input);
            assert!(result.is_ok(), "Should match digit {}: {:?}", digit, result);
            let xml = result.unwrap();
            assert!(xml.contains(&digit.to_string()));
        }

        // Should fail on non-digit
        let result = parser.parse("a");
        assert!(result.is_err());
    }

    #[test]
    fn test_nonterminal_reference() {
        use crate::grammar_ast::parse_ixml_grammar;

        let grammar_text = r#"
            test: greeting.
            greeting: 'hello'.
        "#;
        let grammar = parse_ixml_grammar(grammar_text).expect("Grammar should parse");
        let parser = NativeParser::new(grammar);

        let result = parser.parse("hello");
        assert!(result.is_ok(), "Parse should succeed: {:?}", result);
        let xml = result.unwrap();
        println!("XML output: {}", xml);
        // Remove whitespace for simpler matching
        let normalized = xml.split_whitespace().collect::<Vec<_>>().join("");
        assert!(normalized.contains("<test>"));
        assert!(normalized.contains("<greeting>"));
        assert!(normalized.contains("hello"));
    }
}
