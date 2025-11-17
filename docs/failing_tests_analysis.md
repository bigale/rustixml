# Failing Tests Analysis - rustixml Conformance Suite

## Overview
- **Total tests**: 49
- **Passing**: 19 (38.8%) ✅
- **Failing**: 0 ✅
- **Timeout**: 19 (38.8%)
- **Error**: 6 (12.2%)
- **Skip**: 5 (10.2%)

## Grammar Parse Errors (3 tests)

These tests fail during grammar parsing, before any input parsing occurs.

### 1. **nested-comment** - Nested brace comments
**Issue**: Grammar contains nested comments with braces
```ixml
{here is a comment
{with a nested comment
b: "c".
}}
```
**Root cause**: Lexer doesn't support nested brace comments
**Fix difficulty**: Medium - need to track comment nesting depth in lexer

### 2. **program** - Complex structured language
**Issue**: Grammar parse error on complex programming language grammar
```ixml
block: "{", S, statement**(";", S), "}", S.
statement: if-statement; while-statement; assignment; call; block; .
```
**Root cause**: Likely interaction between separated repetitions and complex alternatives
**Fix difficulty**: Medium - need to debug specific grammar parsing issue

### 3. **ranges1** - Complex character ranges
**Issue**: Grammar with multiple negated character class definitions
```ixml
punctuation: ~[#a; #d; #30-#39; #41-#5A; #61-#7A]+.
```
**Root cause**: Complex semicolon-separated character class ranges
**Fix difficulty**: Medium - may need better character class parsing

## Input Parse Errors (3 tests)

These tests parse the grammar successfully but fail during input parsing.

### 1. **email** - Email address parsing
**Issue**: Input parse error during email validation
```ixml
-word: letgit+.
-letgit: ["A"-"Z"; "a"-"z"; "0"-"9"].
```
**Root cause**: Unknown - need to debug specific input parsing failure
**Fix difficulty**: Low-Medium - likely a small bug in runtime parser

### 2. **empty-group** - Empty group handling
**Issue**: Grammar contains empty group `()`
```ixml
a: b, (), c.
```
**Root cause**: Empty group action registration not implemented
**Fix difficulty**: Low - add special case for empty groups

### 3. **unicode-range1** - Mixed range types
**Issue**: Character range mixing hex and literal characters
```ixml
chars: [#1-"÷"]+.
```
**Root cause**: Range parsing doesn't handle [hex-literal] format
**Fix difficulty**: Medium - need to extend character class range parsing

## Timeout Tests (19 tests)

These tests timeout after 2 seconds, indicating exponential parsing complexity.

### Category 1: Left-Recursive Expression Grammars (7 tests)
**Tests**: expr, expr1, expr2, expr3, expr4, expr5, expr6

**Pattern**: All contain left-recursive expression parsing rules
```ixml
-expr: term; sum; diff.
sum: expr, -"+", term.  ← left-recursive
diff: expr, "-", term.  ← left-recursive
```

**Root cause**: Earley parsers handle left-recursion but with O(n³) complexity. These grammars create parse forests with exponential growth.

**Example complexity**:
- Input: "a+b+c+d+e"
- Parse trees: Exponential in expression depth
- Time: 2+ seconds even for short inputs

**Fix difficulty**: High - requires:
- Parser optimization (memoization, better forest management)
- Or grammar transformation (eliminate left-recursion)
- Or switching to a faster GLR implementation

### Category 2: Repetition-Heavy Text Parsing (4 tests)
**Tests**: diary, diary2, diary3, address

**Pattern**: Use character class repetitions over large text inputs
```ixml
para: word++s, s?, blank.
-word: (letter; punctuation)+.
-letter: [L].  ← matches all Unicode letters
```

**Root cause**:
- Character class `[L]` matches thousands of Unicode characters
- Creates huge terminal sets in parser
- Input files are large (hundreds of words)
- Separated list parsing creates many alternatives

**Fix difficulty**: Medium-High - requires:
- Better character class terminal handling
- Parser optimizations for repetitions
- May need streaming/lazy evaluation

### Category 3: Complex Structured Formats (4 tests)
**Tests**: json, json1, xml, xml1

**Pattern**: Recursive data structure grammars
```ixml
element: value.
-value: string; number; object; array; ...
object: "{", S, members, "}", S.
array: "[", S, elements, "]", S.
```

**Root cause**:
- Deeply nested structures in input
- Recursive grammar rules
- Many alternatives at each level
- Parse forest explosion

**Examples**:
- JSON: Nested objects/arrays
- XML: Nested elements with attributes

**Fix difficulty**: High - requires:
- Parser performance improvements
- Better ambiguity resolution
- Possible incremental parsing

### Category 4: Contact/Record Formats (1 test)
**Tests**: vcard

**Pattern**: Line-based record format with many fields
```ixml
card: -"BEGIN:", name, eoln, property+, -"END:", endname, eoln.
property: name, parameters, -":", attribute++-";", -eoln.
```

**Root cause**: Large input files with many properties per card
**Fix difficulty**: Medium - similar to diary series

### Category 5: Complex Expression Languages (2 tests)
**Tests**: xpath, poly

**Pattern**: Very large grammars with many production rules

**xpath**: Full XPath 3.1 expression language
- 100+ grammar rules
- Multiple levels of operator precedence
- Complex expression nesting

**poly**: Polynomial notation
- Uses Unicode superscript characters for exponents
- Pattern matching for coefficient extraction

**Root cause**:
- Grammar size creates large parse tables
- Many alternatives at each step
- Input ambiguity

**Fix difficulty**: High - requires significant parser optimization

### Category 6: Unicode Category Testing (1 test)
**Tests**: unicode-classes

**Pattern**: Tests all Unicode character categories
```ixml
-line: ( C; Cc; Cf; Cn; Co; Cs; L; LC; Ll; Lm; Lo; Lt; Lu;
         M; Mc; Me; Mn; N; Nd; Nl; No; P; Pc; Pd; Pe; Pf;
         Pi; Po; Ps; S; Sc; Sk; Sm; So; Z; Zl; Zp; Zs), newline.
```

**Root cause**:
- 30+ Unicode category rules
- Large input testing all categories
- Character class implementation may be inefficient

**Fix difficulty**: Medium - optimize Unicode category handling

## Summary by Fix Difficulty

### Low (1 test)
- **empty-group**: Add empty group handling

### Low-Medium (1 test)
- **email**: Debug input parsing failure

### Medium (5 tests)
- **nested-comment**: Implement nested comment support
- **program**: Debug complex grammar parsing
- **ranges1**: Better character class parsing
- **unicode-range1**: Mixed range type support
- **unicode-classes**: Optimize Unicode categories
- **vcard**: Optimize repetition handling

### Medium-High (4 tests)
- **diary, diary2, diary3, address**: Optimize character class and repetition performance

### High (14 tests)
- **expr series (7)**: Left-recursion optimization
- **json, json1, xml, xml1 (4)**: Recursive structure optimization
- **xpath, poly (2)**: Large grammar optimization
- **poly**: Unicode superscript handling

## Recommended Priorities

### Quick Wins (implement first)
1. **empty-group** - Simple fix, immediate gain
2. **email** - Likely a small bug
3. **nested-comment** - Lexer enhancement

### Medium-term (parser improvements)
4. **Character class optimization** - Helps diary, address, unicode-classes
5. **Repetition optimization** - Helps diary series, vcard
6. **Grammar parsing bugs** - Fixes program, ranges1, unicode-range1

### Long-term (architectural changes)
7. **Left-recursion handling** - Requires parser algorithm improvements
8. **Recursive structure handling** - Requires parse forest optimization
9. **Large grammar support** - May need GLR enhancements or different parser strategy

## Performance Bottleneck Analysis

The main bottleneck is **Earley parser performance** on:
1. **Left-recursive grammars** (expr series) - O(n³) complexity
2. **Ambiguous grammars with repetitions** (diary, json, xml)
3. **Large terminal sets** (Unicode character classes)

Potential solutions:
- **Memoization**: Cache parse results to avoid recomputation
- **Parse forest pruning**: Discard equivalent subtrees earlier
- **Better GLR**: Switch to more efficient GLR implementation
- **Incremental parsing**: Parse incrementally instead of all at once
- **Grammar transformation**: Automatically eliminate left-recursion
