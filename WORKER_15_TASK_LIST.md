# Worker 15 Task List - CFA Edge Cases

## Current Task
All CFA edge case tasks completed!

## Queue
(none yet)

## Completed
- [x] **CFA-28: Enable strict mode by default for type checking**
  - Changed `let strict = false` to `let strict = true` in lib.rs
  - Enables proper TS2454 (variable used before assigned) detection
  - Required for conformance with TypeScript's strict mode behavior
  - Affects checkSourceFile(), getTypeOfNode(), and getLspDiagnostics()
- [x] **CFA-27: Add class heritage clause CFA**
  - Added `find_enclosing_heritage_clause` helper to detect if code is inside an extends/implements clause
  - Added `find_class_for_heritage_clause` helper to get the enclosing class/interface
  - Added `is_variable_used_before_declaration_in_heritage_clause` for TDZ checking
  - Integrated heritage clause TDZ check into `get_type_of_identifier`
  - Emits TS2454 when a variable is used in a heritage clause before its declaration
  - Also fixed duplicate ValidationError enum definition in thin_binder.rs
- [x] **CFA-26: Add computed property name CFA**
  - Added `find_enclosing_computed_property` helper to detect if code is inside a computed property expression
  - Added `find_class_for_computed_property` helper to get the enclosing class
  - Added `is_variable_used_before_declaration_in_computed_property` for TDZ checking
  - Integrated computed property TDZ check into `get_type_of_identifier`
  - Emits TS2454 when a variable is used in a computed property name before its declaration
  - Also fixed duplicate ValidationError enum definition in thin_binder.rs
- [x] **CFA-25: Implement static block definite assignment analysis**
  - Added `find_enclosing_static_block` helper to detect if code is inside a static block
  - Added `find_class_for_static_block` helper to get the enclosing class
  - Added `is_variable_used_before_declaration_in_static_block` for TDZ checking
  - Integrated static block TDZ check into `get_type_of_identifier`
  - Emits TS2454 when a variable is used in a static block before its declaration
