# Quote Handling Design Principles for rustixml

## iXML Spec Compliance

Based on the official [iXML 1.0 specification](https://invisiblexml.org/1.0/), this document outlines our quote handling strategy.

## Core Principle: Quote Interchangeability

**From the spec:** "An optionally marked string of one or more characters, enclosed with single or double quotes."

**Key rule:** Single (`'`) and double (`"`) quotes are **completely interchangeable** with no semantic difference.

### Examples from the spec:
- `'Isn''t it?'` ≡ `"Isn't it?"`  (escaping different quote type)
- `"He said ""Don't!"""` ≡ `'He said "Don''t!"'`  (nested quotes)

## Implementation Strategy

### 1. ✅ Literal Strings (COMPLETE)
**Location:** `src/lexer.rs:212-242`

Both quote styles already handled:
```rust
fn read_string(&mut self) -> Result<Token, String>      // Handles "text"
fn read_char_literal(&mut self) -> Result<Token, String> // Handles 'text'
```
✅ Quotes stripped at lexer level, both produce `Token::String(content)`

### 2. ✅ Character Classes (COMPLETE)
**Location:** `src/runtime_parser.rs`

**Pattern implemented:**
```rust
/// Helper function to normalize character class content
fn normalize_charclass_content(content: &str) -> String {
    content.replace("-", "_").replace("'", "").replace("\"", "").replace(" ", "")
}
```

**Used in 8 locations:**
- Terminal naming (lines 46, 48)
- Repetition collection (lines 165, 167)
- Factor conversion (lines 412, 414)
- Symbol list building (lines 840, 842)
- Factor symbol extraction (lines 884, 886)

**Character class parsing** (lines 270-286):
```rust
// Supports BOTH quote styles:
if part.contains('-') && (part.contains('\'') || part.contains('"'))
    let start = range_parts[0].trim().trim_matches('\'').trim_matches('"')
    let end = range_parts[1].trim().trim_matches('\'').trim_matches('"')
```

**Test coverage:**
- `mixed-quotes-single` - ['a'-'z'] ✅
- `mixed-quotes-double` - ["a"-"z"] ✅
- `mixed-quotes-mixed` - ['a"-"z'] ✅

### 3. 🔜 Future Features Requiring Quote Handling

#### A. Hex-Encoded Characters (Not yet implemented)
**From spec:** `#9`, `#a`, `#d`, `#a0`, etc.

Example usage in character classes:
```ixml
digit: [#30-#39].     {# Hex range for 0-9 #}
nbsp: #a0.            {# Non-breaking space #}
tab: #9.              {# Tab character #}
```

**Recommendation:** When implementing, ensure hex codes work with both quote styles:
```ixml
special: [#20-#7e].   {# No quotes - direct hex #}
special: ["#20"-"#7e"].  {# Quoted hex - should also work #}
```

#### B. Unicode Category Classes (Partially implemented)
**From spec:** `[Ll]`, `[Lu]`, `[Nd]`, `[Ll; Lu]`

Currently parsed but not fully tested with mixed quotes:
```ixml
letter: [Ll; Lu].     {# Any letter, upper or lower #}
digit: [Nd].          {# Any numeric digit #}
```

**Action needed:** Add tests for Unicode categories with different quote combinations.

#### C. Escaped Quotes in Literals (Not yet implemented)
**From spec:** Quote doubling for escape: `''` → `'`, `""` → `"`

Examples:
```ixml
single-quote: "'".    {# Would need '' in single-quoted literal #}
quote-test: "He said ""Hello""".  {# Embedded quotes #}
```

**Current status:** Lexer doesn't handle quote doubling yet.

#### D. Character Ranges with Mixed Notation (Future enhancement)
Combining quoted chars and hex codes:
```ixml
printable: ['!'-'~'].        {# ASCII printable, quoted #}
printable: [#21-#7e].        {# ASCII printable, hex #}
printable: ['!'-#7e].        {# Mixed - future feature? #}
```

## Design Patterns Established

### Pattern 1: Dual-Quote Stripping
```rust
// ❌ OLD: Only single quotes
content.replace("'", "")

// ✅ NEW: Both quote types
content.replace("'", "").replace("\"", "")

// ✅ BEST: Use helper function
normalize_charclass_content(&content)
```

### Pattern 2: Conditional Parsing
```rust
// ❌ OLD: Only check one quote type
if part.contains('\'') { ... }

// ✅ NEW: Check both quote types
if part.contains('\'') || part.contains('"') { ... }
```

### Pattern 3: Sequential Trimming
```rust
// ✅ CORRECT: Chain both trim_matches
let ch = part.trim_matches('\'').trim_matches('"').chars().next()

// ❌ WRONG: Only one trim
let ch = part.trim_matches('\'').chars().next()
```

## Testing Strategy

### Current Test Coverage (16/18 = 88.9%)
1. ✅ Literal strings with both quote types
2. ✅ Character classes with single quotes: `['a'-'z']`
3. ✅ Character classes with double quotes: `["a"-"z"]`
4. ✅ Character classes with mixed quotes: `['a"-"z']`
5. ✅ Repetition operators with character classes

### Recommended Future Tests
1. **Escaped quotes in literals:**
   ```ixml
   test: "Don''t".    {# Output: Don't #}
   test: 'Can''t'.    {# Output: Can't #}
   ```

2. **Hex codes with quotes:**
   ```ixml
   tab: "#9".         {# Quoted hex code #}
   tab: '#9'.         {# Single-quoted hex #}
   ```

3. **Unicode categories:**
   ```ixml
   letter: ["L"; "M"].   {# Double-quoted categories #}
   letter: ['L'; 'M'].   {# Single-quoted categories #}
   ```

4. **Complex character class combinations:**
   ```ixml
   alphanum: ["a"-"z"; "A"-"Z"; "0"-"9"; "_"].
   alphanum: ['a'-'z'; 'A'-'Z'; '0'-'9'; '_'].
   alphanum: ['a"-"z"; 'A"-'Z"; "0"-'9'; '_'].  {# Maximum mixing #}
   ```

## Implementation Checklist

### ✅ Completed
- [x] Lexer handles both quote types for literals
- [x] Character class parsing supports both quotes
- [x] Terminal naming strips both quote types
- [x] Helper function `normalize_charclass_content()` created
- [x] All 8 usage locations refactored
- [x] Mixed quote tests added and passing

### 🔜 Future Work
- [ ] Implement quote doubling for escaped quotes (`''` → `'`)
- [ ] Add hex character support (`#9`, `#a0`, etc.)
- [ ] Test hex codes with quoted ranges
- [ ] Comprehensive Unicode category testing
- [ ] Add spec error handling (S07-S11)
- [ ] Document quote handling in user-facing docs

## Maintenance Guidelines

**When adding new features that parse quoted content:**

1. ✅ **Always use the helper function** when stripping quotes for identifiers
2. ✅ **Check both quote types** when parsing content
3. ✅ **Chain trim_matches** for both `'` and `"` when extracting values
4. ✅ **Add tests** for both single and double quote variants
5. ✅ **Document** any quote-related edge cases in code comments

**Code review checklist:**
- [ ] Does this code parse quoted strings or character classes?
- [ ] Are both `'` and `"` handled equally?
- [ ] Is `normalize_charclass_content()` used where appropriate?
- [ ] Are there tests for both quote styles?

## References

- [iXML 1.0 Specification](https://invisiblexml.org/1.0/)
- Implementation: `src/runtime_parser.rs`
- Tests: `test_runner.rs`, `/tmp/ixml-tests/mixed-quotes-*`
- Related commits:
  - `1e74fd6` - Support double-quoted character classes
  - (Next) - Refactor to use helper function
