# Strategy Document: Static Ambiguity Detection

**Project:** rustixml
**Author:** AI Agent
**Date:** 2025-11-21
**Version:** 1.0

## 1. Overview & Goal

The primary objective of this experiment is to develop a method for **statically detecting ambiguity** in a user-provided iXML grammar. This analysis will occur before parsing any user input, providing direct feedback on the grammar itself.

A core constraint of this experiment is to leverage the existing **Recursive Descent Parser (RDP)** architecture, using it as a "reflection" engine to analyze the grammar's properties.

The proposed methodology is **Parse-Driven Grammar Analysis**. This approach involves three phases: instrumenting the RDP to detect ambiguity, analyzing the grammar AST to generate "witness strings" for potential ambiguities, and using the instrumented RDP to perform a "dry run" on these strings to confirm ambiguity.

## 2. Core Methodology

A grammar is ambiguous if a single input string can be parsed into multiple distinct parse trees. Statically proving ambiguity for all cases is an undecidable problem. However, we can detect many common forms of ambiguity by focusing on a key indicator: **rules where multiple alternatives can match the same input string.**

Our strategy is to:
1.  **Identify** rules whose alternatives start with a common prefix.
2.  **Extract** that common prefix as a "witness string".
3.  **Parse** the witness string using a modified RDP that is capable of finding multiple parse trees.
4.  If the parser finds more than one valid parse for the witness string under the rule in question, the grammar is confirmed to be ambiguous.

## 3. Implementation Phases

### Phase I: Instrument the RDP for Ambiguity Detection

**Objective:** Enhance the `NativeParser` to identify and report all valid parses for a given input, rather than just the longest one.

**Key Changes:**

1.  **Modify `native_parser::parse_alternatives`:**
    -   The function will be altered to no longer return immediately after finding the longest match.
    -   It will iterate through **all** alternatives of a given rule.
    -   It will collect every successful `ParseResult` into a list.

2.  **Group and Analyze Results:**
    -   After trying all alternatives, the collected `ParseResult`s will be grouped by the number of characters they consumed.
    -   The **ambiguity condition** is met if any group (for a given consumption length) contains more than one `ParseResult`. This signifies that multiple alternatives successfully parsed an identical segment of the input string, resulting in different parse trees.

3.  **New Parser Mode:**
    -   These changes will be implemented within a new "ambiguity detection" mode or a distinct `AmbiguityDetectingParser` struct to avoid impacting the performance of standard parsing. The output in this mode will be a list of all valid parse trees found.

### Phase II: Static Analysis for Candidate Generation

**Objective:** Statically analyze the grammar's AST to identify rules that are likely to be ambiguous and generate witness strings.

**Implementation:**

1.  **`find_ambiguity_candidates` Function:**
    -   A new standalone function will be created: `find_ambiguity_candidates(grammar: &IxmlGrammar) -> Vec<Candidate>`.

2.  **`Candidate` Data Structure:**
    -   This function will return a list of `Candidate` structs, defined as:
        ```rust
        struct Candidate {
            rule_name: String,
            witness_string: String,
        }
        ```

3.  **Candidate Identification Algorithm:**
    -   The function will iterate through each rule in the grammar.
    -   For each rule, it will compare all pairs of its alternatives (e.g., `alt1` vs. `alt2`).
    -   It will analyze the sequence of `Factor`s at the beginning of each alternative to find a **common terminal prefix**.
    -   **Example:** For `rule: "a" "b" | "a" "c" .`, the algorithm would identify `"a"` as the common prefix and generate a `Candidate { rule_name: "rule", witness_string: "a" }`.
    -   This initial implementation will focus only on prefixes composed of terminal strings and characters.

### Phase III: Verification via "Dry Run" Execution

**Objective:** Use the instrumented RDP from Phase I to confirm or deny the potential ambiguities found in Phase II.

**Implementation:**

1.  **`detect_static_ambiguity` Function:**
    -   A new top-level function will orchestrate the process: `detect_static_ambiguity(grammar: &IxmlGrammar) -> Vec<AmbiguityReport>`.

2.  **`AmbiguityReport` Data Structure:**
    -   The function will return a list of confirmed ambiguities, defined as:
        ```rust
        struct AmbiguityReport {
            rule_name: String,
            witness_string: String,
            parse_trees: Vec<String>, // e.g., XML string representation of each tree
        }
        ```

3.  **Verification Process:**
    -   The function will first call `find_ambiguity_candidates` to get the list of potential issues.
    -   It will then instantiate the `AmbiguityDetectingParser` from Phase I.
    -   For each `Candidate`, it will invoke a specialized method on the parser, such as `parser.check_ambiguity_for_rule(&candidate.rule_name, &candidate.witness_string)`.
    -   This method will run the parsing logic on the `witness_string`, starting from the specified rule.
    -   If the parser's modified `parse_alternatives` function reports an ambiguity (i.e., returns multiple valid parse trees for the witness string), an `AmbiguityReport` is generated and added to the final results.

## 4. Expected Outcomes & Success Criteria

-   **Primary Deliverable:** A function that takes a grammar and returns a list of confirmed static ambiguities, including the rule, the input string that demonstrates the ambiguity, and the resulting parse trees.
-   **Success Criteria:** The system will be considered successful if it can correctly identify simple, prefix-based ambiguities in test grammars, such as:
    -   `A: "x" | "x" .`
    -   `A: "a" "b" | "a" "c" .`
-   This functionality will provide grammar authors with immediate, actionable feedback, allowing them to resolve ambiguities before deploying the grammar.

## 5. Limitations & Future Work

-   **Scope:** This method is a heuristic and is not guaranteed to find all possible ambiguities in a context-free grammar, as that is an undecidable problem.
-   **Initial Limitation:** The first version of this experiment will only detect ambiguities arising from common prefixes of **terminal symbols** (strings and characters).

-   **Future Enhancements:**
    1.  **Non-Terminal Prefixes:** Extend the candidate generation logic to handle cases where alternatives share a common non-terminal prefix (e.g., `A: B "x" | B "y" .`).
    2.  **Integration:** Integrate the ambiguity detection mechanism into the `IxmlParser::new()` constructor to automatically warn users of ambiguous grammars.
    3.  **More Complex Patterns:** Investigate algorithms for detecting other ambiguity patterns beyond common prefixes.
