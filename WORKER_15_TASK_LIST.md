# Worker 15 Task List - CFA Edge Cases

## Current Task
- [ ] **CFA-26: Add computed property name CFA**
  - Track side effects in computed property name evaluation
  - Handle variable access within computed property expressions
  - Test class declarations with computed property names

## Queue
(none yet)

## Completed
- [x] **CFA-25: Implement static block definite assignment analysis**
  - Added `find_enclosing_static_block` helper to detect if code is inside a static block
  - Added `find_class_for_static_block` helper to get the enclosing class
  - Added `is_variable_used_before_declaration_in_static_block` for TDZ checking
  - Integrated static block TDZ check into `get_type_of_identifier`
  - Emits TS2454 when a variable is used in a static block before its declaration
