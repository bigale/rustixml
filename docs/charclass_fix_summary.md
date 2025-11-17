# Character Class OR Operator Fix - Summary

## Achievement
Fixed character class `|` (OR) operator handling, eliminating ALL test failures!

## Results
- **Before**: 17 passing, 2 failures (lf, para-test)
- **After**: 19 passing (+2), 0 failures (-2) ✅

## Tests Fixed
1. ✅ **lf** - Line parsing with negated character class `~[#a | #d]*`
2. ✅ **para-test** - Paragraph parsing (also uses character classes with `|`)

## Root Cause
The `parse_char_class()` function in `src/runtime_parser.rs` was only splitting character class content by `;` and `,`, but not by `|`. 

In iXML character classes, `|` is an OR operator that separates alternatives, just like `,`.

Example from lf test:
```
line: ~[#a | #d]*.
```

This should match characters that are NOT (`#a` OR `#d`), meaning NOT (linefeed OR carriage return).

Without `|` splitting, `#a | #d` was treated as a single malformed element instead of two separate characters.

## Fix Applied
**File**: `src/runtime_parser.rs:415`

Changed:
```rust
let elements: Vec<&str> = part.split(',').map(|s| s.trim()).collect();
```

To:
```rust
let elements: Vec<&str> = part.split(|c| c == ',' || c == '|').map(|s| s.trim()).collect();
```

## Impact
- All formatting-related failures resolved (by semantic XML comparison)
- All parsing failures resolved (by this fix)
- **Zero failing tests!** 🎉

## Remaining Work
- 19 timeout tests (performance/complexity issues)
- 6 error tests (grammar or input parsing errors)

## Next Priority
Investigate timeout tests to improve performance, especially:
- expr series (7 tests)
- diary series (3 tests)  
- json, xml, xpath, vcard
