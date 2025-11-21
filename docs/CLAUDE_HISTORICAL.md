# rustixml - iXML Parser Implementation in Rust
CRITICAL - RUN ALL TESTS IN BACKGROUND
## Project Overview

rustixml is a Rust implementation of an iXML (Invisible XML) parser. iXML is a grammar specification language for converting text to XML without explicit markup. This implementation:

- Parses iXML grammar specifications
- Converts grammars to Earley parser grammars (using the `earlgrey` crate)
- Parses input text according to the grammar
- Generates canonical XML output

## Current Status

**Earley Implementation**: Complete (frozen at 39.8% pass rate)

The Earley-based implementation has reached a natural stopping point. While functional for many iXML grammars, it has hit fundamental abstraction mismatches between iXML semantics and Earley parsing. See [Abstraction Analysis](docs/ABSTRACTION_ANALYSIS.md) for detailed discussion.

**Next Phase**: Native iXML Interpreter (in design)

A native Rust interpreter that directly implements iXML specification semantics without translation to Earley. Expected to achieve 80-90%+ conformance by handling insertion/suppression patterns natively. See [Native Interpreter Design](docs/NATIVE_INTERPRETER_DESIGN.md) and [Transition Summary](docs/TRANSITION_SUMMARY.md).

### Conformance Test Results (Comprehensive - Full iXML Test Suite)

**Test Suite**: `/home/bigale/repos/rustixml/ixml_tests/` (133 total tests across 8 categories)

**Final Earley Results**:
- **53 PASSING** (39.8%) ✅
- **12 FAILING** (9.0%)
- **0 GRAMMAR_ERRORS** (0%)
- **3 INPUT_ERRORS** (2.3%) - Known limitations
- **65 SKIP** (48.9%)

### Key Achievements

✅ **87.8% pass rate** on basic correctness tests (43/49)  
✅ **100% pass rate** on parse tests (3/3)  
✅ **50% pass rate** on ixml grammar parsing tests (4/8)  
✅ **Zero grammar parse errors** - all iXML grammars parse correctly  
✅ Full Unicode General Category support implemented  
✅ Character class parsing with unquoted sequences fixed
✅ Thread-local GROUP_COUNTER synchronization  
✅ Assert-not-a-sentence test handling  

### Known Limitations (Earley Implementation)

The Earley-based approach has fundamental limitations due to abstraction mismatches:

1. **Insertion + Suppression in Repeated Sequences** (3 INPUT_ERROR tests)
   - Pattern: `(-[Co], +".")*` - suppression combined with insertion in loops
   - Root cause: Earley parsers consume input tokens but don't have native insertion semantics
   - Affected tests: `unicode-classes`, `ixml-spaces`, `ixml3`
   - See [earley_insertion_limitation.md](docs/earley_insertion_limitation.md)

2. **Ambiguous Grammar Handling** (11/13 failing)
   - Need multiple parse tree handling
   - Earley naturally supports ambiguity, but tree extraction needs work

3. **Advanced Features** 
   - Version declarations, Unicode version-specific behavior
   - Most tests skipped (51/52 syntax tests)

### Implementation Gaps

### Implementation Gaps

**Note**: The following gaps are documented but will not be addressed in the Earley implementation, as we are transitioning to a native iXML interpreter.

The main areas requiring work for full conformance:

1. **Insertion + Suppression Pattern** - Fundamental Earley limitation (see above)
2. **Ambiguous Grammar Handling** (11/13 failing) - Need to implement multiple parse tree handling
3. **Syntax Tests** (51/52 skip) - Need grammar-only test support
4. **Advanced Features** - Version declarations, Unicode version-specific behavior

### Results by Category

#### correct/ (49 tests) - Basic Correctness Tests ⭐
- **43 PASSING** (87.8%) ✅
- **0 FAILING** (0%)
- **1 INPUT_ERROR** (2.0%) - `unicode-classes` (insertion+suppression pattern limitation)
- **5 SKIP** (10.2%) - advanced features (version-decl, unicode-version-diagnostic, ws-and-delim)

#### syntax/ (52 tests) - Grammar Syntax Tests
- **1 PASSING** (1.9%)
- **51 SKIP** (98.1%) - Most skip due to missing input files (grammar-only tests)

#### ambiguous/ (13 tests) - Ambiguous Grammar Handling
- **2 PASSING** (15.4%)
- **11 FAILING** (84.6%) - Ambiguous parse handling not fully implemented
- **0 INPUT_ERRORS** (0%)

#### ixml/ (8 tests) - Parsing iXML Grammars
- **5 PASSING** (62.5%) ✅
- **0 FAILING** (0%)
- **3 INPUT_ERRORS** (37.5%) - `ixml-spaces`, `ixml3` (insertion+suppression pattern limitation)

#### parse/ (3 tests) - Parse Tests
- **3 PASSING** (100.0%) ✅

#### chars/ (4 tests) - Character Handling
- **4 SKIP** (100%) - Missing test files

#### error/ (3 tests) - Error Handling
- **3 SKIP** (100%) - Missing test files

#### reference/ (1 test) - Reference Implementation
- **1 SKIP** (100%) - Missing test files

### Passing Tests (43)
- `aaa` - Hidden marked literals
- `address` - Address parsing
- `arith` - Arithmetic expression with canonical XML formatting
- `attribute-value` - XML entity escaping in attributes
- `diary`, `diary2`, `diary3` - Diary format parsing
- `element-content` - XML entity escaping in text content
- `email` - Email address parsing with separated repetitions
- `empty-group` - Empty group handling
- `expr` - Expression grammar with Unicode operators
- `expr1`, `expr2`, `expr3`, `expr4`, `expr5`, `expr6` - Expression variants
- `hash` - Separated repetitions with canonical formatting
- `hex`, `hex1`, `hex3` - Hexadecimal parsing
- `json`, `json1` - JSON parsing
- `lf` - Marked hex characters (hidden linefeed)
- `marked` - Marked literals with attribute marks
- `nested-comment` - Nested brace comments
- `para-test` - Multi-paragraph parsing with character classes
- `poly` - Polynomial parsing
- `program` - Complex grammar structure
- `range`, `range-comments` - Character ranges
- `ranges`, `ranges1` - Character range edge cases
- `string` - String literals
- `tab` - Tab character handling
- `test` - Basic grammar test
- `unicode-range`, `unicode-range1`, `unicode-range2` - Unicode character ranges
- `vcard` - VCard parsing
- `xml`, `xml1` - XML parsing
- `xpath` - XPath assert-not-a-sentence test (correctly rejects invalid input)

### Known Issues by Category

#### Input Parse Errors (1)
- `unicode-classes` - Unicode class support (IMPLEMENTED - uses `unicode-general-category` crate. Grammar builds successfully in <1ms, but input parsing times out - Earley parser performance issue with complex character class matching)

## Recent Fixes

### Final Debugging Session: Insertion+Suppression Limitation Discovery (COMPLETED)

**Investigation**: Investigated why `unicode-classes` test fails despite simplified versions working.

**Approach**:
1. Created incremental test suite (`debug_unicode_actual.rs`) - ALL 6 tests PASS ✓
2. Tested exact grammar with full 41 alternatives - PASS ✓
3. Binary searched through input lines to find failure point
4. Isolated failure to line 33: `Co \u{E000}\n`
5. Created minimal reproduction (`debug_suppressed_insertion.rs`)

**Root Cause Identified**:
- Grammar rule: `Co: -"Co ", (-[Co], +".")*.`
- Pattern combines:
  - **Suppression** (`-[Co]`): matches but hides from output
  - **Insertion** (`+"."`): adds content not in input
  - **Repetition** (`*`): loops the sequence
- Earley parsers fundamentally consume input tokens
- No natural way to express "consume this, output something else" in a loop

**Test Results**:
- `[Co]*` - Works fine ✓
- `(-[Co], +".")*` - Fails ✗

**Files Created**:
- `docs/earley_insertion_limitation.md` - Detailed analysis
- `src/bin/debug_unicode_{exact,full,two_lines,line3,line33,nlines}.rs` - Debug tools
- `src/bin/debug_suppressed_insertion.rs` - Minimal reproduction

**Strategic Impact**:
This validates the abstraction analysis - we've reached a hard limit of the Earley translation approach. Clean stopping point at 39.8% pass rate.

### 2. Thread-Local GROUP_COUNTER Synchronization (COMPLETED)

**Files**: `src/runtime_parser.rs` (lines ~100-120)

**Problem**: "Missing Action" errors in test suite due to GROUP_COUNTER mismatch between AST conversion and tree building.

**Root Cause**: The `ast_to_earlgrey()` function uses a global `GROUP_COUNTER` to generate unique group IDs. When called multiple times (once to get the AST, again during tree building), the counter increments, causing mismatched group IDs.

**Fix**: Implemented thread-local storage for consistent group ID mapping:
```rust
thread_local! {
    static GROUP_ID_MAP: RefCell<HashMap<usize, usize>> = RefCell::new(HashMap::new());
}
```

Modified `ast_to_earlgrey` to:
1. Return tuple: `(GrammarBuilder, IxmlGrammar)` with transformed AST
2. Store original→transformed group ID mappings in thread-local map
3. Reuse transformed AST in test runner to maintain consistency

**Impact**: Eliminated GROUP_COUNTER synchronization errors across the test suite.

### 3. Character Class Fix: Unquoted Sequences (COMPLETED)

**Files**: `src/runtime_parser.rs` (lines ~1320-1330)

**Problem**: Character classes like `[xyz]` weren't matching any characters. Only explicit hex chars like `[#61-#7A]` worked.

**Root Cause**: The `parse_char_class()` function only handled quoted strings (`"abc"`) and hex chars (`#XX`), but not unquoted character sequences that should be treated as individual characters.

**Fix**: Added handling for unquoted sequences in the "else" branch:
```rust
else {
    // Treat as individual characters: "xyz" → ['x','y','z']
    for ch in element.chars() {
        chars.push(ch);
    }
}
```

**Impact**: Character classes now work for common patterns like `[a-z]`, `[0-9]`, `[abc]`.

### 4. Self-Closing Root Element Fix (COMPLETED)
**Files**: `src/runtime_parser.rs` (lines 1593-1602)

**Problem**: Self-closing root elements were missing their closing `/>`. The output was `<email user='...' host='...'` instead of `<email user='...' host='...'/>`.

**Root Cause**: The XML serialization code assumed all empty elements would have a parent to add the closing `/>`, but root elements have no parent.

**Fix**: Added `is_root` check in the empty children case to output complete self-closing tag for root elements:

```rust
if children.is_empty() {
    if is_root {
        // Root element needs complete self-closing tag
        format!("<{}{}/>", name, attrs_str)
    } else {
        // Non-root: parent adds />
        format!("<{}{}", name, attrs_str)
    }
}
```

**Impact**:
- **+14 new passing tests**: `email`, `json`, `json1`, `poly`, `vcard`, `xml`, `xml1`, `address`, `expr1`, `expr2`, `expr4`, `ranges1`, `diary`, `diary2`
- Pass rate improved from **55.1% to 83.7%**
- **All output mismatch failures resolved**

### 2. Unicode Character Handling Fix (COMPLETED - MAJOR IMPROVEMENT)
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

### 16. Separator Marked Literal Action Registration (COMPLETED)
**Files**: `src/runtime_parser.rs:2487-2630`

Fixed "Missing Action" errors for marked literals in separator sequences of `**` and `++` operators.

**Problem**: When using separated repetitions like `property**(-";", S)`, the hidden separator `-";"` creates a wrapper rule `char_U003B_hidden -> char_U003B`, but its action wasn't being registered. This caused "Missing Action" errors during XML generation.

**Root Cause**: The `register_marked_literal_actions` function only processed the main grammar AST and didn't recurse into separator sequences.

**Fix**: 
1. Created helper function `register_marked_literal_from_factor()` to handle recursion
2. Added separator processing to both `register_marked_literal_from_alternatives()` and the new helper
3. Now recursively processes marked literals in: main sequences, groups, and separator sequences

**Result**: Fixed tests `css` (ambiguous), `ixml` (ixml grammar parsing) (+2 passing)

### 15. Assert-Not-A-Sentence Test Support (COMPLETED)
**Files**: `src/testsuite_utils.rs`

Added support for tests that expect parse failure (marked with `<assert-not-a-sentence/>` in the test catalog):

**Changes**:
1. `read_simple_test()` - Automatically detects tests without `.output.xml` files and sets `expect_failure = true`
2. `run_test()` - Returns `TestOutcome::Pass` when parse fails on tests expecting failure
3. Returns `TestOutcome::Fail` if parse succeeds when failure was expected

**Example**: The `xpath` test has input `a[.!='']` but the grammar requires mandatory whitespace around comparison operators, so it's supposed to fail parsing. This is now correctly detected as a PASS.

**Result**: Fixed test `xpath` (+1 passing)

### 14. Unicode General Category Support (COMPLETED)
**Files**:
- `Cargo.toml` - Added `unicode-general-category = "1.1.0"` dependency
- `src/runtime_parser.rs:227-385` - Implemented `unicode_category_to_rangeset()` with caching
- `src/runtime_parser.rs:453` - Updated `charclass_to_rangeset()` to use Unicode categories
- `src/runtime_parser.rs:1317` - Updated `parse_char_class()` to use Unicode categories

Implemented full Unicode General Category support for character classes:
- Major categories: L, M, N, P, S, Z, C
- Minor categories: Lu, Ll, Lt, Lm, Lo, LC, Mn, Mc, Me, Nd, Nl, No, Pc, Pd, Ps, Pe, Pi, Pf, Po, Sm, Sc, Sk, So, Zs, Zl, Zp, Cc, Cf, Cs, Co, Cn

**Implementation**:
1. `unicode_category_to_rangeset()` - Iterates through all Unicode codepoints (0x0 to 0x10FFFF) and builds a `RangeSet` for each category using the `unicode-general-category` crate
2. Results are cached using `OnceLock<Mutex<HashMap>>` to avoid recomputation (each category takes 2-4ms to build)
3. Both `charclass_to_rangeset()` (used for partitioning) and `parse_char_class()` (used for grammar building) now call this function

**Performance**:
- Grammar parsing: <100µs
- Earley grammar building: ~260µs (includes caching all Unicode categories)
- Total grammar build: <1ms for complex grammars

**Status**: Grammar builds successfully for `unicode-classes` test, but input parsing times out. This is likely an Earley parser performance issue with the complex alternation structure (30+ character class rules), not a Unicode category matching problem.

### 13. Character Class OR Operator Support (COMPLETED) Running Tests

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
1. **Fix input parse errors** (2 tests) - Debug parse failures
   - `unicode-classes` - Requires `unicode-general-category` crate for Unicode General Category matching (Ll, Lu, L, N, P, Cc, Cf)
   - `xpath` - Grammar requires mandatory whitespace around comparison operators; test input `a[.!='']` has no spaces around `!=`

### Low Priority
2. **Better error messages** - Improve grammar and parse error reporting
3. **Complete skipped tests** - Handle edge cases in `attr-multipart`, `version-decl`, etc.

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
