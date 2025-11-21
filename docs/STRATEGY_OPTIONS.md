# Strategic Options for Improving rustixml Conformance

This document analyzes different approaches to achieve higher iXML conformance, comparing markup-blitz strategies with alternatives suitable for Rust.

## Current Architecture: Native Recursive Descent Parser

**What we have**:
- Direct interpretation of iXML grammar AST
- Recursive descent with backtracking
- No intermediate compilation or table generation
- O(n) for simple grammars, O(n²-n³) for complex/ambiguous ones

**Pros**:
- Simple to implement and debug
- Direct mapping to iXML semantics
- No preprocessing overhead
- Easy to understand errors

**Cons**:
- Performance degrades with ambiguity
- No natural ambiguity detection
- Repeated work on backtracking
- Hard to optimize automatically

## Option 1: Adopt markup-blitz's LALR+GLR Approach

### What It Means

**LALR (Look-Ahead LR) Parsing**:
- **Precomputed tables**: Before parsing any input, the grammar is analyzed and converted into state transition tables
- **State machine**: Each state represents a parsing position, tables tell you what to do next
- **O(1) decision time**: Looking up the next action is constant time (just index into a table)
- **Deterministic where possible**: No backtracking needed for unambiguous grammars

**GLR (Generalized LR) Parsing**:
- Extension of LR parsing that handles ambiguity
- When multiple actions are possible, creates "forks" - parallel parse stacks
- Explores all possibilities simultaneously
- Merges results that reach the same state
- Naturally detects ambiguity (multiple successful parses)

**Precomputed Tables Example**:
```rust
// Instead of this (what we do now):
fn parse_rule(&self, rule: &Rule) -> Result<Node> {
    // Walk AST, make decisions at runtime
    for alt in &rule.alternatives {
        if let Ok(result) = self.parse_alternative(alt) {
            return Ok(result);
        }
    }
    Err("No match")
}

// LALR does this once before parsing:
struct ParseTables {
    // State 0: On 'a', shift to state 5
    // State 0: On Rule<X>, goto state 12
    action_table: Vec<Vec<Action>>,  // [state][token] -> Action
    goto_table: Vec<Vec<usize>>,     // [state][nonterminal] -> next_state
}

// Then parsing becomes:
fn parse_lalr(&self, tables: &ParseTables, input: &str) -> Result<Node> {
    let mut state_stack = vec![0];
    for token in tokenize(input) {
        let action = tables.action_table[state_stack.last()][token];
        match action {
            Action::Shift(next_state) => state_stack.push(next_state),
            Action::Reduce(rule) => { /* apply rule */ },
            Action::Accept => return Ok(result),
            Action::Error => return Err("Parse error"),
        }
    }
}
```

### Pros
- ✅ **Best performance**: O(n) for unambiguous grammars, O(n³) worst case for GLR
- ✅ **Natural ambiguity detection**: Multiple parse stacks = ambiguous
- ✅ **Proven approach**: markup-blitz passes all 5168 tests
- ✅ **Extensive theory**: Decades of research on LALR/GLR parsing

### Cons
- ❌ **Major rewrite**: Complete replacement of parsing engine
- ❌ **Complex implementation**: LALR table generation is non-trivial
- ❌ **Abstraction mismatch**: LR parsers expect tokens, iXML works on characters
- ❌ **iXML semantic gaps**: Insertions, hidden elements, attribute marks need special handling
- ❌ **Rust LR libraries limited**: No mature LALR+GLR library like Java has

### Effort Estimate
- **6-12 months** full-time development
- Need to implement or adapt:
  1. LALR table generator (2-3 months)
  2. GLR parsing engine (2-3 months)
  3. iXML semantic translation (2-3 months)
  4. Integration and testing (2-3 months)

### Existing Rust Libraries
- **`lalrpop`**: LALR parser generator, but macro-based (compile-time grammars)
- **`santiago`**: GLR parser, but incomplete documentation
- **`peg`**: PEG parser generator, different paradigm
- **None directly usable**: Would need significant custom work

### Risk Assessment
- **High risk**: Complex implementation, unclear if LR paradigm maps cleanly to iXML
- **High reward**: Could achieve 95%+ conformance

---

## Option 2: Enhance Native Parser with Targeted Optimizations

### Strategy: Pre-processing + Heuristics + Post-processing

Keep recursive descent architecture but add optimizations at different stages.

### 2.1 Pre-processing Optimizations

**Character Class Partitioning** (Already partially implemented!)

```rust
// BEFORE parsing, transform grammar AST:
// Grammar: [a-z], [a-m], [m-z]
// Partitions: [a-l], [m], [n-z]
// 
// Original rule: letter: [a-z].
// Transformed:   letter: ([a-l] | [m] | [n-z]).

fn preprocess_grammar(ast: &mut GrammarAst) {
    let all_charsets = collect_character_classes(ast);
    let partitions = compute_disjoint_partitions(all_charsets);
    replace_charsets_with_partitions(ast, partitions);
}
```

**Status**: Implemented in `src/charclass.rs` but disabled due to bugs
**Effort**: 1-2 weeks to fix and enable
**Impact**: High for grammars with overlapping character classes (15-20% speedup)

**Left-Recursion Detection and Transformation**

```rust
// Detect patterns like: expr: expr, "+", term | term.
// Transform to right-recursion with careful result reconstruction:
//   expr: term, expr_tail.
//   expr_tail: "+", term, expr_tail | .

fn detect_left_recursion(rule: &Rule) -> bool {
    for alt in &rule.alternatives {
        if let Some(first) = alt.sequence.first() {
            if first.is_nonterminal(&rule.name) {
                return true;
            }
        }
    }
    false
}

fn transform_left_to_right_recursion(rule: &mut Rule) {
    // Create helper rule with right recursion
    // Adjust semantic actions to preserve tree structure
}
```

**Status**: Not implemented
**Effort**: 2-3 weeks
**Impact**: Medium - fixes 2-3 failing tests (`expr`, `xpath`)

**Nonterminal Inlining**

```rust
// Inline simple wrapper rules to reduce call depth
// 
// BEFORE: digit: ["0"-"9"]. number: digit+.
// AFTER:  number: ["0"-"9"]+.  (digit inlined)

fn inline_simple_nonterminals(ast: &mut GrammarAst) {
    let simple_rules = find_inlinable_rules(ast);
    for rule in simple_rules {
        replace_references_with_definition(ast, rule);
    }
}

fn is_inlinable(rule: &Rule) -> bool {
    rule.alternatives.len() == 1 &&
    rule.alternatives[0].sequence.len() == 1 &&
    matches!(rule.alternatives[0].sequence[0], Factor::CharClass(_))
}
```

**Status**: Not implemented
**Effort**: 1-2 weeks
**Impact**: Low-medium - reduces call stack, 5-10% speedup

### 2.2 Runtime Heuristics

**Memoization (Packrat Parsing)**

```rust
// Cache parsing results to avoid re-parsing same position
struct ParseCache {
    // (rule_id, input_position) -> ParseResult
    cache: HashMap<(usize, usize), ParseResult>,
}

impl NativeParser {
    fn parse_rule_memoized(&mut self, rule: &Rule, pos: usize) -> ParseResult {
        let key = (rule.id, pos);
        if let Some(result) = self.cache.get(&key) {
            return result.clone();
        }
        let result = self.parse_rule(rule, pos);
        self.cache.insert(key, result.clone());
        result
    }
}
```

**Status**: Not implemented
**Effort**: 1 week
**Impact**: High for grammars with backtracking (30-50% speedup)
**Trade-off**: Memory usage increases

**Ambiguity Detection via Multiple Parses**

```rust
// Track alternative parse paths during parsing
struct ParseState {
    current_parse: Vec<Node>,
    alternative_parses: Vec<Vec<Node>>,
    ambiguity_detected: bool,
}

fn parse_with_ambiguity_detection(&mut self, input: &str) -> ParseResult {
    // Try each alternative at decision points
    // If multiple succeed with same input consumed, mark ambiguous
    for alt in alternatives {
        if let Ok(result) = self.parse_alternative(alt) {
            successful_parses.push(result);
        }
    }
    
    if successful_parses.len() > 1 {
        return ParseResult::Ambiguous(successful_parses);
    }
}
```

**Status**: Not implemented
**Effort**: 2-3 weeks
**Impact**: High - fixes all 11 ambiguity tests
**Trade-off**: Slower parsing (explores more paths)

**Better Error Recovery**

```rust
// When parse fails, try to identify the furthest point reached
struct ParseError {
    position: usize,
    expected: Vec<String>,
    context: String,
    alternatives_tried: usize,
}

fn enhanced_error_reporting(&self, error: &ParseError) -> String {
    format!(
        "Parse error at line {}, column {}: Expected {} but found {}\n\
         Context: ...{}...\n\
         Tried {} alternatives",
        error.line, error.column, error.expected.join(" or "),
        error.found, error.context, error.alternatives_tried
    )
}
```

**Status**: Partially implemented
**Effort**: 1 week
**Impact**: Low - better UX, doesn't fix tests

### 2.3 Post-processing Optimizations

**Result Tree Normalization**

```rust
// After parsing, normalize the XML tree
fn normalize_parse_tree(tree: &mut XmlNode) {
    // 1. Merge adjacent text nodes
    // 2. Remove empty hidden elements
    // 3. Flatten unnecessary nesting
    // 4. Sort attributes
    
    merge_adjacent_text(tree);
    remove_empty_hidden(tree);
    flatten_sequences(tree);
    sort_attributes(tree);
}
```

**Status**: Partially implemented (flatten_sequences exists)
**Effort**: 1 week
**Impact**: Low - improves output quality

### Pros of Option 2
- ✅ **Incremental improvement**: Can implement piece by piece
- ✅ **Lower risk**: Each optimization is independent
- ✅ **Preserves architecture**: Recursive descent stays intact
- ✅ **Rust-friendly**: No need for complex libraries

### Cons
- ❌ **May not reach 100%**: Fundamental limitations remain
- ❌ **Complexity accumulates**: Many small optimizations vs. one clean architecture
- ❌ **Performance ceiling**: Will never match LALR speed

### Effort Estimate
- **2-4 months** for significant improvements
- Realistic target: **90-95% conformance**

---

## Option 3: Hybrid Model

### Strategy: Use Different Parsers for Different Grammar Types

```rust
enum ParserStrategy {
    FastPath,      // Simple grammars: optimized recursive descent
    MemoizedPath,  // Complex grammars: packrat parsing
    AmbiguityPath, // Ambiguous grammars: explore all paths
}

impl NativeParser {
    fn choose_strategy(&self, grammar: &Grammar) -> ParserStrategy {
        let complexity = analyze_grammar(grammar);
        
        match complexity {
            Complexity::Simple => ParserStrategy::FastPath,
            Complexity::Recursive => ParserStrategy::MemoizedPath,
            Complexity::Ambiguous => ParserStrategy::AmbiguityPath,
        }
    }
}

fn analyze_grammar(grammar: &Grammar) -> Complexity {
    let has_left_recursion = detect_left_recursion(grammar);
    let has_ambiguity_markers = detect_ambiguous_patterns(grammar);
    let rule_depth = compute_max_depth(grammar);
    
    // Use heuristics to classify
    if rule_depth < 5 && !has_left_recursion {
        Complexity::Simple
    } else if has_ambiguity_markers {
        Complexity::Ambiguous
    } else {
        Complexity::Recursive
    }
}
```

### Pattern Detection Examples

**Simple Grammars** (use fast path):
```ixml
date: year, "-", month, "-", day.
year: digit, digit, digit, digit.
```
- No recursion
- No ambiguity
- Direct translation

**Recursive Grammars** (use memoization):
```ixml
expr: term | expr, "+", term.
term: factor | term, "*", factor.
```
- Left recursion detected
- Enable memoization cache

**Ambiguous Grammars** (use full exploration):
```ixml
a: "x" | "x", "y".
```
- Multiple parses possible
- Track all alternatives

### Pros
- ✅ **Best of both worlds**: Fast for simple cases, correct for complex ones
- ✅ **Adaptive**: Automatically chooses right approach
- ✅ **User-friendly**: Works well without configuration

### Cons
- ❌ **Complex implementation**: Need multiple parsing modes
- ❌ **Analysis overhead**: Grammar analysis takes time
- ❌ **Testing burden**: Must test all paths

### Effort Estimate
- **3-5 months**
- Realistic target: **92-97% conformance**

---

## Option 4: Profiling-Driven Optimization

### Strategy: Measure, Don't Guess

```bash
# Profile current implementation on failing tests
cargo build --release
perf record target/release/conformance_test
perf report

# Analyze hotspots
cargo flamegraph --bin conformance_test
```

**Common Hotspots in Parsers**:
1. Character classification (UTF-8 decoding)
2. Backtracking overhead
3. Memory allocation (Vec growth)
4. String copying
5. Hash map lookups

**Targeted Optimizations**:

```rust
// If profiling shows UTF-8 decoding is slow:
use bstr::ByteSlice; // Work with bytes directly

// If backtracking is expensive:
struct SavePoint {
    position: usize,
    stack_depth: usize,
}
// Lightweight save/restore instead of cloning

// If allocation is costly:
struct ParseArena {
    nodes: Vec<Node>,
    current: usize,
}
// Arena allocation for parse nodes
```

### Pros
- ✅ **Evidence-based**: Optimize what actually matters
- ✅ **High ROI**: Focus effort where it helps most
- ✅ **Measurable**: Clear before/after metrics

### Cons
- ❌ **May not fix correctness**: Only improves speed
- ❌ **Platform-specific**: Optimizations may vary by CPU
- ❌ **Diminishing returns**: Easy wins first, hard ones remain

### Effort Estimate
- **2-4 weeks** for initial profiling and fixes
- **Ongoing**: 1 day/month for continuous improvement

---

## Recommended Strategy: **Option 2 + 4 (Enhanced Native Parser + Profiling)**

### Rationale

1. **Proven Architecture**: Recursive descent is well-understood and debuggable
2. **Incremental Path**: Can ship improvements in minor versions (v0.3, v0.4)
3. **Rust Strengths**: Plays to Rust's zero-cost abstractions and memory safety
4. **Realistic Goals**: 90-95% conformance is excellent for most users
5. **Lower Risk**: Don't bet the project on a complete rewrite

### Implementation Roadmap

#### Phase 1: Quick Wins (v0.3.0 - 1 month)
1. ✅ Enable character class partitioning (fix bugs)
2. ✅ Implement basic memoization
3. ✅ Add ambiguity detection for simple cases
4. ✅ Profile and optimize hotspots

**Expected**: 87-90% conformance (43-44 tests passing)

#### Phase 2: Structural Improvements (v0.4.0 - 2 months)
1. ✅ Left-recursion transformation
2. ✅ Nonterminal inlining
3. ✅ Full ambiguity detection with multiple parse paths
4. ✅ Better error messages

**Expected**: 92-95% conformance (45-47 tests passing)

#### Phase 3: Polish (v0.5.0 - 1 month)
1. ✅ Edge case handling
2. ✅ Performance tuning
3. ✅ Memory optimization
4. ✅ Documentation

**Expected**: 95%+ conformance (47+ tests passing)

### Why NOT Option 1 (LALR+GLR)?

1. **Overkill for 95%**: Can reach high conformance without it
2. **Time vs. Value**: 6-12 months for 5% more tests
3. **Maintenance burden**: Complex code is harder to maintain
4. **Community contribution**: Easier for others to contribute to simpler code
5. **iXML evolution**: Spec may change, simpler code adapts easier

### When to Reconsider Option 1

- If aiming for **100% conformance** becomes critical
- If **performance** becomes the bottleneck (currently not an issue)
- If a mature **Rust LALR+GLR library** emerges
- If building a **production parser framework** (not just an iXML parser)

---

## Comparison Table

| Approach | Effort | Conformance Target | Risk | Maintainability | Performance |
|----------|--------|-------------------|------|-----------------|-------------|
| **Current** | 0 | 83.7% | - | ✅✅✅ | ✅✅ |
| **LALR+GLR** | 12 months | 98-100% | ⚠️⚠️⚠️ | ⚠️ | ✅✅✅ |
| **Enhanced Native** | 4 months | 92-95% | ✅✅ | ✅✅ | ✅✅ |
| **Hybrid** | 5 months | 95-97% | ⚠️⚠️ | ⚠️ | ✅✅✅ |
| **Profile-Only** | 1 month | 83.7% | ✅✅✅ | ✅✅✅ | ✅✅✅ |

---

## Conclusion

**Ship v0.2.0 now** with current 83.7% conformance and excellent documentation.

**Plan for v0.3-v0.5** using Option 2 (Enhanced Native Parser) with profiling.

**Keep Option 1** (LALR+GLR) as a potential v2.0 if needed.

This balances:
- ✅ Time to market
- ✅ User value (83.7% is great for most use cases)
- ✅ Continuous improvement path
- ✅ Code maintainability
- ✅ Community contribution potential

The perfect is the enemy of the good. Ship now, improve incrementally.
