# rustixml Conformance Roadmap

This document outlines the known issues in rustixml and provides a clear roadmap for improving iXML conformance.

## Current Conformance (v0.2.0 - Post-fixes): 75.4%

- **Total Tests**: 49 / 65 passing
- **Correctness**: 44 / 49 (89.8%)
- **Ambiguous**: 2 / 13 (15.4%)
- **Error**: 3 / 3 (100.0%)

This is an improvement from the initial v0.2.0 state of 69.2% (45/65) passing.

---

## v0.3.0: Foundational Improvements

**Target Conformance**: ~80-85%
**Focus**: Address the most straightforward correctness issues.

### 1. Grammar Parser Enhancements
- **Status**: `Done`
- **Goal**: Implement missing iXML 1.0 grammar features.
- **Fixes (4 tests)**:
    - `correct/version-decl` (Version declaration)
    - `correct/version-decl.2` (Version declaration with any string)
    - `correct/ws-and-delim` (`=` as rule separator)
    - `error/invalid-char` (`+#hex` insertion syntax)

### 2. Line Ending Normalization
- **Status**: `To Do`
- **Goal**: Handle `CR`, `LF`, and `CRLF` line endings consistently by normalizing them in the input stream.
- **Fixes**: `correct/vcard` and potentially other tests sensitive to newlines.
- **Note**: A previous attempt at this did not resolve the `vcard` failure, which points to a more complex underlying issue. The `vcard` test fails with a mysterious error (`1 alternatives tried` for a rule with 2 alternatives) that requires deeper investigation.

### 3. Basic Left-Recursion Handling
- **Status**: `To Do`
- **Goal**: Detect direct left-recursion and apply a transformation to prevent infinite loops. This is a precursor to full left-recursion support.
- **Fixes**: `correct/expr`, `correct/xpath`, and potentially some ambiguity tests like `ambiguous/expr0`.

---

## v0.4.0: Ambiguity & Advanced Grammars

**Target Conformance**: ~90-95%
**Focus**: Tackle ambiguity and implement key optimizations for complex grammars.

### 1. Full Ambiguity Detection
- **Status**: `To Do`
- **Goal**: Modify the parser to detect all possible valid parses and, if more than one exists, wrap the output with `ixml:state="ambiguous"`.
- **Fixes (11 tests)**: This is expected to resolve most, if not all, of the `ambiguous/*` test failures.

### 2. Advanced Lexing for Identifiers
- **Status**: `To Do`
- **Goal**: Update the lexer to correctly tokenize identifiers that contain dots (e.g., `unicode-6.1`), as required by the iXML specification.
- **Fixes**: `correct/unicode-version-diagnostic`.
- **Note**: This is a challenging task due to the ambiguity between `.` in identifiers and `.` as a rule terminator. It requires careful changes to the lexer.

---

## Architectural Considerations

### Current Approach: Native Recursive Descent
rustixml uses a native recursive descent parser that directly interprets the iXML grammar. This approach is simple and maps cleanly to iXML's unique semantics (like insertion and suppression). However, it requires explicit implementation of features like ambiguity detection and left-recursion handling.

### Alternative: LALR+GLR
The `markup-blitz` reference implementation uses an LALR+GLR parser, which handles ambiguity and left-recursion automatically. While powerful, this represents a complete architectural rewrite and is significantly more complex.

**Decision**: For the foreseeable future, we will enhance the existing native parser. This strategy allows for incremental improvements and lower development risk. The LALR+GLR approach remains a consideration for a future major version (e.g., v2.0) if 100% conformance becomes a critical requirement.

## How to Contribute
The best way to contribute is to pick a task from the roadmap!
1. **Medium**: Investigate the `correct/vcard` failure.
2. **Hard**: Implement left-recursion transformation.
3. **Expert**: Design and implement a robust lexer for advanced identifier syntax.

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more details on setting up the development environment and submitting changes.
