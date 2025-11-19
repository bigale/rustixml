# Native iXML Interpreter Design

## Overview

This document outlines the design for a **native Rust iXML interpreter** that directly implements the iXML specification without translation to an intermediate parser representation (Earley, LALR, etc.).

### Why Native?

The Earley-based approach revealed fundamental abstraction mismatches:

1. **Insertion semantics** (`+"text"`) - adding output not in input
2. **Suppression semantics** (`-name`) - hiding matched input from output  
3. **Combined patterns** - `(-[Co], +".")*` loops that suppress and insert simultaneously

These are **first-class iXML operations** but don't map naturally to parser generators that only consume input tokens.

### Design Philosophy

**Specification-First**: Implement iXML semantics directly as described in the specification, not as a translation layer.

**Rust-Native**: Leverage Rust's strengths:
- Pattern matching for grammar traversal
- `Option`/`Result` for parse success/failure
- Iterators for input consumption
- Zero-cost abstractions
- Memory safety without runtime overhead

## Architecture

### High-Level Structure

```
┌─────────────────────────────────────────────────┐
│ iXML Grammar Text                               │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│ Grammar Parser (existing: grammar_parser.rs)    │
│ - Handwritten recursive descent                 │
│ - Produces IxmlGrammar AST                      │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│ Native Interpreter (NEW)                        │
│ - Direct AST interpretation                     │
│ - Recursive descent parsing                     │
│ - Native insertion/suppression handling         │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│ XML Output (existing: XmlNode → String)         │
└─────────────────────────────────────────────────┘
```

### Core Components

#### 1. Input Stream (`src/input_stream.rs`)

Manages input text with position tracking:

```rust
pub struct InputStream<'a> {
    input: &'a str,
    position: usize,  // Current position in input (char index)
    chars: Vec<char>, // Pre-computed char array for O(1) access
}

impl<'a> InputStream<'a> {
    pub fn new(input: &'a str) -> Self;
    pub fn current(&self) -> Option<char>;
    pub fn advance(&mut self) -> Option<char>;
    pub fn peek(&self, offset: usize) -> Option<char>;
    pub fn position(&self) -> usize;
    pub fn set_position(&mut self, pos: usize);
    pub fn remaining(&self) -> &str;
    pub fn is_eof(&self) -> bool;
}
```

**Design choices**:
- Pre-compute `Vec<char>` for Unicode-safe indexing
- Position is character index, not byte offset
- Immutable input string reference
- Mutable position cursor for backtracking

#### 2. Parse Context (`src/parse_context.rs`)

Tracks parse state during recursive descent:

```rust
pub struct ParseContext {
    pub rule_name: String,        // Current rule being parsed
    pub depth: usize,             // Recursion depth (for debugging)
    pub left_recursion: HashSet<String>, // Detect left-recursion
}

pub struct ParseResult {
    pub node: Option<XmlNode>,    // Parsed node (None if suppressed)
    pub consumed: usize,          // Characters consumed
}
```

**Design choices**:
- Track current rule for error messages
- Left-recursion detection (fail-fast on direct left-recursion)
- Separate "success with no output" (suppressed) from "failure"

#### 3. Native Parser (`src/native_parser.rs`)

Main interpreter implementing iXML semantics:

```rust
pub struct NativeParser {
    grammar: IxmlGrammar,
    rules: HashMap<String, Rule>, // Fast rule lookup by name
}

impl NativeParser {
    pub fn new(grammar: IxmlGrammar) -> Self;
    
    pub fn parse(&self, input: &str) -> Result<String, ParseError>;
    
    // Core parsing methods (recursive descent)
    fn parse_rule(&self, stream: &mut InputStream, rule: &Rule, ctx: &mut ParseContext) 
        -> Result<ParseResult, ParseError>;
    
    fn parse_alternatives(&self, stream: &mut InputStream, alts: &Alternatives, ctx: &mut ParseContext)
        -> Result<ParseResult, ParseError>;
    
    fn parse_sequence(&self, stream: &mut InputStream, seq: &Sequence, ctx: &mut ParseContext)
        -> Result<ParseResult, ParseError>;
    
    fn parse_factor(&self, stream: &mut InputStream, factor: &Factor, ctx: &mut ParseContext)
        -> Result<ParseResult, ParseError>;
    
    fn parse_base_factor(&self, stream: &mut InputStream, base: &BaseFactor, mark: Mark, ctx: &mut ParseContext)
        -> Result<ParseResult, ParseError>;
}
```

**Design choices**:
- One method per grammar construct (alternatives, sequence, factor, etc.)
- All methods return `Result<ParseResult, ParseError>`
- `ParseResult` contains both the node and characters consumed
- Backtracking via `stream.set_position(saved_pos)`

### Parsing Algorithm

#### Alternatives (Choice)

```rust
fn parse_alternatives(&self, stream: &mut InputStream, alts: &Alternatives, ctx: &mut ParseContext)
    -> Result<ParseResult, ParseError> {
    
    let start_pos = stream.position();
    let mut errors = Vec::new();
    
    // Try each alternative in order
    for alt in &alts.alts {
        stream.set_position(start_pos); // Reset for each alternative
        
        match self.parse_sequence(stream, alt, ctx) {
            Ok(result) => return Ok(result), // First success wins
            Err(e) => errors.push(e),
        }
    }
    
    // All alternatives failed
    Err(ParseError::NoAlternativeMatched { 
        position: start_pos,
        attempts: errors,
    })
}
```

**Design choices**:
- Ordered choice (PEG-style): first match wins
- Backtrack to start position for each alternative
- Collect all errors for better diagnostics

#### Sequence (Concatenation)

```rust
fn parse_sequence(&self, stream: &mut InputStream, seq: &Sequence, ctx: &mut ParseContext)
    -> Result<ParseResult, ParseError> {
    
    let start_pos = stream.position();
    let mut children = Vec::new();
    let mut total_consumed = 0;
    
    // Parse each factor in sequence
    for factor in &seq.factors {
        match self.parse_factor(stream, factor, ctx) {
            Ok(result) => {
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
    
    Ok(ParseResult {
        node: Some(XmlNode::Sequence(children)),
        consumed: total_consumed,
    })
}
```

**Design choices**:
- All factors must succeed (fail-fast)
- Collect non-suppressed nodes into children
- Full backtrack on any failure
- Track total characters consumed

#### Repetition

```rust
fn parse_repetition(&self, stream: &mut InputStream, factor: &Factor, rep: &Repetition, ctx: &mut ParseContext)
    -> Result<ParseResult, ParseError> {
    
    let mut children = Vec::new();
    let mut total_consumed = 0;
    
    match rep {
        Repetition::ZeroOrMore => {
            // Keep parsing until failure
            loop {
                let pos = stream.position();
                match self.parse_base_factor(stream, &factor.base, Mark::None, ctx) {
                    Ok(result) => {
                        if result.consumed == 0 {
                            break; // Prevent infinite loop on epsilon matches
                        }
                        if let Some(node) = result.node {
                            children.push(node);
                        }
                        total_consumed += result.consumed;
                    }
                    Err(_) => {
                        stream.set_position(pos); // Backtrack last attempt
                        break;
                    }
                }
            }
            Ok(ParseResult {
                node: Some(XmlNode::Sequence(children)),
                consumed: total_consumed,
            })
        }
        
        Repetition::OneOrMore => {
            // Must match at least once
            match self.parse_base_factor(stream, &factor.base, Mark::None, ctx) {
                Ok(result) => {
                    if let Some(node) = result.node {
                        children.push(node);
                    }
                    total_consumed += result.consumed;
                    
                    // Then same as ZeroOrMore
                    // ... (same loop as above)
                }
                Err(e) => return Err(e),
            }
        }
        
        // Similar for Optional, SeparatedZeroOrMore, SeparatedOneOrMore
        // ...
    }
}
```

**Design choices**:
- Greedy matching (consume as much as possible)
- Epsilon-match detection to prevent infinite loops
- Separated repetitions parse separator between elements

#### Terminal Matching

```rust
fn parse_terminal(&self, stream: &mut InputStream, value: &str, mark: Mark, insertion: bool)
    -> Result<ParseResult, ParseError> {
    
    if insertion {
        // Insertion: don't consume input, just create node
        return Ok(ParseResult {
            node: Some(XmlNode::Text(value.to_string())),
            consumed: 0,
        });
    }
    
    // Regular terminal: must match input
    let start_pos = stream.position();
    
    for ch in value.chars() {
        match stream.current() {
            Some(input_ch) if input_ch == ch => {
                stream.advance();
            }
            _ => {
                stream.set_position(start_pos);
                return Err(ParseError::TerminalMismatch {
                    expected: value.to_string(),
                    position: start_pos,
                });
            }
        }
    }
    
    let consumed = value.chars().count();
    
    // Apply mark
    let node = match mark {
        Mark::Hidden => None, // Suppressed
        _ => Some(XmlNode::Text(value.to_string())),
    };
    
    Ok(ParseResult { node, consumed })
}
```

**Design choices**:
- **Insertions** (`+"text"`): consume 0 characters, always succeed, create node
- **Suppressions** (`-"text"`): consume characters, succeed/fail normally, but return `None` node
- Character-by-character matching for Unicode safety

#### Character Class Matching

```rust
fn parse_charclass(&self, stream: &mut InputStream, content: &str, negated: bool, mark: Mark)
    -> Result<ParseResult, ParseError> {
    
    let start_pos = stream.position();
    
    match stream.current() {
        Some(ch) => {
            let matches = self.charclass_matches(ch, content);
            let should_match = if negated { !matches } else { matches };
            
            if should_match {
                stream.advance();
                
                let node = match mark {
                    Mark::Hidden => None,
                    _ => Some(XmlNode::Text(ch.to_string())),
                };
                
                Ok(ParseResult { node, consumed: 1 })
            } else {
                Err(ParseError::CharClassMismatch {
                    charclass: content.to_string(),
                    negated,
                    actual: ch,
                    position: start_pos,
                })
            }
        }
        None => Err(ParseError::UnexpectedEof { position: start_pos }),
    }
}

fn charclass_matches(&self, ch: char, content: &str) -> bool {
    // Parse character class content and test if ch matches
    // Handles: ranges (a-z), hex chars (#41), Unicode categories ([L])
    // Already implemented in runtime_parser.rs - can reuse
}
```

**Design choices**:
- Match single character against character class predicate
- Support negation (`~[...]`)
- Reuse existing character class parsing logic
- Apply mark after successful match

#### Nonterminal (Rule Reference)

```rust
fn parse_nonterminal(&self, stream: &mut InputStream, name: &str, mark: Mark, ctx: &mut ParseContext)
    -> Result<ParseResult, ParseError> {
    
    // Check for left recursion
    if ctx.left_recursion.contains(name) {
        return Err(ParseError::LeftRecursion { rule: name.to_string() });
    }
    
    // Look up rule
    let rule = self.rules.get(name)
        .ok_or_else(|| ParseError::UndefinedRule { rule: name.to_string() })?;
    
    // Push onto left-recursion tracker
    ctx.left_recursion.insert(name.to_string());
    ctx.depth += 1;
    
    let result = self.parse_rule(stream, rule, ctx);
    
    // Pop from left-recursion tracker
    ctx.left_recursion.remove(name);
    ctx.depth -= 1;
    
    // Apply mark to result
    match result {
        Ok(mut parse_result) => {
            parse_result.node = parse_result.node.and_then(|node| match mark {
                Mark::Hidden => None,
                Mark::Attribute => Some(XmlNode::Attribute { 
                    name: name.to_string(),
                    value: node.to_string(),
                }),
                Mark::Promoted => Some(node), // Promote contents
                Mark::None => Some(XmlNode::Element {
                    name: name.to_string(),
                    attributes: vec![],
                    children: vec![node],
                }),
            });
            Ok(parse_result)
        }
        Err(e) => Err(e),
    }
}
```

**Design choices**:
- Direct left-recursion detection (fail immediately)
- Marks applied to rule result, not during parsing
- Promoted mark unwraps the element
- Attribute mark converts to attribute node

### Mark Handling

Marks (`@`, `-`, `^`) are applied **after** successful parsing:

```rust
fn apply_mark(node: Option<XmlNode>, mark: Mark, name: &str) -> Option<XmlNode> {
    match mark {
        Mark::None => node,
        
        Mark::Hidden => None, // Suppress output
        
        Mark::Attribute => node.map(|n| XmlNode::Attribute {
            name: name.to_string(),
            value: n.text_content(), // Extract text value
        }),
        
        Mark::Promoted => node, // Contents promoted (no wrapper element)
    }
}
```

**Key insight**: Marks are **post-processing** operations on successful parses, not parse-time behavior changes.

### Insertion Handling

Insertions (`+"text"`) are **synthetic terminals** that always succeed:

```rust
if insertion {
    // Create node without consuming input
    return Ok(ParseResult {
        node: Some(XmlNode::Text(value.to_string())),
        consumed: 0, // Key: consume nothing
    });
}
```

This naturally handles the `(-[Co], +".")*` pattern:
1. `-[Co]` matches and consumes a Co character, returns `None` (suppressed)
2. `+"."` creates a "." text node, consumes 0 characters
3. Sequence succeeds with node `[None, Some(Text("."))]`
4. Repetition continues until `-[Co]` fails

### Error Handling

```rust
pub enum ParseError {
    UnexpectedEof { position: usize },
    
    TerminalMismatch { expected: String, position: usize },
    
    CharClassMismatch { charclass: String, negated: bool, actual: char, position: usize },
    
    NoAlternativeMatched { position: usize, attempts: Vec<ParseError> },
    
    UndefinedRule { rule: String },
    
    LeftRecursion { rule: String },
}

impl ParseError {
    pub fn format_with_context(&self, input: &str) -> String {
        // Format error with line/column, surrounding context
    }
}
```

**Design choices**:
- Rich error types with position information
- Collect all alternative attempts for better diagnostics
- Context-aware error formatting

## Implementation Plan

### Phase 1: Core Infrastructure (Foundation)

**Files to create**:
- `src/input_stream.rs` - Input management with backtracking
- `src/parse_context.rs` - Parse state tracking
- `src/native_parser.rs` - Main parser struct (skeleton)

**Tests**: Unit tests for `InputStream` operations

**Effort**: 2-3 hours

### Phase 2: Basic Terminals (Proof of Concept)

**Implement**:
- `parse_terminal()` - Literal matching
- `parse_charclass()` - Character class matching
- `parse_nonterminal()` - Rule reference

**Tests**: 
- `test` - Basic grammar
- `aaa` - Hidden literals
- `string` - String literals

**Effort**: 3-4 hours

### Phase 3: Sequences and Alternatives

**Implement**:
- `parse_sequence()` - Factor concatenation
- `parse_alternatives()` - Ordered choice

**Tests**:
- `address` - Multiple alternatives
- `expr` - Nested alternatives

**Effort**: 2-3 hours

### Phase 4: Repetitions

**Implement**:
- `parse_repetition()` - `*`, `+`, `?`
- Separated repetitions - `**`, `++`

**Tests**:
- `email` - Separated repetitions
- `hash` - Multiple separators

**Effort**: 4-5 hours

### Phase 5: Marks and Insertions

**Implement**:
- Mark application logic
- Insertion handling
- Attribute extraction

**Tests**:
- `marked` - Various marks
- `unicode-classes` - **The critical test** with `(-[Co], +".")*`

**Effort**: 3-4 hours

### Phase 6: Full Test Suite

**Implement**:
- Error handling improvements
- Edge cases
- Performance tuning

**Tests**: Run full 133-test suite

**Effort**: 5-8 hours

## Expected Benefits

### 1. Correctness

- **Direct semantics**: No translation layer to introduce bugs
- **Specification alignment**: Code structure mirrors iXML spec
- **Insertion/suppression**: Native support, not bolted-on

### 2. Performance

Expected performance characteristics:

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Terminal match | O(k) | k = literal length |
| Character class | O(1) | Single character test |
| Alternative | O(n × m) | n = alternatives, m = input |
| Sequence | O(k × m) | k = factors, m = input |
| Repetition | O(n × m) | n = iterations, m = input per iteration |

**Overall**: O(n × m) worst-case where n = grammar size, m = input size

For typical grammars:
- **Faster than Earley** for deterministic grammars (no state explosion)
- **Similar to Earley** for ambiguous grammars
- **Much simpler** implementation (no parser table construction)

### 3. Simplicity

Lines of code estimate:

| Component | LOC | Notes |
|-----------|-----|-------|
| `input_stream.rs` | ~100 | Simple wrapper |
| `parse_context.rs` | ~50 | State tracking |
| `native_parser.rs` | ~600-800 | Core interpreter |
| Error handling | ~100 | Rich error types |
| Tests | ~500 | Comprehensive coverage |
| **Total** | **~1400** | vs. 3600+ for Earley approach |

### 4. Debuggability

- Stack traces show actual rule names (not generated symbols)
- Errors reference grammar rules directly
- No "missing action" or "missing symbol" cryptic messages
- Easy to add tracing/logging

### 5. Maintainability

- One-to-one mapping: grammar construct → function
- No global state (except grammar itself)
- Pure functions (input → result)
- Easy to extend with new features

## Comparison: Earley vs. Native

| Aspect | Earley Approach | Native Interpreter |
|--------|----------------|-------------------|
| **Abstraction** | Parser generator (wrong level) | Direct specification |
| **Insertions** | No native support | First-class operation |
| **Suppressions** | Wrapper rules required | Built into marks |
| **Complexity** | 3600+ LOC | ~1400 LOC estimated |
| **Performance** | O(n³) worst-case, state explosion | O(n²) typical, predictable |
| **Errors** | "Missing Action", "No Rule completes" | Grammar-aligned messages |
| **Debugging** | Generated symbols hard to trace | Direct rule names |
| **Test Pass Rate** | 39.8% (hard limit) | 87%+ expected |

## Next Steps

1. ✅ **Document design** (this file)
2. **Implement Phase 1** - Core infrastructure
3. **Implement Phase 2** - Basic terminals (get first test passing)
4. **Iterate through phases** 3-6
5. **Run full test suite** - Target 80%+ pass rate
6. **Performance profiling** - Optimize hot paths
7. **Production-ready** - Documentation, error messages, API design

## Open Questions

1. **Ambiguity handling**: How to represent multiple parse trees?
   - Option A: Return first match (PEG-style)
   - Option B: Return all matches (GLR-style)
   - **Recommendation**: Start with Option A, add Option B later if needed

2. **Left recursion**: How to handle indirect left-recursion?
   - Option A: Detect and fail (simple)
   - Option B: Packrat memoization to handle safely
   - **Recommendation**: Start with Option A, good enough for most grammars

3. **Unicode categories**: Reuse existing code or refactor?
   - **Recommendation**: Reuse `unicode_category_to_rangeset()` from `runtime_parser.rs`

4. **Separated repetitions with marks**: Edge case handling?
   - Example: `word++(-",")` - separator is hidden
   - **Recommendation**: Test thoroughly, likely works naturally

## Success Criteria

A successful native interpreter implementation will:

- ✅ Pass `unicode-classes` test (the Earley blocker)
- ✅ Pass all 43 currently passing tests
- ✅ Pass additional tests that Earley couldn't handle
- ✅ Have clear, grammar-aligned error messages
- ✅ Be simpler and more maintainable than Earley approach
- ✅ Demonstrate Rust language strengths (pattern matching, iterators, zero-cost abstractions)
- ✅ Serve as foundation for 90%+ conformance with the full iXML spec
