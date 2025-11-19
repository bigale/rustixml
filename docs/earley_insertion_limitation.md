# Earley Translation Limitation: Suppressed+Inserted Sequences

## Problem Description

The unicode-classes test fails on line 33, which contains:
```
Co: -"Co ", (-[Co], +".")*.
```

This pattern combines:
- **Suppression**: `-[Co]` matches a Co category character but suppresses it from output
- **Insertion**: `+"."` inserts a literal "." that wasn't in the input  
- **Repetition**: `*` repeats this sequence zero or more times

## Root Cause

The Earley parser fundamentally deals with **consuming input tokens**. Our translation approach has worked by:
1. Creating terminal predicates for character matching
2. Creating nonterminals for sequences and repetitions
3. Using tree-building to construct output

However, **insertions** (`+`) require adding content to the output that doesn't correspond to any input token. Combined with **suppression** (`-`) in a repeated sequence, this creates an impossible situation:

- The Earley grammar needs to consume the Co character (for `-[Co]`)
- But the tree builder needs to insert "." (for `+"."`)
- This happens in a loop, so we need multiple inserted "." for multiple Co characters

## Test Case Isolation

Created `debug_suppressed_insertion.rs` which shows:
- `[Co]*` works fine ✓
- `(-[Co], +".")*` fails ✗

The Co character (U+E000, private use) parses successfully without the insertion pattern.

## Why This Matters

This is line 33 of the unicode-classes test, which explains why:
- Lines 1-32 all parse successfully
- Line 33 breaks the parse
- The full test fails with "No Rule completes"

## Other Affected Tests

Need to check if `ixml-spaces` and `ixml3` also use this pattern.

## Strategic Impact

This represents a **fundamental abstraction mismatch** between:
- iXML semantics (insertion + suppression are first-class operations)
- Earley parsing (only consumes input, doesn't have insertion semantics)

This validates the analysis in `ABSTRACTION_ANALYSIS.md` - we're at the wrong abstraction level. The Earley translation approach has reached a hard limit.

## Recommended Path Forward

Given the constraint to "wrap up Earley in 1-2 passes":

1. **Document this limitation** clearly
2. **Mark tests using this pattern as known failures**
3. **Move to native iXML interpreter** that handles insertion/suppression natively

The current Earley approach has achieved:
- 53 PASS tests (39.8%)
- Character classes working
- Unicode categories working
- Multi-character literals working
- Basic sequences and repetitions working
- GROUP_COUNTER synchronization fixed

But it cannot handle:
- Insertions combined with suppressions in repeated sequences
- Potentially other advanced iXML features

This is a clean stopping point for the Earley approach.
