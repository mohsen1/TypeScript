# Comprehensive Code Review: Language Service Implementation

**Generated:** 2026-01-04
**Reviewer:** Gemini 3 Pro (via ask-gemini.mjs)
**Scope:** `wasm/src/services/mod.rs` and related language service functionality

---

## Executive Summary

This document contains a comprehensive code review of the newly merged Language Service implementation for the TypeScript-to-Rust migration. The review covers all major language service features: completions, go-to-definition, hover/quick-info, diagnostics, references/rename, formatting, and document symbols.

### Overall Assessment

The language service implementation is **a functional skeleton** that demonstrates the correct API structure but lacks the deep semantic integration required for production use. The fundamental issue is that **symbol resolution only works for top-level declarations**, making most features non-functional for local variables, class members, and cross-file navigation.

### Issue Counts by Severity

| Severity | Count | Description |
|----------|-------|-------------|
| **BLOCKER** | 5 | Core functionality broken, cannot be used |
| **CRITICAL** | 7 | Major features missing or incorrect |
| **MAJOR** | 8 | Significant gaps in implementation |
| **MINOR** | 4 | Style, optimization, or minor functionality |

---

## General Architecture Issues

### [BLOCKER] Symbol Resolution Limited to Top-Level Scope

**Problem:** `get_symbol_at_location` in `checker/state.rs` only looks up symbols in `file_locals` (top-level scope). It cannot resolve:
- Local variables inside functions
- Function parameters
- Class members
- Block-scoped variables

**Evidence:**
```rust
Some(Node::Identifier(id)) => {
    // Look up in file locals (top-level declarations)
    self.file_locals.get(&id.escaped_text)
}
```

**Impact:** This breaks:
- Find References (only finds top-level references)
- Rename (only renames top-level symbols)
- Go-to-Definition (only works for top-level)
- Completions (no member completions)

**Fix:**
1. Attach `SymbolId` to AST nodes during binding
2. Store node-to-symbol mapping in `CheckerState`
3. Update `get_symbol_at_location` to use this mapping

### [BLOCKER] Property Access Resolution Broken

**Problem:** `obj.prop` resolution delegates to identifier lookup, searching for a global variable named `prop` instead of looking up the member on the type of `obj`.

**Evidence:**
```rust
Some(Node::PropertyAccessExpression(pae)) => {
    self.get_symbol_at_location(pae.name)  // Wrong!
}
```

**Fix:**
1. Get type of `pae.expression` (left side)
2. Look up symbol `pae.name` in the `members` of that type
3. Return that symbol

### [CRITICAL] No Cross-File Navigation Support

**Problem:** All definition info hardcodes the current filename. Go-to-definition is fundamentally a cross-file operation.

**Evidence:**
```rust
file_name: self.file_name.clone(),
```

**Fix:** Track source file origin in `NodeBase` or Symbol.

### [CRITICAL] Code Duplication - type_to_string

**Problem:** `LanguageService::type_to_string` (~100 lines) duplicates `CheckerState::type_to_string`, risking inconsistency.

**Fix:** Remove local method and call `self.checker.type_to_string(type_id)`.

### [MAJOR] LanguageService Lacks Source Text Access

**Problem:** The `LanguageService` struct has no reference to source text, limiting accurate refactorings and text-based operations.

**Fix:** Add `source_text: &'a str` field to `LanguageService`.

---

## Completions (Auto-Complete)

**Verdict:** 2 Critical, 2 Major issues. **Placeholder implementation.**

### [CRITICAL] Context-Agnostic Completions

**Problem:** Returns all global symbols and keywords regardless of cursor position. Typing `myObject.` shows `const`, `class`, etc. instead of object members.

**Evidence:**
```rust
pub fn get_completions_at_position(&self, position: u32) -> Option<CompletionInfo> {
    // For now, provide all file-level symbols as completions
    for (name, symbol_id) in self.checker.get_file_symbols() {
        // ... adds everything ...
    }
}
```

**Fix:** Determine context from AST:
- If in `PropertyAccessExpression`, return member completions
- If in type position (after `:`), return types
- If after `new`, prioritize classes

### [MAJOR] Missing Completion Kinds

**Problem:** No support for:
- String literal completions (imports)
- Constructor completions (after `new`)
- Type completions (after `:`)

### [MAJOR] Performance - Keyword Allocation

**Problem:** Every completion request allocates new Strings for all keywords.

**Fix:** Use `Cow<'static, str>` or cache keyword entries.

---

## Go-to-Definition

**Verdict:** 2 Blockers, 2 Critical, 2 Major issues. **Non-functional for most cases.**

### [BLOCKER] Incorrect Symbol Resolution

See General Architecture Issues above.

### [BLOCKER] Property Access Fails

Clicking on `obj.prop` jumps to a global variable `prop` (if one exists) instead of the property definition.

### [CRITICAL] No Cross-File Navigation

Always returns current filename.

### [CRITICAL] Missing Go-to-Type-Definition

`get_type_definition_at_position` is not implemented.

### [MAJOR] Incomplete Declaration Handling

Missing handlers for: `Constructor`, `MethodDeclaration`, `GetAccessor`, `SetAccessor`.

### [MAJOR] Missing Alias Resolution

Clicking on import `import { Foo } from './bar'` should jump to original definition in `bar.ts`, not the import specifier.

---

## Quick Info / Hover

**Verdict:** 0 Blockers, 1 Critical, 2 Major, 2 Minor issues.

### [CRITICAL] Incorrect Display Format

**Problem:** All symbols formatted as `keyword name: type`. For functions, shows `function foo: (x: number) => void` instead of correct `function foo(x: number): void`.

### [MAJOR] Inaccurate Keyword Mapping

**Problem:** `VariableElement` always shows `var`, regardless of whether it's `const`, `let`, or `var`.

**Fix:** Inspect declaration node for variable declaration list flags.

### [MAJOR] Missing JSDoc Support

**Evidence:**
```rust
documentation: Vec::new(), // TODO: extract JSDoc
```

### [MINOR] Code Duplication (type_to_string)

### [MINOR] Unsafe Transmute in get_node_kind

Using `transmute` for AST node kinds > 166 is undefined behavior.

---

## Diagnostics

**Verdict:** 1 Blocker, 1 Critical, 2 Major issues.

### [BLOCKER] Missing Diagnostic Methods

**Problem:** `get_semantic_diagnostics` and `get_syntactic_diagnostics` are **completely missing** from `LanguageService`.

### [CRITICAL] Inaccurate Error Spans

**Problem:** Uses entire node range for diagnostic span instead of narrowing to specific child (e.g., identifier name).

**Evidence:**
```rust
self.diagnostics.push(Diagnostic::error(
    base.pos,
    base.end - base.pos,  // Full node width
));
```

**Fix:** Implement `get_error_span_for_node` helper.

### [MAJOR] Incomplete Diagnostic Codes

Only ~30 error codes defined vs hundreds in TypeScript.

### [MAJOR] Hardcoded Severity

All diagnostics are `Error`. No support for Warnings, Suggestions, or Messages.

---

## Find References & Rename

**Verdict:** 2 Blockers, 1 Critical, 2 Major, 1 Minor issues.

### [BLOCKER] Symbol Resolution Broken for Locals

Only works for top-level file-scope variables. Cannot find references to:
- Local variables
- Function parameters
- Class members

### [CRITICAL] Incorrect Write Access Detection

**Problem:** Returns `true` only if node is a declaration. Doesn't detect assignments (`x = 1`) or update expressions (`x++`).

**Fix:** Check parent for assignment/update expression context.

### [MAJOR] Missing Rename Validation

`can_rename: true` for any symbol found. Should reject:
- Keywords
- External library symbols
- String literals in some contexts

### [MAJOR] Missing Alias Resolution

Comparing symbols by identity fails for imports/exports where imported symbol is an alias.

### [MINOR] Inefficient AST Traversal

Traverses entire AST for every request.

---

## Formatting & Indentation

**Verdict:** Non-Existent (Skeleton Only)

### [CRITICAL] No Formatting Implementation

Configuration structures exist (`FormatCodeSettings`, `IndentStyle`, `SemicolonPreference`) but no actual formatting logic:
- No `get_formatting_edits_for_range`
- No `format_document`
- No `get_indentation_at_position`

---

## Document Symbols (Navigation)

**Verdict:** Partial / Incomplete

### [MAJOR] Missing Node Types

Not included in `collect_navigation_items`:
- `PropertyDeclaration`
- `ConstructorDeclaration`
- `GetAccessorDeclaration`
- `SetAccessorDeclaration`
- `ModuleDeclaration`
- `EnumMember`

### [MAJOR] No Declaration Merging

TypeScript merges declarations (namespace + function with same name). Current implementation returns distinct items.

---

## Outlining (Folding Ranges)

**Verdict:** Rudimentary

### [MAJOR] Missing Folding Ranges

- Import declarations not folded
- Comments not folded (no trivia scanning)
- No `#region` / `#endregion` support
- No JSX element folding

### [MINOR] Incorrect Heuristic

Uses byte-length check (`end > start + 10`) instead of line-based logic.

---

## Feature Comparison with TypeScript

| Feature | TypeScript | Rust Migration |
|---------|------------|----------------|
| **Symbol Resolution** | Full scope chain | Top-level only |
| **Property Access** | Type-based lookup | Global lookup (broken) |
| **Cross-File Navigation** | Full support | Not implemented |
| **Completions** | Context-aware | Context-agnostic |
| **Member Completions** | Yes | No |
| **Go-to-Definition** | Full support | Top-level only |
| **Go-to-Type-Definition** | Yes | Not implemented |
| **Quick Info** | Accurate signatures | Wrong format |
| **JSDoc** | Full support | Not implemented |
| **Diagnostics API** | Full | Not exposed |
| **Find References** | Full scope | Top-level only |
| **Rename** | Validated | No validation |
| **Formatting** | Full engine | Not implemented |
| **Document Symbols** | Complete | Missing many kinds |
| **Folding Ranges** | AST + Comments | AST only |

---

## Recommendations

### Immediate Priority (Blockers)

1. **Fix symbol resolution** - Attach symbols to AST nodes during binding
2. **Fix property access resolution** - Use type-based member lookup
3. **Implement diagnostics API** - Expose `get_semantic_diagnostics` / `get_syntactic_diagnostics`

### High Priority (Critical)

1. **Add cross-file navigation support** - Track source file in nodes/symbols
2. **Fix completion context detection** - Member completions, type completions
3. **Implement go-to-type-definition**
4. **Fix quick info display format** - Proper function signatures

### Medium Priority (Major)

1. **Implement write access detection** - Check assignment context
2. **Add rename validation** - Reject keywords, library symbols
3. **Complete document symbols** - Add missing node types
4. **Improve folding ranges** - Line-based heuristics, comments

### Low Priority (Minor)

1. **Optimize allocations** - Cache keyword completions
2. **Add JSDoc support**
3. **Implement formatting engine**

---

## Conclusion

The Language Service implementation provides the correct API structure and demonstrates understanding of the TypeScript language service architecture. However, the **fundamental limitation** is that symbol resolution only works for top-level declarations due to the binder discarding local scope information.

Until the symbol-to-node mapping is implemented, most language service features will only work for trivial cases (global variables in single-file projects). This is the single most important fix needed to make the language service functional.

**Total: 5 blockers, 7 critical, 8 major, 4 minor issues requiring attention.**
