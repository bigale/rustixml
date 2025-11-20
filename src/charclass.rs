//! Character class handling for iXML parser
//!
//! This module provides functionality for parsing and matching iXML character classes.

use std::collections::HashMap;
use unicode_general_category::{get_general_category, GeneralCategory};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RangeSet {
    /// Sorted, non-overlapping ranges stored as (start, end) inclusive
    ranges: Vec<(char, char)>,
}

impl Default for RangeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl RangeSet {
    /// Create an empty RangeSet
    pub fn new() -> Self {
        RangeSet { ranges: Vec::new() }
    }

    /// Create a RangeSet from a single character
    pub fn from_char(ch: char) -> Self {
        RangeSet {
            ranges: vec![(ch, ch)],
        }
    }

    /// Create a RangeSet from a range
    pub fn from_range(start: char, end: char) -> Self {
        if start <= end {
            RangeSet {
                ranges: vec![(start, end)],
            }
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
                        new_ranges
                            .push((start, char::from_u32(sub_start as u32 - 1).unwrap_or(start)));
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

    /// Get the number of ranges in this set
    pub fn num_ranges(&self) -> usize {
        self.ranges.len()
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

/// Split character class content by separator characters while respecting quoted strings
/// In character classes, `;`, `,`, and `|` are separators, but not inside quotes
fn split_charclass_content(content: &str) -> Vec<String> {
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';

    for ch in content.chars() {
        if in_quote {
            current.push(ch);
            if ch == quote_char {
                in_quote = false;
            }
        } else if ch == '"' || ch == '\'' {
            in_quote = true;
            quote_char = ch;
            current.push(ch);
        } else if ch == ';' || ch == ',' || ch == '|' {
            // Separator - save current element if non-empty
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                elements.push(trimmed);
            }
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    // Don't forget the last element
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        elements.push(trimmed);
    }

    elements
}

/// Convert a Unicode General Category name to a RangeSet
/// Supports both major categories (L, M, N, P, S, Z, C) and minor categories (Lu, Ll, etc.)
/// Convert a Unicode category name to a RangeSet.
/// This function is cached internally to avoid recomputing expensive ranges.
pub fn unicode_category_to_rangeset(category_name: &str) -> Option<RangeSet> {
    use std::sync::{Mutex, OnceLock};

    // Cache for Unicode category rangesets
    static UNICODE_CACHE: OnceLock<Mutex<HashMap<String, RangeSet>>> = OnceLock::new();

    // Get or initialize the cache
    let cache = UNICODE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Check if we have it cached
    {
        let cache_lock = cache.lock().unwrap();
        if let Some(rangeset) = cache_lock.get(category_name) {
            return Some(rangeset.clone());
        }
    }

    // Not cached, compute it
    let mut result = RangeSet::new();

    // Check if this is a valid Unicode category name
    let is_major = matches!(category_name, "L" | "M" | "N" | "P" | "S" | "Z" | "C");
    let is_minor = matches!(
        category_name,
        "Lu" | "Ll"
            | "Lt"
            | "Lm"
            | "Lo"
            | "LC"
            | "Mn"
            | "Mc"
            | "Me"
            | "Nd"
            | "Nl"
            | "No"
            | "Pc"
            | "Pd"
            | "Ps"
            | "Pe"
            | "Pi"
            | "Pf"
            | "Po"
            | "Sm"
            | "Sc"
            | "Sk"
            | "So"
            | "Zs"
            | "Zl"
            | "Zp"
            | "Cc"
            | "Cf"
            | "Cs"
            | "Co"
            | "Cn"
    );

    if !is_major && !is_minor {
        return None;
    }

    // Helper to check if a GeneralCategory matches a category name
    let matches_category = |cat: GeneralCategory, name: &str| -> bool {
        match name {
            // Major categories
            "L" => matches!(
                cat,
                GeneralCategory::UppercaseLetter
                    | GeneralCategory::LowercaseLetter
                    | GeneralCategory::TitlecaseLetter
                    | GeneralCategory::ModifierLetter
                    | GeneralCategory::OtherLetter
            ),
            "LC" => matches!(
                cat,
                GeneralCategory::UppercaseLetter
                    | GeneralCategory::LowercaseLetter
                    | GeneralCategory::TitlecaseLetter
            ),
            "M" => matches!(
                cat,
                GeneralCategory::NonspacingMark
                    | GeneralCategory::SpacingMark
                    | GeneralCategory::EnclosingMark
            ),
            "N" => matches!(
                cat,
                GeneralCategory::DecimalNumber
                    | GeneralCategory::LetterNumber
                    | GeneralCategory::OtherNumber
            ),
            "P" => matches!(
                cat,
                GeneralCategory::ConnectorPunctuation
                    | GeneralCategory::DashPunctuation
                    | GeneralCategory::OpenPunctuation
                    | GeneralCategory::ClosePunctuation
                    | GeneralCategory::InitialPunctuation
                    | GeneralCategory::FinalPunctuation
                    | GeneralCategory::OtherPunctuation
            ),
            "S" => matches!(
                cat,
                GeneralCategory::MathSymbol
                    | GeneralCategory::CurrencySymbol
                    | GeneralCategory::ModifierSymbol
                    | GeneralCategory::OtherSymbol
            ),
            "Z" => matches!(
                cat,
                GeneralCategory::SpaceSeparator
                    | GeneralCategory::LineSeparator
                    | GeneralCategory::ParagraphSeparator
            ),
            "C" => matches!(
                cat,
                GeneralCategory::Control
                    | GeneralCategory::Format
                    | GeneralCategory::Surrogate
                    | GeneralCategory::PrivateUse
                    | GeneralCategory::Unassigned
            ),
            // Minor categories
            "Lu" => cat == GeneralCategory::UppercaseLetter,
            "Ll" => cat == GeneralCategory::LowercaseLetter,
            "Lt" => cat == GeneralCategory::TitlecaseLetter,
            "Lm" => cat == GeneralCategory::ModifierLetter,
            "Lo" => cat == GeneralCategory::OtherLetter,
            "Mn" => cat == GeneralCategory::NonspacingMark,
            "Mc" => cat == GeneralCategory::SpacingMark,
            "Me" => cat == GeneralCategory::EnclosingMark,
            "Nd" => cat == GeneralCategory::DecimalNumber,
            "Nl" => cat == GeneralCategory::LetterNumber,
            "No" => cat == GeneralCategory::OtherNumber,
            "Pc" => cat == GeneralCategory::ConnectorPunctuation,
            "Pd" => cat == GeneralCategory::DashPunctuation,
            "Ps" => cat == GeneralCategory::OpenPunctuation,
            "Pe" => cat == GeneralCategory::ClosePunctuation,
            "Pi" => cat == GeneralCategory::InitialPunctuation,
            "Pf" => cat == GeneralCategory::FinalPunctuation,
            "Po" => cat == GeneralCategory::OtherPunctuation,
            "Sm" => cat == GeneralCategory::MathSymbol,
            "Sc" => cat == GeneralCategory::CurrencySymbol,
            "Sk" => cat == GeneralCategory::ModifierSymbol,
            "So" => cat == GeneralCategory::OtherSymbol,
            "Zs" => cat == GeneralCategory::SpaceSeparator,
            "Zl" => cat == GeneralCategory::LineSeparator,
            "Zp" => cat == GeneralCategory::ParagraphSeparator,
            "Cc" => cat == GeneralCategory::Control,
            "Cf" => cat == GeneralCategory::Format,
            "Cs" => cat == GeneralCategory::Surrogate,
            "Co" => cat == GeneralCategory::PrivateUse,
            "Cn" => cat == GeneralCategory::Unassigned,
            _ => false,
        }
    };

    // Iterate through all valid Unicode codepoints and collect matching characters
    // Unicode goes up to 0x10FFFF (1,114,111 code points)
    let mut range_start: Option<char> = None;
    let mut prev_char: Option<char> = None;

    for codepoint in 0u32..=0x10FFFF {
        if let Some(ch) = char::from_u32(codepoint) {
            let cat = get_general_category(ch);
            if matches_category(cat, category_name) {
                if range_start.is_none() {
                    range_start = Some(ch);
                }
                prev_char = Some(ch);
            } else {
                // End of range
                if let (Some(start), Some(end)) = (range_start, prev_char) {
                    result.add_range(start, end);
                }
                range_start = None;
                prev_char = None;
            }
        }
    }

    // Don't forget the last range
    if let (Some(start), Some(end)) = (range_start, prev_char) {
        result.add_range(start, end);
    }

    // Cache the result before returning
    {
        let mut cache_lock = cache.lock().unwrap();
        cache_lock.insert(category_name.to_string(), result.clone());
    }

    Some(result)
}

/// Parse a character class content string into a RangeSet
/// This handles the same formats as parse_char_class but returns a RangeSet
pub fn charclass_to_rangeset(content: &str) -> RangeSet {
    let mut result = RangeSet::new();

    // Split while respecting quoted strings
    let elements = split_charclass_content(content);

    for element in elements {
        let element = element.trim();
        if element.is_empty() {
            continue;
        }

        // Check for hex character range: #30-#39 or #1-"÷"
        if element.starts_with('#') && element.contains('-') {
            if let Some(dash_pos) = element[1..].find('-') {
                let actual_dash_pos = dash_pos + 1;
                let start_part = &element[..actual_dash_pos];
                let end_part = &element[actual_dash_pos + 1..];

                if end_part.starts_with('#') {
                    // Hex-to-hex range: #30-#39
                    if let (Some(start), Some(end)) =
                        (parse_hex_char(start_part), parse_hex_char(end_part))
                    {
                        result.add_range(start, end);
                        continue;
                    }
                } else if end_part.starts_with('"') || end_part.starts_with('\'') {
                    // Hex-to-literal range: #1-"÷"
                    let quote = if end_part.starts_with('"') { '"' } else { '\'' };
                    if let Some(close_pos) = end_part[1..].find(quote) {
                        let end_str = &end_part[1..close_pos + 1];
                        let end_char = end_str.chars().next();
                        if let (Some(start), Some(end)) = (parse_hex_char(start_part), end_char) {
                            result.add_range(start, end);
                            continue;
                        }
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
                        let end_quote = if after_dash.starts_with('\'') {
                            '\''
                        } else {
                            '"'
                        };
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
            // Only trim the quote character that was actually used
            let inner = if element.starts_with('\'') {
                element.trim_matches('\'')
            } else {
                element.trim_matches('"')
            };
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
        else if (element.starts_with('\'') && element.ends_with('\''))
            || (element.starts_with('"') && element.ends_with('"'))
        {
            // Only trim the quote character that was actually used
            let inner = if element.starts_with('\'') {
                element.trim_matches('\'')
            } else {
                element.trim_matches('"')
            };
            for ch in inner.chars() {
                result.add_char(ch);
            }
        }
        // Unicode category - try to match category names like L, Ll, Lu, etc.
        else if let Some(category_rangeset) = unicode_category_to_rangeset(element) {
            result = result.union(&category_rangeset);
        }
    }

    result
}

/// Parse a hexadecimal character code like #30 or #1F600
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
