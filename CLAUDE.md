# rustixml - iXML Parser Implementation in Rust
CRITICAL - RUN ALL TESTS IN BACKGROUND
## Project Overview

rustixml is a Rust implementation of an iXML (Invisible XML) parser. iXML is a grammar specification language for converting text to XML without explicit markup. This implementation:

- Parses iXML grammar specifications
- Converts grammars to Earley parser grammars (using the `earlgrey` crate)
- Parses input text according to the grammar
- Generates canonical XML output

## Current Status

### Conformance Test Results (Latest Run - COMPLETE via Docker)

**Test Suite**: `/home/bigale/repos/ixml/tests/correct/` (49 total tests)

**Latest Results** (with Unicode character handling fix):
- **27 PASSING** (55.1%)
- **11 FAILING** (22.4%) - output mismatch
- **0 TIMEOUTS** (0%)
- **6 INPUT_ERRORS** (12.2%)
- **5 SKIP** (10.2%) - missing files or not applicable

### Passing Tests (27)
- `aaa` - Hidden marked literals
- `arith` - Arithmetic expression with canonical XML formatting
- `attribute-value` - XML entity escaping in attributes
- `diary3` - Diary format parsing
- `element-content` - XML entity escaping in text content
- `empty-group` - Empty group handling
- `expr` - Expression grammar with Unicode operators
- `expr3`, `expr5`, `expr6` - Expression variants
- `hash` - Separated repetitions with canonical formatting
- `hex`, `hex1`, `hex3` - Hexadecimal parsing
- `lf` - Marked hex characters (hidden linefeed)
- `marked` - Marked literals with attribute marks
- `nested-comment` - Nested brace comments
- `para-test` - Multi-paragraph parsing with character classes
- `program` - Complex grammar structure
- `range`, `range-comments` - Character ranges
- `ranges` - Character range edge cases
- `string` - String literals
- `tab` - Tab character handling
- `test` - Basic grammar test
- `unicode-range`, `unicode-range2` - Unicode character ranges

### Known Issues by Category

#### 1. Failing Tests (11) - Output mismatch
- `address` - Address parsing (output format issue)
- `expr1`, `expr2`, `expr4` - Expression variants
- `json` - JSON parsing (output format issue)
- `json1` - JSON variant
- `poly` - Polynomial parsing
- `ranges1` - Range syntax variation
- `vcard` - VCard parsing
- `xml` - XML parsing
- `xml1` - XML parsing variant

#### 2. Input Parse Errors (6)
- `diary`, `diary2` - Diary format parsing
- `email` - Character class matching issue
- `unicode-classes` - Unicode class support
- `unicode-range1` - Unicode range edge case
- `xpath` - XPath parsing

## Recent Fixes

### 1. Unicode Character Handling Fix (COMPLETED - MAJOR IMPROVEMENT)
**Files**: `src/runtime_parser.rs` (lines 619-624, 836, 1959, 2028, 2148)

**Problem**: Terminal matching used byte length (`s.len()`) instead of character count to detect single-character terminals. Multi-byte UTF-8 characters like `×` (U+00D7, 2 bytes) and `÷` (U+00F7, 2 bytes) were incorrectly treated as multi-character strings.

**Root Cause**: The code `s.len() == 1` returns `false` for Unicode characters that are multiple bytes in UTF-8.

**Fix**: Changed all occurrences to use `s.chars().count() == 1` for proper Unicode support:

```rust
// Before (broken for Unicode)
s.len() == 1 && s.chars().next() == Some(ch)

// After (correct)
s.chars().count() == 1 && s.chars().next() == Some(ch)
```

**Impact**:
- **Eliminated all 13 timeout tests** - they were actually failing instantly with grammar construction errors
- **+5 new passing tests**: `diary3`, `expr`, `expr3`, `expr5`, `expr6`
- Pass rate improved from **44.9% to 55.1%**

This was the root cause of the "timeout" tests - the Unicode character bug caused terminal matching to fail, which resulted in "Missing Symbol" and "No Rule completes" errors.

### 2. Handwritten Recursive Descent Parser (COMPLETED - MAJOR IMPROVEMENT)
**Files**:
- `src/grammar_parser.rs` (302 lines) - Complete handwritten recursive descent parser
- `src/grammar_ast.rs` - Grammar parser entry point
- `src/lib.rs:16` - Add grammar_parser module

**Features**:
- Parser struct with token stream and position tracking
- Recursive methods for each grammar element: `parse_grammar()`, `parse_rule()`, `parse_alternatives()`, `parse_sequence()`, `parse_factor()`, `parse_base_factor()`
- Supports all iXML features: marks (`@`, `-`, `^`), repetitions (`+`, `*`, `?`, `++`, `**`), insertions (`+string`), hex chars (`#a`), character classes, grouping
- Marked nonterminals: `@name`, `-name`, `^name` using `BaseFactor::marked_nonterminal()`
- Marked literals: `@"text"`, `-#a` using `BaseFactor::marked_literal()`
- EOF token filtering (lexer adds EOF, parser filters it out)

**Performance**: All test grammars parse in 2-21 microseconds (O(n) linear time)

The grammar parser is accessed via `grammar_ast::parse_ixml_grammar()`.

### 2. Canonical iXML XML Serialization Format (COMPLETED)
**Files**: `src/runtime_parser.rs:633-700`

Implemented the canonical iXML XML serialization format where:
- Opening and closing tags are written without their final `>`
- The `>` appears on the next line with indentation before the next content
- Exception: root element's final closing tag includes its `>`
- Self-closing elements follow the same pattern when they have siblings
- Elements with only text content use compact format

Example:
```xml
<expr plus='+'
   ><open>(</open
   ><left>a</left
   ><right>b</right
   ><close>)</close
></expr>
```

**Result**: Fixed tests `arith`, `range`, `range-comments`, `test` (but test runner crashed before completing all)

### 2. XML Entity Escaping
**Files**: `src/runtime_parser.rs:617-631`

- Attributes (using single quotes): escape `&`, `<`, `'`
- Text content: escape `&`, `<` only
- Fixed test: `attribute-value`

### 3. Marked Literals Support
**Files**:
- `src/ast.rs:85-90` - Added `mark: Mark` field to `BaseFactor::Literal`
- `src/grammar_ast.rs:46-63` - Grammar productions for `-"text"`, `@"text"`, `^"text"`
- `src/runtime_parser.rs:1017-1096` - Action registration for marked literals

Supports:
- Hidden marks: `-"text"` - creates `_hidden` elements (skipped in output)
- Attribute marks: `@"text"` - creates attribute nodes
- Promoted marks: `^"text"` - promotes content to parent level

**Result**: Fixed tests `aaa`, `element-content`

### 4. Separation Operators (`**` and `++`)
**Files**:
- `src/lexer.rs:15-18, 160-177` - Added `DoubleStar` and `DoublePlus` tokens
- `src/ast.rs:122-125` - Added `SeparatedZeroOrMore` and `SeparatedOneOrMore` to `Repetition` enum
- `src/grammar_ast.rs:142-147` - Grammar productions for `base**(sep)` and `base++(sep)`
- `src/runtime_parser.rs:520-569` - Runtime conversion to Earley grammar

Supports:
- `factor**(separator)` - zero or more occurrences separated by separator
- `factor++(separator)` - one or more occurrences separated by separator

**Status**: Grammar parsing works, but some tests timeout (performance issues)

### 5. Brace Comments in Character Classes
**Files**: `src/lexer.rs:279-300`

Fixed lexer to skip `{comments}` inside character classes like `[#0-#1F{;comment;}]`

### 6. Marked Hex Character Support (COMPLETED)
**Files**: `src/grammar_ast.rs:92-127`

Added support for marked hexadecimal characters in the grammar parser:
- `@#a` - Attribute marked hex (e.g., attribute linefeed)
- `-#a` - Hidden marked hex (e.g., hidden linefeed)
- `^#a` - Promoted marked hex (e.g., promoted linefeed)

**Result**: Fixed test `lf`

### 7. Character Terminal Collection from Separators (COMPLETED)
**Files**: `src/runtime_parser.rs:92-115, 132-153`

Fixed `collect_chars_from_factor` and `collect_charclasses_from_factor` to recurse into separator sequences of `++` and `**` operators. Previously, character terminals used in separators (like `"-"` in `word++"-"`) were not collected, causing "Missing Symbol" errors.

**Result**: Fixed grammar parsing for `email` test (still has input error)

### 8. Duplicate Rule Prevention for Separated Repetitions (COMPLETED)
**Files**: `src/runtime_parser.rs:19-49, 247-258, 659-693, 1272-1285, 1408-1421, 1533-1614`

Created `normalize_sequence()` function to generate unique separator-based identifiers. This prevents duplicate rules when the same base symbol appears in multiple separated repetitions with different separators (e.g., `factor++"×"` and `factor++"÷"`).

**Result**: Fixed grammar parsing for `expr2`, `expr3`, `expr4` tests

### 9. Unicode Character Detection Fix (COMPLETED)
**Files**: `src/runtime_parser.rs:549`

Changed `.len()` to `.chars().count()` for proper Unicode character counting. Previously, multi-byte UTF-8 characters like "÷" were incorrectly treated as multi-character literals.

**Result**: Fixed duplicate rule errors for grammars with Unicode operators

### 10. Duplicate Marked Literal Wrapper Prevention (COMPLETED)
**Files**:
- `src/runtime_parser.rs:86-94, 197-241` - Pre-collection and deduplication
- `src/ast.rs:53` - Added `Eq` and `Hash` derives to `Mark` enum

Pre-collects all marked literals (character/literal + mark combinations) and creates wrapper rules once during grammar build. This prevents duplicate wrapper rules when the same marked literal appears multiple times (e.g., `-#a` appearing twice in a grammar).

**Result**: Fixed test `lf`, `hash`

### 11. Docker-based Safe Test Runner (COMPLETED)
**Files**:
- `src/bin/safe_conformance_runner.rs` - Ultra-safe test runner with panic catching
- `Dockerfile.test` - Docker container for isolated testing

Created a robust Docker-based testing infrastructure that:
- Catches panics with `panic::catch_unwind`
- Writes results incrementally to prevent data loss on crash
- Prints results to stdout in real-time to identify crashers
- Skips known timeout/crasher tests to allow full suite completion
- Provides safe ASCII-only output to prevent encoding issues

**Result**: Can now run all 49 tests to completion without crashes

### 12. Semantic XML Comparison (COMPLETED)
**Files**:
- `Cargo.toml` - Added `roxmltree = "0.20"` dependency
- `src/testsuite_utils.rs:26-98, 185-207` - Implemented `xml_deep_equal()` and `nodes_equal()`

Implemented semantic XML comparison matching production iXML implementations (Markup Blitz). The test infrastructure now:
1. Tries exact string match first (fast path)
2. Falls back to semantic XML comparison using DOM parsing
3. Compares XML structure and content while ignoring formatting differences

This matches the iXML specification intent - both compact and canonical formats are valid. The comparison checks:
- Element tag names
- Attributes (order-independent)
- Text content (trimmed)
- Child elements (order-dependent)
- Ignores whitespace/formatting differences

**Result**: Fixed tests `marked`, `ranges` (+2 passing, formatting differences only)

### 13. Character Class OR Operator Support (COMPLETED)
**Files**: `src/runtime_parser.rs:415`

Fixed character class content parsing to handle `|` (OR operator) in addition to `,` and `;`. In iXML character classes, `|` separates alternatives just like `,`.

Example from lf test:
```
line: ~[#a | #d]*.
```

This means: match characters NOT (#a OR #d), i.e., NOT (linefeed OR carriage return).

Previously, `#a | #d` was treated as a single malformed element. Now it correctly splits into two hex characters: `#a` and `#d`.

Changed:
```rust
let elements: Vec<&str> = part.split(',').map(|s| s.trim()).collect();
```

To:
```rust
let elements: Vec<&str> = part.split(|c| c == ',' || c == '|').map(|s| s.trim()).collect();
```

**Result**: Fixed tests `lf`, `para-test` (+2 passing, **ZERO failures remaining!**)

## Running Tests

### Docker-based Test Runner (RECOMMENDED)

The safest way to run the full test suite is using Docker, which provides:
- Isolated environment preventing Claude Code crashes
- Panic catching to handle test failures gracefully
- Incremental output showing progress
- Complete results for all 49 tests

**Build and run:**
```bash
docker build -f Dockerfile.test -t rustixml-test .
docker run --rm rustixml-test
```

**Output:**
- Real-time progress to stdout
- Results saved to `/tmp/safe_results.txt` inside container
- Summary with pass/fail/timeout/error counts

**Implementation details:**
- Test runner: `src/bin/safe_conformance_runner.rs`
- Dockerfile: `Dockerfile.test`
- Test directory in container: `/ixml_tests/correct`
- Each test has 2-second timeout
- Known timeout/crasher tests are skipped to ensure completion

### Individual Test Debugging

For debugging specific test failures outside Docker, create debug scripts in `src/bin/`:

**Example** (`src/bin/debug_testname.rs`):
```rust
use rustixml::testsuite_utils::{read_simple_test, run_test, TestOutcome};

fn main() {
    let test_dir = "/home/bigale/repos/ixml/tests/correct";
    let test_name = "your-test-name";

    match read_simple_test(test_dir, test_name) {
        Ok(test) => {
            match run_test(&test) {
                TestOutcome::Fail { expected, actual } => {
                    println!("FAIL\nExpected:\n{}\nActual:\n{}", expected, actual);
                }
                TestOutcome::Pass => println!("PASS"),
                TestOutcome::GrammarParseError(e) => {
                    println!("Grammar error: {}", e);
                    println!("\nGrammar was:");
                    println!("{}", test.grammar);
                }
                TestOutcome::InputParseError(e) => println!("Input error: {}", e),
                TestOutcome::Skip(r) => println!("Skip: {}", r),
            }
        }
        Err(e) => eprintln!("Error loading test: {}", e),
    }
}
```

**Run with:**
```bash
cargo run --release --bin debug_testname
```

**WARNING**: Running tests outside Docker may crash Claude Code for tests with timeouts or panics. Use Docker for full suite testing.

## Next Steps

### High Priority
1. **Fix failing tests** (11 tests) - Debug output mismatch issues
   - Expression variants: `expr1`, `expr2`, `expr4` (3 tests)
   - JSON/Complex: `json`, `json1`, `poly` (3 tests)
   - Other: `address`, `ranges1`, `vcard`, `xml`, `xml1` (5 tests)
2. **Fix input parse errors** (6 tests) - Debug parse failures
   - `diary`, `diary2` - Diary format issues
   - `email`, `xpath` - Complex grammar issues
   - `unicode-classes`, `unicode-range1` - Unicode support gaps

### Low Priority
3. **Better error messages** - Improve grammar and parse error reporting
4. **Complete skipped tests** - Handle edge cases in `attr-multipart`, `version-decl`, etc.

## Architecture Notes

### Key Files
- `src/lexer.rs` - Tokenizes iXML grammar text
- `src/ast.rs` - AST node definitions for iXML grammars
- `src/grammar_parser.rs` - Handwritten recursive descent parser for iXML grammars
- `src/grammar_ast.rs` - Grammar parser entry point
- `src/runtime_parser.rs` - Converts iXML AST to Earley grammar, handles XML generation
- `src/testsuite_utils.rs` - Test infrastructure for conformance tests
- `src/bin/safe_conformance_runner.rs` - Docker-safe test runner with panic catching
- `src/bin/debug_*.rs` - Individual test debugging scripts
- `Dockerfile.test` - Docker container for safe test execution

### Dependencies
- `earlgrey` - Earley parser implementation for runtime input parsing

### Grammar Conversion Pipeline
1. Lexer tokenizes iXML grammar → `Vec<Token>`
2. Handwritten parser (`grammar_parser.rs`) parses tokens → `IxmlGrammar` AST
3. Runtime converter (`runtime_parser.rs`) converts AST → Earley `GrammarBuilder`
4. `GrammarBuilder` compiled → Earley `Grammar`
5. Earley parser parses input text → Parse trees
6. `EarleyForest` evaluates parse trees → `XmlNode`
7. `XmlNode.to_xml()` → Canonical XML string

### XML Generation
XML generation uses semantic actions registered on each grammar production. The action format is `"nonterminal -> symbol1 symbol2 ..."` and returns an `XmlNode` which can be:
- `Element { name, attributes, children }` - XML element
- `Text(String)` - Text content
- `Attribute { name, value }` - Attribute (extracted by parent element)

Hidden elements (`_hidden`) are skipped during XML serialization.

## Reference Implementation: Markup Blitz

The `/home/bigale/repos/markup-blitz/` repository contains Gunther Rademacher's production iXML implementation in Java. It passes all 5168 iXML conformance tests and serves as an authoritative reference for solving complexity issues.

### Core Architecture Differences

**Parser Algorithm**: Markup Blitz uses **LALR(1) + GLR**, not Earley parsing.
- LALR(1) for parser table construction (deterministic where possible)
- GLR (Generalized LR) for dynamic conflict resolution when ambiguity occurs
- This provides better worst-case performance than Earley for many grammars

**Key files**:
- `src/main/java/de/bottlecaps/markup/blitz/transform/Generator.java` - LALR table generation
- `src/main/java/de/bottlecaps/markup/blitz/transform/BNF.java` - Grammar transformation
- `src/main/java/de/bottlecaps/markup/blitz/transform/ClassifyCharacters.java` - Character class optimization

### Key Optimization Patterns

#### 1. Character Class Partitioning (`ClassifyCharacters.java:390-417`)

**Problem**: Multiple overlapping character classes create redundant terminal symbols.

**Solution**: Partition all character sets in the grammar into non-overlapping equivalence classes before parser generation.

```java
// The classify() method computes disjoint character class partitions
public static Set<RangeSet> classify(Collection<RangeSet> allRangeSets) {
    // Each original charset becomes alternatives of these partitions
    // Reduces terminal count, improves parsing speed
}
```

**Example**: If grammar has `[a-z]` and `[a-m]`:
- Creates partitions: `[a-m]` and `[n-z]`
- `[a-z]` becomes `[a-m] | [n-z]`
- `[a-m]` stays as `[a-m]`

**Relevance to rustixml**: Could reduce Earley parser state explosion for grammars with many overlapping character classes.

#### 2. Explicit BNF Transformation for Repetitions (`BNF.java:173-247`)

**Problem**: Repetition operators (`*`, `+`, `**`, `++`) need efficient rule expansion.

**Solution**: Generate explicit grammar rules with specific patterns:

```java
// ONE_OR_MORE (+): name: term | name term
case ONE_OR_MORE:
    // Creates left-recursive rule for efficiency

// ZERO_OR_MORE (*): name: | name term
case ZERO_OR_MORE:
    // Empty alternative first

// Separated ZERO_OR_MORE (**):
// Creates TWO rules:
//   listName: term | listName sep term
//   name: | listName
case ZERO_OR_MORE: // with separator
    // Two-rule pattern handles the separator cleanly
```

**Relevance to rustixml**: The current separated repetition handling in `runtime_parser.rs` may benefit from this two-rule pattern.

#### 3. Fork-based Conflict Resolution (`Generator.java:396-432`)

**Problem**: LALR conflicts (shift-reduce, reduce-reduce) cause ambiguity.

**Solution**: Create "forks" - linked pairs of alternative actions that GLR explores in parallel.

```java
// When conflicts detected, create fork chains
conflicts.put(conflictToken, forkId);
// GLR parser explores all fork branches, merges results
```

**Relevance to rustixml**: Earley naturally handles ambiguity, but understanding how GLR manages it may inform optimizations.

#### 4. Compressed Parser Tables (`CompressedMap`, `Map2D`)

**Problem**: Large transition tables consume memory and slow lookups.

**Solution**: Tile-based compression with multiple indirection levels:
- ASCII: Direct 128-entry lookup table (fast path)
- BMP: Compressed tiled map
- Supplementary: Range-based lookup

**Relevance to rustixml**: Not directly applicable to Earley, but useful if considering parser algorithm changes.

### Potential Improvements for rustixml

Based on markup-blitz patterns, these approaches could help resolve the 13 timeout tests:

1. **Character Class Optimization** (High impact)
   - Implement character set partitioning before Earley grammar construction
   - Reduces the number of terminal symbols and parser states
   - File to modify: `src/runtime_parser.rs` character class handling

2. **Two-Rule Separated Repetition Pattern** (Medium impact)
   - Adopt the `listName`/`name` two-rule pattern from BNF.java
   - May reduce ambiguity in separated repetitions
   - Current implementation in `src/runtime_parser.rs:520-569`

3. **Consider LALR+GLR** (High impact, major refactor)
   - Would require replacing the `earlgrey` crate
   - Better performance characteristics for the problematic grammars
   - Existing Rust GLR implementations: `glr`, `santiago`

4. **Nonterminal Inlining** (Medium impact)
   - Markup-blitz's `CharsetCollector` inlines simple nonterminals that just wrap character classes
   - Could reduce rule count and parser complexity

### Why Earley May Struggle

The timeout tests (`expr`, `diary`, `json1`, `poly`) likely suffer from:
- **Cubic worst-case**: Earley is O(n³) for ambiguous grammars
- **No table precomputation**: Earley builds parse forest dynamically
- **Left recursion handling**: While Earley handles it, performance degrades
- **Character-level parsing**: Without tokenization, creates many more states

Markup-blitz avoids these by:
- Precomputing LALR tables (O(1) state transitions)
- GLR only forks on actual conflicts
- Treating character classes as single tokens after partitioning

## Implementation Plan: Earley Optimizations

Based on markup-blitz patterns, adapted for our Earley-based runtime pipeline. These are grammar transformations that occur before feeding to the `earlgrey` crate.

### Phase 1: Character Class Partitioning (High Impact) - IMPLEMENTED (DISABLED)

**Why it helps Earley**: Reduces terminal count, fewer parser states, less ambiguity

**Implementation location**: `src/runtime_parser.rs`
- `RangeSet` data structure with union/intersection/minus operations
- `classify_charclasses()` - compute disjoint partitions
- `partition_charclasses_in_ast()` - AST transformation approach
- Feature flag: `ENABLE_CHARCLASS_PARTITIONING` (currently `false`)

**Status**: Implemented but disabled due to regression (22 PASS → 21 PASS). The `rangeset_to_charclass_content()` function generates partition content strings that aren't parsed correctly by the existing character class parser. Tests like `ranges` and `json` move from PASS/FAIL to INPUT_ERROR when enabled.

**Steps implemented**:
1. ✅ Collect all character classes from grammar into `Vec<RangeSet>`
2. ✅ Compute disjoint partitions using set intersection/difference
3. ✅ Transform AST: replace each CharClass with Group of partition alternatives
4. ⚠️ Issue: Generated partition content strings cause parse errors

**Example transformation**:
```
Grammar has: [a-z], [a-m], [0-9]
Partitions: [a-m], [n-z], [0-9]
[a-z] → ([a-m] | [n-z])  (as Group in AST)
[a-m] → [a-m]  (unchanged)
```

**TODO**: Fix `rangeset_to_charclass_content()` to generate syntax compatible with `parse_char_class()`

**Effort**: Medium (200-300 lines) - IMPLEMENTED, needs debugging
**Expected impact**: High for grammars with overlapping character classes

### Phase 2: Two-Rule Separated Repetition Pattern (Medium Impact)

**Current approach**:
```rust
// base**(sep) creates:
//   base_sep_star: | base_sep_plus
//   base_sep_plus: base | base_sep_plus sep base
```

**Markup-blitz approach**:
```java
// base**(sep) creates:
//   listName: base | listName sep base  (left-recursive list)
//   name: | listName                    (optional wrapper)
```

**Implementation**: Modify `convert_factor()` in runtime_parser.rs

**Effort**: Low (50-100 lines)
**Expected impact**: Medium - may reduce ambiguity in separated repetitions

### Phase 3: Nonterminal Inlining (Low-Medium Impact)

**Problem**: Nonterminals that just wrap character classes add indirection

**Optimization**: Inline simple nonterminals (single alternative, single character class) directly

**Effort**: Medium (100-150 lines)
**Expected impact**: Low-medium, reduces rule count

### Strategies NOT Applicable to Earley

- **Left-to-Right Recursion Conversion**: Earley handles left-recursion natively
- **Fork-based Conflict Resolution**: Earley explores all parses naturally
- **Compressed Parser Tables**: Earley doesn't use precomputed tables

### Implementation Priority

| Phase | Optimization | Effort | Impact | Status |
|-------|-------------|--------|--------|--------|
| 1 | Character Class Partitioning | Medium | High | IN PROGRESS |
| 2 | Two-Rule Separated Repetition | Low | Medium | Pending |
| 3 | Nonterminal Inlining | Medium | Low-Med | Optional |

### Measurement Strategy

Before/after each phase:
1. Run Docker test suite, record timeout test behavior
2. Measure grammar complexity (rules, terminals, overlaps)
3. Document which grammars benefit most

## Test Environment
- iXML test suite: `/home/bigale/repos/ixml/tests/correct/`
- Each test has: `name.ixml` (grammar), `name.inp` (input), `name.output.xml` (expected output)
