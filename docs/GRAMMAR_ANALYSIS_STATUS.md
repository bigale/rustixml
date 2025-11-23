# Grammar Analysis Implementation Status

**Date**: 2025-11-22
**Status**: Implemented but disabled due to recursion depth issues

## What We Built

### 1. Grammar Analysis Module (`src/grammar_analysis.rs`)

Complete implementation of grammar structure analysis including:

- ✅ **Recursion detection** - Identifies directly and indirectly recursive rules
- ✅ **Left-recursion detection** - Identifies rules that start with themselves
- ✅ **Rule categorization** - Identifies hidden, promoted, and attribute rules
- ✅ **Complexity scoring** - Calculates complexity based on alternatives and nesting
- ✅ **Reporting** - Generates human-readable analysis reports
- ✅ **Test coverage** - 3 passing tests for core functionality

### 2. NativeParser Integration

- ✅ Added `GrammarAnalysis` field to `NativeParser`
- ✅ Integrated analysis into constructor
- ✅ Warning messages for problematic grammars

## Current Status

**Grammar analysis is DISABLED** due to stack overflow on complex grammars.

### The Problem

When analyzing certain grammars (like `ambiguous/ambig`), the left-recursion detection causes stack overflow due to deep or infinite recursion in the analysis itself.

**Error**:
```
Running ambiguous/ambig: ambig...
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```

### Root Cause

The `is_left_recursive` and `is_nullable` functions use recursion to traverse the grammar tree. For grammars with:
- Deep nesting
- Complex mutual recursion
- Many interconnected rules

The recursion depth exceeds the stack limit.

### What Works

Simple grammars work perfectly:

**Test case** - Left-recursive expression grammar:
```ixml
expr: expr, "+", term | term.
term: digit+.
digit: ["0"-"9"].
```

**Output**:
```
[rustixml] Grammar analysis:
⚠️  Left-recursive rules (may cause infinite loops):
   - expr
```

This correctly detects the left-recursion in `expr`.

## Files Created/Modified

### New Files
- ✅ `/home/bigale/repos/rustixml/src/grammar_analysis.rs` (508 lines)
  - `GrammarAnalysis` struct
  - Recursion detection
  - Left-recursion detection
  - Nullable detection
  - Complexity scoring
  - Reporting functions

### Modified Files
- ✅ `/home/bigale/repos/rustixml/src/lib.rs` - Added `grammar_analysis` module
- ✅ `/home/bigale/repos/rustixml/src/native_parser.rs` - Integration (currently disabled)

### Documentation
- ✅ `/home/bigale/repos/rustixml/docs/NORMALIZATION_STRATEGY.md` - Initial design
- ✅ `/home/bigale/repos/rustixml/docs/NORMALIZATION_LESSONS.md` - Lessons learned
- ✅ `/home/bigale/repos/rustixml/docs/GRAMMAR_ANALYSIS_STATUS.md` - This file

## Attempted Fixes

### Attempt 1: Depth Limiting (MAX_ANALYSIS_DEPTH)
**Status**: ❌ Failed

Added depth parameter to all analysis functions with limits of 100, 50, and 20. None prevented stack overflow on `ambiguous/ambig` grammar.

**Code changes**:
- Added `const MAX_ANALYSIS_DEPTH: usize = 20`
- Modified all recursive functions to check depth before recursing
- Functions updated: `is_recursive`, `is_left_recursive`, `is_nullable`, `is_sequence_nullable`, `is_factor_nullable`, `check_alternatives_for_recursion`, `check_sequence_for_recursion`, `check_factor_for_recursion`

**Result**: Stack overflow still occurred, suggesting the problem is deeper than just recursion depth.

### Attempt 2: Larger Stack (8MB Thread)
**Status**: ❌ Failed

Ran analysis in a separate thread with 8MB stack (4x default size).

**Code changes**:
```rust
std::thread::Builder::new()
    .stack_size(8 * 1024 * 1024) // 8MB stack
    .spawn(move || GrammarAnalysis::analyze(&grammar_clone))
```

**Result**: Stack overflow still occurred on `ambiguous/ambig`. This grammar is exceptionally complex.

### Attempt 3: Panic Catching
**Status**: ❌ Not viable

Tried using `catch_unwind` to catch stack overflow panics, but stack overflow triggers `abort()` which cannot be caught in Rust.

## How to Fix the Stack Overflow (Future Work)

### Option 1: Iteration Instead of Recursion (RECOMMENDED)

Replace recursive traversal with iterative traversal using an explicit stack:

```rust
fn is_left_recursive_iterative(
    rule_name: &str,
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
) -> bool {
    let mut stack = vec![(rule_name, alternatives, 0)]; // (target, alts, alt_index)
    let mut visited = HashSet::new();

    while let Some((target, alts, idx)) = stack.pop() {
        if idx >= alts.alts.len() {
            continue;
        }

        let alt = &alts.alts[idx];

        // Process alternative
        // Push next alternative
        stack.push((target, alts, idx + 1));

        // Process factors in this alternative
        // ...
    }

    false
}
```

### Option 2: Depth Limiting

Add maximum depth parameter to prevent infinite recursion:

```rust
const MAX_ANALYSIS_DEPTH: usize = 100;

fn is_left_recursive(
    rule_name: &str,
    alternatives: &Alternatives,
    rule_map: &HashMap<String, &Rule>,
    depth: usize,
) -> bool {
    if depth > MAX_ANALYSIS_DEPTH {
        eprintln!("Warning: Analysis depth limit reached for rule '{}'", rule_name);
        return false; // Assume not left-recursive if too deep
    }

    // ... rest of function with depth + 1
}
```

### Option 3: Memoization

Cache results of `is_nullable` and `is_left_recursive` to avoid recomputation:

```rust
struct AnalysisCache {
    nullable: HashMap<String, bool>,
    left_recursive: HashMap<String, bool>,
}

impl AnalysisCache {
    fn is_nullable(&mut self, rule_name: &str, rule_map: &HashMap<String, &Rule>) -> bool {
        if let Some(&result) = self.nullable.get(rule_name) {
            return result;
        }

        let result = compute_nullable(rule_name, rule_map, self);
        self.nullable.insert(rule_name.to_string(), result);
        result
    }
}
```

## Next Steps

### Immediate (v0.3.0)
1. **Fix stack overflow** using Option 1 (iteration) or Option 2 (depth limiting)
2. **Re-enable analysis** in `NativeParser::new()`
3. **Test thoroughly** on all 65 conformance test grammars
4. **Add more tests** for edge cases

### Future (v0.4.0)
1. **Use analysis for optimizations**:
   - Memoization for recursive rules
   - Special handling for left-recursive rules
   - Warn users about problematic patterns

2. **Selective inlining**:
   - Inline hidden rules (`-name`) only
   - Preserve XML structure
   - Measure performance impact

3. **Grammar transformation**:
   - Transform left-recursion to right-recursion
   - Optimize nullable sequences
   - Simplify complex patterns

## Test Results

### With Analysis Disabled
- Conformance: **75.4%** (49/65 tests passing)
- All test categories functional
- No regressions

### With Analysis Enabled (Simple Grammar)
```bash
$ echo -n "1+2" | ./rustixml /tmp/expr.ixml -
[rustixml] Grammar analysis:
⚠️  Left-recursive rules (may cause infinite loops):
   - expr
```

✅ Correctly detects left-recursion
❌ Causes stack overflow on complex grammars

## Conclusion

Grammar analysis implementation is **complete and functional** for simple grammars, but **disabled due to stack overflow** on complex grammars that cannot be solved with depth limiting or larger stacks.

**What works**:
- ✅ Recursion detection (direct and indirect)
- ✅ Left-recursion detection on simple grammars (tested with `expr`)
- ✅ Nullable detection
- ✅ Complexity scoring
- ✅ Human-readable reporting
- ✅ 3 passing unit tests

**What doesn't work**:
- ❌ Analysis of highly complex grammars like `ambiguous/ambig`
- ❌ Depth limiting (tried 100, 50, 20 - all failed)
- ❌ Larger stack sizes (tried 8MB - still failed)

**Current Status** (2025-11-22 Update):
- ✅ **Iterative recursion detection WORKING** - Successfully analyzes all grammars including `ambiguous/ambig` without stack overflow
- ⚠️  **Left-recursion detection DISABLED** - Nullable check still has recursion issues
- Grammar analysis enabled in `/home/bigale/repos/rustixml/src/native_parser.rs:25-31` with iterative algorithms
- Conformance maintained at **75.4%** (49/65 tests passing)
- All code compiles successfully

**What's Fixed**:
- ✅ `is_recursive()` rewritten with explicit stack - handles arbitrarily complex grammars
- ✅ Successfully completes analysis on `ambiguous/ambig` grammar (previously caused stack overflow)
- ✅ No more stack overflow on conformance tests

**What Still Needs Work**:
- ⚠️ Left-recursion detection temporarily disabled (line 51 in grammar_analysis.rs)
- ⚠️ Nullable detection still uses recursion (needs full rewrite with explicit stack)

**Recommendation**: Complete the iterative rewrite of nullable detection to re-enable left-recursion detection.

**Estimated effort**: 1-2 days to rewrite nullable detection with explicit stack and re-enable left-recursion detection.
