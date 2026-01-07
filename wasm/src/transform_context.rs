//! Transform Context - Projection Layer for AST Transforms
//!
//! This module implements the "Projection Layer" approach to separate Transform logic
//! from Print logic. Since our ThinNode AST is read-only (Data-Oriented Design), we
//! cannot mutate nodes. Instead, we create lightweight "transform directives" that tell
//! the printer how to emit a node differently than its literal AST representation.
//!
//! # Architecture
//!
//! ## Phase 1: Lowering Pass
//! The `LoweringPass` walks the AST and produces `TransformDirective`s for nodes that
//! need transformation (ES5 classes, module exports, etc.). These directives are stored
//! in a `TransformContext`.
//!
//! ## Phase 2: Print Pass
//! The `ThinPrinter` checks the `TransformContext` before emitting each node. If a
//! directive exists, it uses that to guide emission; otherwise it emits the node directly.
//!
//! # Benefits
//!
//! - ✅ AST remains read-only (DOD compliance)
//! - ✅ Transforms are testable independently from printing
//! - ✅ No intermediate allocations (just a HashMap of directives)
//! - ✅ Composable transforms via `Chain` directive
//! - ✅ Clear separation of concerns

use crate::parser::NodeIndex;
use rustc_hash::FxHashMap;

/// Transform directives tell the printer how to emit a node differently
/// than its literal AST representation.
#[derive(Debug, Clone)]
pub enum TransformDirective {
    /// Emit node as-is (identity transform)
    Identity,

    /// ES5 Class: Transform class to IIFE pattern
    ///
    /// ```typescript
    /// class Foo { constructor(x) { this.x = x; } }
    /// ```
    ///
    /// Becomes:
    ///
    /// ```javascript
    /// var Foo = /** @class */ (function () {
    ///     function Foo(x) { this.x = x; }
    ///     return Foo;
    /// }());
    /// ```
    ES5Class {
        /// Original class node index
        class_node: NodeIndex,
        /// Extracted class name (for the constructor function)
        class_name: Option<String>,
        /// Heritage clause (extends)
        heritage: Option<NodeIndex>,
        /// Members to transform
        members: Vec<NodeIndex>,
    },

    /// ES5 Namespace: Transform namespace to IIFE pattern
    ES5Namespace {
        /// Original namespace node index
        namespace_node: NodeIndex,
    },

    /// ES5 Enum: Transform enum to IIFE pattern
    ES5Enum {
        /// Original enum node index
        enum_node: NodeIndex,
    },

    /// CommonJS Export: Wrap declaration with exports assignment
    ///
    /// ```typescript
    /// export class Foo {}
    /// ```
    ///
    /// Becomes:
    ///
    /// ```javascript
    /// class Foo {}
    /// exports.Foo = Foo;
    /// ```
    CommonJSExport {
        /// Names to export
        names: Vec<String>,
        /// Whether this is a default export
        is_default: bool,
        /// The inner directive to apply first
        inner: Box<TransformDirective>,
    },

    /// CommonJS default export for anonymous class/function declarations.
    ///
    /// ```typescript
    /// export default function () {}
    /// ```
    ///
    /// Becomes:
    ///
    /// ```javascript
    /// exports.default = function () {};
    /// ```
    CommonJSExportDefaultExpr,

    /// ES5 Arrow Function: Transform arrow to regular function
    ///
    /// ```typescript
    /// const f = (x) => x + 1;
    /// ```
    ///
    /// Becomes:
    ///
    /// ```javascript
    /// var f = function (x) { return x + 1; };
    /// ```
    ES5ArrowFunction {
        /// Original arrow function node
        arrow_node: NodeIndex,
        /// Whether this captures 'this' (needs _this = this)
        captures_this: bool,
    },

    /// ES5 Async/Await: Transform to __awaiter helper
    ES5AsyncFunction {
        /// Original async function node
        function_node: NodeIndex,
    },

    /// ES5 For-Of: Transform to iterator loop with __values helper
    ES5ForOf {
        /// Original for-of statement node
        for_of_node: NodeIndex,
    },

    /// ES5 Object Literal: Transform computed properties and spread to assignments
    ///
    /// ```typescript
    /// const obj = { a: 1, [k]: 2, ...rest };
    /// ```
    ///
    /// Becomes:
    ///
    /// ```javascript
    /// var obj = (_a = { a: 1 }, _a[k] = 2, Object.assign(_a, rest), _a);
    /// ```
    ES5ObjectLiteral {
        /// Original object literal node
        object_literal: NodeIndex,
    },

    /// Module Wrapper: Wrap entire file for AMD/System/UMD
    ModuleWrapper {
        /// Module format (AMD, System, UMD)
        format: ModuleFormat,
        /// Dependencies
        dependencies: Vec<String>,
        /// Module body nodes
        body: Vec<NodeIndex>,
    },

    /// Chain multiple transforms (composition)
    /// Transforms are applied in order.
    Chain(Vec<TransformDirective>),
}

/// Module formats that require wrapping transforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleFormat {
    CommonJS,
    AMD,
    System,
    UMD,
    ES6,
}

/// Transform context maps node indices to their transform directives
#[derive(Clone)]
pub struct TransformContext {
    /// Map of NodeIndex -> TransformDirective
    /// Only contains entries for nodes that need transformation
    directives: FxHashMap<NodeIndex, TransformDirective>,
}

impl TransformContext {
    /// Create a new empty transform context
    pub fn new() -> Self {
        TransformContext {
            directives: FxHashMap::default(),
        }
    }

    /// Register a transform directive for a node
    pub fn insert(&mut self, node: NodeIndex, directive: TransformDirective) {
        self.directives.insert(node, directive);
    }

    /// Get the transform directive for a node, if any
    pub fn get(&self, node: NodeIndex) -> Option<&TransformDirective> {
        self.directives.get(&node)
    }

    /// Check if a node has a transform directive
    pub fn has_transform(&self, node: NodeIndex) -> bool {
        self.directives.contains_key(&node)
    }

    /// Clear all directives (for reuse)
    pub fn clear(&mut self) {
        self.directives.clear();
    }

    /// Get the number of registered transforms
    pub fn len(&self) -> usize {
        self.directives.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.directives.is_empty()
    }
}

impl Default for TransformContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_context_basic() {
        let mut ctx = TransformContext::new();
        let node = NodeIndex(42);

        // Initially empty
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert!(!ctx.has_transform(node));

        // Insert a directive
        ctx.insert(node, TransformDirective::Identity);
        assert!(!ctx.is_empty());
        assert_eq!(ctx.len(), 1);
        assert!(ctx.has_transform(node));

        // Get the directive
        let directive = ctx.get(node);
        assert!(directive.is_some());
        assert!(matches!(directive.unwrap(), TransformDirective::Identity));

        // Clear
        ctx.clear();
        assert!(ctx.is_empty());
        assert!(!ctx.has_transform(node));
    }

    #[test]
    fn test_es5_class_directive() {
        let mut ctx = TransformContext::new();
        let class_node = NodeIndex(10);

        ctx.insert(
            class_node,
            TransformDirective::ES5Class {
                class_node,
                class_name: Some("MyClass".to_string()),
                heritage: None,
                members: vec![],
            },
        );

        let directive = ctx.get(class_node).unwrap();
        match directive {
            TransformDirective::ES5Class { class_name, .. } => {
                assert_eq!(class_name.as_ref().unwrap(), "MyClass");
            }
            _ => panic!("Expected ES5Class directive"),
        }
    }

    #[test]
    fn test_commonjs_export_chain() {
        let mut ctx = TransformContext::new();
        let class_node = NodeIndex(10);

        // Chain ES5 class transform with CommonJS export
        let directive = TransformDirective::CommonJSExport {
            names: vec!["MyClass".to_string()],
            is_default: false,
            inner: Box::new(TransformDirective::ES5Class {
                class_node,
                class_name: Some("MyClass".to_string()),
                heritage: None,
                members: vec![],
            }),
        };

        ctx.insert(class_node, directive);

        let retrieved = ctx.get(class_node).unwrap();
        match retrieved {
            TransformDirective::CommonJSExport { names, inner, .. } => {
                assert_eq!(names, &["MyClass".to_string()]);
                assert!(matches!(**inner, TransformDirective::ES5Class { .. }));
            }
            _ => panic!("Expected CommonJSExport directive"),
        }
    }
}
