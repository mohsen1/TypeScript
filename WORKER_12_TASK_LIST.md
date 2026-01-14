# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Switch solver default fallback from `Any` to `Unknown` to expose hidden bugs

## Context

**The "Any" Poisoning Problem**
Currently, when the compiler encounters something it doesn't understand (a missing symbol, a complex generic, or a syntax error), it defaults to `Any`. In TypeScript, `Any` shuts off type checking - if a variable becomes `Any` because the binder couldn't find its definition, **all** downstream errors related to that variable disappear.

This creates an illusion of progress. You implement new features (like Control Flow Analysis), but because the underlying variables resolved to `Any` (due to binding failures), the new checks simply say "Looks good!" and emit nothing.

**Why This Task Matters**
Switching to `Unknown` or `Error` as the default fallback will:
1. **Expose bugs** - Instead of silently accepting invalid code, the compiler will emit real errors
2. **Reveal progress** - Fixes that were being masked by `Any` will become visible in conformance metrics
3. **Force correctness** - We can't hide behind permissive defaults

**Expected Outcome**
- Conformance metrics may temporarily worsen (more "extra errors")
- This is GOOD - it means we're seeing real errors instead of masked failures
- The errors will point to actual bugs that need fixing in Binder and Solver

## Queue
- [ ] After switching fallback, analyze new errors to identify top bug patterns
- [ ] Coordinate with Binder Squad (TS2304 issues revealed)
- [ ] Coordinate with Parser Squad (syntax errors causing resolution failures)

## Implementation Steps

1. **Locate the fallback logic**
   - Search for `TypeId::ANY` in `wasm/src/solver/`
   - Find where `solve_subtype` and other solver functions return `Any` on failure

2. **Make the switch**
   - Replace `TypeId::ANY` with `TypeId::UNKNOWN` or `TypeId::ERROR`
   - Update any error handling that assumes `Any` as the safe default

3. **Test and measure**
   - Run `./wasm/test.sh` to verify compilation
   - Run `./wasm/differential-test/run-conformance.sh --all`
   - Compare before/after reports to identify exposed bugs

4. **Document findings**
   - What error codes increased the most?
   - Which files/tests reveal the most binding failures?
   - Create prioritized bug list for other squads

## Ready for Merge
No (task in progress)
