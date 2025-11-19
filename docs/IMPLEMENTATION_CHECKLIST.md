# Native Interpreter Implementation Checklist

Quick reference for implementing the native iXML interpreter. See `NATIVE_INTERPRETER_DESIGN.md` for full details.

## Phase 1: Core Infrastructure ⏱️ 2-3 hours

### Files to Create

- [ ] `src/input_stream.rs` (~100 LOC)
  - [ ] `InputStream` struct with char vector
  - [ ] `new(input: &str)` constructor
  - [ ] `current()` - get current character
  - [ ] `advance()` - move forward one character
  - [ ] `peek(offset)` - look ahead
  - [ ] `position()` / `set_position()` - backtracking support
  - [ ] `is_eof()` - end of input check
  - [ ] Unit tests for all operations

- [ ] `src/parse_context.rs` (~50 LOC)
  - [ ] `ParseContext` struct with rule tracking
  - [ ] `ParseResult` struct with node + consumed count
  - [ ] Helper methods for context management

- [ ] `src/native_parser.rs` (~100 LOC skeleton)
  - [ ] `NativeParser` struct with grammar + rules HashMap
  - [ ] `new(grammar)` constructor
  - [ ] `parse(input)` main entry point
  - [ ] Method stubs for all parse functions

### Tests

```bash
cargo test --lib input_stream
cargo test --lib parse_context
```

## Phase 2: Basic Terminals ⏱️ 3-4 hours

### Implement in `native_parser.rs`

- [ ] `parse_terminal()` (~40 LOC)
  - [ ] Character-by-character matching
  - [ ] Insertion handling (consume 0 chars)
  - [ ] Mark application (hidden/attribute/promoted)
  - [ ] Backtracking on mismatch

- [ ] `parse_charclass()` (~60 LOC)
  - [ ] Single character matching
  - [ ] Reuse existing `charclass_matches()` logic
  - [ ] Negation support (`~[...]`)
  - [ ] Mark application

- [ ] `parse_nonterminal()` (~80 LOC)
  - [ ] Rule lookup from HashMap
  - [ ] Left-recursion detection
  - [ ] Recursive call to `parse_rule()`
  - [ ] Mark application to result

### Tests

Create `src/bin/native_test_*.rs` debug scripts:

```bash
# Should pass:
cargo run --release --bin native_test_basic
cargo run --release --bin native_test_aaa
cargo run --release --bin native_test_string
```

Target tests: `test`, `aaa`, `string`

## Phase 3: Sequences & Alternatives ⏱️ 2-3 hours

### Implement in `native_parser.rs`

- [ ] `parse_sequence()` (~50 LOC)
  - [ ] Loop through factors
  - [ ] Collect non-suppressed nodes
  - [ ] Backtrack on any failure
  - [ ] Return sequence node with consumed count

- [ ] `parse_alternatives()` (~50 LOC)
  - [ ] Try each alternative in order
  - [ ] Reset position before each try
  - [ ] Return first success (PEG-style)
  - [ ] Collect errors for diagnostics

- [ ] `parse_rule()` (~30 LOC)
  - [ ] Look up rule
  - [ ] Call `parse_alternatives()`
  - [ ] Apply rule-level mark

### Tests

```bash
# Should pass:
cargo run --release --bin native_test_address
cargo run --release --bin native_test_expr
```

Target tests: `address`, `expr` (simple alternatives)

## Phase 4: Repetitions ⏱️ 4-5 hours

### Implement in `native_parser.rs`

- [ ] `parse_repetition()` (~150 LOC)
  - [ ] `Repetition::ZeroOrMore` (greedy loop)
  - [ ] `Repetition::OneOrMore` (require first match)
  - [ ] `Repetition::Optional` (try once)
  - [ ] Epsilon-match detection (prevent infinite loops)
  
- [ ] `parse_separated_repetition()` (~80 LOC)
  - [ ] `SeparatedZeroOrMore` - list with separator
  - [ ] `SeparatedOneOrMore` - non-empty list
  - [ ] Parse element, then loop (separator, element)

### Tests

```bash
# Should pass:
cargo run --release --bin native_test_email
cargo run --release --bin native_test_hash
cargo run --release --bin native_test_para
```

Target tests: `email`, `hash`, `para-test`

## Phase 5: Marks & Insertions ⏱️ 3-4 hours

### Implement in `native_parser.rs`

- [ ] `apply_mark()` helper (~40 LOC)
  - [ ] `Mark::None` - wrap in element
  - [ ] `Mark::Hidden` - return None (suppress)
  - [ ] `Mark::Attribute` - convert to attribute node
  - [ ] `Mark::Promoted` - unwrap element

- [ ] Insertion handling in `parse_terminal()` (already done in Phase 2)
  - [ ] Verify consume=0 behavior
  - [ ] Test in loops

- [ ] Attribute extraction (~30 LOC)
  - [ ] Collect attribute nodes from children
  - [ ] Move to parent element's attribute list

### Critical Test

```bash
# The test that broke Earley:
cargo run --release --bin native_test_unicode_classes
```

**Success criteria**: Line 33 (`Co \u{E000}\n`) with pattern `(-[Co], +".")*` parses correctly.

Expected output: `<Co>.</Co>` (suppressed Co character, inserted ".")

### Other Tests

```bash
cargo run --release --bin native_test_marked
cargo run --release --bin native_test_lf
```

## Phase 6: Full Test Suite ⏱️ 5-8 hours

### Integration

- [ ] Create `src/bin/native_conformance_runner.rs`
  - [ ] Copy test runner structure from `comprehensive_test_runner.rs`
  - [ ] Use `NativeParser` instead of Earley
  - [ ] Run all 133 tests

- [ ] Error handling improvements
  - [ ] Better error messages with context
  - [ ] Line/column information
  - [ ] Suggestions for common mistakes

- [ ] Edge cases
  - [ ] Empty sequences
  - [ ] Empty alternatives
  - [ ] Deeply nested grammars
  - [ ] Large character classes

### Performance

- [ ] Profile hot paths
  - [ ] Character matching
  - [ ] Backtracking
  - [ ] Node construction

- [ ] Optimize if needed
  - [ ] Memoization for expensive rules
  - [ ] Character class caching
  - [ ] String interning for common nodes

### Target Results

**Goal**: 80-90% pass rate (106-119 of 133 tests)

Expected improvements over Earley:
- ✅ `unicode-classes` - insertion+suppression pattern
- ✅ `ixml-spaces` - similar pattern
- ✅ `ixml3` - similar pattern
- ✅ Additional ambiguous tests (better error handling)

## Verification Steps

After each phase:

1. **Unit tests pass**
   ```bash
   cargo test --lib
   ```

2. **Target tests pass**
   ```bash
   cargo run --release --bin native_test_<name>
   ```

3. **No regressions**
   - Previous phase tests still pass
   - Compare output with expected XML

4. **Code quality**
   - Run `cargo clippy`
   - Run `cargo fmt`
   - Review for clarity

## Success Metrics

| Phase | LOC | Tests Passing | Cumulative Time |
|-------|-----|---------------|-----------------|
| 1 | ~250 | 0 (infrastructure) | 2-3h |
| 2 | ~430 | 3-5 | 5-7h |
| 3 | ~530 | 8-12 | 7-10h |
| 4 | ~760 | 20-30 | 11-15h |
| 5 | ~830 | 40-50 | 14-19h |
| 6 | ~1400 | 80-90+ | 20-28h |

## Key Implementation Notes

### Backtracking Pattern

```rust
let saved_pos = stream.position();
match try_parse() {
    Ok(result) => return Ok(result),
    Err(_) => {
        stream.set_position(saved_pos);
        // Try next alternative
    }
}
```

### Insertion Pattern

```rust
if insertion {
    return Ok(ParseResult {
        node: Some(XmlNode::Text(value.to_string())),
        consumed: 0, // Critical: consume nothing
    });
}
```

### Mark Pattern

```rust
match mark {
    Mark::Hidden => return Ok(ParseResult { node: None, consumed }),
    Mark::Attribute => /* convert to attribute */,
    Mark::Promoted => /* unwrap element */,
    Mark::None => /* wrap in element */,
}
```

## Debugging Tips

1. **Add tracing**: Use `println!` or `tracing` crate to see parse flow
2. **Small tests first**: Start with simplest grammars
3. **Compare with Earley**: Use same input, compare results
4. **Check position**: Always verify `consumed` matches actual advancement
5. **Test insertions**: Verify `consumed=0` for `+"text"`

## When Things Go Wrong

**Parse fails unexpectedly**:
- Check backtracking (position restored?)
- Verify mark handling (suppressed node?)
- Test in isolation (create minimal grammar)

**Infinite loop**:
- Check epsilon-match detection
- Verify repetition advances position
- Add recursion depth limit

**Wrong output**:
- Check mark application order
- Verify attribute extraction
- Compare with expected XML structure

**Performance issues**:
- Profile with `cargo flamegraph`
- Add memoization for recursive rules
- Cache expensive operations

## Ready to Start?

1. Read `NATIVE_INTERPRETER_DESIGN.md` for architecture details
2. Follow this checklist phase-by-phase
3. Test incrementally
4. Ask questions when stuck!

**Good luck!** 🚀
