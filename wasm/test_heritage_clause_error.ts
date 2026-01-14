// Test file to verify HeritageClauseElement specific error message

// Scenario 1: class extends with invalid token
class A extends ! {}  // Should report "Class name or type expression expected"

// Scenario 2: class extends with nothing after (semicolon)
class B extends ; {}  // Should report "Class name or type expression expected"

// Scenario 3: class implements with invalid token
class C implements + {}  // Should report "Class name or type expression expected"

// Scenario 4: interface extends with invalid token
interface D extends , {}  // Should report "Class name or type expression expected"

// Scenario 5: Multiple implements with one invalid
class E implements Foo, !, Bar {}  // Should report "Class name or type expression expected"
