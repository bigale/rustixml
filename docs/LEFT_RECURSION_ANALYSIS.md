# Left-Recursion Analysis

**Date**: 2025-11-23
**Status**: Root cause identified, solution designed

## Problem Statement

4 tests in the "correct" category are failing due to incomplete left-recursion support:

1. `correct/expr` - Expression grammar with operator precedence
2. `correct/unicode-classes` - Character class parsing
3. `correct/vcard` - vCard format parsing
4. `correct/xpath` - XPath expression parsing

**Current conformance**: 75.4% (49/65 tests)
**Target conformance**: 81.5% (53/65 tests) by fixing these 4 tests

## Root Cause

### What We Have

**Packrat memoization** (commit 39c76b1, Nov 21, 2025):
- Caches `(rule, position) → Result` mappings
- 24% performance improvement
- Active in `/home/bigale/repos/rustixml/src/native_parser.rs:103,128`

### What We're Missing

**Seed-growing algorithm** for left-recursion:
- Start with failure seed
- Iteratively grow the parse
- Continue until fixed point (no growth)

## Example: correct/expr Test

### Grammar
```ixml
expression: expr.
-expr: term; sum; diff.
sum: expr, -"+", term.
diff: expr, "-", term.
-term: factor; prod; div.
prod: term, -"×", factor.
div: term, "÷", factor.
-factor: id; number; bracketed.
bracketed: -"(", expr, -")".
id: @name.
name: letter+.
number: @value.
value: digit+.
-letter: ["a"-"z"].
-digit: ["0"-"9"].
```

### Input
```
pi+(10×b)
```

### Expected Output
```xml
<expression>
  <sum>
    <id name='pi'/>
    <bracketed>
      <prod>
        <number value='10'/>
        <id name='b'/>
      </prod>
    </bracketed>
  </sum>
</expression>
```

### Actual Behavior
```
Parse error: Parse succeeded but input remains: "+(10×b)"
```

**What happened**:
1. `expression` → `expr`
2. `expr` tries first alternative: `term`
3. `term` → `factor` → `id` matches `pi` ✓
4. Returns successfully
5. Input `+(10×b)` remains unparsed ✗

**Why it failed**:
- PEG ordered choice: First successful alternative wins
- `expr: term` succeeds, so `sum` and `diff` never tried
- To parse as `sum`, need: `expr, "+", term`
- But `expr` → `term` succeeds immediately (left-recursion)
- Without seed-growing, can't build up longer parse

## Solution: Seed-Growing Algorithm

### Algorithm (Warth et al., 2008)

```rust
fn parse_rule_with_seed_growing(
    rule_name: &str,
    position: usize,
    ctx: &mut ParseContext
) -> Result<ParseResult> {
    let memo_key = (rule_name.to_string(), position);

    // Check if already in recursion set (detecting left-recursion)
    if ctx.recursion_stack.contains(&memo_key) {
        // Left-recursion detected! Start with failure seed
        return Err(ParseError::LeftRecursion);
    }

    // Check cache
    if let Some(cached) = ctx.memo_cache.get(&memo_key) {
        return cached.clone();
    }

    // Mark as in-progress (detect recursion)
    ctx.recursion_stack.insert(memo_key.clone());

    // Seed with failure
    let mut seed = Err(ParseError::LeftRecursion);
    ctx.memo_cache.insert(memo_key.clone(), seed.clone());

    // Try to grow the seed
    loop {
        // Remove from recursion stack to allow re-entry
        ctx.recursion_stack.remove(&memo_key);

        // Try to parse (will use cached seed for recursive calls)
        let result = parse_rule_direct(rule_name, position, ctx);

        // Re-add to recursion stack
        ctx.recursion_stack.insert(memo_key.clone());

        match (&seed, &result) {
            // Grew! Update seed and try again
            (Err(_), Ok(new_result)) |
            (Ok(old_result), Ok(new_result))
                if new_result.end_pos > old_result.end_pos => {
                seed = result.clone();
                ctx.memo_cache.insert(memo_key.clone(), seed.clone());
            },
            // No growth - we've reached fixed point
            _ => break,
        }
    }

    // Cleanup
    ctx.recursion_stack.remove(&memo_key);

    seed
}
```

### How It Works on `pi+(10×b)`

**Iteration 1**:
- Try `expr` at position 0
- Recursive call to `expr` returns failure seed
- `term` alternative succeeds: matches `pi` (positions 0-2)
- `sum` tries: `expr` (failure), skip
- Result: `pi` (0-2)
- **Growth**: Yes! Store in cache

**Iteration 2**:
- Try `expr` at position 0 again
- Recursive call returns cached `pi` (0-2)
- `term` still just `pi`
- `sum` tries: `expr` (cached: `pi` at 0-2), `+` (at 2), `term` (parses `(10×b)` at 2-8)
- Result: `sum` containing `pi+(10×b)` (0-8)
- **Growth**: Yes! Store in cache

**Iteration 3**:
- Try `expr` at position 0 again
- Recursive call returns cached `sum` (0-8)
- `term` still just `pi`
- `sum` tries: `expr` (cached: full expression), `+` fails (no more `+`)
- Result: Same as iteration 2 (0-8)
- **No growth** - Fixed point reached!

**Final result**: `sum` node containing `pi+(10×b)` ✓

## Implementation Plan

### Phase 1: Add Recursion Tracking (1-2 days)

Modify `ParseContext` in `/home/bigale/repos/rustixml/src/parse_context.rs`:

```rust
pub struct ParseContext {
    pub input: String,
    pub position: usize,
    pub memo_cache: HashMap<(String, usize), Result<ParseResult, ParseError>>,
    pub recursion_stack: HashSet<(String, usize)>,  // NEW
}
```

### Phase 2: Implement Seed-Growing (2-3 days)

Modify `parse_rule()` in `/home/bigale/repos/rustixml/src/native_parser.rs`:

1. Detect left-recursion using `recursion_stack`
2. Seed with failure
3. Grow iteratively
4. Cache at each iteration
5. Stop at fixed point

### Phase 3: Testing (1-2 days)

1. Test on `correct/expr` - should fix operator precedence
2. Test on simple left-recursive grammar
3. Test on mutually recursive grammar
4. Verify no regression on existing 49 passing tests
5. Measure performance impact

### Total Effort: 4-7 days

## Expected Impact

**Fixes**:
- `correct/expr` - Left-recursive expression parsing ✓
- Potentially `correct/xpath` - Complex recursive grammar ✓

**May not fix**:
- `correct/unicode-classes` - May be character encoding issue
- `correct/vcard` - May be line ending handling issue

**Best case**: +2-4 tests = 78-82% conformance (51-53/65 tests)

## Performance Considerations

**Overhead**:
- Extra `recursion_stack` lookups: O(log n) per rule call
- Iteration until fixed point: Typically 2-3 iterations max
- Additional memoization storage: Minimal

**Optimization**:
- Only apply seed-growing for rules flagged as left-recursive by static analysis
- Use grammar analysis to identify which rules need special handling
- Cache the "is left-recursive" determination

**Estimated performance impact**: 5-10% slowdown on left-recursive grammars, 0% on non-recursive grammars

## References

- **Warth et al., 2008**: "Packrat Parsers Can Support Left Recursion"
  - Original seed-growing algorithm
  - Theoretical foundation

- **GAP_RESEARCH.md**: Dimension 3, variation #3
  - Identified seed parsing as preferred approach
  - Effort estimate: 2-4 weeks
  - Our estimate: 4-7 days (already have memoization infrastructure)

## Decision

**Implement seed-growing for left-recursion** as the next priority:

1. Highest impact: Fixes 2-4 failing tests
2. Builds on existing memoization  (commit 39c76b1)
3. Well-understood algorithm
4. Minimal performance cost
5. Enables broader grammar support

This is a better investment than exhaustive parsing for ambiguity (11 tests, 4-6 days, complex edge cases).

## Files to Modify

1. `/home/bigale/repos/rustixml/src/parse_context.rs` - Add `recursion_stack`
2. `/home/bigale/repos/rustixml/src/native_parser.rs` - Implement seed-growing in `parse_rule()`
3. `/home/bigale/repos/rustixml/src/grammar_analysis.rs` - Use existing left-recursion detection to flag rules
4. `/home/bigale/repos/rustixml/tests/` - Add unit tests for left-recursion

## Next Steps

1. Implement recursion tracking
2. Implement basic seed-growing
3. Test on `correct/expr`
4. Refine and optimize
5. Run full conformance suite
6. Document results
