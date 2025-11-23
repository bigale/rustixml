# Ambiguity Tracking Analysis

**Date**: 2025-11-23
**Status**: Analysis complete, implementation deferred

## Current State

### What We Achieved

1. **Static Ambiguity Detection** - WORKING ✓
   - Grammar normalization (Pemberton's approach)
   - Fixpoint nullable detection
   - Pattern 3: Consecutive nullable nonterminals
   - Successfully detects ambig2, ambig3 patterns
   - Zero regressions, 75.4% conformance maintained

2. **Detection Methodology**
   - Normalize grammar to inline hidden/promoted rules
   - Build nullable set using fixpoint iteration
   - Check for three ambiguity patterns:
     - Pattern 1: Nullable alternatives
     - Pattern 2: Overlapping alternatives
     - Pattern 3: Consecutive nullable nonterminals
   - Mark output with `ixml:state="ambiguous"`

### What We Discovered

The iXML test suite expects **detailed diagnostic comments** for ambiguous grammars:

```xml
<!-- AMBIGUOUS
     The input from line.pos 1.1 to 1.6 can be interpreted as 'a' in 4 different ways:
     1: a[1.1:]:  "a"[:1.2] spaces[:1.2] b[:1.6]
     2: a[1.1:]:  "a"[:1.2] spaces[:1.3] b[:1.6]
     3: a[1.1:]:  "a"[:1.2] spaces[:1.4] b[:1.6]
     4: a[1.1:]:  "a"[:1.2] spaces[:1.5] b[:1.6]
-->
<a ixml:state="ambiguous" xmlns:ixml="http://invisiblexml.org/NS">
  <!-- One of the 4 valid parse trees -->
</a>
```

This is a **test suite convention** (Steven Pemberton's tests), not a strict iXML spec requirement.

## Exhaustive Parsing Approach

### Requirements to Pass Ambiguous Tests

To generate the diagnostic comments, we would need:

1. **Exhaustive parsing** - Return ALL valid parse trees, not just the first
2. **Position tracking** - Record which rules matched at which positions
3. **Diagnostic generation** - Format the comment describing all alternatives

### Implementation Estimate

- Modify parser for exhaustive parsing: 2-3 days
- Position tracking and diagnostic generation: 1-2 days
- Testing and refinement: 1 day
- **Total effort**: 4-6 days

### Potential Impact

- **Best case**: +11 ambiguous tests = 92% conformance (60/65 tests)
- **Current**: 75.4% (49/65 tests) with correct detection

### Architecture Sketch

```rust
// Phase 1: Static Analysis (already done)
let analysis = GrammarAnalysis::analyze(&grammar);

if analysis.is_potentially_ambiguous {
    // Phase 2: Parse exhaustively
    let all_parses = parse_all_alternatives(&input, &grammar);

    if all_parses.len() > 1 {
        // Phase 3: Generate diagnostic
        let comment = format!(
            "<!-- AMBIGUOUS\n     The input from line.pos {} to {} \
             can be interpreted as '{}' in {} different ways:\n{}\n-->",
            start, end, root, all_parses.len(),
            describe_all_parses(&all_parses)
        );

        // Phase 4: Return first parse with comment
        return format!("{}\n{}", comment, all_parses[0].to_xml());
    }
}
```

## Why We're Deferring This

### Concerns About Stacked Ambiguities

**Real-world grammars can have complex, nested ambiguities**:

1. **Combinatorial explosion**: Grammar with N ambiguous choice points → 2^N possible parses
2. **Nested ambiguities**: Ambiguity within ambiguity creates exponential parse trees
3. **Performance**: Exhaustive parsing could be 10-100x slower on complex grammars
4. **Memory**: Storing all parse trees could exhaust memory on pathological inputs

**Example pathological case**:
```ixml
expr: a | b.
a: term | term, " ".
b: term | term, " ".
term: atom | atom, " ".
atom: "x" | "x", " ".
```

Input: `x     ` (5 spaces) → Potentially hundreds of valid parses

### The Basic Tests Might Pass But...

The conformance tests are **carefully constructed examples** with limited ambiguity depth. Real-world grammars (like the EDI X12 grammars we're targeting) could have:

- Deeply nested optional elements
- Multiple nullable sequences
- Complex interaction between rules

We need to understand the **worst-case behavior** before implementing exhaustive parsing.

## Decision

**Focus on fixing the 4 "correct" category parsing failures first**:

1. `correct/expr` - Parse succeeded but input remains
2. `correct/unicode-classes` - Parse succeeded but input remains
3. `correct/vcard` - Parse error in 'eoln' rule
4. `correct/xpath` - Parse succeeded but input remains

These represent **actual parser bugs** that affect correctness, not just diagnostic formatting.

**Ambiguity tracking improvements are deferred** until we:
1. Understand performance implications better
2. Fix core parsing issues
3. Evaluate whether real-world use cases need exhaustive parsing

## What We Keep

The **static ambiguity detection** remains enabled and valuable:

- Warns users about potentially ambiguous grammars
- Marks output with `ixml:state="ambiguous"`
- Provides grammar analysis diagnostics
- Zero performance cost during parsing (one-time analysis at grammar load)

This gives users actionable information without the complexity and performance cost of exhaustive parsing.

## Next Steps

### Immediate (Current Sprint)

1. Fix `correct/expr` - likely operator precedence or left-recursion issue
2. Fix `correct/unicode-classes` - likely character class handling
3. Fix `correct/vcard` - specific rule matching issue
4. Fix `correct/xpath` - complex grammar parsing issue

**Target**: 53/65 tests (81.5% conformance) by fixing these 4 tests

### Future (If Needed)

1. Analyze real-world EDI grammar ambiguity patterns
2. Implement selective exhaustive parsing (only for flagged ambiguous grammars)
3. Add depth/parse-count limits to prevent combinatorial explosion
4. Benchmark on pathological cases
5. Consider hybrid approach: static detection + optional exhaustive mode

## Files Modified

- `/home/bigale/repos/rustixml/src/grammar_analysis.rs` - Grammar normalization, Pattern 3 detection
- `/home/bigale/repos/rustixml/src/native_parser.rs` - Uses analysis to set `ixml:state="ambiguous"`

## Performance

Grammar analysis overhead: **< 1ms** per grammar (one-time cost at load)
Zero runtime parsing overhead
