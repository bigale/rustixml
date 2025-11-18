//! Runtime parser using Earlgrey
//!
//! This module provides runtime parsing capabilities by converting iXML AST
//! to Earlgrey grammars and using them to parse input.

use earlgrey::{EarleyParser, GrammarBuilder, EarleyForest};
use crate::ast::{IxmlGrammar, Rule, Alternatives, Sequence, Factor, BaseFactor, Repetition, Mark};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::{HashMap, BTreeSet};

// ============================================================================
// Character Class Partitioning (Phase 1 Optimization)
// ============================================================================

/// A set of character ranges, used for character class partitioning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RangeSet {
    /// Sorted, non-overlapping ranges stored as (start, end) inclusive
    ranges: Vec<(char, char)>,
}

impl RangeSet {
    /// Create an empty RangeSet
    pub fn new() -> Self {
        RangeSet { ranges: Vec::new() }
    }

    /// Create a RangeSet from a single character
    pub fn from_char(ch: char) -> Self {
        RangeSet { ranges: vec![(ch, ch)] }
    }

    /// Create a RangeSet from a range
    pub fn from_range(start: char, end: char) -> Self {
        if start <= end {
            RangeSet { ranges: vec![(start, end)] }
        } else {
            RangeSet::new()
        }
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Add a character to the set
    pub fn add_char(&mut self, ch: char) {
        self.add_range(ch, ch);
    }

    /// Add a range to the set
    pub fn add_range(&mut self, start: char, end: char) {
        if start > end {
            return;
        }
        self.ranges.push((start, end));
        self.normalize();
    }

    /// Normalize ranges: sort and merge overlapping/adjacent ranges
    fn normalize(&mut self) {
        if self.ranges.len() <= 1 {
            return;
        }
        self.ranges.sort_by_key(|r| r.0);
        let mut merged = Vec::with_capacity(self.ranges.len());
        let mut current = self.ranges[0];

        for &(start, end) in &self.ranges[1..] {
            // Check if ranges overlap or are adjacent
            if start as u32 <= current.1 as u32 + 1 {
                // Merge ranges
                current.1 = current.1.max(end);
            } else {
                merged.push(current);
                current = (start, end);
            }
        }
        merged.push(current);
        self.ranges = merged;
    }

    /// Union of two RangeSets
    pub fn union(&self, other: &RangeSet) -> RangeSet {
        let mut result = self.clone();
        for &(start, end) in &other.ranges {
            result.add_range(start, end);
        }
        result
    }

    /// Intersection of two RangeSets
    pub fn intersection(&self, other: &RangeSet) -> RangeSet {
        let mut result = RangeSet::new();

        for &(a_start, a_end) in &self.ranges {
            for &(b_start, b_end) in &other.ranges {
                let int_start = a_start.max(b_start);
                let int_end = a_end.min(b_end);
                if int_start <= int_end {
                    result.ranges.push((int_start, int_end));
                }
            }
        }
        result.normalize();
        result
    }

    /// Subtract other from self (self - other)
    pub fn minus(&self, other: &RangeSet) -> RangeSet {
        let mut result = self.clone();

        for &(sub_start, sub_end) in &other.ranges {
            let mut new_ranges = Vec::new();

            for &(start, end) in &result.ranges {
                if sub_end < start || sub_start > end {
                    // No overlap, keep the range
                    new_ranges.push((start, end));
                } else {
                    // Overlap exists, split the range
                    if start < sub_start {
                        // Keep part before subtraction
                        new_ranges.push((start, char::from_u32(sub_start as u32 - 1).unwrap_or(start)));
                    }
                    if end > sub_end {
                        // Keep part after subtraction
                        new_ranges.push((char::from_u32(sub_end as u32 + 1).unwrap_or(end), end));
                    }
                }
            }
            result.ranges = new_ranges;
        }
        result.normalize();
        result
    }

    /// Check if the set contains a character
    pub fn contains(&self, ch: char) -> bool {
        for &(start, end) in &self.ranges {
            if ch >= start && ch <= end {
                return true;
            }
        }
        false
    }

    /// Generate a unique name for this RangeSet
    pub fn to_name(&self) -> String {
        let mut parts = Vec::new();
        for &(start, end) in &self.ranges {
            if start == end {
                parts.push(format!("{:X}", start as u32));
            } else {
                parts.push(format!("{:X}_{:X}", start as u32, end as u32));
            }
        }
        format!("cc_{}", parts.join("_"))
    }

    /// Create a predicate function for this RangeSet
    pub fn to_predicate(&self) -> Box<dyn Fn(&str) -> bool + Send + Sync> {
        let ranges = self.ranges.clone();
        Box::new(move |s: &str| {
            if s.chars().count() != 1 {
                return false;
            }
            let ch = s.chars().next().unwrap();
            for &(start, end) in &ranges {
                if ch >= start && ch <= end {
                    return true;
                }
            }
            false
        })
    }
}

/// Parse a character class content string into a RangeSet
/// This handles the same formats as parse_char_class but returns a RangeSet
fn charclass_to_rangeset(content: &str) -> RangeSet {
    let mut result = RangeSet::new();

    // Split by semicolon to get inclusion and exclusion parts
    let parts: Vec<&str> = content.split(';').collect();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Split by comma or pipe to get individual elements
        let elements: Vec<&str> = part.split(|c| c == ',' || c == '|').map(|s| s.trim()).collect();

        for element in elements {
            let element = element.trim();
            if element.is_empty() {
                continue;
            }

            // Check for hex character range: #30-#39
            if element.starts_with('#') && element.contains('-') {
                if let Some(dash_pos) = element[1..].find('-') {
                    let actual_dash_pos = dash_pos + 1;
                    let start_part = &element[..actual_dash_pos];
                    let end_part = &element[actual_dash_pos + 1..];

                    if end_part.starts_with('#') {
                        if let (Some(start), Some(end)) = (parse_hex_char(start_part), parse_hex_char(end_part)) {
                            result.add_range(start, end);
                            continue;
                        }
                    }
                }
                // Not a range, treat as single hex char
                if let Some(ch) = parse_hex_char(element) {
                    result.add_char(ch);
                }
            }
            // Check for quoted character range: "a"-"z"
            else if (element.starts_with('\'') || element.starts_with('"')) && element.contains('-') {
                let quote = if element.starts_with('\'') { '\'' } else { '"' };
                if let Some(first_close) = element[1..].find(quote) {
                    let first_close = first_close + 1;
                    let after_close = &element[first_close + 1..];
                    if after_close.starts_with('-') && after_close.len() > 1 {
                        let after_dash = &after_close[1..];
                        if after_dash.starts_with('\'') || after_dash.starts_with('"') {
                            let start_str = &element[1..first_close];
                            let start_char = start_str.chars().next();
                            let end_quote = if after_dash.starts_with('\'') { '\'' } else { '"' };
                            if let Some(end_close) = after_dash[1..].find(end_quote) {
                                let end_str = &after_dash[1..end_close + 1];
                                let end_char = end_str.chars().next();
                                if let (Some(start), Some(end)) = (start_char, end_char) {
                                    result.add_range(start, end);
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Not a range, treat as quoted characters
                let inner = element.trim_matches('\'').trim_matches('"');
                for ch in inner.chars() {
                    result.add_char(ch);
                }
            }
            // Single hex character
            else if element.starts_with('#') {
                if let Some(ch) = parse_hex_char(element) {
                    result.add_char(ch);
                }
            }
            // Single quoted string
            else if (element.starts_with('\'') && element.ends_with('\'')) ||
                    (element.starts_with('"') && element.ends_with('"')) {
                let inner = element.trim_matches('\'').trim_matches('"');
                for ch in inner.chars() {
                    result.add_char(ch);
                }
            }
            // Unicode category - skip for now (handled separately)
        }
    }

    result
}

/// Classify character classes into disjoint partitions
/// This is the key optimization from markup-blitz
pub fn classify_charclasses(all_charclasses: &[(String, bool)]) -> (Vec<RangeSet>, HashMap<(String, bool), Vec<usize>>) {
    // Convert all character classes to RangeSets
    let mut rangesets: Vec<(RangeSet, String, bool)> = Vec::new();
    for (content, negated) in all_charclasses {
        let rs = charclass_to_rangeset(content);
        if !rs.is_empty() {
            rangesets.push((rs, content.clone(), *negated));
        }
    }

    if rangesets.is_empty() {
        return (Vec::new(), HashMap::new());
    }

    // Compute disjoint partitions
    // Start with the union of all ranges
    let mut partitions: Vec<RangeSet> = Vec::new();
    let mut all_union = RangeSet::new();
    for (rs, _, _) in &rangesets {
        all_union = all_union.union(rs);
    }
    partitions.push(all_union);

    // For each original charset, split existing partitions
    for (rangeset, _, _) in &rangesets {
        let mut new_partitions = Vec::new();

        for partition in &partitions {
            let intersection = rangeset.intersection(partition);
            if !intersection.is_empty() {
                if intersection != *partition {
                    // Split the partition
                    let remainder = partition.minus(&intersection);
                    if !remainder.is_empty() {
                        new_partitions.push(remainder);
                    }
                    new_partitions.push(intersection);
                } else {
                    // Partition is fully contained, keep it
                    new_partitions.push(partition.clone());
                }
            } else {
                // No intersection, keep partition as-is
                new_partitions.push(partition.clone());
            }
        }

        // Deduplicate partitions
        let mut seen = BTreeSet::new();
        partitions = new_partitions.into_iter().filter(|p| {
            let key = format!("{:?}", p.ranges);
            if seen.contains(&key) {
                false
            } else {
                seen.insert(key);
                true
            }
        }).collect();
    }

    // Map each original charset to its partition indices
    let mut mapping: HashMap<(String, bool), Vec<usize>> = HashMap::new();

    for (rangeset, content, negated) in &rangesets {
        let mut indices = Vec::new();
        for (i, partition) in partitions.iter().enumerate() {
            // Check if this partition is part of the original rangeset
            let intersection = rangeset.intersection(partition);
            if !intersection.is_empty() && intersection == *partition {
                indices.push(i);
            }
        }
        mapping.insert((content.clone(), *negated), indices);
    }

    (partitions, mapping)
}

/// Transform the AST to replace character classes with partitioned alternatives
/// This is the AST transformation approach for character class partitioning
pub fn partition_charclasses_in_ast(grammar: &IxmlGrammar) -> IxmlGrammar {
    // Collect all character classes
    let mut charclasses_seen = std::collections::HashSet::new();
    collect_charclasses(grammar, &mut charclasses_seen);

    // Convert to vector for partitioning
    let charclass_list: Vec<(String, bool)> = charclasses_seen.iter().cloned().collect();

    // Compute partitions
    let (partitions, partition_mapping) = classify_charclasses(&charclass_list);

    // If no partitions or single charclass, return unchanged
    if partitions.is_empty() || charclass_list.len() <= 1 {
        return grammar.clone();
    }

    // Transform the grammar by replacing CharClass with partitioned Groups
    let transformed_rules: Vec<Rule> = grammar.rules.iter().map(|rule| {
        Rule {
            name: rule.name.clone(),
            mark: rule.mark,
            alternatives: transform_alternatives(&rule.alternatives, &partitions, &partition_mapping),
        }
    }).collect();

    IxmlGrammar { rules: transformed_rules }
}

fn transform_alternatives(
    alts: &Alternatives,
    partitions: &[RangeSet],
    mapping: &HashMap<(String, bool), Vec<usize>>,
) -> Alternatives {
    Alternatives {
        alts: alts.alts.iter().map(|seq| transform_sequence(seq, partitions, mapping)).collect(),
    }
}

fn transform_sequence(
    seq: &Sequence,
    partitions: &[RangeSet],
    mapping: &HashMap<(String, bool), Vec<usize>>,
) -> Sequence {
    Sequence {
        factors: seq.factors.iter().map(|factor| transform_factor(factor, partitions, mapping)).collect(),
    }
}

fn transform_factor(
    factor: &Factor,
    partitions: &[RangeSet],
    mapping: &HashMap<(String, bool), Vec<usize>>,
) -> Factor {
    let transformed_base = match &factor.base {
        BaseFactor::CharClass { content, negated, mark } => {
            // Check if this charclass should be partitioned
            if let Some(indices) = mapping.get(&(content.clone(), *negated)) {
                if indices.len() > 1 {
                    // Create a Group with alternatives for each partition
                    let partition_alts: Vec<Sequence> = indices.iter().map(|&idx| {
                        let partition = &partitions[idx];
                        // Convert partition RangeSet to charclass content string
                        let partition_content = rangeset_to_charclass_content(partition);
                        Sequence {
                            factors: vec![Factor {
                                base: BaseFactor::CharClass {
                                    content: partition_content,
                                    negated: false, // Partitions are always positive
                                    mark: *mark,
                                },
                                repetition: Repetition::None,
                            }],
                        }
                    }).collect();

                    BaseFactor::Group {
                        alternatives: Box::new(Alternatives { alts: partition_alts }),
                    }
                } else {
                    // Single partition, keep as-is (but could use partition content)
                    factor.base.clone()
                }
            } else {
                // Not in mapping, keep as-is
                factor.base.clone()
            }
        }
        BaseFactor::Group { alternatives } => {
            BaseFactor::Group {
                alternatives: Box::new(transform_alternatives(alternatives, partitions, mapping)),
            }
        }
        _ => factor.base.clone(),
    };

    // Also transform separator sequences in repetitions
    let transformed_rep = match &factor.repetition {
        Repetition::SeparatedZeroOrMore(sep) => {
            Repetition::SeparatedZeroOrMore(Box::new(transform_sequence(sep, partitions, mapping)))
        }
        Repetition::SeparatedOneOrMore(sep) => {
            Repetition::SeparatedOneOrMore(Box::new(transform_sequence(sep, partitions, mapping)))
        }
        _ => factor.repetition.clone(),
    };

    Factor {
        base: transformed_base,
        repetition: transformed_rep,
    }
}

/// Convert a RangeSet back to a charclass content string
fn rangeset_to_charclass_content(rs: &RangeSet) -> String {
    let mut parts = Vec::new();
    for &(start, end) in &rs.ranges {
        if start == end {
            // Single character
            if start as u32 <= 0x7F && start.is_alphanumeric() {
                parts.push(format!("\"{}\"", start));
            } else {
                parts.push(format!("#{:X}", start as u32));
            }
        } else {
            // Range
            let start_str = if start as u32 <= 0x7F && start.is_alphanumeric() {
                format!("\"{}\"", start)
            } else {
                format!("#{:X}", start as u32)
            };
            let end_str = if end as u32 <= 0x7F && end.is_alphanumeric() {
                format!("\"{}\"", end)
            } else {
                format!("#{:X}", end as u32)
            };
            parts.push(format!("{}-{}", start_str, end_str));
        }
    }
    parts.join(", ")
}

// Global counter for generating unique group IDs
static GROUP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Helper function to normalize character class content by removing quote characters
/// Supports both single quotes (') and double quotes (") per iXML spec flexibility
fn normalize_charclass_content(content: &str) -> String {
    // Escape all special characters to create unique, safe symbol names
    content.chars().map(|ch| {
        match ch {
            ' ' => "SPACE".to_string(),
            '\t' => "TAB".to_string(),
            '\n' => "NEWLINE".to_string(),
            '\r' => "RETURN".to_string(),
            '"' => "QUOTE".to_string(),
            '\'' => "APOS".to_string(),
            '<' => "LT".to_string(),
            '>' => "GT".to_string(),
            '&' => "AMP".to_string(),
            '/' => "SLASH".to_string(),
            '\\' => "BACKSLASH".to_string(),
            '|' => "PIPE".to_string(),
            '?' => "QUEST".to_string(),
            '!' => "EXCL".to_string(),
            '=' => "EQ".to_string(),
            '-' => "DASH".to_string(),
            '[' => "LBRACK".to_string(),
            ']' => "RBRACK".to_string(),
            '{' => "LBRACE".to_string(),
            '}' => "RBRACE".to_string(),
            '(' => "LPAREN".to_string(),
            ')' => "RPAREN".to_string(),
            '#' => "HASH".to_string(),
            _ if ch.is_alphanumeric() => ch.to_string(),
            _ => format!("U{:04X}", ch as u32),
        }
    }).collect::<Vec<_>>().join("_")
}

/// Helper function to normalize separator symbols into a unique identifier
/// This ensures different separators create different nonterminal names
fn normalize_separator(symbols: &[String]) -> String {
    symbols.join("_")
        .replace("-", "_DASH_")
        .replace("char_", "")
        .replace("U00", "")
        .replace("lit_seq_", "")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Helper function to normalize multi-character literals into safe symbol names
/// Escapes all special characters to prevent "Missing Symbol" errors
fn normalize_literal_sequence(value: &str) -> String {
    value.chars().map(|ch| {
        match ch {
            ' ' => "SPACE".to_string(),
            '\t' => "TAB".to_string(),
            '\n' => "NEWLINE".to_string(),
            '\r' => "RETURN".to_string(),
            '"' => "QUOTE".to_string(),
            '\'' => "APOS".to_string(),
            '<' => "LT".to_string(),
            '>' => "GT".to_string(),
            '&' => "AMP".to_string(),
            '/' => "SLASH".to_string(),
            '\\' => "BACKSLASH".to_string(),
            '|' => "PIPE".to_string(),
            '?' => "QUEST".to_string(),
            '!' => "EXCL".to_string(),
            '=' => "EQ".to_string(),
            _ if ch.is_alphanumeric() => ch.to_string(),
            _ => format!("U{:04X}", ch as u32),
        }
    }).collect::<Vec<_>>().join("_")
}

/// Helper function to normalize a separator sequence into a unique identifier
/// This allows computing separator-based names without first converting to symbols
fn normalize_sequence(seq: &Sequence) -> String {
    seq.factors.iter().map(|factor| {
        match &factor.base {
            BaseFactor::Literal { value, .. } => value.chars()
                .map(|ch| format!("{:X}", ch as u32))
                .collect::<Vec<_>>()
                .join(""),
            BaseFactor::Nonterminal { name, .. } => name.clone(),
            BaseFactor::CharClass { content, negated, mark: _ } => {
                let prefix = if *negated { "NEG" } else { "CC" };
                format!("{}{}", prefix, normalize_charclass_content(content))
            }
            BaseFactor::Group { .. } => "GRP".to_string(),
        }
    }).collect::<Vec<_>>().join("_")
}

// Feature flag for character class partitioning
// Set to true to enable AST transformation for partitioning
// NOTE: Currently disabled - causes regression on ranges test
// TODO: Fix rangeset_to_charclass_content to generate valid syntax
const ENABLE_CHARCLASS_PARTITIONING: bool = false;

/// Convert an iXML AST to an Earlgrey grammar
///
/// This is the "translator" that takes our parsed iXML grammar and converts it
/// to Earlgrey's format so we can use it to parse input at runtime.
pub fn ast_to_earlgrey(grammar: &IxmlGrammar) -> Result<GrammarBuilder, String> {
    // Reset group counter for deterministic group naming across conversion and XML generation
    GROUP_COUNTER.store(0, Ordering::SeqCst);

    // Optionally transform the AST for character class partitioning
    let grammar = if ENABLE_CHARCLASS_PARTITIONING {
        partition_charclasses_in_ast(grammar)
    } else {
        grammar.clone()
    };
    let grammar = &grammar;

    let mut builder = GrammarBuilder::default();

    // First pass: collect all unique characters from literals and define terminals
    let mut chars_seen = std::collections::HashSet::new();
    collect_literal_chars(grammar, &mut chars_seen);

    for ch in chars_seen {
        let term_name = char_terminal_name(ch);
        builder = builder.terminal(&term_name, move |s: &str| {
            // Use chars().count() for proper Unicode support, not byte length
            s.chars().count() == 1 && s.chars().next() == Some(ch)
        });
    }

    // Collect and define character class terminals
    let mut charclasses_seen = std::collections::HashSet::new();
    collect_charclasses(grammar, &mut charclasses_seen);

    // Define terminals for each character class
    for (content, negated) in charclasses_seen {
        let class_name = if negated {
            format!("charclass_neg_{}", normalize_charclass_content(&content))
        } else {
            format!("charclass_{}", normalize_charclass_content(&content))
        };
        let predicate = parse_char_class(&content, negated);
        builder = builder.terminal(&class_name, predicate);
    }

    // Declare sequence nonterminals for multi-character literals BEFORE creating marked literal wrappers
    // This ensures the base literal sequences exist when marked wrappers reference them
    declare_literal_sequences(grammar, &mut builder);

    // Collect and pre-create marked literal wrapper rules
    let mut marked_literals = std::collections::HashSet::new();
    collect_marked_literals(grammar, &mut marked_literals);

    for (base_name, mark) in marked_literals {
        let marked_name = format!("{}_{}", base_name, mark_suffix(mark));
        builder = builder.nonterm(&marked_name);
        builder = builder.rule(&marked_name, &[&base_name]);
    }

    // Second pass: declare all nonterminals (including multi-char literal sequences and repetitions)
    for rule in &grammar.rules {
        builder = builder.nonterm(&rule.name);
    }

    // Declare repetition helper nonterminals (e.g., letter_star, word_plus, etc.)
    let mut repetition_nonterminals = std::collections::HashSet::new();
    collect_repetition_nonterminals(grammar, &mut repetition_nonterminals);
    for nonterm in repetition_nonterminals {
        builder = builder.nonterm(&nonterm);
    }

    // Third pass: add all the rules
    let mut created_repetitions = std::collections::HashSet::new();
    for rule in &grammar.rules {
        builder = convert_rule(builder, rule, &mut created_repetitions)?;
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
    match &factor.base {
        BaseFactor::Literal { value, .. } => {
            for ch in value.chars() {
                chars.insert(ch);
            }
        }
        BaseFactor::Group { alternatives } => {
            // Recurse into group alternatives
            collect_chars_from_alternatives(alternatives, chars);
        }
        _ => {}, // Nonterminal and CharClass don't contain literal chars
    }

    // Also collect chars from separator sequences in repetition operators
    match &factor.repetition {
        Repetition::SeparatedZeroOrMore(sep) | Repetition::SeparatedOneOrMore(sep) => {
            for sep_factor in &sep.factors {
                collect_chars_from_factor(sep_factor, chars);
            }
        }
        _ => {}, // Other repetitions don't have separators
    }
}

/// Collect all unique character classes from the grammar
fn collect_charclasses(grammar: &IxmlGrammar, charclasses: &mut std::collections::HashSet<(String, bool)>) {
    for rule in &grammar.rules {
        collect_charclasses_from_alternatives(&rule.alternatives, charclasses);
    }
}

fn collect_charclasses_from_alternatives(alts: &Alternatives, charclasses: &mut std::collections::HashSet<(String, bool)>) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            collect_charclasses_from_factor(factor, charclasses);
        }
    }
}

fn collect_charclasses_from_factor(factor: &Factor, charclasses: &mut std::collections::HashSet<(String, bool)>) {
    match &factor.base {
        BaseFactor::CharClass { content, negated, .. } => {
            charclasses.insert((content.clone(), *negated));
        }
        BaseFactor::Group { alternatives } => {
            // Recurse into group alternatives
            collect_charclasses_from_alternatives(alternatives, charclasses);
        }
        _ => {}, // Literal and Nonterminal don't contain character classes
    }

    // Also collect charclasses from separator sequences in repetition operators
    match &factor.repetition {
        Repetition::SeparatedZeroOrMore(sep) | Repetition::SeparatedOneOrMore(sep) => {
            for sep_factor in &sep.factors {
                collect_charclasses_from_factor(sep_factor, charclasses);
            }
        }
        _ => {}, // Other repetitions don't have separators
    }
}

/// Collect all unique marked literals (char/literal + mark combinations)
fn collect_marked_literals(grammar: &IxmlGrammar, marked_literals: &mut std::collections::HashSet<(String, Mark)>) {
    for rule in &grammar.rules {
        collect_marked_from_alternatives(&rule.alternatives, marked_literals);
    }
}

fn collect_marked_from_alternatives(alts: &Alternatives, marked_literals: &mut std::collections::HashSet<(String, Mark)>) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            collect_marked_from_factor(factor, marked_literals);
        }
    }
}

fn collect_marked_from_factor(factor: &Factor, marked_literals: &mut std::collections::HashSet<(String, Mark)>) {
    match &factor.base {
        BaseFactor::Literal { value, mark, .. } => {
            if *mark != Mark::None {
                // Compute the base name (same logic as convert_factor)
                let base_name = if value.chars().count() == 1 {
                    let ch = value.chars().next().unwrap();
                    char_terminal_name(ch)
                } else {
                    format!("lit_seq_{}", normalize_literal_sequence(value))
                };
                marked_literals.insert((base_name, *mark));
            }
        }
        BaseFactor::CharClass { content, negated, mark, .. } => {
            if *mark != Mark::None {
                // Compute the base name (same logic as convert_factor)
                let base_name = if *negated {
                    format!("charclass_neg_{}", normalize_charclass_content(content))
                } else {
                    format!("charclass_{}", normalize_charclass_content(content))
                };
                marked_literals.insert((base_name, *mark));
            }
        }
        BaseFactor::Group { alternatives } => {
            collect_marked_from_alternatives(alternatives, marked_literals);
        }
        _ => {}, // Only Nonterminals don't have marks
    }

    // Also collect from separator sequences in repetition operators
    match &factor.repetition {
        Repetition::SeparatedZeroOrMore(sep) | Repetition::SeparatedOneOrMore(sep) => {
            for sep_factor in &sep.factors {
                collect_marked_from_factor(sep_factor, marked_literals);
            }
        }
        _ => {}, // Other repetitions don't have separators
    }
}

/// Collect all repetition helper nonterminals that will be created
fn collect_repetition_nonterminals(grammar: &IxmlGrammar, nonterminals: &mut std::collections::HashSet<String>) {
    for rule in &grammar.rules {
        collect_repetition_from_alternatives(&rule.alternatives, nonterminals);
    }
}

fn collect_repetition_from_alternatives(alts: &Alternatives, nonterminals: &mut std::collections::HashSet<String>) {
    for seq in &alts.alts {
        collect_nonterminals_from_sequence(nonterminals, seq);
    }
}

fn collect_nonterminals_from_sequence(nonterminals: &mut std::collections::HashSet<String>, seq: &Sequence) {
    for factor in &seq.factors {
        collect_repetition_from_factor(factor, nonterminals);
    }
}

fn collect_repetition_from_factor(factor: &Factor, nonterminals: &mut std::collections::HashSet<String>) {
    // Get the base name for this factor
    let base_name = match &factor.base {
        BaseFactor::Literal { value, mark, .. } => {
            // Literals become terminals (single char) or sequences (multi-char)
            let base = if value.chars().count() == 1 {
                char_terminal_name(value.chars().next().unwrap())
            } else {
                format!("lit_seq_{}", normalize_literal_sequence(value))
            };

            // If marked, the actual base name includes the mark suffix (wrapper rule)
            if *mark != Mark::None {
                format!("{}_{}", base, mark_suffix(*mark))
            } else {
                base
            }
        }
        BaseFactor::Nonterminal { name, .. } => name.clone(),
        BaseFactor::CharClass { content, negated, mark, .. } => {
            // Character classes become terminals, use the same naming convention
            let base = if *negated {
                format!("charclass_neg_{}", normalize_charclass_content(content))
            } else {
                format!("charclass_{}", normalize_charclass_content(content))
            };

            // If marked, the actual base name includes the mark suffix (wrapper rule)
            if *mark != Mark::None {
                format!("{}_{}", base, mark_suffix(*mark))
            } else {
                base
            }
        }
        BaseFactor::Group { alternatives } => {
            // Recurse into group to collect repetitions inside
            collect_repetition_from_alternatives(alternatives, nonterminals);
            // Groups get unique names based on GROUP_COUNTER, but we can't predict those
            // They're handled dynamically during rule conversion
            return;
        }
    };

    // Add the repetition helper nonterminal based on the repetition type
    match &factor.repetition {
        Repetition::None => {}, // No helper needed
        Repetition::OneOrMore => {
            nonterminals.insert(format!("{}_plus", base_name));
        }
        Repetition::ZeroOrMore => {
            nonterminals.insert(format!("{}_star", base_name));
        }
        Repetition::Optional => {
            nonterminals.insert(format!("{}_opt", base_name));
        }
        Repetition::SeparatedZeroOrMore(sep) => {
            // base**(sep) generates both _sep_star and _sep_plus nonterminals
            let sep_id = normalize_sequence(sep);
            nonterminals.insert(format!("{}_sep_{}_star", base_name, sep_id));
            nonterminals.insert(format!("{}_sep_{}_plus", base_name, sep_id));
            collect_nonterminals_from_sequence(nonterminals, sep);
        }
        Repetition::SeparatedOneOrMore(sep) => {
            let sep_id = normalize_sequence(sep);
            nonterminals.insert(format!("{}_sep_{}_plus", base_name, sep_id));
            collect_nonterminals_from_sequence(nonterminals, sep);
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
    match &factor.base {
        BaseFactor::Literal { value, .. } => {
            if value.len() > 1 {
                let seq_name = format!("lit_seq_{}", normalize_literal_sequence(value));
                if !declared.contains(&seq_name) {
                    let old_builder = std::mem::replace(builder, GrammarBuilder::default());
                    *builder = old_builder.nonterm(&seq_name);
                    declared.insert(seq_name);
                }
            }
        }
        BaseFactor::Group { alternatives } => {
            // Recurse into group alternatives to declare any literal sequences within
            declare_sequences_from_alternatives(alternatives, builder, declared);
        }
        _ => {}, // Nonterminal and CharClass don't need sequence declarations
    }
}

/// Convert a single iXML rule to Earlgrey format
fn convert_rule(builder: GrammarBuilder, rule: &Rule, created_reps: &mut std::collections::HashSet<String>) -> Result<GrammarBuilder, String> {
    // For now, we'll ignore marks on the rule itself
    // We'll handle them later when generating XML

    convert_alternatives(builder, &rule.name, &rule.alternatives, created_reps)
}

/// Convert alternatives (multiple options separated by |)
fn convert_alternatives(mut builder: GrammarBuilder, rule_name: &str, alts: &Alternatives, created_reps: &mut std::collections::HashSet<String>) -> Result<GrammarBuilder, String> {
    for seq in &alts.alts {
        builder = convert_sequence(builder, rule_name, seq, created_reps)?;
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

/// Helper function to parse a hex character from a string like "#9" or "#a0"
fn parse_hex_char(s: &str) -> Option<char> {
    if !s.starts_with('#') {
        return None;
    }
    let hex_part = &s[1..];
    if let Ok(code_point) = u32::from_str_radix(hex_part, 16) {
        char::from_u32(code_point)
    } else {
        None
    }
}

/// Parse a character class content string and return a predicate function
/// Examples: "'a'-'z'" → matches a-z, "L" → matches Unicode Letter category, "#30-#39" → matches 0-9
fn parse_char_class(content: &str, negated: bool) -> Box<dyn Fn(&str) -> bool + 'static> {
    // Parse the content to extract ranges and individual characters
    let mut chars = Vec::new();
    let mut ranges = Vec::new();
    let mut unicode_categories = Vec::new();

    let content = content.trim();
    let parts: Vec<&str> = content.split(';').map(|s| s.trim()).collect();

    for part in parts {
        // Split by comma or pipe to get individual elements
        // In character classes, both , and | separate alternatives (OR)
        let elements: Vec<&str> = part.split(|c| c == ',' || c == '|').map(|s| s.trim()).collect();

        for element in elements {
            // Check for hex character range: #30-#39
            if element.starts_with('#') && element.contains('-') {
                // Try to parse as hex range
                if let Some(dash_pos) = element[1..].find('-') {
                    let actual_dash_pos = dash_pos + 1;
                    let start_part = &element[..actual_dash_pos];
                    let end_part = &element[actual_dash_pos + 1..];

                    if end_part.starts_with('#') {
                        let start_char = parse_hex_char(start_part);
                        let end_char = parse_hex_char(end_part);
                        if let (Some(start), Some(end)) = (start_char, end_char) {
                            ranges.push((start, end));
                            continue;
                        }
                    }
                }
                // If not a range, treat as single hex char
                if let Some(ch) = parse_hex_char(element) {
                    chars.push(ch);
                }
            }
            // Check for quoted character range: "a"-"z" or 'a'-'z'
            else if (element.starts_with('\'') || element.starts_with('"')) && element.contains('-') {
                // Look for pattern: "x"-"y" or 'x'-'y'
                let quote = if element.starts_with('\'') { '\'' } else { '"' };

                // Find the closing quote
                if let Some(first_close) = element[1..].find(quote) {
                    let first_close = first_close + 1;

                    // Check if there's a dash after the closing quote
                    let after_close = &element[first_close + 1..];
                    if after_close.starts_with('-') && after_close.len() > 1 {
                        // Check if there's another quoted char after the dash
                        let after_dash = &after_close[1..];
                        if after_dash.starts_with('\'') || after_dash.starts_with('"') {
                            // This is a range
                            let start_str = &element[1..first_close];
                            let start_char = start_str.chars().next();

                            let end_quote = if after_dash.starts_with('\'') { '\'' } else { '"' };
                            if let Some(end_close) = after_dash[1..].find(end_quote) {
                                let end_str = &after_dash[1..end_close + 1];
                                let end_char = end_str.chars().next();

                                if let (Some(start), Some(end)) = (start_char, end_char) {
                                    ranges.push((start, end));
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Not a range, treat as quoted string of individual characters
                let inner = element.trim_matches('\'').trim_matches('"');
                for ch in inner.chars() {
                    chars.push(ch);
                }
            }
            // Single hex character
            else if element.starts_with('#') {
                if let Some(ch) = parse_hex_char(element) {
                    chars.push(ch);
                }
            }
            // Single quoted string (characters)
            else if (element.starts_with('\'') && element.ends_with('\'')) || (element.starts_with('"') && element.ends_with('"')) {
                let inner = element.trim_matches('\'').trim_matches('"');
                for ch in inner.chars() {
                    chars.push(ch);
                }
            }
            // Unicode category
            else if element.len() == 1 || element.len() == 2 {
                unicode_categories.push(element.to_string());
            }
        }
    }

    // Create the predicate function
    Box::new(move |s: &str| {
        // Check if the string is exactly one character (not one byte)
        if s.chars().count() != 1 {
            return false;
        }
        let ch = s.chars().next().unwrap();

        let mut matches = false;

        // Check individual characters
        if chars.contains(&ch) {
            matches = true;
        }

        // Check ranges
        for (start, end) in &ranges {
            if ch >= *start && ch <= *end {
                matches = true;
                break;
            }
        }

        // Check Unicode categories
        for category in &unicode_categories {
            let category_matches = match category.as_str() {
                "L" => ch.is_alphabetic(),
                "Ll" => ch.is_lowercase(),
                "Lu" => ch.is_uppercase(),
                "Lt" => ch.is_uppercase(), // Titlecase (approximation)
                "Lm" => ch.is_alphabetic(), // Modifier letter (approximation)
                "Lo" => ch.is_alphabetic(), // Other letter (approximation)
                "M" => false, // Mark categories (not easily checked in Rust)
                "N" => ch.is_numeric(),
                "Nd" => ch.is_numeric() && ch.is_ascii_digit(),
                "Nl" => ch.is_numeric(), // Letter number (approximation)
                "No" => ch.is_numeric(), // Other number (approximation)
                "P" => ch.is_ascii_punctuation(),
                "S" => !ch.is_alphanumeric() && !ch.is_whitespace(), // Symbol (approximation)
                "Z" => ch.is_whitespace(),
                "Zs" => ch == ' ', // Space separator
                "Zl" => ch == '\u{2028}', // Line separator
                "Zp" => ch == '\u{2029}', // Paragraph separator
                "C" => ch.is_control(),
                _ => false,
            };
            if category_matches {
                matches = true;
                break;
            }
        }

        // Apply negation if needed
        if negated {
            !matches
        } else {
            matches
        }
    })
}

/// Convert a sequence (multiple factors in a row)
fn convert_sequence(mut builder: GrammarBuilder, rule_name: &str, seq: &Sequence, created_reps: &mut std::collections::HashSet<String>) -> Result<GrammarBuilder, String> {
    // Build a list of symbols (terminals and nonterminals) for this production
    let mut symbols = Vec::new();

    for factor in &seq.factors {
        let (new_builder, symbol_name) = convert_factor(builder, factor, created_reps)?;
        builder = new_builder;
        symbols.push(symbol_name);
    }

    // Add the production rule: rule_name := symbols[0] symbols[1] ...
    let symbol_strs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
    builder = builder.rule(rule_name, &symbol_strs);
    Ok(builder)
}

/// Convert a factor (a single grammar element, possibly with repetition)
/// Helper to get suffix for marked symbols
fn mark_suffix(mark: Mark) -> &'static str {
    match mark {
        Mark::None => "",
        Mark::Attribute => "attr",
        Mark::Hidden => "hidden",
        Mark::Promoted => "promoted",
    }
}

/// Convert a sequence to a list of symbol names (for use in separated repetition)
fn convert_sequence_to_symbols(mut builder: GrammarBuilder, seq: &Sequence, created_reps: &mut std::collections::HashSet<String>) -> Result<(GrammarBuilder, Vec<String>), String> {
    let mut symbols = Vec::new();

    for factor in &seq.factors {
        let (new_builder, symbol_name) = convert_factor(builder, factor, created_reps)?;
        builder = new_builder;
        symbols.push(symbol_name);
    }

    Ok((builder, symbols))
}

fn convert_factor(mut builder: GrammarBuilder, factor: &Factor, created_reps: &mut std::collections::HashSet<String>) -> Result<(GrammarBuilder, String), String> {
    // First get the base symbol name
    let (new_builder, base_name) = match &factor.base {
        BaseFactor::Literal { value, insertion: _, mark } => {
            // For character-level parsing, split literal into individual characters
            let base = if value.chars().count() == 1 {
                // Single character - terminal is already defined, just return the name
                let ch = value.chars().next().unwrap();
                char_terminal_name(ch)
            } else {
                // Multi-character literal - create a sequence rule
                let seq_name = format!("lit_seq_{}", normalize_literal_sequence(value));

                // Only create the rule if it hasn't been created yet
                if !created_reps.contains(&seq_name) {
                    created_reps.insert(seq_name.clone());

                    // Collect character terminal names
                    let char_symbols: Vec<String> = value.chars()
                        .map(|ch| char_terminal_name(ch))
                        .collect();

                    // Create a rule: seq_name := char1 char2 char3 ...
                    builder = builder.rule(&seq_name, &char_symbols.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                }
                seq_name
            };

            // If the literal has a mark, use the pre-created wrapper nonterminal
            if *mark != Mark::None {
                let marked_name = format!("{}_{}", base, mark_suffix(*mark));
                (builder, marked_name)
            } else {
                (builder, base)
            }

            // TODO: Track insertion flag for XML generation
        }
        BaseFactor::Nonterminal { name, mark: _ } => {
            // Just reference the nonterminal by name
            // TODO: Track mark for XML generation
            (builder, name.clone())
        }
        BaseFactor::CharClass { content, negated, mark } => {
            // Terminal was already defined in first pass, just return the name
            let base = if *negated {
                format!("charclass_neg_{}", normalize_charclass_content(content))
            } else {
                format!("charclass_{}", normalize_charclass_content(content))
            };

            // If the character class has a mark, use the pre-created wrapper nonterminal
            if *mark != Mark::None {
                let marked_name = format!("{}_{}", base, mark_suffix(*mark));
                (builder, marked_name)
            } else {
                (builder, base)
            }
        }
        BaseFactor::Group { alternatives } => {
            // Generate a unique name for this group
            let group_id = GROUP_COUNTER.fetch_add(1, Ordering::SeqCst);
            let group_name = format!("group_{}", group_id);

            // Declare the nonterminal for this group
            let mut b = builder.nonterm(&group_name);

            // Convert each alternative in the group to a production rule
            for seq in &alternatives.alts {
                // Build symbols list for this alternative
                let mut symbols = Vec::new();
                for inner_factor in &seq.factors {
                    let (new_builder, symbol_name) = convert_factor(b, inner_factor, created_reps)?;
                    b = new_builder;
                    symbols.push(symbol_name);
                }

                // Add production rule: group_name := symbols[0] symbols[1] ...
                b = b.rule(&group_name, &symbols.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }

            (b, group_name)
        }
    };
    builder = new_builder;

    // Handle repetition by creating helper rules
    match &factor.repetition {
        Repetition::None => Ok((builder, base_name)),
        Repetition::OneOrMore => {
            // Create a new rule: base_name_plus := base_name | base_name_plus base_name
            let plus_name = format!("{}_plus", base_name);

            // Check if we've already created this repetition rule
            if !created_reps.contains(&plus_name) {
                created_reps.insert(plus_name.clone());

                // Groups need dynamic declaration (can't predict group_N names upfront)
                if base_name.starts_with("group_") {
                    builder = builder.nonterm(&plus_name);
                }
                // Other nonterminals already declared in upfront pass
                builder = builder.rule(&plus_name, &[&base_name]);
                builder = builder.rule(&plus_name, &[&plus_name, &base_name]);
            }
            Ok((builder, plus_name))
        }
        Repetition::ZeroOrMore => {
            // Create a new rule: base_name_star := ε | base_name_star base_name (LEFT recursion like +)
            let star_name = format!("{}_star", base_name);

            // Check if we've already created this repetition rule
            if !created_reps.contains(&star_name) {
                created_reps.insert(star_name.clone());

                // Groups need dynamic declaration (can't predict group_N names upfront)
                if base_name.starts_with("group_") {
                    builder = builder.nonterm(&star_name);
                }
                // Other nonterminals already declared in upfront pass
                builder = builder.rule(&star_name, &[] as &[&str]); // epsilon production
                builder = builder.rule(&star_name, &[&star_name, &base_name]); // LEFT recursion
            }
            Ok((builder, star_name))
        }
        Repetition::Optional => {
            // Create a new rule: base_name_opt := ε | base_name
            let opt_name = format!("{}_opt", base_name);

            // Check if we've already created this optional rule
            if !created_reps.contains(&opt_name) {
                created_reps.insert(opt_name.clone());

                // Groups need dynamic declaration (can't predict group_N names upfront)
                if base_name.starts_with("group_") {
                    builder = builder.nonterm(&opt_name);
                }
                // Other nonterminals already declared in upfront pass
                builder = builder.rule(&opt_name, &[] as &[&str]); // epsilon production
                builder = builder.rule(&opt_name, &[&base_name]);
            }
            Ok((builder, opt_name))
        }
        Repetition::SeparatedZeroOrMore(sep) => {
            // base**(sep) := ε | base_sep_plus
            // Create unique names based on separator to avoid duplicates
            let sep_id = normalize_sequence(sep);
            let star_name = format!("{}_sep_{}_star", base_name, sep_id);
            let plus_name = format!("{}_sep_{}_plus", base_name, sep_id);

            // Check if we've already created this repetition rule
            if !created_reps.contains(&star_name) {
                created_reps.insert(star_name.clone());
                created_reps.insert(plus_name.clone());

                // Convert separator sequence to runtime symbols
                let (new_builder, sep_symbols) = convert_sequence_to_symbols(builder, sep, created_reps)?;
                builder = new_builder;

                if base_name.starts_with("group_") {
                    builder = builder.nonterm(&star_name);
                    builder = builder.nonterm(&plus_name);
                }

                // base_sep_star := ε | base_sep_plus
                builder = builder.rule(&star_name, &[] as &[&str]);
                builder = builder.rule(&star_name, &[&plus_name]);

                // base_sep_plus := base | base_sep_plus sep base
                builder = builder.rule(&plus_name, &[&base_name]);
                let mut plus_rule = vec![plus_name.clone()];
                plus_rule.extend(sep_symbols.iter().map(|s| s.clone()));
                plus_rule.push(base_name.clone());
                let rule_refs: Vec<&str> = plus_rule.iter().map(|s| s.as_str()).collect();
                builder = builder.rule(&plus_name, &rule_refs);
            }

            Ok((builder, star_name))
        }
        Repetition::SeparatedOneOrMore(sep) => {
            // base++(sep) := base | base_sep_plus sep base
            // Create unique name based on separator to avoid duplicates
            let sep_id = normalize_sequence(sep);
            let plus_name = format!("{}_sep_{}_plus", base_name, sep_id);

            // Check if we've already created this repetition rule
            if !created_reps.contains(&plus_name) {
                created_reps.insert(plus_name.clone());

                // Convert separator sequence to runtime symbols
                let (new_builder, sep_symbols) = convert_sequence_to_symbols(builder, sep, created_reps)?;
                builder = new_builder;

                if base_name.starts_with("group_") {
                    builder = builder.nonterm(&plus_name);
                }

                // base_sep_plus := base | base_sep_plus sep base
                builder = builder.rule(&plus_name, &[&base_name]);
                let mut plus_rule = vec![plus_name.clone()];
                plus_rule.extend(sep_symbols.iter().map(|s| s.clone()));
                plus_rule.push(base_name.clone());
                let rule_refs: Vec<&str> = plus_rule.iter().map(|s| s.as_str()).collect();
                builder = builder.rule(&plus_name, &rule_refs);
            }

            Ok((builder, plus_name))
        }
    }
}

/// Simple XML node representation
#[derive(Clone, Debug, PartialEq)]
pub enum XmlNode {
    Element { name: String, attributes: Vec<(String, String)>, children: Vec<XmlNode> },
    Text(String),
    Attribute { name: String, value: String }, // For @mark - to be extracted by parent
}

impl XmlNode {
    fn escape_xml_attr(s: &str) -> String {
        // We use single quotes for attribute values
        // Per XML spec, in attributes we must escape: &, <, ' (when using single quotes)
        // We don't need to escape > or " in single-quoted attributes
        s.replace('&', "&amp;")
         .replace('<', "&lt;")
         .replace('\'', "&apos;")
    }

    fn escape_xml_text(s: &str) -> String {
        // In text content, we must escape: &, <
        // We don't escape > in text content per the iXML spec examples
        s.replace('&', "&amp;")
         .replace('<', "&lt;")
    }

    /// Check if this node is a self-closing element (has no children)
    fn is_self_closing(&self) -> bool {
        matches!(self, XmlNode::Element { children, .. } if children.is_empty())
    }

    pub fn to_xml(&self) -> String {
        // iXML spec does not prescribe specific formatting - both compact and canonical are valid
        // We use canonical format (with newlines) as the default pretty-printing style
        // Note: Some test cases like "marked" have manually whitespace-stripped expected outputs
        // which won't match our canonical format, but both are conformant per iXML spec
        let compact_mode = false;
        self.to_xml_internal(0, true, compact_mode)
    }

    /// Internal XML serialization with canonical iXML formatting
    ///
    /// The canonical format:
    /// - Opening and closing tags are written without their final `>`
    /// - The `>` appears on the next line with indentation before the next content
    /// - Exception: root element's final closing tag includes its `>`
    /// Check if a node produces inline content (text without element tags)
    /// Used for formatting decisions - these nodes should be treated like text
    fn is_inline_content(node: &XmlNode) -> bool {
        match node {
            XmlNode::Text(_) => true,
            XmlNode::Element { name, .. } => {
                // Hidden and promoted elements unwrap to inline content
                // Note: _repeat_container and group are unwrapped but not inline
                // because they may contain elements that need canonical formatting
                name == "__hidden__" || name == "__promoted__" || name == "_hidden" || name == "_promoted"
            }
            XmlNode::Attribute { .. } => true,
        }
    }

    fn to_xml_internal(&self, depth: usize, is_root: bool, compact_mode: bool) -> String {
        match self {
            XmlNode::Element { name, attributes, children } => {
                // Skip rendering hidden, promoted, repeat container, and group wrapper elements
                // Just render their children directly, but apply canonical formatting
                if name == "__hidden__" || name == "__promoted__" || name == "_hidden" || name == "_promoted"
                    || name == "_repeat_container" || name == "group"
                {
                    if compact_mode {
                        return children.iter()
                            .map(|child| child.to_xml_internal(depth, false, compact_mode))
                            .collect::<Vec<_>>()
                            .join("");
                    }

                    // For canonical format, we need to apply element-to-text transition formatting
                    // when unwrapping containers
                    let indent = "   ".repeat(depth);
                    let mut result = String::new();

                    for (i, child) in children.iter().enumerate() {
                        let curr_is_inline = Self::is_inline_content(child);

                        if i > 0 {
                            let prev_child = &children[i - 1];
                            let prev_is_inline = Self::is_inline_content(prev_child);

                            if !prev_is_inline && curr_is_inline {
                                // Previous was element, current is inline
                                // Check if there's an element after this inline content
                                let has_element_after = children.iter().skip(i + 1)
                                    .any(|c| !Self::is_inline_content(c));

                                if has_element_after {
                                    // Need to add newline + indent + > after previous element
                                    result.push('\n');
                                    result.push_str(&indent);
                                    if Self::is_self_closing(prev_child) {
                                        result.push_str("/>");
                                    } else {
                                        result.push('>');
                                    }
                                } else {
                                    // No more elements, close previous inline
                                    if Self::is_self_closing(prev_child) {
                                        result.push_str("/>");
                                    } else {
                                        result.push('>');
                                    }
                                }
                            } else if !prev_is_inline && !curr_is_inline {
                                // Previous was element, current is also element
                                result.push('\n');
                                result.push_str(&indent);
                                if Self::is_self_closing(prev_child) {
                                    result.push_str("/>");
                                } else {
                                    result.push('>');
                                }
                            }
                            // If prev is inline, curr continues inline
                        }

                        result.push_str(&child.to_xml_internal(depth, false, compact_mode));
                    }

                    return result;
                }

                let indent = "   ".repeat(depth);

                let attrs_str = if attributes.is_empty() {
                    String::new()
                } else {
                    format!(" {}", attributes.iter()
                        .map(|(k, v)| format!("{}='{}'", k, Self::escape_xml_attr(v)))
                        .collect::<Vec<_>>()
                        .join(" "))
                };

                // In fully compact mode, serialize everything on one line
                if compact_mode {
                    let content: String = children.iter()
                        .map(|child| child.to_xml_internal(depth, false, true))
                        .collect();
                    return format!("<{}{}>{}</{}>", name, attrs_str, content, name);
                }

                // Check if this element only contains text (no element children)
                let only_text = children.iter().all(|c| Self::is_inline_content(c));

                if children.is_empty() {
                    // Self-closing element - in canonical format, just the opening tag without >
                    // The /> will be added by the parent when iterating children
                    format!("<{}{}", name, attrs_str)
                } else if only_text {
                    // Element with only text content - use compact format
                    // Closing tag gets final > only if this is the root element
                    let text_content: String = children.iter()
                        .filter_map(|c| match c {
                            XmlNode::Text(s) => Some(Self::escape_xml_text(s)),
                            _ => None,
                        })
                        .collect();
                    if is_root {
                        format!("<{}{}>{}</{}>", name, attrs_str, text_content, name)
                    } else {
                        format!("<{}{}>{}</{}", name, attrs_str, text_content, name)
                    }
                } else {
                    // Element with child elements - use canonical format
                    let mut result = format!("<{}{}", name, attrs_str);

                    // Check if there are any non-inline children - if so, use canonical line breaking
                    let has_elements = children.iter().any(|c| !Self::is_inline_content(c));

                    for (i, child) in children.iter().enumerate() {
                        let curr_is_inline = Self::is_inline_content(child);

                        if i == 0 {
                            // First child - in canonical format with elements, always add newline before >
                            if has_elements {
                                result.push('\n');
                                result.push_str(&indent);
                                result.push_str("   ");
                            }
                            // Close parent's opening tag
                            result.push('>');
                        } else {
                            // Not the first child
                            let prev_child = &children[i - 1];
                            let prev_is_inline = Self::is_inline_content(prev_child);

                            if !prev_is_inline && !curr_is_inline {
                                // Previous was an element, current is also an element
                                // Close previous element with > on a new line before current element
                                result.push('\n');
                                result.push_str(&indent);
                                result.push_str("   ");

                                if Self::is_self_closing(prev_child) {
                                    result.push_str("/>");
                                } else {
                                    result.push('>');
                                }

                                // Current element goes inline after the >
                            } else if !prev_is_inline && curr_is_inline {
                                // Previous was an element, current is inline content
                                // Check if there's an element after this inline content (look ahead)
                                let has_element_after = children.iter().skip(i + 1)
                                    .any(|c| !Self::is_inline_content(c));

                                if has_element_after {
                                    // There's an element coming after this inline content
                                    // Close previous element with newline + indent + >
                                    result.push('\n');
                                    result.push_str(&indent);
                                    result.push_str("   ");
                                    if Self::is_self_closing(prev_child) {
                                        result.push_str("/>");
                                    } else {
                                        result.push('>');
                                    }
                                    // Inline content continues inline after the >
                                } else {
                                    // No more elements, just inline content remaining
                                    // Close previous element inline
                                    if Self::is_self_closing(prev_child) {
                                        result.push_str("/>");
                                    } else {
                                        result.push('>');
                                    }
                                    // Inline content continues inline, no newline
                                }
                            }
                            // If prev is inline (text or __hidden__), curr continues inline regardless of type
                        }

                        result.push_str(&child.to_xml_internal(depth + 1, false, compact_mode));
                    }

                    // Close the last child if it's an element (not inline content)
                    // Inline content doesn't need closing, and parent closing tag appears inline after it
                    if let Some(last_child) = children.last() {
                        if Self::is_inline_content(last_child) {
                            // Last child is inline content - parent closing tag goes inline, no newline
                        } else {
                            // Last child is an element - close it on a new line
                            result.push('\n');
                            result.push_str(&indent);
                            if Self::is_self_closing(last_child) {
                                result.push_str("/>");
                            } else {
                                result.push('>');
                            }
                        }
                    }

                    // Close this element
                    if is_root {
                        result.push_str(&format!("</{}>", name));
                    } else {
                        result.push_str(&format!("</{}", name));
                    }

                    result
                }
            }
            XmlNode::Text(s) => Self::escape_xml_text(s),
            XmlNode::Attribute { .. } => {
                // Attributes should have been extracted by parent, shouldn't appear in output
                String::new()
            }
        }
    }
}

/// Build an EarleyForest configured for XML generation
/// Returns the forest which can be used with forest.eval(&parse_trees)
pub fn build_xml_forest(grammar: &IxmlGrammar) -> EarleyForest<'static, XmlNode> {
    // Reset group counter to ensure same group IDs as during grammar conversion
    GROUP_COUNTER.store(0, Ordering::SeqCst);

    // Create an EarleyForest to walk the parse tree
    let mut forest = EarleyForest::new(|_symbol, token| {
        // For terminals (leaves), just return the token text
        XmlNode::Text(token.to_string())
    });

    // Register actions for all productions in the grammar
    // Unlike traditional semantic actions, Earlgrey requires actions per production,
    // not per nonterminal. The format is "nonterminal -> symbol1 symbol2 ..."
    for rule in &grammar.rules {
        register_rule_actions(&mut forest, rule);
    }

    // Also register actions for literal sequence nonterminals
    register_literal_sequence_actions(&mut forest, grammar);

    // Also register actions for group nonterminals
    register_group_actions(&mut forest, grammar);

    // Register actions for marked literal wrappers
    register_marked_literal_actions(&mut forest, grammar);

    forest
}

/// Helper function to register actions for all productions of a rule
/// Production format: "nonterminal -> symbol1 symbol2 ..."
fn register_rule_actions(
    forest: &mut EarleyForest<'static, XmlNode>,
    rule: &Rule,
) {
    use std::collections::HashSet;
    let mut registered = HashSet::new();

    let rule_name = &rule.name;
    let rule_mark = rule.mark;

    for seq in &rule.alternatives.alts {
        // Build the list of symbols for this production
        let mut symbols = Vec::new();
        let mut base_names = Vec::new();  // Track base names for repetition action registration
        let mut factor_marks = Vec::new();

        for factor in &seq.factors {
            let (base_name, symbol_name) = get_factor_symbol(factor);
            symbols.push(symbol_name);
            base_names.push(base_name);

            // Extract mark from nonterminal factors
            let factor_mark = match &factor.base {
                BaseFactor::Nonterminal { mark, .. } => *mark,
                _ => Mark::None,
            };
            factor_marks.push(factor_mark);
        }

        // Create the production string: "rule_name -> symbol1 symbol2 ..."
        let production = if symbols.is_empty() {
            format!("{} -> ", rule_name)  // Empty production (epsilon)
        } else {
            format!("{} -> {}", rule_name, symbols.join(" "))
        };

        // Register the action for this production
        let rule_name_for_closure = rule_name.to_string();
        let base_names_for_closure = base_names.clone();
        forest.action(&production, move |nodes: Vec<XmlNode>| {
            // Separate attribute children from regular children and process marks
            let mut attributes = Vec::new();
            let mut children = Vec::new();

            for (i, node) in nodes.into_iter().enumerate() {
                let factor_mark = if i < factor_marks.len() {
                    factor_marks[i]
                } else {
                    Mark::None
                };
                let factor_base_name = if i < base_names_for_closure.len() {
                    &base_names_for_closure[i]
                } else {
                    ""
                };

                // Handle repetition containers - extract their children
                // Check what type of element this is first
                let should_unwrap = if let XmlNode::Element { ref name, .. } = &node {
                    name == "_repeat_container" || name == "_hidden" || name == "group"
                } else {
                    false
                };

                if should_unwrap {
                    // Now we can destructure and move node
                    if let XmlNode::Element { name, children: inner, .. } = node {
                        if name == "_repeat_container" {
                            // Recursively unwrap any nested containers
                            for child in inner {
                                match child {
                                    XmlNode::Element { name: child_name, children: nested, attributes: child_attrs } => {
                                        if child_name == "_repeat_container" {
                                            // Recursively process nested container - extract attributes
                                            for nested_child in nested {
                                                match nested_child {
                                                    XmlNode::Attribute { name: attr_name, value } => {
                                                        attributes.push((attr_name, value));
                                                    }
                                                    other => children.push(other),
                                                }
                                            }
                                        } else {
                                            children.push(XmlNode::Element { name: child_name, children: nested, attributes: child_attrs });
                                        }
                                    }
                                    // Extract Attribute nodes and add to attributes list
                                    XmlNode::Attribute { name: attr_name, value } => {
                                        attributes.push((attr_name, value));
                                    }
                                    other => children.push(other),
                                }
                            }
                        } else if name == "_hidden" || name == "group" {
                            // Promote all children from hidden elements and groups (not just text)
                            // Recursively unwrap any containers inside the hidden/group
                            for child in inner {
                                match child {
                                    XmlNode::Element { name: child_name, children: nested, attributes: child_attrs } => {
                                        if child_name == "_repeat_container" || child_name == "_hidden" || child_name == "group" {
                                            // Recursively unwrap nested containers/groups and extract attributes
                                            for nested_child in nested {
                                                match nested_child {
                                                    XmlNode::Attribute { name: attr_name, value } => {
                                                        attributes.push((attr_name, value));
                                                    }
                                                    other => children.push(other),
                                                }
                                            }
                                        } else {
                                            children.push(XmlNode::Element { name: child_name, children: nested, attributes: child_attrs });
                                        }
                                    }
                                    // Extract Attribute nodes and add to attributes list
                                    XmlNode::Attribute { name: attr_name, value } => {
                                        attributes.push((attr_name, value));
                                    }
                                    other => children.push(other),
                                }
                            }
                        }
                        continue;
                    }
                }

                match (node, factor_mark) {
                    // Attribute mark on element - convert element to attribute
                    (XmlNode::Element { name, children: inner, .. }, Mark::Attribute) => {
                        let value = extract_text_from_nodes(&inner);
                        attributes.push((name, value));
                    }
                    // Attribute nodes - extract and add to attributes list
                    (XmlNode::Attribute { name, value }, _) => {
                        attributes.push((name, value));
                    }
                    // Hidden nodes - unwrap and promote children and attributes
                    (XmlNode::Element { children: inner, attributes: hidden_attrs, .. }, Mark::Hidden) => {
                        // Promote attributes from hidden element to parent
                        attributes.extend(hidden_attrs);
                        // Promote children
                        children.extend(inner);
                    }
                    // Promoted nodes - make hidden elements visible with proper name
                    (node, Mark::Promoted) => {
                        let promoted_element = match node {
                            // For _hidden wrapper: extract content and create element with base name
                            XmlNode::Element { name, children: inner, attributes: attrs } if name == "_hidden" => {
                                XmlNode::Element {
                                    name: factor_base_name.to_string(),
                                    attributes: attrs,
                                    children: inner,
                                }
                            }
                            // For regular elements: wrap in element with promoted name
                            // This handles the case where hidden rule returned single child without _hidden wrapper
                            other => XmlNode::Element {
                                name: factor_base_name.to_string(),
                                attributes: vec![],
                                children: vec![other],
                            },
                        };
                        // Add directly to children - no wrapper needed
                        children.push(promoted_element);
                    }
                    // Regular nodes - keep as is
                    (node, _) => {
                        children.push(node);
                    }
                }
            }

            // Apply mark from rule definition
            match rule_mark {
                Mark::Hidden => {
                    // Hidden: return children without wrapper
                    // Wrap in _hidden element that gets unwrapped by parent
                    // Include attributes so they can be promoted to parent
                    let attr_nodes: Vec<XmlNode> = attributes.into_iter()
                        .map(|(n, v)| XmlNode::Attribute { name: n, value: v })
                        .collect();
                    if children.len() == 1 && attr_nodes.is_empty() {
                        children.into_iter().next().unwrap()
                    } else {
                        // Combine attributes and children
                        let mut all_children = attr_nodes;
                        all_children.extend(children);
                        XmlNode::Element {
                            name: "_hidden".to_string(),
                            attributes: vec![],
                            children: all_children,
                        }
                    }
                }
                Mark::Attribute => {
                    // Attribute: extract text content and create Attribute node
                    let text_value = extract_text_from_nodes(&children);
                    XmlNode::Attribute {
                        name: rule_name_for_closure.clone(),
                        value: text_value,
                    }
                }
                Mark::Promoted => {
                    // Promoted: return children without wrapper
                    if children.len() == 1 {
                        children.into_iter().next().unwrap()
                    } else {
                        XmlNode::Element {
                            name: "_promoted".to_string(),
                            attributes: vec![],
                            children,
                        }
                    }
                }
                Mark::None => {
                    // Normal: wrap in element
                    XmlNode::Element {
                        name: rule_name_for_closure.clone(),
                        attributes,
                        children,
                    }
                }
            }
        });

        // Also register actions for any helper rules we create for repetition
        for (factor, base_name) in seq.factors.iter().zip(base_names.iter()) {
            register_repetition_actions(forest, factor, base_name, &mut registered);
        }
    }
}

/// Extract text content from a list of nodes
fn extract_text_from_nodes(nodes: &[XmlNode]) -> String {
    let mut result = String::new();
    for node in nodes {
        match node {
            XmlNode::Text(t) => result.push_str(t),
            XmlNode::Element { children, .. } => result.push_str(&extract_text_from_nodes(children)),
            XmlNode::Attribute { value, .. } => result.push_str(value),
        }
    }
    result
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
            let seq_name = format!("lit_seq_{}", normalize_literal_sequence(&literal));

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
            match &factor.base {
                BaseFactor::Literal { value, .. } => {
                    literals.insert(value.clone());
                }
                BaseFactor::Group { alternatives } => {
                    // Recurse into groups
                    collect_literals_from_alternatives(alternatives, literals);
                }
                _ => {},
            }
        }
    }
}

/// Register XML actions for all group nonterminals in the grammar
fn register_group_actions(forest: &mut EarleyForest<'static, XmlNode>, grammar: &IxmlGrammar) {
    // Use a local counter to track group IDs (don't use global GROUP_COUNTER which may have been
    // incremented by other code). We traverse in the same order as during grammar conversion.
    let mut group_counter = 0;

    // Traverse the grammar to find all groups and register their actions
    for rule in &grammar.rules {
        register_group_actions_from_alternatives(forest, &rule.alternatives, &mut group_counter);
    }
}

/// Helper to traverse alternatives and register actions for any groups found
fn register_group_actions_from_alternatives(
    forest: &mut EarleyForest<'static, XmlNode>,
    alts: &Alternatives,
    group_counter: &mut usize,
) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            if let BaseFactor::Group { alternatives } = &factor.base {
                // Assign ID to this group
                let group_id = *group_counter;
                *group_counter += 1;
                let group_name = format!("group_{}", group_id);

                // Register actions for each alternative in the group
                for group_seq in &alternatives.alts {
                    // Build symbol list for this alternative
                    let symbols = build_symbol_list_for_sequence(group_seq, group_counter);

                    // Create production string
                    let production = if symbols.is_empty() {
                        format!("{} -> ", group_name)  // Trailing space for epsilon production
                    } else {
                        format!("{} -> {}", group_name, symbols.join(" "))
                    };

                    // Action: groups just pass through their child nodes
                    forest.action(&production, move |nodes: Vec<XmlNode>| {
                        if nodes.is_empty() {
                            // Empty group - return hidden element (will be filtered out)
                            XmlNode::Element {
                                name: "_hidden".to_string(),
                                attributes: vec![],
                                children: vec![],
                            }
                        } else if nodes.len() == 1 {
                            nodes.into_iter().next().unwrap()
                        } else {
                            // Multiple nodes - wrap in hidden element (will be unwrapped)
                            XmlNode::Element {
                                name: "_hidden".to_string(),
                                attributes: vec![],
                                children: nodes,
                            }
                        }
                    });
                }

                // Recurse into the group's alternatives to find nested groups
                // Note: build_symbol_list_for_sequence already incremented counter for nested groups
                register_group_actions_from_alternatives(forest, alternatives, group_counter);
            }
        }
    }
}

/// Helper to build symbol list for a sequence (used in group action registration)
/// This version uses a local counter instead of the global GROUP_COUNTER
fn build_symbol_list_for_sequence(seq: &Sequence, group_counter: &mut usize) -> Vec<String> {
    let mut symbols = Vec::new();
    for factor in &seq.factors {
        let base_name = match &factor.base {
            BaseFactor::Literal { value, mark, .. } => {
                let base = if value.chars().count() == 1 {
                    let ch = value.chars().next().unwrap();
                    char_terminal_name(ch)
                } else {
                    format!("lit_seq_{}", normalize_literal_sequence(value))
                };

                // If marked, use the marked wrapper name
                if *mark != Mark::None {
                    format!("{}_{}", base, mark_suffix(*mark))
                } else {
                    base
                }
            }
            BaseFactor::Nonterminal { name, .. } => name.clone(),
            BaseFactor::CharClass { content, negated, .. } => {
                if *negated {
                    format!("charclass_neg_{}", normalize_charclass_content(content))
                } else {
                    format!("charclass_{}", normalize_charclass_content(content))
                }
            }
            BaseFactor::Group { .. } => {
                // For groups, use current counter (incremented by register_group_actions)
                let group_id = *group_counter;
                format!("group_{}", group_id)
            }
        };

        // Handle repetition
        let symbol_name = match &factor.repetition {
            Repetition::None => base_name,
            Repetition::OneOrMore => format!("{}_plus", base_name),
            Repetition::ZeroOrMore => format!("{}_star", base_name),
            Repetition::Optional => format!("{}_opt", base_name),
            Repetition::SeparatedZeroOrMore(sep) => {
                let sep_id = normalize_sequence(sep);
                format!("{}_sep_{}_star", base_name, sep_id)
            }
            Repetition::SeparatedOneOrMore(sep) => {
                let sep_id = normalize_sequence(sep);
                format!("{}_sep_{}_plus", base_name, sep_id)
            }
        };

        symbols.push(symbol_name);
    }
    symbols
}

/// Get the symbol name for a factor (matches the logic in ast_to_earlgrey)
/// Returns (base_name, symbol_name) where base_name is without repetition suffix
/// Register actions for marked literal wrappers (e.g., char_dot_hidden -> char_dot)
fn register_marked_literal_actions(forest: &mut EarleyForest<'static, XmlNode>, grammar: &IxmlGrammar) {
    for rule in &grammar.rules {
        register_marked_literal_from_alternatives(forest, &rule.alternatives);
    }
}

fn register_marked_literal_from_alternatives(
    forest: &mut EarleyForest<'static, XmlNode>,
    alts: &Alternatives,
) {
    for seq in &alts.alts {
        for factor in &seq.factors {
            // Check if this factor is a marked literal
            if let BaseFactor::Literal { value, mark, .. } = &factor.base {
                if *mark != Mark::None {
                    // Get the base symbol name
                    let base = if value.chars().count() == 1 {
                        let ch = value.chars().next().unwrap();
                        char_terminal_name(ch)
                    } else {
                        format!("lit_seq_{}", normalize_literal_sequence(value))
                    };

                    let marked_name = format!("{}_{}", base, mark_suffix(*mark));
                    let production = format!("{} -> {}", marked_name, base);

                    // Register action based on mark type
                    match mark {
                        Mark::Hidden => {
                            // Hidden: don't include in output
                            forest.action(&production, |_nodes| {
                                // Return an empty element that will be unwrapped by parent
                                XmlNode::Element {
                                    name: "_hidden".to_string(),
                                    attributes: vec![],
                                    children: vec![],
                                }
                            });
                        }
                        Mark::Attribute => {
                            // Attribute: extract text and create attribute node
                            let name_clone = marked_name.clone();
                            forest.action(&production, move |nodes| {
                                let text = extract_text_from_nodes(&nodes);
                                XmlNode::Attribute {
                                    name: name_clone.clone(),
                                    value: text,
                                }
                            });
                        }
                        Mark::Promoted => {
                            // Promoted: pass through without wrapper
                            forest.action(&production, |mut nodes| {
                                if nodes.len() == 1 {
                                    nodes.pop().unwrap()
                                } else {
                                    XmlNode::Element {
                                        name: "_promoted".to_string(),
                                        attributes: vec![],
                                        children: nodes,
                                    }
                                }
                            });
                        }
                        Mark::None => {}
                    }
                }
            }

            // Check if this factor is a marked character class
            if let BaseFactor::CharClass { content, negated, mark } = &factor.base {
                if *mark != Mark::None {
                    // Get the base symbol name
                    let base = if *negated {
                        format!("charclass_neg_{}", normalize_charclass_content(content))
                    } else {
                        format!("charclass_{}", normalize_charclass_content(content))
                    };

                    let marked_name = format!("{}_{}", base, mark_suffix(*mark));
                    let production = format!("{} -> {}", marked_name, base);

                    // Register action based on mark type
                    match mark {
                        Mark::Hidden => {
                            // Hidden: don't include in output
                            forest.action(&production, |_nodes| {
                                // Return an empty element that will be unwrapped by parent
                                XmlNode::Element {
                                    name: "_hidden".to_string(),
                                    attributes: vec![],
                                    children: vec![],
                                }
                            });
                        }
                        Mark::Attribute => {
                            // Attribute: extract text and create attribute node
                            let name_clone = marked_name.clone();
                            forest.action(&production, move |nodes| {
                                let text = extract_text_from_nodes(&nodes);
                                XmlNode::Attribute {
                                    name: name_clone.clone(),
                                    value: text,
                                }
                            });
                        }
                        Mark::Promoted => {
                            // Promoted: pass through without wrapper
                            forest.action(&production, |mut nodes| {
                                if nodes.len() == 1 {
                                    nodes.pop().unwrap()
                                } else {
                                    XmlNode::Element {
                                        name: "_promoted".to_string(),
                                        attributes: vec![],
                                        children: nodes,
                                    }
                                }
                            });
                        }
                        Mark::None => {}
                    }
                }
            }

            // Recurse into groups
            if let BaseFactor::Group { alternatives } = &factor.base {
                register_marked_literal_from_alternatives(forest, alternatives);
            }
        }
    }
}

fn get_factor_symbol(factor: &Factor) -> (String, String) {
    let base_name = match &factor.base {
        BaseFactor::Literal { value, insertion: _, mark } => {
            let base = if value.chars().count() == 1 {
                // Single character - use char terminal name
                let ch = value.chars().next().unwrap();
                char_terminal_name(ch)
            } else {
                // Multi-character literal - use sequence name
                format!("lit_seq_{}", normalize_literal_sequence(value))
            };

            // If marked, use the marked wrapper name
            if *mark != Mark::None {
                format!("{}_{}", base, mark_suffix(*mark))
            } else {
                base
            }
        }
        BaseFactor::Nonterminal { name, mark: _ } => name.clone(),
        BaseFactor::CharClass { content, negated, mark } => {
            // Use the same naming as in convert_factor
            let base = if *negated {
                format!("charclass_neg_{}", normalize_charclass_content(content))
            } else {
                format!("charclass_{}", normalize_charclass_content(content))
            };

            // If marked, use the marked wrapper name (same as Literal handling)
            if *mark != Mark::None {
                format!("{}_{}", base, mark_suffix(*mark))
            } else {
                base
            }
        }
        BaseFactor::Group { .. } => {
            // Use the global counter to match convert_factor
            let group_id = GROUP_COUNTER.fetch_add(1, Ordering::SeqCst);
            format!("group_{}", group_id)
        }
    };

    // Handle repetition by using the helper rule name
    let symbol_name = match &factor.repetition {
        Repetition::None => base_name.clone(),
        Repetition::OneOrMore => format!("{}_plus", base_name),
        Repetition::ZeroOrMore => format!("{}_star", base_name),
        Repetition::Optional => format!("{}_opt", base_name),
        Repetition::SeparatedZeroOrMore(sep) => {
            let sep_id = normalize_sequence(sep);
            format!("{}_sep_{}_star", base_name, sep_id)
        }
        Repetition::SeparatedOneOrMore(sep) => {
            let sep_id = normalize_sequence(sep);
            format!("{}_sep_{}_plus", base_name, sep_id)
        }
    };

    (base_name, symbol_name)
}

/// Register actions for repetition helper rules
/// base_name is the symbol name without repetition suffix (passed from get_factor_symbol)
fn register_repetition_actions(
    forest: &mut EarleyForest<'static, XmlNode>,
    factor: &Factor,
    base_name: &str,
    registered: &mut std::collections::HashSet<String>,
) {
    // Use the passed base_name directly instead of recalculating it
    // This ensures we don't increment GROUP_COUNTER a second time for groups

    match &factor.repetition {
        Repetition::OneOrMore => {
            let plus_name = format!("{}_plus", base_name);
            if !registered.contains(&plus_name) {
                registered.insert(plus_name.clone());

                // Register actions for both productions: base and recursive
                // Repetitions pass through children unchanged
                forest.action(&format!("{} -> {}", plus_name, base_name), |nodes| {
                    // Base case - just pass through the child nodes
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                });
                forest.action(&format!("{} -> {} {}", plus_name, plus_name, base_name), |mut nodes| {
                    // Recursive case - flatten children from both sides
                    if nodes.len() >= 2 {
                        let right = nodes.pop().unwrap();
                        let left = nodes.pop().unwrap();

                        let mut all_children = vec![];
                        // Extract children from left (the recursive _plus result)
                        if let XmlNode::Element { children, name, .. } = left {
                            if name == "_repeat_container" {
                                all_children.extend(children);
                            } else {
                                all_children.push(XmlNode::Element { name, attributes: vec![], children });
                            }
                        } else {
                            all_children.push(left);
                        }

                        // Add right child
                        all_children.push(right);

                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: all_children }
                    } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                    }
                });
            }
        }
        Repetition::ZeroOrMore => {
            let star_name = format!("{}_star", base_name);
            if !registered.contains(&star_name) {
                registered.insert(star_name.clone());

                // Register actions for epsilon and recursive productions (LEFT recursion)
                // Epsilon production uses "_repeat_container" to match OneOrMore pattern
                let epsilon_prod = format!("{} -> ", star_name);
                forest.action(&epsilon_prod, |_nodes| {
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] }
                });
                // LEFT recursion: star_name base_name (same as OneOrMore)
                let recursive_prod = format!("{} -> {} {}", star_name, star_name, base_name);
                forest.action(&recursive_prod, |mut nodes| {
                    // Same pattern as OneOrMore - flatten children
                    if nodes.len() >= 2 {
                        let right = nodes.pop().unwrap();
                        let left = nodes.pop().unwrap();

                        let mut all_children = vec![];
                        if let XmlNode::Element { children, name, .. } = left {
                            if name == "_repeat_container" {
                                all_children.extend(children);
                            } else {
                                all_children.push(XmlNode::Element { name, attributes: vec![], children });
                            }
                        } else {
                            all_children.push(left);
                        }

                        all_children.push(right);

                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: all_children }
                    } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                    }
                });
            }
        }
        Repetition::Optional => {
            let opt_name = format!("{}_opt", base_name);
            if !registered.contains(&opt_name) {
                registered.insert(opt_name.clone());

                // Register actions for epsilon and base productions
                // Epsilon production - return empty container
                forest.action(&format!("{} -> ", opt_name), |_nodes| {
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] }
                });
                // Base production - pass through single nodes, wrap multiple
                forest.action(&format!("{} -> {}", opt_name, base_name), |mut nodes| {
                    if nodes.len() == 1 {
                        nodes.pop().unwrap()
                    } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                    }
                });
            }
        }
        Repetition::None => {}
        Repetition::SeparatedZeroOrMore(sep) => {
            // For base**(sep), we have: base_sep_star -> ε | base_sep_plus
            // and base_sep_plus -> base | base_sep_plus sep base
            let sep_id = normalize_sequence(sep);
            let star_name = format!("{}_sep_{}_star", base_name, sep_id);
            let plus_name = format!("{}_sep_{}_plus", base_name, sep_id);

            if !registered.contains(&star_name) {
                registered.insert(star_name.clone());

                // Register star actions
                forest.action(&format!("{} -> ", star_name), |_nodes| {
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] }
                });
                forest.action(&format!("{} -> {}", star_name, plus_name), |mut nodes| {
                    // nodes contains exactly one element: the _repeat_container from plus
                    // Pass it through directly instead of re-wrapping
                    if nodes.len() == 1 {
                        nodes.pop().unwrap()
                    } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                    }
                });
            }

            if !registered.contains(&plus_name) {
                registered.insert(plus_name.clone());

                // Get separator symbol names
                let mut group_counter = 0;
                let sep_symbols = build_symbol_list_for_sequence(&sep, &mut group_counter);

                // Register base case: base_sep_plus -> base
                forest.action(&format!("{} -> {}", plus_name, base_name), |nodes| {
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                });

                // Register recursive case: base_sep_plus -> base_sep_plus sep... base
                let mut recursive_pattern = format!("{} -> {}", plus_name, plus_name);
                for sep_sym in &sep_symbols {
                    recursive_pattern.push_str(&format!(" {}", sep_sym));
                }
                recursive_pattern.push_str(&format!(" {}", base_name));

                let sep_len = sep_symbols.len(); // Store the length to avoid capturing vec
                forest.action(&recursive_pattern, move |mut nodes| {
                    // Extract left (recursive result), separators (include them!), and right (base)
                    if nodes.is_empty() {
                        return XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] };
                    }

                    let right = nodes.pop().unwrap();
                    // Collect separator nodes instead of skipping them
                    let mut separators = vec![];
                    for _ in 0..sep_len {
                        if !nodes.is_empty() {
                            separators.push(nodes.pop().unwrap());
                        }
                    }
                    separators.reverse(); // Restore original order

                    let left = if !nodes.is_empty() { nodes.pop().unwrap() } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] }
                    };

                    let mut all_children = vec![];
                    if let XmlNode::Element { children, name, .. } = left {
                        if name == "_repeat_container" {
                            all_children.extend(children);
                        } else {
                            all_children.push(XmlNode::Element { name, attributes: vec![], children });
                        }
                    } else {
                        all_children.push(left);
                    }

                    // Add separators to output
                    all_children.extend(separators);
                    all_children.push(right);

                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: all_children }
                });
            }
        }
        Repetition::SeparatedOneOrMore(sep) => {
            // For base++(sep), we have: base_sep_plus -> base | base_sep_plus sep base
            let sep_id = normalize_sequence(sep);
            let plus_name = format!("{}_sep_{}_plus", base_name, sep_id);

            if !registered.contains(&plus_name) {
                registered.insert(plus_name.clone());

                // Get separator symbol names
                let mut group_counter = 0;
                let sep_symbols = build_symbol_list_for_sequence(&sep, &mut group_counter);

                // Register base case: base_sep_plus -> base
                forest.action(&format!("{} -> {}", plus_name, base_name), |nodes| {
                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: nodes }
                });

                // Register recursive case: base_sep_plus -> base_sep_plus sep... base
                let mut recursive_pattern = format!("{} -> {}", plus_name, plus_name);
                for sep_sym in &sep_symbols {
                    recursive_pattern.push_str(&format!(" {}", sep_sym));
                }
                recursive_pattern.push_str(&format!(" {}", base_name));

                let sep_len = sep_symbols.len(); // Store the length to avoid capturing vec
                forest.action(&recursive_pattern, move |mut nodes| {
                    // Extract left (recursive result), separators (include them!), and right (base)
                    if nodes.is_empty() {
                        return XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] };
                    }

                    let right = nodes.pop().unwrap();
                    // Collect separator nodes instead of skipping them
                    let mut separators = vec![];
                    for _ in 0..sep_len {
                        if !nodes.is_empty() {
                            separators.push(nodes.pop().unwrap());
                        }
                    }
                    separators.reverse(); // Restore original order

                    let left = if !nodes.is_empty() { nodes.pop().unwrap() } else {
                        XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: vec![] }
                    };

                    let mut all_children = vec![];
                    if let XmlNode::Element { children, name, .. } = left {
                        if name == "_repeat_container" {
                            all_children.extend(children);
                        } else {
                            all_children.push(XmlNode::Element { name, attributes: vec![], children });
                        }
                    } else {
                        all_children.push(left);
                    }

                    // Add separators to output
                    all_children.extend(separators);
                    all_children.push(right);

                    XmlNode::Element { name: "_repeat_container".to_string(), attributes: vec![], children: all_children }
                });
            }
        }
    }

    // Recurse into groups to register actions for nested factors
    if let BaseFactor::Group { alternatives } = &factor.base {
        for alt in &alternatives.alts {
            for nested_factor in &alt.factors {
                let (nested_base_name, _) = get_factor_symbol(nested_factor);
                register_repetition_actions(forest, nested_factor, &nested_base_name, registered);
            }
        }
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

    #[test]
    fn test_attribute_mark() {
        // Test that @name creates an attribute
        let ixml = r#"
            element: @name body.
            name: "foo".
            body: "bar".
        "#;

        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("element").expect("Failed to build grammar");
        let parser = EarleyParser::new(grammar);

        let input_str = "foobar";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                let forest = build_xml_forest(&ast);
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        let xml_string = tree.to_xml();
                        println!("Attribute mark XML: {}", xml_string);
                        // Should have name as an attribute
                        assert!(xml_string.contains("name="), "Should contain name attribute");
                        assert!(xml_string.contains("foo"), "Attribute value should be 'foo'");
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_hidden_mark() {
        // Test that -name hides the element
        let ixml = r#"
            sentence: word -space word.
            word: "hello" | "world".
            space: " ".
        "#;

        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("sentence").expect("Failed to build grammar");
        let parser = EarleyParser::new(grammar);

        let input_str = "hello world";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                let forest = build_xml_forest(&ast);
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        let xml_string = tree.to_xml();
                        println!("Hidden mark XML: {}", xml_string);
                        // Should not contain <space> element
                        assert!(!xml_string.contains("<space"), "Should not contain space element");
                        assert!(xml_string.contains("hello"), "Should contain 'hello'");
                        assert!(xml_string.contains("world"), "Should contain 'world'");
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_promoted_mark() {
        // Test that ^name promotes children to parent level
        let ixml = r#"
            container: ^wrapper body.
            wrapper: "prefix".
            body: "content".
        "#;

        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");
        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("container").expect("Failed to build grammar");
        let parser = EarleyParser::new(grammar);

        let input_str = "prefixcontent";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                let forest = build_xml_forest(&ast);
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        let xml_string = tree.to_xml();
                        println!("Promoted mark XML: {}", xml_string);
                        // wrapper should be promoted, so we shouldn't see <wrapper>
                        assert!(!xml_string.contains("<wrapper"), "Should not contain wrapper element");
                        assert!(xml_string.contains("prefix"), "Should contain 'prefix'");
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_char_class_range() {
        // Test character class with range like [a-z]
        let ixml = r#"letter: ['a'-'z']."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("letter").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test lowercase letters
        for ch in ['a', 'm', 'z'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_ok(), "Failed to parse '{}'", ch);
        }

        // Test that uppercase letters don't match
        let input_str = "A";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse uppercase 'A'");
    }

    #[test]
    fn test_char_class_individual_chars() {
        // Test character class with individual characters like ['a', 'e', 'i']
        let ixml = r#"vowel: ['a', 'e', 'i', 'o', 'u']."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("vowel").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test vowels
        for ch in ['a', 'e', 'i', 'o', 'u'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_ok(), "Failed to parse vowel '{}'", ch);
        }

        // Test that consonants don't match
        let input_str = "b";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse consonant 'b'");
    }

    #[test]
    fn test_char_class_negated() {
        // Test negated character class like ~['0'-'9']
        let ixml = r#"nondigit: ~['0'-'9']."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("nondigit").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test that letters match
        for ch in ['a', 'x', 'Z'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_ok(), "Failed to parse non-digit '{}'", ch);
        }

        // Test that digits don't match
        for ch in ['0', '5', '9'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_err(), "Should not parse digit '{}'", ch);
        }
    }

    #[test]
    fn test_char_class_unicode_category() {
        // Test Unicode category like [L] for letters
        let ixml = r#"letter: [L]."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("letter").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test that letters match
        for ch in ['a', 'Z', 'ñ'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_ok(), "Failed to parse letter '{}'", ch);
        }

        // Test that numbers don't match
        let input_str = "5";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse digit '5'");
    }

    #[test]
    fn test_char_class_with_repetition() {
        // Test character class with repetition like ['a'-'z']+
        let ixml = r#"word: ['a'-'z']+."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("word").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Parse "hello"
        let input_str = "hello";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        assert!(result.is_ok(), "Failed to parse 'hello'");

        // Generate XML
        if let Ok(trees) = result {
            let forest = build_xml_forest(&ast);
            let xml_result = forest.eval(&trees);

            match xml_result {
                Ok(tree) => {
                    let xml_string = tree.to_xml();
                    println!("Character class repetition XML: {}", xml_string);
                    // Check that all characters are present (will be wrapped in <repeat> tags)
                    assert!(xml_string.contains("h"));
                    assert!(xml_string.contains("e"));
                    assert!(xml_string.contains("l"));
                    assert!(xml_string.contains("o"));
                    assert!(xml_string.contains("<word>"));
                    assert!(xml_string.contains("</word>"));
                }
                Err(e) => panic!("Failed to build XML tree: {}", e),
            }
        }
    }

    #[test]
    fn test_char_class_xml_generation() {
        // Test that character classes generate proper XML
        let ixml = r#"digit: ['0'-'9']."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("digit").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        let input_str = "7";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                let forest = build_xml_forest(&ast);
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        let xml_string = tree.to_xml();
                        println!("Character class XML: {}", xml_string);
                        assert_eq!(xml_string, "<digit>7</digit>");
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_group_simple() {
        // Test simple group like (a | b)
        let ixml = r#"choice: ("a" | "b")."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("choice").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test 'a'
        let input_str = "a";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'a'");

        // Test 'b'
        let input_str = "b";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'b'");

        // Test 'c' should fail
        let input_str = "c";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse 'c'");
    }

    #[test]
    fn test_group_with_repetition() {
        // Test group with repetition like (a | b)+
        let ixml = r#"word: ("a" | "b")+."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("word").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test single character
        let input_str = "a";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'a'");

        // Test multiple characters
        let input_str = "abba";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'abba'");

        // Test invalid character should fail
        let input_str = "abc";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse 'abc'");
    }

    #[test]
    fn test_group_with_sequence() {
        // Test group with sequences like ("hello" | "world")
        let ixml = r#"greeting: ("hello" | "world")."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("greeting").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test "hello"
        let input_str = "hello";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'hello'");

        // Test "world"
        let input_str = "world";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_ok(), "Failed to parse 'world'");

        // Test "goodbye" should fail
        let input_str = "goodbye";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse 'goodbye'");
    }

    #[test]
    fn test_group_xml_generation() {
        // Test that groups generate proper XML
        let ixml = r#"choice: ("a" | "b")."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("choice").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        let input_str = "a";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));

        match result {
            Ok(trees) => {
                let forest = build_xml_forest(&ast);
                let xml_result = forest.eval(&trees);

                match xml_result {
                    Ok(tree) => {
                        let xml_string = tree.to_xml();
                        println!("Group XML: {}", xml_string);
                        assert!(xml_string.contains("<choice>"));
                        assert!(xml_string.contains("a"));
                    }
                    Err(e) => panic!("Failed to build XML tree: {}", e),
                }
            }
            Err(e) => panic!("Parse failed: {:?}", e),
        }
    }

    #[test]
    fn test_nested_groups() {
        // Test nested groups like (("a" | "b") | "c")
        let ixml = r#"choice: (("a" | "b") | "c")."#;
        let ast = parse_ixml_grammar(ixml).expect("Failed to parse iXML grammar");

        let builder = ast_to_earlgrey(&ast).expect("Failed to convert to Earlgrey");
        let grammar = builder.into_grammar("choice").expect("Failed to build grammar");

        let parser = EarleyParser::new(grammar);

        // Test all three options
        for ch in ['a', 'b', 'c'] {
            let input_str = ch.to_string();
            let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
            let result = parser.parse(tokens.iter().map(|s| s.as_str()));
            assert!(result.is_ok(), "Failed to parse '{}'", ch);
        }

        // Test invalid character
        let input_str = "d";
        let tokens: Vec<String> = input_str.chars().map(|c| c.to_string()).collect();
        let result = parser.parse(tokens.iter().map(|s| s.as_str()));
        assert!(result.is_err(), "Should not parse 'd'");
    }
}

