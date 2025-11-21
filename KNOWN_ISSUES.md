# Known Issues and Limitations

This document describes known limitations of rustixml v0.2.0 and potential paths for improvement.

## Current Conformance: 69.2% (45/65 tests)

### Breakdown by Category:
- **correct**: 41/49 (83.7%) ✅
- **ambiguous**: 2/13 (15.4%)
- **error**: 2/3 (66.7%)

## Test Failures

### Grammar Parse Errors (5 tests)

These tests use iXML features not yet implemented in our grammar parser:

1. **`correct/unicode-version-diagnostic`** - Unicode version pragma
2. **`correct/version-decl`** - Version declaration syntax
3. **`correct/version-decl.2`** - Version declaration variant
4. **`correct/ws-and-delim`** - Advanced whitespace handling in grammar
5. **`error/invalid-char`** - Error test with intentional invalid characters

**Impact**: Low - These are advanced iXML 1.0 features not commonly used.

**Fix Path**: Implement iXML version pragma and advanced delimiter syntax parsing.

### Parse Failures (15 tests)

Tests where the parser fails or produces incorrect output:

#### Ambiguity Tests (11 failures)
Tests in the `ambiguous/` category expect the parser to detect and report ambiguous parses:

- `ambiguous/ambig` - Left recursion: "a+a+a"
- `ambiguous/ambig2` - Multiple parse trees
- `ambiguous/ambig3` - Whitespace ambiguity
- `ambiguous/ambig4` - Line ending ambiguity
- `ambiguous/ambig5` - Digit grouping ambiguity
- `ambiguous/ambig6` - Rule block ambiguity
- `ambiguous/ambig7` - Sequential element ambiguity
- `ambiguous/css` - CSS selector ambiguity
- `ambiguous/date` - Date format ambiguity
- `ambiguous/expr0` - Expression ambiguity
- `ambiguous/lf2` - Line feed ambiguity

**Root Cause**: Our parser currently:
1. Does not detect ambiguity (picks first successful parse)
2. Does not add `ixml:state="ambiguous"` attributes
3. May fail on certain ambiguous grammars

**Impact**: Medium - Ambiguity detection is part of the iXML spec but not critical for basic parsing.

**Fix Path**: Implement ambiguity detection by tracking multiple parse paths and adding appropriate XML attributes.

#### Correctness Tests (4 failures)

- **`correct/expr`** - Left-recursive arithmetic expressions: "a+(10×b)"
- **`correct/unicode-classes`** - Unicode general category classes with complex patterns
- **`correct/vcard`** - vCard format with multiple line endings
- **`correct/xpath`** - XPath expression with predicates: "[.!='']"

**Root Causes**:
1. **Left recursion handling** - Some left-recursive patterns cause parse errors or incomplete parses
2. **Complex character class operations** - Unicode general categories with exclusions
3. **Line ending normalization** - Different newline styles in input
4. **Operator precedence** - Complex expressions with multiple operators

**Impact**: Medium-High - These affect real-world grammars (arithmetic, XPath, data formats).

**Fix Path**:
- Improve left-recursion handling in native parser
- Better character class optimization (see markup-blitz patterns below)
- Implement proper line ending normalization
- Review operator precedence rules

## Insights from markup-blitz (Reference Implementation)

The [markup-blitz](https://github.com/GuntherRademacher/markup-blitz) project by Gunther Rademacher achieves **100% conformance** (all 5168 iXML tests pass). Key differences:

### Architecture

**Parser Algorithm**: markup-blitz uses **LALR(1) + GLR** (Generalized LR parsing)
- LALR(1) for deterministic parsing where possible
- GLR for handling ambiguity dynamically
- Better worst-case performance than Earley for many grammars

**rustixml** uses a **Native Recursive Descent Parser**
- Direct implementation of iXML semantics
- O(n) for non-ambiguous grammars
- O(n²) or worse for ambiguous/left-recursive grammars

### Key Optimizations in markup-blitz

#### 1. Character Class Partitioning
**Problem**: Overlapping character classes create redundant states.

**Solution**: Partition all character sets into non-overlapping equivalence classes.

**Example**:
```
Grammar has: [a-z], [a-m], [0-9]
Partitions: [a-m], [n-z], [0-9]
[a-z] → [a-m] | [n-z]
[a-m] → [a-m]
```

**Relevance to rustixml**: Could reduce parser complexity for grammars with many overlapping character classes (like `unicode-classes`).

**Status**: Partially implemented in `src/charclass.rs` but not yet integrated into parser.

#### 2. Two-Rule Separated Repetition Pattern
**Problem**: Repetitions with separators (`a++sep`) need efficient rule expansion.

**markup-blitz approach**:
```
base**(sep) creates TWO rules:
  listName: base | listName sep base  (left-recursive list)
  name: | listName                    (optional wrapper)
```

**rustixml current approach**: Single rule with complex sequence matching.

**Relevance**: May improve performance and correctness for separated repetitions.

#### 3. Ambiguity Detection
markup-blitz's GLR parser naturally detects ambiguity by tracking fork points where multiple parse paths exist.

**rustixml needs**: Explicit ambiguity tracking by:
- Recording alternative parse paths
- Detecting when multiple paths succeed
- Adding `ixml:state="ambiguous"` to output

#### 4. Precomputed Parse Tables
LALR precomputes state transition tables, making parsing O(1) per token.

**rustixml uses**: Dynamic recursive descent, which recomputes parse decisions at runtime.

**Trade-off**: Recursive descent is simpler to implement and debug, but slower for complex grammars.

## Future Improvement Paths

### Short Term (v0.3)
1. ✅ **Grammar Parser**: Add version pragma and advanced delimiter support (5 tests)
2. ✅ **Line Ending Normalization**: Handle CR, LF, CRLF properly (1-2 tests)
3. ✅ **Better Left Recursion**: Improve detection and handling (2-3 tests)

**Expected**: 50-55 passing tests (77-85%)

### Medium Term (v0.4)
1. ✅ **Ambiguity Detection**: Track multiple parse paths, add ixml:state attribute (11 tests)
2. ✅ **Character Class Optimization**: Implement partitioning (1-2 tests)
3. ✅ **Better Error Messages**: Improve parse error reporting

**Expected**: 58-60 passing tests (89-92%)

### Long Term (v1.0)
1. ✅ **Consider GLR/LALR**: Evaluate parser algorithm change for better performance
2. ✅ **Full Unicode Support**: All Unicode general categories and properties
3. ✅ **Extensive Testing**: Test against all 5168 markup-blitz test cases

**Target**: 95%+ conformance

## Performance Considerations

Current performance is acceptable for most use cases:
- Simple grammars: < 1ms
- Medium grammars: 1-10ms
- Complex grammars: 10-100ms

The 15 failing tests are not primarily performance issues but correctness issues:
- Ambiguity not detected
- Some left-recursive patterns fail
- Complex character class operations

## Why This Is Still v0.2.0

Despite the known issues, rustixml v0.2.0 is **production-ready** for:

✅ **Most iXML grammars** (83.7% of correctness tests pass)
✅ **Date/time parsing**
✅ **CSV and simple data formats**
✅ **Simple expression languages**
✅ **Configuration file parsing**
✅ **WebAssembly deployment** (WASMZ pattern)

Not recommended for:
❌ Complex ambiguous grammars
❌ Left-recursive arithmetic expressions with multiple operators
❌ Grammars requiring advanced iXML 1.0 features

## Contributing

If you'd like to help improve conformance:

1. **Easy**: Add tests for the 5 grammar parse errors
2. **Medium**: Implement ambiguity detection
3. **Hard**: Optimize character class handling
4. **Expert**: Consider GLR/LALR algorithm

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## References

- [iXML Specification](https://invisiblexml.org/1.0/)
- [markup-blitz](https://github.com/GuntherRademacher/markup-blitz) - Reference Java implementation
- [CLAUDE.md](CLAUDE.md) - Detailed architectural notes and markup-blitz analysis
