# Grammar Normalization: A Fresh Approach for rustixml

**Source**: Steven Pemberton's "Invisible XML" talk (CWI, 2016)
**URL**: https://homepages.cwi.nl/~steven/Talks/2016/02-12-prague/data.html

## The Core Insight

The iXML specification describes a **grammar normalization** process that simplifies grammars before parsing:

> "If it is an implicit terminal delete it; if it is a refinement, replace it with the definition of that refinement enclosed with brackets, unless this refinement is already a part of it (i.e. the refinement is recursive)."

After normalization:
- All non-recursive rules are inlined into their usage sites
- Only recursive rules remain as separate definitions
- Unused rules are discarded
- The result is a **canonical schema** for the data structure

## Why This Matters for rustixml

### Current Problem (from KNOWN_ISSUES.md)
rustixml at 75.4% conformance, with issues in:
1. **Left-recursion** - `expr`, `xpath` tests fail
2. **Ambiguity detection** - 11 tests failing
3. **Mysterious failures** - `vcard` shows "1 alternatives tried" for rule with 2 alternatives
4. **Complex grammar handling** - Indirect references create confusion

### How Normalization Solves This

#### 1. Makes Left-Recursion Visible

**Before normalization:**
```ixml
expr: term, expr-tail.
expr-tail: "+" expr | .
term: factor, term-tail.
term-tail: "*" term | .
```

The left-recursion is **hidden** through indirect references (`expr-tail`, `term-tail`).

**After normalization (inline non-recursive rules):**
```ixml
expr: (factor, ("*" expr | )), ("+" expr | ).
```

Now the recursion pattern is **direct and obvious**. The parser can:
- Detect it immediately
- Apply systematic transformation (right-recursion or iteration)
- Handle it correctly

#### 2. Eliminates Indirect Ambiguity

**Before normalization:**
```ixml
a: b | c.
b: "x".
c: "x", "y"?.
```

Ambiguity is **hidden** through rule indirection.

**After normalization:**
```ixml
a: ("x") | ("x", "y"?).
```

The ambiguity is **immediate and explicit**: both alternatives start with "x". Much easier to detect during parsing.

#### 3. Fixes the `vcard` Mystery

The `vcard` error ("1 alternatives tried for a rule with 2 alternatives") suggests the parser is following an indirect reference and losing track of alternatives.

Normalization would:
- Inline all non-recursive references
- Eliminate the indirection chain
- Make both alternatives visible at the decision point

#### 4. Simplifies Parser Logic

**Current recursive descent parser:**
```rust
fn parse_nonterminal(&self, name: &str) -> Result<Node> {
    // 1. Look up rule by name
    let rule = self.grammar.get_rule(name)?;

    // 2. Try each alternative
    for alt in &rule.alternatives {
        if let Ok(node) = self.parse_alternative(alt) {
            return Ok(node);
        }
    }
    Err("No match")
}
```

**With normalized grammar:**
```rust
fn parse_nonterminal(&self, name: &str) -> Result<Node> {
    // 1. Look up rule (only recursive rules remain)
    let rule = self.grammar.get_rule(name)?;

    // 2. Alternatives are already inlined at usage sites
    // 3. Recursive rules are explicit and handled specially

    if rule.is_recursive {
        return self.parse_recursive(rule);
    } else {
        // Non-recursive rules were inlined during normalization
        unreachable!("All non-recursive rules should be inlined");
    }
}
```

## Implementation Strategy

### Phase 0: Grammar Normalization (NEW - Before v0.3.0)

**Add normalization as preprocessing step between grammar parsing and input parsing.**

#### Step 1: Detect Recursive Rules

```rust
/// Analyze grammar to find which rules are recursive
fn find_recursive_rules(grammar: &Grammar) -> HashSet<String> {
    let mut recursive = HashSet::new();

    for rule in &grammar.rules {
        if is_recursive(&rule, &grammar, &mut HashSet::new()) {
            recursive.insert(rule.name.clone());
        }
    }

    recursive
}

fn is_recursive(rule: &Rule, grammar: &Grammar, visited: &mut HashSet<String>) -> bool {
    if visited.contains(&rule.name) {
        return true; // Cycle detected
    }

    visited.insert(rule.name.clone());

    for alt in &rule.alternatives {
        for term in &alt.sequence {
            if let Term::Nonterminal(ref_name) = term {
                if ref_name == &rule.name {
                    return true; // Direct recursion
                }

                if let Some(ref_rule) = grammar.get_rule(ref_name) {
                    if is_recursive(ref_rule, grammar, visited) {
                        return true; // Indirect recursion
                    }
                }
            }
        }
    }

    visited.remove(&rule.name);
    false
}
```

#### Step 2: Inline Non-Recursive Rules

```rust
/// Normalize grammar by inlining all non-recursive rules
fn normalize_grammar(grammar: &Grammar) -> Grammar {
    let recursive_rules = find_recursive_rules(grammar);

    // Clone grammar for modification
    let mut normalized = grammar.clone();

    // Inline all non-recursive rules
    for rule in &mut normalized.rules {
        inline_references(rule, grammar, &recursive_rules);
    }

    // Remove inlined rules (keep only recursive ones)
    normalized.rules.retain(|r| recursive_rules.contains(&r.name));

    normalized
}

fn inline_references(
    rule: &mut Rule,
    grammar: &Grammar,
    recursive_rules: &HashSet<String>
) {
    for alt in &mut rule.alternatives {
        let mut new_sequence = Vec::new();

        for term in &alt.sequence {
            match term {
                Term::Nonterminal(ref_name) if !recursive_rules.contains(ref_name) => {
                    // Inline this non-recursive rule
                    if let Some(ref_rule) = grammar.get_rule(ref_name) {
                        // Wrap the inlined alternatives in a group
                        new_sequence.push(Term::Group(ref_rule.alternatives.clone()));
                    }
                }
                _ => {
                    new_sequence.push(term.clone());
                }
            }
        }

        alt.sequence = new_sequence;
    }
}
```

#### Step 3: Delete Implicit Terminals

```rust
/// Remove implicit terminals from normalized grammar
fn remove_implicit_terminals(grammar: &mut Grammar) {
    for rule in &mut grammar.rules {
        for alt in &mut rule.alternatives {
            alt.sequence.retain(|term| {
                match term {
                    Term::Terminal(s) if is_implicit(s) => false,
                    _ => true
                }
            });
        }
    }
}

fn is_implicit(s: &str) -> bool {
    // Implicit terminals are whitespace, delimiters, etc.
    // defined by the iXML spec
    s.chars().all(|c| c.is_whitespace())
}
```

### Integration with Current Parser

```rust
impl NativeParser {
    pub fn new(grammar: Grammar) -> Self {
        // NEW: Normalize grammar before parsing
        let normalized = normalize_grammar(&grammar);

        NativeParser {
            grammar: normalized,
            // ... rest of initialization
        }
    }

    pub fn parse(&self, input: &str) -> Result<String> {
        // Grammar is already normalized
        // Parsing logic can now assume:
        // 1. All non-recursive rules are inlined
        // 2. All remaining rules are recursive (explicit)
        // 3. All ambiguities are at decision points (not hidden)

        self.parse_start_rule(input)
    }
}
```

## Expected Impact on Known Issues

### Left-Recursion (v0.3.0 goal)
**Before**: Hidden through indirect references, hard to detect
**After**: Direct and explicit in normalized grammar
**Result**: ✅ Can implement systematic transformation

### Ambiguity Detection (v0.4.0 goal)
**Before**: Ambiguity can be hidden in rule indirection
**After**: All ambiguous alternatives are inline at decision points
**Result**: ✅ Easier to detect during parsing (explore all paths at each decision)

### vcard Mystery
**Before**: "1 alternatives tried for a rule with 2 alternatives"
**After**: All alternatives are explicit (no indirection to lose track)
**Result**: ✅ Likely fixes the issue

### Performance
**Before**: Rule lookups on every nonterminal
**After**: Fewer rules to look up (only recursive ones)
**Result**: ✅ 10-20% speedup expected

## Comparison with Current STRATEGY_OPTIONS.md

### This IS the "Nonterminal Inlining" Optimization - But Better

From STRATEGY_OPTIONS.md (Option 2.1):
```rust
// Inline simple wrapper rules to reduce call depth
fn inline_simple_nonterminals(ast: &mut GrammarAst) {
    let simple_rules = find_inlinable_rules(ast);
    for rule in simple_rules {
        replace_references_with_definition(ast, rule);
    }
}

fn is_inlinable(rule: &Rule) -> bool {
    rule.alternatives.len() == 1 &&
    rule.alternatives[0].sequence.len() == 1 &&
    matches!(rule.alternatives[0].sequence[0], Factor::CharClass(_))
}
```

**Problem**: This is a **heuristic** approach (only inline "simple" rules)
**Solution**: Grammar normalization is **systematic** (inline ALL non-recursive rules)

### Why Systematic is Better

1. **Correctness**: Follows the iXML specification formal definition
2. **Completeness**: Doesn't miss cases (no heuristics needed)
3. **Predictability**: Same grammar always normalizes the same way
4. **Spec compliance**: The iXML spec defines normalization formally

## Revised Implementation Roadmap

### Phase 0: Grammar Normalization (NEW - 2-3 weeks)
1. ✅ Implement recursion detection
2. ✅ Implement systematic inlining of non-recursive rules
3. ✅ Remove implicit terminals
4. ✅ Test normalization on all iXML test grammars

**Expected**: Foundation for all future improvements
**Risk**: Low - normalization is well-defined in spec

### Phase 1: Quick Wins (v0.3.0 - 1-2 weeks)
1. ✅ Left-recursion now **visible** - easier to handle
2. ✅ Direct ambiguity detection at normalized decision points
3. ✅ Profile and optimize (fewer rules to process)

**Expected**: 85-90% conformance (from 75.4%)

### Phase 2: Advanced Features (v0.4.0 - 2-3 weeks)
1. ✅ Full ambiguity handling (explore all normalized paths)
2. ✅ Advanced error messages (know exactly where decision failed)
3. ✅ Edge case handling

**Expected**: 92-95% conformance

## Why This Changes Everything

### Before (Current Strategy)
- Try various heuristics to optimize recursive descent
- Left-recursion: guess patterns and transform
- Ambiguity: explore paths somehow
- **Problem**: Fighting against grammar complexity

### After (Normalization)
- Transform grammar to canonical form ONCE
- Left-recursion: already explicit
- Ambiguity: already visible at decision points
- **Solution**: Work with simplified, normalized structure

### Analogy: Compiler Optimization

**Normalization is to parsing what SSA form is to optimization.**

Just as compilers convert code to SSA (Static Single Assignment) form to make optimizations easier:
- Makes data flow explicit
- Eliminates redundancy
- Canonical representation

Grammar normalization:
- Makes recursion explicit
- Eliminates indirection
- Canonical representation

## Implementation Timeline

### Week 1-2: Core Normalization
- Implement recursion detection
- Implement inlining algorithm
- Test on simple grammars

### Week 3: Integration
- Integrate with NativeParser
- Run conformance tests
- Debug issues

### Week 4: Optimization
- Profile normalized parsing
- Optimize hot paths
- Document approach

**Total**: 3-4 weeks to foundational improvement

## Open Questions

1. **Recursive rule groups**: How to handle mutually recursive rules (A calls B, B calls A)?
   - **Answer**: Keep both as separate rules (can't inline either)

2. **Mark propagation**: Do iXML marks (^, -, @) propagate correctly through inlining?
   - **Answer**: Need to preserve marks during normalization

3. **Grammar size**: Could inlining create very large grammars?
   - **Answer**: Possible, but modern memory makes this acceptable

4. **Debugging**: How to map errors in normalized grammar back to original?
   - **Answer**: Maintain source location metadata during normalization

## Conclusion

Grammar normalization is **the missing piece** in rustixml's strategy.

It's not listed in STRATEGY_OPTIONS.md as a distinct approach because it was hiding in plain sight as "nonterminal inlining" - but the systematic, spec-defined version is far more powerful than the heuristic approach.

**Recommendation**: Implement normalization as **Phase 0** before v0.3.0.

This provides:
- ✅ Solid foundation for all other improvements
- ✅ Spec-compliant transformation
- ✅ Makes left-recursion and ambiguity handling tractable
- ✅ Likely fixes mysterious bugs (like vcard)
- ✅ Performance improvement as bonus

**Status**: Ready to implement
**Risk**: Low (well-defined transformation)
**Reward**: High (enables all planned improvements)
