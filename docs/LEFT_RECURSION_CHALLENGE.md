# Left-Recursion Challenge in Native iXML Parser

## Problem Statement

The native interpreter currently fails on **left-recursive grammars** - grammars where a rule can match itself at the same position through its first alternative. This affects ~15-20 tests (23% of test suite).

### Example: Expression Grammar

```ixml
expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
```

**Input**: `pi+(10×b)`

**Problem**: 
- When parsing `expr` at position 0, we try alternatives: `term`, `sum`, `diff`
- `term` matches "pi" successfully (2 chars)
- `sum` is `expr, -"+", term` - starts by parsing `expr` at position 0
- This triggers left-recursion detection → `sum` fails
- `diff` also starts with `expr` → also fails
- Result: Only "pi" consumed, `+(10×b)` remains

## Current Implementation

### Position-Based Detection

We track `(rule_name, position)` pairs to detect left-recursion:

```rust
pub struct ParseContext {
    pub left_recursion: HashSet<(String, usize)>,
}

pub fn enter_rule(&mut self, rule_name: &str, position: usize) -> bool {
    let key = (rule_name.to_string(), position);
    if self.left_recursion.contains(&key) {
        return false; // Left recursion detected
    }
    self.left_recursion.insert(key);
    true
}
```

**Benefits**:
- Prevents infinite loops
- Allows same rule at different positions
- Simple and fast

**Limitations**:
- Blocks **valid** left-recursive parses
- Left-recursion is common in expression grammars
- iXML spec allows these grammars

### Longest-Match Alternative Selection

We try all alternatives and pick the longest match:

```rust
fn parse_alternatives(&self, stream: &mut InputStream, alts: &Alternatives, ctx: &mut ParseContext) -> Result<ParseResult, ParseError> {
    let mut best_result: Option<(ParseResult, usize)> = None;
    
    for alt in &alts.alts {
        stream.set_position(start_pos);
        match self.parse_sequence(stream, alt, ctx) {
            Ok(result) => {
                let end_pos = stream.position();
                if best_result.is_none() || end_pos > best_result.unwrap().1 {
                    best_result = Some((result, end_pos));
                }
            }
            Err(_) => continue,
        }
    }
    
    match best_result {
        Some((result, end_pos)) => {
            stream.set_position(end_pos);
            Ok(result)
        }
        None => Err(ParseError::NoAlternativeMatched { ... })
    }
}
```

**Benefits**:
- Greedy matching (consume as much as possible)
- Handles ambiguous grammars correctly
- iXML-compliant semantics

**Limitations**:
- Still can't handle left-recursion
- All alternatives must parse successfully first

## Affected Tests

### Failing with "input remains":
- `expr`, `expr1`, `expr2`, `expr3`, `expr4`, `expr5`, `expr6`
- `expr0` (ambiguous)
- `xpath`
- Possibly others with expression-like structures

### Impact:
- ~15-20 tests blocked (23-31% of test suite)
- Prevents reaching 80%+ pass rate goal
- Common pattern in real-world grammars

## Solution Options

### Option 1: Memoization with Seed Parsing (Packrat-style)

**How it works**:
1. Cache parse results by `(rule, position)`
2. When left-recursion detected, return "seed" (empty match)
3. Repeatedly re-parse with seed as base, growing result
4. Stop when result stops growing

**Example**:
```rust
struct MemoEntry {
    result: Option<ParseResult>,
    in_progress: bool,
}

type MemoTable = HashMap<(String, usize), MemoEntry>;

fn parse_rule_with_memo(&self, rule: &str, pos: usize, memo: &mut MemoTable) -> ParseResult {
    // Check memo
    if let Some(entry) = memo.get(&(rule, pos)) {
        if entry.in_progress {
            // Left-recursion! Return seed
            return ParseResult::empty();
        }
        return entry.result.clone();
    }
    
    // Mark in progress
    memo.insert((rule, pos), MemoEntry { result: None, in_progress: true });
    
    // Parse with seed
    let mut result = self.parse_alternatives(...);
    
    // Grow seed
    loop {
        let new_result = self.parse_alternatives_with_seed(result, ...);
        if new_result.consumed <= result.consumed {
            break; // Stop growing
        }
        result = new_result;
    }
    
    // Cache and return
    memo.insert((rule, pos), MemoEntry { result: Some(result.clone()), in_progress: false });
    result
}
```

**Pros**:
- Handles all left-recursive grammars
- Maintains longest-match semantics
- Well-studied technique (Warth et al. 2008)
- Performance benefit from memoization

**Cons**:
- Significant complexity (~200-300 LOC)
- Memory overhead for memo table
- Requires careful lifetime management
- Seed growing loop can be slow

**Estimated Effort**: 4-6 hours
**Test Improvement**: +15-20 tests (23-31%)

### Option 2: Precedence Climbing for Expressions

**How it works**:
1. Recognize expression grammar patterns
2. Use operator precedence parsing
3. Transform left-recursive rules at parse time

**Example**:
```rust
fn parse_expression(&self, min_prec: u32) -> ParseResult {
    let mut left = self.parse_primary(); // id, number, etc.
    
    while let Some(op) = self.peek_operator() {
        if op.precedence < min_prec {
            break;
        }
        self.consume_operator();
        let right = self.parse_expression(op.precedence + 1);
        left = combine(left, op, right);
    }
    
    left
}
```

**Pros**:
- Fast and efficient
- Natural for expression grammars
- Well-understood technique
- No memoization overhead

**Cons**:
- Only works for expression-like grammars
- Requires grammar analysis to detect patterns
- Not general-purpose
- May not handle all iXML patterns

**Estimated Effort**: 3-4 hours
**Test Improvement**: +7-10 tests (expr family only)

### Option 3: Grammar Transformation (Preprocessing)

**How it works**:
1. Analyze grammar for left-recursion
2. Transform to right-recursive form
3. Parse with transformed grammar
4. Transform result back

**Example**:
```
Original: expr: term | expr "+" term
Transform: expr: term (("+" term)*)
```

**Pros**:
- No runtime overhead
- Clean separation of concerns
- Proven technique (taught in compilers courses)

**Cons**:
- Complex grammar analysis
- May not preserve exact semantics
- Hard to reverse-transform results
- Requires AST manipulation

**Estimated Effort**: 6-8 hours
**Test Improvement**: +15-20 tests (but risky)

### Option 4: Hybrid Approach

**How it works**:
1. Use precedence climbing for common expression patterns
2. Fall back to memoization for complex cases
3. Detect grammar type automatically

**Pros**:
- Best performance for common cases
- Handles all cases eventually
- Pragmatic engineering

**Cons**:
- Most complex to implement
- Two systems to maintain
- Pattern detection can be tricky

**Estimated Effort**: 8-10 hours
**Test Improvement**: +15-20 tests with good performance

## Recommendation

**Start with Option 1: Memoization with Seed Parsing**

### Rationale:
1. **General-purpose**: Handles all left-recursive grammars
2. **Well-documented**: Based on published research
3. **Performance boost**: Memoization helps overall
4. **Achievable**: Can implement in one session
5. **Testing friendly**: Easy to validate correctness

### Implementation Plan:

1. **Add MemoTable to ParseContext** (~30 LOC)
   - `HashMap<(String, usize), MemoEntry>`
   - Thread through all parse functions

2. **Implement seed parsing in parse_rule** (~80 LOC)
   - Check memo before parsing
   - Mark in-progress during parse
   - Return seed if left-recursion detected
   - Grow seed until fixed point

3. **Update alternative selection** (~50 LOC)
   - Try alternatives with current seed
   - Pick longest result
   - Cache final result

4. **Add tests** (~40 LOC)
   - Test expr grammar
   - Test nested recursion
   - Test performance

**Total estimated code**: ~200 LOC
**Time**: 4-6 hours
**Risk**: Medium (well-studied technique)
**Benefit**: +15-20 tests → 42-47 passing (65-72%)

## References

- Warth, A., Douglass, J. R., & Millstein, T. (2008). "Packrat parsers can support left recursion."
- Dubroy, P. & Warth, A. (2018). "Incremental packrat parsing."
- Ford, B. (2002). "Packrat parsing: Simple, powerful, lazy, linear time."

## Next Steps

1. Read Warth 2008 paper (section 4 on left-recursion)
2. Implement MemoTable structure
3. Add seed parsing to parse_rule
4. Test with expr grammar
5. Run full test suite
6. Document results

**Target**: 42-47 tests passing (65-72%) after implementation
