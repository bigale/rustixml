//! Grammar Normalization
//!
//! Implements the normalization process described in the iXML specification.
//! The core transformation:
//! 1. Detect which rules are recursive (directly or indirectly)
//! 2. Inline all non-recursive rules into their usage sites
//! 3. Remove implicit terminals
//! 4. Discard unused rules
//!
//! This creates a canonical "schema" representation that makes parsing easier:
//! - Left-recursion becomes explicit
//! - Ambiguity appears at decision points
//! - Fewer rule lookups during parsing
//!
//! Reference: https://homepages.cwi.nl/~steven/Talks/2016/02-12-prague/data.html

use crate::ast::{
    Alternatives, BaseFactor, Factor, IxmlGrammar, Mark, Rule, Sequence,
};
#[cfg(test)]
use crate::ast::Repetition;
use std::collections::{HashMap, HashSet};

/// Normalize an iXML grammar by inlining non-recursive rules
pub fn normalize_grammar(grammar: &IxmlGrammar) -> IxmlGrammar {
    // Step 1: Build a map of rule names to rules for quick lookup
    let rule_map: HashMap<String, &Rule> = grammar
        .rules
        .iter()
        .map(|r| (r.name.clone(), r))
        .collect();

    // Step 2: Detect which rules are recursive
    let recursive_rules = find_recursive_rules(grammar, &rule_map);

    println!(
        "[normalize] Found {} recursive rules: {:?}",
        recursive_rules.len(),
        recursive_rules
    );

    // Step 3: Inline non-recursive rules in all rules
    let mut normalized_rules = Vec::new();
    for rule in &grammar.rules {
        let mut normalized_rule = rule.clone();
        inline_in_alternatives(&mut normalized_rule.alternatives, &rule_map, &recursive_rules);
        normalized_rules.push(normalized_rule);
    }

    // Step 4: Keep only recursive rules and the start rule
    // (The start rule is typically the first rule)
    let start_rule_name = grammar.rules.first().map(|r| r.name.clone());

    normalized_rules.retain(|r| {
        recursive_rules.contains(&r.name)
            || start_rule_name.as_ref() == Some(&r.name)
    });

    println!(
        "[normalize] Reduced from {} rules to {} rules",
        grammar.rules.len(),
        normalized_rules.len()
    );

    IxmlGrammar::new(normalized_rules)
}

/// Find all rules that are directly or indirectly recursive
fn find_recursive_rules(
    grammar: &IxmlGrammar,
    rule_map: &HashMap<String, &Rule>,
) -> HashSet<String> {
    let mut recursive = HashSet::new();

    for rule in &grammar.rules {
        let mut visited = HashSet::new();
        if is_recursive(&rule.name, rule_map, &mut visited) {
            recursive.insert(rule.name.clone());
        }
    }

    recursive
}

/// Check if a rule is recursive (directly or indirectly)
fn is_recursive(
    rule_name: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
) -> bool {
    // If we've already visited this rule in the current path, it's a cycle
    if visited.contains(rule_name) {
        return true;
    }

    // Get the rule definition
    let rule = match rule_map.get(rule_name) {
        Some(r) => r,
        None => return false, // Undefined rule, not recursive
    };

    // Mark this rule as visited in the current path
    visited.insert(rule_name.to_string());

    // Check all nonterminals referenced in this rule
    let is_rec = check_alternatives_for_recursion(&rule.alternatives, rule_name, rule_map, visited);

    // Remove from visited path (backtrack)
    visited.remove(rule_name);

    is_rec
}

/// Check if alternatives contain recursion
fn check_alternatives_for_recursion(
    alternatives: &Alternatives,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
) -> bool {
    for seq in &alternatives.alts {
        if check_sequence_for_recursion(seq, target_rule, rule_map, visited) {
            return true;
        }
    }
    false
}

/// Check if a sequence contains recursion
fn check_sequence_for_recursion(
    seq: &Sequence,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
) -> bool {
    for factor in &seq.factors {
        if check_factor_for_recursion(factor, target_rule, rule_map, visited) {
            return true;
        }
    }
    false
}

/// Check if a factor contains recursion
fn check_factor_for_recursion(
    factor: &Factor,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
) -> bool {
    match &factor.base {
        BaseFactor::Nonterminal { name, .. } => {
            // Direct recursion
            if name == target_rule {
                return true;
            }

            // Indirect recursion
            is_recursive(name, rule_map, visited)
        }
        BaseFactor::Group { alternatives } => {
            check_alternatives_for_recursion(alternatives, target_rule, rule_map, visited)
        }
        _ => false, // Literals and character classes can't be recursive
    }
}

/// Inline non-recursive rules in alternatives
fn inline_in_alternatives(
    alternatives: &mut Alternatives,
    rule_map: &HashMap<String, &Rule>,
    recursive_rules: &HashSet<String>,
) {
    for seq in &mut alternatives.alts {
        inline_in_sequence(seq, rule_map, recursive_rules);
    }
}

/// Inline non-recursive rules in a sequence
fn inline_in_sequence(
    seq: &mut Sequence,
    rule_map: &HashMap<String, &Rule>,
    recursive_rules: &HashSet<String>,
) {
    let mut new_factors = Vec::new();

    for factor in &seq.factors {
        match inline_factor(factor, rule_map, recursive_rules) {
            InlineResult::Keep(f) => new_factors.push(f),
            InlineResult::Replace(factors) => new_factors.extend(factors),
        }
    }

    seq.factors = new_factors;
}

/// Result of inlining a factor
enum InlineResult {
    Keep(Factor),          // Keep the factor as-is
    #[allow(dead_code)]
    Replace(Vec<Factor>),  // Replace with multiple factors (reserved for future use)
}

/// Inline a factor if it's a non-recursive nonterminal
fn inline_factor(
    factor: &Factor,
    rule_map: &HashMap<String, &Rule>,
    recursive_rules: &HashSet<String>,
) -> InlineResult {
    match &factor.base {
        BaseFactor::Nonterminal { name, mark } => {
            // Don't inline recursive rules
            if recursive_rules.contains(name) {
                return InlineResult::Keep(factor.clone());
            }

            // Look up the rule definition
            let target_rule = match rule_map.get(name) {
                Some(r) => r,
                None => return InlineResult::Keep(factor.clone()), // Undefined rule
            };

            // Clone the alternatives and recursively inline within them
            let mut inlined_alternatives = target_rule.alternatives.clone();
            inline_in_alternatives(&mut inlined_alternatives, rule_map, recursive_rules);

            // Wrap the inlined alternatives in a group
            let mut inlined_base = BaseFactor::Group {
                alternatives: Box::new(inlined_alternatives),
            };

            // Preserve the mark from the nonterminal reference
            if *mark != Mark::None {
                // If the nonterminal had a mark, we need to apply it to the group
                // This is a simplification - a complete implementation would need
                // to propagate marks through the inlined content
                inlined_base = apply_mark_to_base(inlined_base, *mark);
            }

            // Create a new factor with the same repetition
            let inlined_factor = Factor::new(inlined_base, factor.repetition.clone());

            InlineResult::Keep(inlined_factor)
        }
        BaseFactor::Group { alternatives } => {
            // Recursively inline within groups
            let mut inlined_alternatives = (**alternatives).clone();
            inline_in_alternatives(&mut inlined_alternatives, rule_map, recursive_rules);

            let inlined_factor = Factor::new(
                BaseFactor::Group {
                    alternatives: Box::new(inlined_alternatives),
                },
                factor.repetition.clone(),
            );

            InlineResult::Keep(inlined_factor)
        }
        _ => InlineResult::Keep(factor.clone()), // Keep literals and character classes as-is
    }
}

/// Apply a mark to a base factor (simplified - full implementation would be more complex)
fn apply_mark_to_base(base: BaseFactor, _mark: Mark) -> BaseFactor {
    match base {
        BaseFactor::Group { alternatives } => {
            // For groups, we can't directly apply the mark
            // This is a limitation of the current simplified implementation
            // A full implementation would need to propagate the mark through the tree
            BaseFactor::Group { alternatives }
        }
        _ => base, // For other types, mark propagation is not straightforward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_recursion_detection() {
        // expr: expr, "+", term | term.
        let grammar = IxmlGrammar::new(vec![
            Rule::new(
                "expr".to_string(),
                Mark::None,
                Alternatives::new(vec![
                    Sequence::new(vec![
                        Factor::simple(BaseFactor::nonterminal("expr".to_string())),
                        Factor::simple(BaseFactor::literal("+".to_string())),
                        Factor::simple(BaseFactor::nonterminal("term".to_string())),
                    ]),
                    Sequence::new(vec![
                        Factor::simple(BaseFactor::nonterminal("term".to_string())),
                    ]),
                ]),
            ),
            Rule::new(
                "term".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![
                    Factor::simple(BaseFactor::literal("x".to_string())),
                ])),
            ),
        ]);

        let rule_map: HashMap<_, _> = grammar
            .rules
            .iter()
            .map(|r| (r.name.clone(), r))
            .collect();
        let recursive = find_recursive_rules(&grammar, &rule_map);

        assert!(recursive.contains("expr"));
        assert!(!recursive.contains("term"));
    }

    #[test]
    fn test_indirect_recursion_detection() {
        // a: b. b: c. c: a.
        let grammar = IxmlGrammar::new(vec![
            Rule::new(
                "a".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![
                    Factor::simple(BaseFactor::nonterminal("b".to_string())),
                ])),
            ),
            Rule::new(
                "b".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![
                    Factor::simple(BaseFactor::nonterminal("c".to_string())),
                ])),
            ),
            Rule::new(
                "c".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![
                    Factor::simple(BaseFactor::nonterminal("a".to_string())),
                ])),
            ),
        ]);

        let rule_map: HashMap<_, _> = grammar
            .rules
            .iter()
            .map(|r| (r.name.clone(), r))
            .collect();
        let recursive = find_recursive_rules(&grammar, &rule_map);

        // All three rules are mutually recursive
        assert!(recursive.contains("a"));
        assert!(recursive.contains("b"));
        assert!(recursive.contains("c"));
    }

    #[test]
    fn test_simple_inlining() {
        // number: digit+. digit: ["0"-"9"].
        // After normalization: number: ["0"-"9"]+.
        let grammar = IxmlGrammar::new(vec![
            Rule::new(
                "number".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![Factor::new(
                    BaseFactor::nonterminal("digit".to_string()),
                    Repetition::OneOrMore,
                )])),
            ),
            Rule::new(
                "digit".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![Factor::simple(
                    BaseFactor::charclass("\"0\"-\"9\"".to_string()),
                )])),
            ),
        ]);

        let normalized = normalize_grammar(&grammar);

        // Should keep start rule and inline digit
        assert_eq!(normalized.rules.len(), 1);
        assert_eq!(normalized.rules[0].name, "number");

        // Check that digit was inlined
        let first_alt = &normalized.rules[0].alternatives.alts[0];
        let first_factor = &first_alt.factors[0];
        assert_eq!(first_factor.repetition, Repetition::OneOrMore);

        // The base should be a group containing the inlined digit rule
        match &first_factor.base {
            BaseFactor::Group { alternatives } => {
                assert_eq!(alternatives.alts.len(), 1);
            }
            _ => panic!("Expected a Group after inlining"),
        }
    }
}
