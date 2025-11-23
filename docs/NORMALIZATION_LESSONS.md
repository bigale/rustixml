# Grammar Normalization: Lessons Learned

**Date**: 2025-11-22
**Status**: Initial implementation - needs refinement

## What We Implemented

Successfully implemented the core normalization algorithm from Steven Pemberton's iXML talk:
1. ✅ Recursion detection (direct and indirect)
2. ✅ Non-recursive rule inlining
3. ✅ Integration with NativeParser

## The Problem We Discovered

**Normalization broke conformance: 75.4% → 24.6%**

### Root Cause

The normalization described in the paper is designed for **Earley parsers**, which work differently from our **recursive descent parser**:

**Earley Parser (from the paper)**:
- Grammar is a "schema" describing data structure
- Rules don't directly create XML elements
- Normalization simplifies the grammar structure
- XML output is generated from the parse tree based on the schema

**Our Recursive Descent Parser**:
- Each rule creates an XML element with that rule's name
- Inlining removes rules → removes XML elements
- This changes the output structure!

### Example

**Original Grammar**:
```ixml
word: letter+.
letter: ["a"-"z"].
```

**Input**: `abc`

**Expected Output**:
```xml
<word><letter>a</letter><letter>b</letter><letter>c</letter></word>
```

**After Full Normalization**:
```ixml
word: (["a"-"z"])+.
```

**Actual Output** (with current naive inlining):
```xml
<word>abc</word>
```

The `<letter>` elements disappeared because we inlined the `letter` rule!

## Why This Matters

The normalization paper assumes the parser architecture separates:
1. **Parsing** (recognizing structure)
2. **Serialization** (generating XML from recognized structure)

Our parser combines both steps - rules ARE the XML structure.

## The Correct Approach for rustixml

We need **selective normalization** that preserves XML structure:

### Option 1: Only Inline Transparent Rules

Inline only rules that don't create XML elements:
- **Hidden rules** (`-name`): Don't create elements, safe to inline
- **Promoted rules** (`^name`): Replace parent with child, complex to inline
- **Attribute rules** (`@name`): Create attributes, not elements

**Algorithm**:
```rust
fn should_inline(rule: &Rule) -> bool {
    // Only inline if rule is hidden (doesn't create XML element)
    rule.mark == Mark::Hidden
}
```

### Option 2: Use Normalization for Analysis Only

Don't modify the grammar, but use normalization information for optimization:
- Detect which rules are recursive
- Detect potential left-recursion
- Identify inlining opportunities for performance
- Keep original grammar for parsing

**Algorithm**:
```rust
struct NormalizationInfo {
    recursive_rules: HashSet<String>,
    inlinable_rules: HashSet<String>,
    left_recursive_rules: HashSet<String>,
}

impl NativeParser {
    pub fn new(grammar: IxmlGrammar) -> Self {
        // Analyze grammar structure
        let info = analyze_grammar(&grammar);

        // Use info for optimizations, but don't modify grammar
        NativeParser {
            grammar,
            optimization_info: info,
        }
    }
}
```

### Option 3: Normalization with XML Reconstruction

Keep track of inlined rules and reconstruct their XML elements during parsing:
- Inline for performance
- Remember original structure
- Recreate XML elements as needed

This is complex and error-prone.

## Recommended Next Steps

### Short Term (v0.3.0)
**Use Option 2: Analysis-Only Normalization**

1. Keep `normalize.rs` for recursion detection
2. Create `GrammarAnalysis` struct with optimization hints
3. Use analysis to:
   - Detect left-recursion (error or transform)
   - Enable memoization for recursive rules
   - Identify optimization opportunities

**Benefits**:
- Zero risk to conformance
- Provides foundation for future optimizations
- Makes grammar structure explicit

### Medium Term (v0.4.0)
**Add Option 1: Selective Inlining**

1. Inline hidden rules only
2. Test thoroughly to ensure no output changes
3. Measure performance improvement

### Long Term (v2.0)
**Reconsider Architecture**

If we want full normalization benefits, we might need to:
- Separate parsing from serialization
- Use normalized grammar internally
- Map back to original structure for XML output

Or accept that our architecture is fundamentally different from the paper's approach.

## Code Changes Made

### `/home/bigale/repos/rustixml/src/normalize.rs`
- Implemented recursion detection ✅
- Implemented inlining ✅
- **Status**: Disabled due to XML structure issues

### `/home/bigale/repos/rustixml/src/native_parser.rs`
- Added normalization call (now commented out)
- **TODO**: Use normalization for analysis only

## Tests Results

### Before Normalization
- Conformance: 75.4% (49/65 passing)
- All three test categories working

### With Full Normalization
- Conformance: 24.6% (16/65 passing)
- Many tests failing due to missing XML elements

### After Disabling Normalization
- Should return to 75.4%
- Need to verify

## Key Insights

1. **Not all optimizations from research papers apply directly**
   - Papers assume specific architectures
   - Need to adapt techniques to our design

2. **XML output preservation is critical**
   - iXML is fundamentally about XML serialization
   - Can't change output structure

3. **Normalization still valuable for analysis**
   - Recursion detection helps with left-recursion
   - Structure analysis helps with optimization
   - Don't need to modify grammar to get benefits

4. **Incremental approach is wise**
   - Start with analysis-only
   - Add selective optimizations carefully
   - Test thoroughly at each step

## Next Implementation

Create `grammar_analysis.rs` that provides:
```rust
pub struct GrammarAnalysis {
    pub recursive_rules: HashSet<String>,
    pub left_recursive_rules: HashSet<String>,
    pub hidden_rules: HashSet<String>,
    pub complexity_score: HashMap<String, usize>,
}

pub fn analyze_grammar(grammar: &IxmlGrammar) -> GrammarAnalysis {
    // Reuse recursion detection from normalize.rs
    // Add left-recursion detection
    // Identify optimization candidates
}
```

Then use this in `NativeParser` to:
- Warn about left-recursion
- Enable targeted optimizations
- Provide debugging information

## Conclusion

Grammar normalization from the paper is valuable but needs adaptation:
- ✅ Recursion detection works perfectly
- ❌ Full inlining breaks XML structure
- ✅ Analysis-based approach is the way forward

The code we wrote is not wasted - it's the foundation for grammar analysis that will enable targeted optimizations without breaking conformance.
