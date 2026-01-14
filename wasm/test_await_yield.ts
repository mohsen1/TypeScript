// Test file to verify await/yield don't trigger false TS1109

// Scenario 1: await in non-async context
const value = await 5;  // Should NOT trigger TS1109 for 'await'

// Scenario 2: yield in non-generator context
const result = yield 10;  // Should NOT trigger TS1109 for 'yield'

// Scenario 3: await after missing semicolon (error recovery)
const x = 1  // TS1005: missing semicolon
await Promise.resolve()  // Should NOT trigger TS1109 due to cascading fix

// Scenario 4: yield after missing semicolon (error recovery)
function* gen() {
  const y = 5  // TS1005: missing semicolon
  yield 10  // Should NOT trigger TS1109 due to cascading fix
}

// Scenario 5: await with missing operand
const z = await;  // Should trigger TS1109 (actual error - missing operand)
