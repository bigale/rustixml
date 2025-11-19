# Project Transition Summary: Earley → Native Interpreter

**Date**: November 19, 2025  
**Status**: Design phase complete, ready for implementation

## Executive Summary

The Earley-based iXML implementation has reached **39.8% conformance** (53/133 tests passing) and hit a **fundamental limitation** with insertion+suppression patterns. We're now transitioning to a **native Rust interpreter** that directly implements iXML semantics.

## Earley Implementation: Final Status

### Achievements ✅

- **53 tests passing** (39.8% of 133 total)
- **Zero grammar parse errors** - all iXML grammars parse correctly
- **87.8% pass rate** on basic correctness tests (43/49)
- **Full Unicode General Category support** using `unicode-general-category` crate
- **Character class parsing** with ranges, hex chars, Unicode categories
- **Thread-local GROUP_COUNTER** synchronization fix
- **Comprehensive test infrastructure** with Docker-based safe runner

### Hard Limit Reached 🚫

**Pattern**: `(-[Co], +".")*`  
**Problem**: Combining suppression (`-`) with insertion (`+`) in repeated sequences

**Why it fails**:
- Earley parsers fundamentally **consume input tokens**
- Insertions (`+"text"`) add output not present in input
- No natural way to express "consume this, output something else" in a loop
- Translation to Earley creates action mismatches

**Affected tests**: `unicode-classes`, `ixml-spaces`, `ixml3`

**Analysis**: See `docs/earley_insertion_limitation.md` and `docs/ABSTRACTION_ANALYSIS.md`

### What Works

✅ **Terminals**: Literals, character classes, hex chars  
✅ **Nonterminals**: Rule references  
✅ **Alternatives**: Choice expressions  
✅ **Sequences**: Concatenation  
✅ **Repetitions**: `*`, `+`, `?`, `**`, `++`  
✅ **Marks**: `@` (attribute), `-` (hidden), `^` (promoted)  
✅ **Simple insertions**: `+"text"` in non-looping contexts  
✅ **Unicode**: Full General Category support, multi-byte characters  

### What Doesn't Work

❌ **Combined insertion+suppression in loops**: `(-[x], +"y")*`  
❌ **Ambiguous parse trees**: Only returns one parse  
❌ **Some complex grammar structures**: Performance issues  

## Native Interpreter: Design

### Architecture

**Specification-First Approach**: Direct implementation of iXML semantics, no translation layer.

**Core Components**:

1. **InputStream** (`src/input_stream.rs`) - Input management with backtracking
2. **ParseContext** (`src/parse_context.rs`) - State tracking during parsing
3. **NativeParser** (`src/native_parser.rs`) - Main interpreter (recursive descent)

### Key Design Decisions

**Insertion Handling**: 
```rust
if insertion {
    return Ok(ParseResult {
        node: Some(XmlNode::Text(value)),
        consumed: 0, // Key: consume nothing
    });
}
```
- Insertions consume 0 characters
- Always succeed
- Create output node
- **Naturally handles** `(-[Co], +".")*` pattern

**Mark Handling**:
- Applied **after** successful parsing
- Post-processing operation, not parse-time behavior
- Clean separation of concerns

**Error Handling**:
- Rich error types with position info
- Grammar-aligned error messages
- No more "Missing Action" or "No Rule completes"

### Implementation Phases

| Phase | Focus | Estimated LOC | Effort |
|-------|-------|---------------|--------|
| 1 | Core infrastructure | ~150 | 2-3 hours |
| 2 | Basic terminals | ~200 | 3-4 hours |
| 3 | Sequences & alternatives | ~150 | 2-3 hours |
| 4 | Repetitions | ~200 | 4-5 hours |
| 5 | Marks & insertions | ~150 | 3-4 hours |
| 6 | Full test suite | ~150 | 5-8 hours |
| **Total** | | **~1400** | **20-28 hours** |

Compare to Earley approach: **3600+ LOC**

### Expected Performance

| Grammar Type | Native | Earley |
|--------------|--------|--------|
| Deterministic | **O(n)** | O(n³) |
| Ambiguous | O(n²) | O(n³) |
| With insertions | **O(n)** | ❌ Fails |

Where n = input length

### Success Criteria

A successful implementation will:

- ✅ Pass `unicode-classes` test (the Earley blocker)
- ✅ Pass all 43 currently passing tests
- ✅ Pass additional tests Earley couldn't handle
- ✅ Have ~1400 LOC (vs. 3600+ for Earley)
- ✅ Demonstrate Rust strengths (pattern matching, iterators)
- ✅ Target 80-90%+ conformance

## Key Documents

| Document | Purpose |
|----------|---------|
| `CLAUDE.md` | Main project documentation, test results, implementation notes |
| `docs/ARCHITECTURE.md` | Earley implementation architecture |
| `docs/ABSTRACTION_ANALYSIS.md` | Why Earley approach hit limits |
| `docs/earley_insertion_limitation.md` | Detailed analysis of insertion+suppression bug |
| `docs/NATIVE_INTERPRETER_DESIGN.md` | Complete native interpreter design (this is the implementation blueprint) |
| `PROGRESS.md` | Historical development progress |

## Next Steps

1. **Implement Phase 1** - Core infrastructure (InputStream, ParseContext)
2. **Implement Phase 2** - Basic terminals (get first test passing)
3. **Iterate** through phases 3-6
4. **Validate** with full test suite
5. **Optimize** performance hotspots
6. **Document** API and usage patterns

## Comparison: Before & After

### Before (Earley)
```rust
// Complex translation with helper rules
// word**"-" becomes:
//   word_sep_star -> epsilon | word_sep_plus
//   word_sep_plus -> word | word_sep_plus "-" word
// Insertion creates "Missing Action" errors
```

### After (Native)
```rust
// Direct implementation
fn parse_repetition(factor, sep, kind) {
    let mut results = vec![parse_factor(factor)?];
    loop {
        let saved = stream.position();
        if let Ok(_) = parse_sequence(sep) {
            if let Ok(item) = parse_factor(factor) {
                results.push(item);
                continue;
            }
        }
        stream.set_position(saved);
        break;
    }
    Ok(results)
}
// Insertions work naturally - consume 0, always succeed
```

### Code Complexity

| Aspect | Earley | Native |
|--------|--------|--------|
| Total LOC | 3663 | ~1400 (est.) |
| Complexity | High (translation layer) | Low (direct) |
| Debugging | Cryptic errors | Grammar-aligned |
| Maintainability | Hard | Easy |
| Extensibility | Limited | High |

## Why This Will Work

1. **Specification Alignment**: Code structure mirrors iXML spec
2. **Rust Strengths**: Pattern matching, Option/Result, iterators
3. **Proven Pattern**: Recursive descent is well-understood
4. **Natural Semantics**: Insertion/suppression are first-class
5. **Simplicity**: Less code = fewer bugs

## Risk Mitigation

**Risk**: Performance with large grammars  
**Mitigation**: Profile and optimize, add memoization if needed

**Risk**: Ambiguous grammars  
**Mitigation**: Start with first-match (PEG), add multi-parse later

**Risk**: Complex edge cases  
**Mitigation**: Comprehensive test suite, incremental implementation

**Risk**: Time to implement  
**Mitigation**: Clear design, phased approach, ~20-28 hours estimated

## Conclusion

The Earley implementation taught us valuable lessons about abstraction mismatches. The native interpreter addresses these issues by **directly implementing iXML semantics** rather than translating to an intermediate representation.

**Bottom line**: We're trading a complex, limited implementation (3600+ LOC, 39.8% pass rate) for a simpler, more capable one (~1400 LOC, 80-90%+ expected pass rate).

**Status**: Ready to begin Phase 1 implementation.
