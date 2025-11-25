//! Grammar Analysis
//!
//! Analyzes iXML grammar structure without modifying it.
//! Provides information for optimizations, warnings, and debugging.
//!
//! This uses the normalization concepts from Steven Pemberton's work
//! but applies them for analysis only, preserving the original grammar.

use crate::ast::{Alternatives, BaseFactor, Factor, IxmlGrammar, Mark, Repetition, Rule, Sequence};
use std::collections::{HashMap, HashSet};

/// Maximum recursion depth for grammar analysis to prevent stack overflow
#[allow(dead_code)]
const MAX_ANALYSIS_DEPTH: usize = 20;

/// Analysis results for an iXML grammar
#[derive(Debug, Clone)]
pub struct GrammarAnalysis {
    /// Rules that are recursive (directly or indirectly)
    pub recursive_rules: HashSet<String>,

    /// Rules that are left-recursive (first symbol can derive itself)
    pub left_recursive_rules: HashSet<String>,

    /// Rules marked as hidden (-name) - candidates for inlining
    pub hidden_rules: HashSet<String>,

    /// Rules marked as promoted (^name)
    pub promoted_rules: HashSet<String>,

    /// Rules marked as attributes (@name)
    pub attribute_rules: HashSet<String>,

    /// Complexity score for each rule (number of alternatives + nesting depth)
    pub complexity_scores: HashMap<String, usize>,

    /// Whether the grammar is potentially ambiguous
    pub is_potentially_ambiguous: bool,
}

impl GrammarAnalysis {
    /// Analyze an iXML grammar
    pub fn analyze(grammar: &IxmlGrammar) -> Self {
        let rule_map: HashMap<String, &Rule> = grammar
            .rules
            .iter()
            .map(|r| (r.name.clone(), r))
            .collect();

        // Find recursive rules (using iterative algorithm)
        let recursive_rules = find_recursive_rules(grammar, &rule_map);

        // Find left-recursive rules (using fixpoint iteration - fixed exponential explosion issue)
        let left_recursive_rules = find_left_recursive_rules(grammar, &rule_map, &recursive_rules);

        // Categorize rules by mark
        let mut hidden_rules = HashSet::new();
        let mut promoted_rules = HashSet::new();
        let mut attribute_rules = HashSet::new();

        for rule in &grammar.rules {
            match rule.mark {
                Mark::Hidden => {
                    hidden_rules.insert(rule.name.clone());
                }
                Mark::Promoted => {
                    promoted_rules.insert(rule.name.clone());
                }
                Mark::Attribute => {
                    attribute_rules.insert(rule.name.clone());
                }
                Mark::None => {}
            }
        }

        // Calculate complexity scores
        let complexity_scores = grammar
            .rules
            .iter()
            .map(|rule| {
                let score = calculate_complexity(&rule.alternatives);
                (rule.name.clone(), score)
            })
            .collect();

        // Normalize grammar for more precise analysis
        let normalized = normalize_grammar(grammar);
        let normalized_map: HashMap<String, &Rule> = normalized
            .rules
            .iter()
            .map(|r| (r.name.clone(), r))
            .collect();

        // Detect potentially ambiguous patterns using normalized grammar
        let is_potentially_ambiguous = detect_ambiguity_patterns(
            &normalized,
            &normalized_map,
            &recursive_rules,
        );

        GrammarAnalysis {
            recursive_rules,
            left_recursive_rules,
            hidden_rules,
            promoted_rules,
            attribute_rules,
            complexity_scores,
            is_potentially_ambiguous,
        }
    }

    /// Check if a rule is recursive
    pub fn is_recursive(&self, rule_name: &str) -> bool {
        self.recursive_rules.contains(rule_name)
    }

    /// Check if a rule is left-recursive
    pub fn is_left_recursive(&self, rule_name: &str) -> bool {
        self.left_recursive_rules.contains(rule_name)
    }

    /// Get complexity score for a rule
    pub fn complexity(&self, rule_name: &str) -> usize {
        self.complexity_scores.get(rule_name).copied().unwrap_or(0)
    }

    /// Get human-readable report of grammar issues
    pub fn report(&self) -> String {
        let mut report = String::new();

        if self.is_potentially_ambiguous {
            report.push_str("⚠️  Grammar may be ambiguous (multiple parse trees possible)\n");
            report.push_str("   Parse output will be marked with ixml:state=\"ambiguous\"\n");
            report.push('\n');
        }

        if !self.left_recursive_rules.is_empty() {
            report.push_str("⚠️  Left-recursive rules (may cause infinite loops):\n");
            for rule in &self.left_recursive_rules {
                report.push_str(&format!("   - {}\n", rule));
            }
            report.push('\n');
        }

        if !self.recursive_rules.is_empty() {
            report.push_str("ℹ️  Recursive rules (normal, but watch for performance):\n");
            for rule in &self.recursive_rules {
                if !self.left_recursive_rules.contains(rule) {
                    report.push_str(&format!("   - {}\n", rule));
                }
            }
            report.push('\n');
        }

        let high_complexity: Vec<_> = self
            .complexity_scores
            .iter()
            .filter(|(_, &score)| score > 10)
            .collect();

        if !high_complexity.is_empty() {
            report.push_str("ℹ️  High complexity rules (may be slow to parse):\n");
            for (rule, score) in high_complexity {
                report.push_str(&format!("   - {} (complexity: {})\n", rule, score));
            }
            report.push('\n');
        }

        if report.is_empty() {
            report.push_str("✅ No issues detected\n");
        }

        report
    }
}

/// Find all recursive rules (directly or indirectly)
fn find_recursive_rules(
    grammar: &IxmlGrammar,
    rule_map: &HashMap<String, &Rule>,
) -> HashSet<String> {
    let mut recursive = HashSet::new();

    for rule in &grammar.rules {
        let mut visited = HashSet::new();
        if is_recursive(&rule.name, rule_map, &mut visited, 0) {
            recursive.insert(rule.name.clone());
        }
    }

    recursive
}

/// Detect patterns that indicate potential ambiguity in the grammar
fn detect_ambiguity_patterns(
    grammar: &IxmlGrammar,
    rule_map: &HashMap<String, &Rule>,
    _recursive_rules: &HashSet<String>,
) -> bool {
    // Compute nullable set once for reuse
    let nullable_set = compute_nullable_set(rule_map);

    for rule in &grammar.rules {
        // Pattern 1: Multiple nullable alternatives in the same rule
        // Example: a: "a"* ; "b"* (both alternatives match empty string)
        if rule.alternatives.alts.len() > 1 {
            let mut nullable_alts = 0;
            for alt in &rule.alternatives.alts {
                let mut all_nullable = true;
                for factor in &alt.factors {
                    if !is_factor_nullable_simple(factor, &nullable_set) {
                        all_nullable = false;
                        break;
                    }
                }
                if all_nullable {
                    nullable_alts += 1;
                }
            }
            if nullable_alts > 1 {
                return true;
            }
        }

        // Pattern 2: Alternatives starting with the same nullable nonterminal
        // Example: a: spaces, "x" | spaces, "y" where spaces is nullable
        // This can cause ambiguity in how spaces are attributed
        if rule.alternatives.alts.len() > 1 {
            let mut first_nullable_nonterminals: HashMap<String, usize> = HashMap::new();

            for alt in &rule.alternatives.alts {
                // Skip empty sequences
                if alt.factors.is_empty() {
                    continue;
                }

                // Check if first factor is a nullable nonterminal
                if let BaseFactor::Nonterminal { name, .. } = &alt.factors[0].base {
                    if nullable_set.contains(name) {
                        *first_nullable_nonterminals.entry(name.clone()).or_insert(0) += 1;
                    }
                }
            }

            // If same nullable nonterminal appears at start of multiple alternatives
            for count in first_nullable_nonterminals.values() {
                if *count > 1 {
                    return true;
                }
            }
        }

        // Pattern 3: Consecutive nullable nonterminals in a sequence
        // Example: a: "a", spaces, b.  b: spaces, "b".  (where spaces is nullable)
        // When spaces is nullable, the distribution of matching content is ambiguous
        for alt in &rule.alternatives.alts {
            for (i, factor) in alt.factors.iter().enumerate() {
                // Skip last factor (no consecutive after it)
                if i >= alt.factors.len() - 1 {
                    continue;
                }

                // Check if this factor is a nullable nonterminal
                if let BaseFactor::Nonterminal { name, .. } = &factor.base {
                    if nullable_set.contains(name) {
                        // Check if next factor could expand to start with the same nullable nonterminal
                        let next_factor = &alt.factors[i + 1];
                        if could_start_with_nullable(next_factor, name, rule_map, &nullable_set) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check if a factor could start with a specific nullable nonterminal
fn could_start_with_nullable(
    factor: &Factor,
    nullable_name: &str,
    rule_map: &HashMap<String, &Rule>,
    nullable_set: &HashSet<String>,
) -> bool {
    match &factor.base {
        BaseFactor::Nonterminal { name, .. } => {
            // Direct match
            if name == nullable_name && nullable_set.contains(name) {
                return true;
            }

            // Check if this nonterminal's first factor could be the nullable
            if let Some(rule) = rule_map.get(name.as_str()) {
                for alt in &rule.alternatives.alts {
                    if alt.factors.is_empty() {
                        continue;
                    }
                    // Recursively check first factor
                    if let BaseFactor::Nonterminal { name: first_name, .. } = &alt.factors[0].base {
                        if first_name == nullable_name && nullable_set.contains(nullable_name) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Check if a rule is recursive (iterative version to avoid stack overflow)
fn is_recursive(
    rule_name: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    _depth: usize,
) -> bool {
    // Use explicit stack for iteration
    let mut work_stack: Vec<(String, bool)> = vec![(rule_name.to_string(), false)];
    let mut local_visited = HashSet::new();

    while let Some((current_rule, returning)) = work_stack.pop() {
        if returning {
            // We're returning from exploring this rule - remove from visited
            local_visited.remove(&current_rule);
            continue;
        }

        // Check if we've already visited this rule (cycle detected)
        if local_visited.contains(&current_rule) || visited.contains(&current_rule) {
            return true;
        }

        let rule = match rule_map.get(current_rule.as_str()) {
            Some(r) => r,
            None => continue,
        };

        // Mark as visiting
        local_visited.insert(current_rule.clone());

        // Push return marker
        work_stack.push((current_rule.clone(), true));

        // Check if this rule references our target rule
        if current_rule != rule_name {
            // Explore this rule's alternatives
            for alt in &rule.alternatives.alts {
                for factor in &alt.factors {
                    if let BaseFactor::Nonterminal { name, .. } = &factor.base {
                        if name == rule_name {
                            return true; // Found reference to target rule
                        }
                        // Add to work stack to explore
                        work_stack.push((name.clone(), false));
                    } else if let BaseFactor::Group { alternatives } = &factor.base {
                        // Need to check groups too
                        for group_alt in &alternatives.alts {
                            for group_factor in &group_alt.factors {
                                if let BaseFactor::Nonterminal { name, .. } = &group_factor.base {
                                    if name == rule_name {
                                        return true;
                                    }
                                    work_stack.push((name.clone(), false));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Exploring the target rule itself
            for alt in &rule.alternatives.alts {
                for factor in &alt.factors {
                    if let BaseFactor::Nonterminal { name, .. } = &factor.base {
                        work_stack.push((name.clone(), false));
                    } else if let BaseFactor::Group { alternatives } = &factor.base {
                        for group_alt in &alternatives.alts {
                            for group_factor in &group_alt.factors {
                                if let BaseFactor::Nonterminal { name, .. } = &group_factor.base {
                                    work_stack.push((name.clone(), false));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Find left-recursive rules
fn find_left_recursive_rules(
    grammar: &IxmlGrammar,
    rule_map: &HashMap<String, &Rule>,
    recursive_rules: &HashSet<String>,
) -> HashSet<String> {
    let mut left_recursive = HashSet::new();

    for rule in &grammar.rules {
        // Only check rules we know are recursive
        if !recursive_rules.contains(&rule.name) {
            continue;
        }

        // Check if any alternative starts with the rule itself (direct left-recursion)
        // or can derive to the rule itself through nullable symbols (indirect)
        if is_left_recursive(&rule.name, &rule.alternatives, rule_map) {
            left_recursive.insert(rule.name.clone());
        }
    }

    left_recursive
}

/// Check if a rule is left-recursive using fixpoint iteration
/// This computes the "left-reachable set" - which nonterminals can appear
/// at the leftmost position (accounting for nullable prefixes)
fn is_left_recursive(
    rule_name: &str,
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
) -> bool {
    // Precompute nullable set once
    let nullable_set = compute_nullable_set(rule_map);

    // Compute left-reachable nonterminals for this rule
    let left_reachable = compute_left_reachable(rule_name, alternatives, rule_map, &nullable_set);

    // Left-recursive if rule can reach itself
    left_reachable.contains(rule_name)
}

/// Compute which nonterminals can appear at the leftmost position
/// (accounting for nullable prefixes) using fixpoint iteration
fn compute_left_reachable(
    _rule_name: &str,
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
    nullable_set: &HashSet<String>,
) -> HashSet<String> {
    let mut reachable = HashSet::new();
    let mut changed = true;

    // Iterate until fixpoint (no more changes)
    while changed {
        changed = false;

        // For each alternative
        for alt in &alternatives.alts {
            // Process factors in sequence
            for factor in &alt.factors {
                match &factor.base {
                    BaseFactor::Nonterminal { name, .. } => {
                        // This nonterminal is left-reachable
                        if reachable.insert(name.clone()) {
                            changed = true;
                        }

                        // If we can reach a rule, we can reach what IT can reach
                        if let Some(ref_rule) = rule_map.get(name.as_str()) {
                            let ref_reachable = compute_left_reachable_direct(
                                &ref_rule.alternatives,
                                nullable_set
                            );
                            for ref_name in ref_reachable {
                                if reachable.insert(ref_name) {
                                    changed = true;
                                }
                            }
                        }

                        // If not nullable, stop here
                        if !nullable_set.contains(name) {
                            break;
                        }
                    }
                    BaseFactor::Literal { value, .. } => {
                        // Empty literals are nullable, continue
                        if !value.is_empty() {
                            break;
                        }
                    }
                    BaseFactor::CharClass { .. } => {
                        // Character classes block
                        break;
                    }
                    BaseFactor::Group { alternatives: group_alts } => {
                        // Inline group analysis
                        let group_reachable = compute_left_reachable_direct(group_alts, nullable_set);
                        for name in group_reachable {
                            if reachable.insert(name) {
                                changed = true;
                            }
                        }

                        // If group not nullable, stop
                        if !is_alternatives_nullable(group_alts, nullable_set) {
                            break;
                        }
                    }
                }

                // Check if factor is nullable via repetition
                match factor.repetition {
                    Repetition::ZeroOrMore | Repetition::Optional | Repetition::SeparatedZeroOrMore(_) => {
                        // Nullable, continue to next factor
                        continue;
                    }
                    _ => {
                        // If base was nullable, we already handled it above
                        // If not nullable and required, we already broke
                    }
                }
            }
        }
    }

    reachable
}

/// Compute left-reachable nonterminals for alternatives (one pass, no recursion)
fn compute_left_reachable_direct(
    alternatives: &Alternatives,
    nullable_set: &HashSet<String>,
) -> HashSet<String> {
    let mut reachable = HashSet::new();

    for alt in &alternatives.alts {
        for factor in &alt.factors {
            match &factor.base {
                BaseFactor::Nonterminal { name, .. } => {
                    reachable.insert(name.clone());

                    // If not nullable, stop here
                    if !nullable_set.contains(name) {
                        break;
                    }
                }
                BaseFactor::Literal { value, .. } => {
                    if !value.is_empty() {
                        break;
                    }
                }
                BaseFactor::CharClass { .. } => {
                    break;
                }
                BaseFactor::Group { alternatives: group_alts } => {
                    // Recursively get group's reachable (bounded depth)
                    let group_reachable = compute_left_reachable_direct(group_alts, nullable_set);
                    reachable.extend(group_reachable);

                    if !is_alternatives_nullable(group_alts, nullable_set) {
                        break;
                    }
                }
            }

            // Check repetition
            match factor.repetition {
                Repetition::ZeroOrMore | Repetition::Optional | Repetition::SeparatedZeroOrMore(_) => {
                    continue;
                }
                _ => {
                    // Already handled blocking above
                }
            }
        }
    }

    reachable
}

/// Check if alternatives are nullable (any alternative is nullable)
fn is_alternatives_nullable(alternatives: &Alternatives, nullable_set: &HashSet<String>) -> bool {
    alternatives.alts.iter().any(|alt| {
        alt.factors.iter().all(|f| is_factor_nullable_simple(f, nullable_set))
    })
}

/// Compute nullable set for all rules using fixpoint iteration (completely iterative)
fn compute_nullable_set(rule_map: &HashMap<String, &Rule>) -> HashSet<String> {
    let mut nullable_rules: HashSet<String> = HashSet::new();
    let mut changed = true;

    // Iterate until fixpoint (no more changes)
    while changed {
        changed = false;

        for (rule_name, rule) in rule_map.iter() {
            // Skip if already marked nullable
            if nullable_rules.contains(rule_name.as_str()) {
                continue;
            }

            // Check if any alternative is nullable
            for alt in &rule.alternatives.alts {
                let mut seq_nullable = true;

                // All factors in sequence must be nullable
                for factor in &alt.factors {
                    if !is_factor_nullable_simple(factor, &nullable_rules) {
                        seq_nullable = false;
                        break;
                    }
                }

                if seq_nullable {
                    // Found nullable alternative - mark rule as nullable
                    nullable_rules.insert(rule_name.to_string());
                    changed = true;
                    break;
                }
            }
        }
    }

    nullable_rules
}

/// Check if factor is nullable (fully iterative version)
fn is_factor_nullable_simple(factor: &Factor, nullable_rules: &HashSet<String>) -> bool {
    // Use a work stack to avoid recursion for nested groups
    let mut work_stack: Vec<&Factor> = vec![factor];
    let mut results_stack: Vec<bool> = Vec::new();

    while let Some(current_factor) = work_stack.pop() {
        // Check repetition first
        match current_factor.repetition {
            Repetition::ZeroOrMore | Repetition::Optional | Repetition::SeparatedZeroOrMore(_) => {
                results_stack.push(true);
                continue;
            }
            _ => {}
        }

        // Check base factor
        match &current_factor.base {
            BaseFactor::Literal { value, .. } => {
                results_stack.push(value.is_empty());
            }
            BaseFactor::CharClass { .. } => {
                results_stack.push(false);
            }
            BaseFactor::Nonterminal { name, .. } => {
                // Check if this rule is in our nullable set
                results_stack.push(nullable_rules.contains(name));
            }
            BaseFactor::Group { alternatives } => {
                // For groups, check if any alternative is nullable
                let mut group_nullable = false;
                for alt in &alternatives.alts {
                    let mut seq_nullable = true;
                    for seq_factor in &alt.factors {
                        // Simple inline check for this factor
                        let factor_nullable = match seq_factor.repetition {
                            Repetition::ZeroOrMore | Repetition::Optional | Repetition::SeparatedZeroOrMore(_) => true,
                            _ => match &seq_factor.base {
                                BaseFactor::Literal { value, .. } => value.is_empty(),
                                BaseFactor::CharClass { .. } => false,
                                BaseFactor::Nonterminal { name, .. } => nullable_rules.contains(name),
                                BaseFactor::Group { .. } => {
                                    // Nested group - limit depth by assuming not nullable
                                    // This avoids infinite recursion on deeply nested groups
                                    false
                                }
                            }
                        };

                        if !factor_nullable {
                            seq_nullable = false;
                            break;
                        }
                    }
                    if seq_nullable {
                        group_nullable = true;
                        break;
                    }
                }
                results_stack.push(group_nullable);
            }
        }
    }

    // Return the last result (should be only one for single factor check)
    results_stack.pop().unwrap_or(false)
}

/// Check if alternatives can match empty string (nullable) - uses precomputed set
#[allow(dead_code)]
fn is_nullable(
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
    _visited: &mut HashSet<String>,
    _depth: usize,
) -> bool {
    // Compute nullable set for entire grammar
    let nullable_rules = compute_nullable_set(rule_map);

    // Check if any alternative is nullable
    for alt in &alternatives.alts {
        let mut seq_nullable = true;
        for factor in &alt.factors {
            if !is_factor_nullable_simple(factor, &nullable_rules) {
                seq_nullable = false;
                break;
            }
        }
        if seq_nullable {
            return true;
        }
    }
    false
}

/// Fully iterative nullable check with memoization cache
#[allow(dead_code)]
fn is_nullable_with_cache(
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
    visited: &HashSet<String>,
    cache: &mut HashMap<String, bool>,
    depth: usize,
) -> bool {
    if depth > 5 {
        return false;
    }
    // At least one alternative must be nullable
    for seq in &alternatives.alts {
        if is_sequence_nullable_with_cache(seq, rule_map, visited, cache, depth + 1) {
            return true;
        }
    }
    false
}

/// Check if sequence is nullable with caching
#[allow(dead_code)]
fn is_sequence_nullable_with_cache(
    seq: &Sequence,
    rule_map: &HashMap<String, &Rule>,
    visited: &HashSet<String>,
    cache: &mut HashMap<String, bool>,
    depth: usize,
) -> bool {
    if depth > 5 {
        return false;
    }
    // All factors must be nullable
    for factor in &seq.factors {
        if !is_factor_nullable_with_cache(factor, rule_map, visited, cache, depth + 1) {
            return false;
        }
    }
    true
}

/// Check if factor is nullable with caching and depth limiting
#[allow(dead_code)]
fn is_factor_nullable_with_cache(
    factor: &Factor,
    rule_map: &HashMap<String, &Rule>,
    visited: &HashSet<String>,
    cache: &mut HashMap<String, bool>,
    depth: usize,
) -> bool {
    if depth > 5 {
        return false;
    }

    // Check repetition first
    match factor.repetition {
        Repetition::ZeroOrMore | Repetition::Optional | Repetition::SeparatedZeroOrMore(_) => {
            return true
        }
        _ => {}
    }

    // Check base factor
    match &factor.base {
        BaseFactor::Literal { value, .. } => value.is_empty(),
        BaseFactor::CharClass { .. } => false,
        BaseFactor::Nonterminal { name, .. } => {
            // Check cache first
            if let Some(&result) = cache.get(name) {
                return result;
            }

            // Avoid infinite loops - if currently visiting, assume not nullable
            if visited.contains(name) {
                return false;
            }

            // Create new visited set for this branch
            let mut local_visited = visited.clone();
            local_visited.insert(name.clone());

            let result = if let Some(rule) = rule_map.get(name.as_str()) {
                is_nullable_with_cache(&rule.alternatives, rule_map, &local_visited, cache, depth + 1)
            } else {
                false
            };

            // Cache the result
            cache.insert(name.clone(), result);
            result
        }
        BaseFactor::Group { alternatives } => {
            is_nullable_with_cache(alternatives, rule_map, visited, cache, depth + 1)
        }
    }
}

/// Iterative helper for checking if a sequence is nullable (kept for compatibility)
#[allow(dead_code)]
fn is_sequence_nullable_iterative(
    seq: &Sequence,
    rule_map: &HashMap<String, &Rule>,
    visited: &HashSet<String>,
) -> bool {
    let mut cache = HashMap::new();
    is_sequence_nullable_with_cache(seq, rule_map, visited, &mut cache, 0)
}

/// Iterative helper for checking if a factor is nullable (uses precomputed set)
#[allow(dead_code)]
fn is_factor_nullable_iterative(
    factor: &Factor,
    rule_map: &HashMap<String, &Rule>,
    _visited: &HashSet<String>,
) -> bool {
    let nullable_rules = compute_nullable_set(rule_map);
    is_factor_nullable_simple(factor, &nullable_rules)
}

/// Check if a sequence can match empty string
#[allow(dead_code)]
fn is_sequence_nullable(
    seq: &Sequence,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> bool {
    // Prevent stack overflow on deeply recursive grammars
    if depth > MAX_ANALYSIS_DEPTH {
        return false; // Assume not nullable if too deep
    }

    // All factors must be nullable
    seq.factors
        .iter()
        .all(|factor| is_factor_nullable(factor, rule_map, visited, depth + 1))
}

/// Check if a factor can match empty string
#[allow(dead_code)]
fn is_factor_nullable(
    factor: &Factor,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> bool {
    // Prevent stack overflow on deeply recursive grammars
    if depth > MAX_ANALYSIS_DEPTH {
        return false; // Assume not nullable if too deep
    }

    // Check repetition first
    match factor.repetition {
        Repetition::ZeroOrMore | Repetition::Optional => return true,
        Repetition::OneOrMore => {
            // Need base to be nullable
        }
        Repetition::None => {
            // Check base
        }
        Repetition::SeparatedZeroOrMore(_) => return true,
        Repetition::SeparatedOneOrMore(_) => {
            // Need base to be nullable
        }
    }

    // Check base factor
    match &factor.base {
        BaseFactor::Literal { value, .. } => value.is_empty(),
        BaseFactor::CharClass { .. } => false, // Character class always matches at least one char
        BaseFactor::Nonterminal { name, .. } => {
            if visited.contains(name) {
                return false; // Avoid infinite loop
            }
            visited.insert(name.clone());

            if let Some(rule) = rule_map.get(name) {
                let result = is_nullable(&rule.alternatives, rule_map, visited, depth + 1);
                visited.remove(name);
                result
            } else {
                visited.remove(name);
                false
            }
        }
        BaseFactor::Group { alternatives } => is_nullable(alternatives, rule_map, visited, depth + 1),
    }
}

/// Helper functions from normalize.rs
#[allow(dead_code)]
fn check_alternatives_for_recursion(
    alternatives: &Alternatives,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> bool {
    // Prevent stack overflow on deeply recursive grammars
    if depth > MAX_ANALYSIS_DEPTH {
        return false;
    }

    for seq in &alternatives.alts {
        if check_sequence_for_recursion(seq, target_rule, rule_map, visited, depth + 1) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn check_sequence_for_recursion(
    seq: &Sequence,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> bool {
    // Prevent stack overflow on deeply recursive grammars
    if depth > MAX_ANALYSIS_DEPTH {
        return false;
    }

    for factor in &seq.factors {
        if check_factor_for_recursion(factor, target_rule, rule_map, visited, depth + 1) {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
fn check_factor_for_recursion(
    factor: &Factor,
    target_rule: &str,
    rule_map: &HashMap<String, &Rule>,
    visited: &mut HashSet<String>,
    depth: usize,
) -> bool {
    // Prevent stack overflow on deeply recursive grammars
    if depth > MAX_ANALYSIS_DEPTH {
        return false;
    }

    match &factor.base {
        BaseFactor::Nonterminal { name, .. } => {
            if name == target_rule {
                return true;
            }
            is_recursive(name, rule_map, visited, depth + 1)
        }
        BaseFactor::Group { alternatives } => {
            check_alternatives_for_recursion(alternatives, target_rule, rule_map, visited, depth + 1)
        }
        _ => false,
    }
}

/// Calculate complexity score for alternatives
fn calculate_complexity(alternatives: &Alternatives) -> usize {
    let mut score = alternatives.alts.len(); // Base: number of alternatives

    for seq in &alternatives.alts {
        score += seq.factors.len(); // Add sequence length

        for factor in &seq.factors {
            score += match &factor.base {
                BaseFactor::Group { alternatives } => calculate_complexity(alternatives),
                _ => 1,
            };
        }
    }

    score
}

//=============================================================================
// Grammar Normalization for Static Analysis
//=============================================================================
//
// Based on Steven Pemberton's normalization algorithm:
// https://homepages.cwi.nl/~steven/Talks/2016/02-12-prague/data.html
//
// This creates a "normalized" (canonical schema) form of the grammar that:
// 1. Inlines hidden (-name) and promoted (^name) nonterminals
// 2. Removes unmarked terminals
// 3. Eliminates unused rules
//
// IMPORTANT: This is for STATIC ANALYSIS only, not for parsing!
// The parser still uses the original grammar to preserve XML structure.

/// Normalize a grammar for static analysis purposes
/// Returns a new grammar with hidden/promoted rules inlined
fn normalize_grammar(grammar: &IxmlGrammar) -> IxmlGrammar {
    let rule_map: HashMap<String, &Rule> = grammar
        .rules
        .iter()
        .map(|r| (r.name.clone(), r))
        .collect();

    // Find rules that should be inlined (hidden and promoted)
    let mut inline_rules: HashSet<String> = HashSet::new();
    for rule in &grammar.rules {
        match rule.mark {
            Mark::Hidden | Mark::Promoted => {
                inline_rules.insert(rule.name.clone());
            }
            _ => {}
        }
    }

    // Create normalized rules (except the ones we're inlining)
    let mut normalized_rules = Vec::new();
    for rule in &grammar.rules {
        if inline_rules.contains(&rule.name) {
            continue; // Skip - this rule will be inlined
        }

        // Normalize the alternatives
        let normalized_alts = normalize_alternatives(&rule.alternatives, &rule_map, &inline_rules, 0);

        normalized_rules.push(Rule::new(
            rule.name.clone(),
            rule.mark,
            normalized_alts,
        ));
    }

    IxmlGrammar::new(normalized_rules)
}

/// Normalize alternatives by inlining marked nonterminals
fn normalize_alternatives(
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
    inline_rules: &HashSet<String>,
    depth: usize,
) -> Alternatives {
    // Prevent infinite recursion
    if depth > 10 {
        return alternatives.clone();
    }

    let normalized_alts: Vec<Sequence> = alternatives
        .alts
        .iter()
        .flat_map(|seq| normalize_sequence(seq, rule_map, inline_rules, depth + 1))
        .collect();

    Alternatives::new(normalized_alts)
}

/// Normalize a sequence, potentially expanding into multiple sequences
fn normalize_sequence(
    sequence: &Sequence,
    rule_map: &HashMap<String, &Rule>,
    inline_rules: &HashSet<String>,
    depth: usize,
) -> Vec<Sequence> {
    // Prevent infinite recursion
    if depth > 10 {
        return vec![sequence.clone()];
    }

    let mut result_sequences = vec![Vec::new()];

    for factor in &sequence.factors {
        let normalized_factors = normalize_factor(factor, rule_map, inline_rules, depth + 1);

        if normalized_factors.len() == 1 {
            // Simple case: one factor -> append to all sequences
            for seq in &mut result_sequences {
                seq.push(normalized_factors[0].clone());
            }
        } else {
            // Complex case: multiple alternatives from inlining
            // Need to create cross-product of sequences
            let mut new_sequences = Vec::new();
            for existing_seq in &result_sequences {
                for new_factor in &normalized_factors {
                    let mut combined = existing_seq.clone();
                    combined.push(new_factor.clone());
                    new_sequences.push(combined);
                }
            }
            result_sequences = new_sequences;
        }
    }

    result_sequences
        .into_iter()
        .map(Sequence::new)
        .collect()
}

/// Normalize a factor, potentially expanding to multiple factors
fn normalize_factor(
    factor: &Factor,
    rule_map: &HashMap<String, &Rule>,
    inline_rules: &HashSet<String>,
    depth: usize,
) -> Vec<Factor> {
    // Prevent infinite recursion
    if depth > 10 {
        return vec![factor.clone()];
    }

    match &factor.base {
        BaseFactor::Nonterminal { name, .. } => {
            // Check if this nonterminal should be inlined
            if inline_rules.contains(name) {
                if let Some(rule) = rule_map.get(name.as_str()) {
                    // Inline this rule's alternatives
                    // This gets complex with repetitions, so simplify:
                    // If no repetition, inline directly
                    // If repetition, keep the nonterminal for now (conservative)
                    match factor.repetition {
                        Repetition::None => {
                            // Inline: collect all factors from all alternatives
                            let mut inlined_factors = Vec::new();
                            for alt in &rule.alternatives.alts {
                                for alt_factor in &alt.factors {
                                    inlined_factors.push(alt_factor.clone());
                                }
                            }
                            return if inlined_factors.is_empty() {
                                vec![]
                            } else {
                                inlined_factors
                            };
                        }
                        _ => {
                            // Keep as-is if there's repetition (too complex to inline)
                            return vec![factor.clone()];
                        }
                    }
                }
            }
            // Not inlined, keep as-is
            vec![factor.clone()]
        }
        BaseFactor::Group { alternatives } => {
            // Normalize the group's alternatives
            let normalized_alts = normalize_alternatives(alternatives, rule_map, inline_rules, depth + 1);
            vec![Factor::new(
                BaseFactor::Group {
                    alternatives: Box::new(normalized_alts),
                },
                factor.repetition.clone(),
            )]
        }
        _ => {
            // Literals and char classes stay as-is
            vec![factor.clone()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Repetition;

    #[test]
    fn test_detect_left_recursion() {
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
                    Sequence::new(vec![Factor::simple(BaseFactor::nonterminal(
                        "term".to_string(),
                    ))]),
                ]),
            ),
            Rule::new(
                "term".to_string(),
                Mark::None,
                Alternatives::single(Sequence::new(vec![Factor::simple(
                    BaseFactor::literal("x".to_string()),
                )])),
            ),
        ]);

        let analysis = GrammarAnalysis::analyze(&grammar);

        assert!(analysis.is_left_recursive("expr"));
        assert!(!analysis.is_left_recursive("term"));
    }

    #[test]
    fn test_no_left_recursion() {
        // number: digit+. digit: ["0"-"9"].
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

        let analysis = GrammarAnalysis::analyze(&grammar);

        assert!(!analysis.is_left_recursive("number"));
        assert!(!analysis.is_left_recursive("digit"));
    }

    #[test]
    fn test_complexity_calculation() {
        let grammar = IxmlGrammar::new(vec![Rule::new(
            "simple".to_string(),
            Mark::None,
            Alternatives::new(vec![
                Sequence::new(vec![Factor::simple(BaseFactor::literal("a".to_string()))]),
                Sequence::new(vec![Factor::simple(BaseFactor::literal("b".to_string()))]),
            ]),
        )]);

        let analysis = GrammarAnalysis::analyze(&grammar);

        // 2 alternatives + 2 sequences (len=1 each) + 2 factors = 6
        assert_eq!(analysis.complexity("simple"), 6);
    }
}
