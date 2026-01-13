# Worker 5 Task List

## Current Task
- [ ] Fix module augmentation resolution
  - Handle `interface Window` merging across files
  - Ensure module augmentations update existing declarations
  - Test with multiple files augmenting same global interface
- [ ] Fix namespace merging behavior
  - Merge declarations across multiple `namespace` blocks
  - Handle nested namespaces
  - Ensure exported symbols are visible to augmentations

## Queue
- [ ] Add module resolution debugging
  - Log symbol table merge operations
  - Track which file each symbol comes from
  - Verify module scope lookup order
- [ ] Test with real-world lib.d.ts augmentations
  - DOM APIs (Window, Document, etc.)
  - Node.js globals (process, Buffer)
  - Verify no TS2304 errors for standard library

## Completed
(none yet)
