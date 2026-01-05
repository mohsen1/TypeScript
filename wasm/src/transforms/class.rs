//! Class Transforms
//!
//! Transforms ES2015 classes to ES5 prototype-based code:
//! - class → function + prototype
//! - constructor → function body
//! - methods → prototype assignments
//! - static methods → function properties
//! - extends → __extends helper
//! - super → Parent.prototype.method.call(this)
//! - property initializers → constructor assignments
//! - accessors → Object.defineProperty

use super::{TransformContext, Transformer};
use crate::parser::{Node, NodeIndex};

/// Class transformation state
pub struct ClassTransformer {
    /// Current class name being transformed
    _current_class_name: Option<String>,
    /// Whether current class extends another
    _has_heritage: bool,
}

impl ClassTransformer {
    pub fn new() -> Self {
        ClassTransformer {
            _current_class_name: None,
            _has_heritage: false,
        }
    }

    /// Transform a class declaration to ES5
    ///
    /// ```ts
    /// class Foo extends Bar {
    ///     constructor(x) { super(x); this.x = x; }
    ///     method() { return this.x; }
    ///     static create() { return new Foo(1); }
    /// }
    /// ```
    ///
    /// Becomes:
    ///
    /// ```js
    /// var Foo = /** @class */ (function (_super) {
    ///     __extends(Foo, _super);
    ///     function Foo(x) {
    ///         var _this = _super.call(this, x) || this;
    ///         _this.x = x;
    ///         return _this;
    ///     }
    ///     Foo.prototype.method = function () { return this.x; };
    ///     Foo.create = function () { return new Foo(1); };
    ///     return Foo;
    /// }(Bar));
    /// ```
    pub fn transform_class_declaration(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Extract class information
        let has_heritage = match ctx.arena.get(node_idx) {
            Some(Node::ClassDeclaration(class_decl)) => {
                class_decl.heritage_clauses.as_ref().map_or(false, |hc| !hc.nodes.is_empty())
            }
            _ => return None,
        };

        if has_heritage {
            ctx.helpers_needed.extends = true;
        }

        // In a full implementation, we would:
        // 1. Create an IIFE wrapper
        // 2. Add __extends call if there's a heritage clause
        // 3. Transform constructor to function
        // 4. Transform methods to prototype assignments
        // 5. Transform static methods to function properties
        // 6. Transform property initializers to constructor assignments
        // 7. Transform accessors to Object.defineProperty

        // For now, mark that we need the extends helper and visit children
        None
    }

    /// Transform a super call
    /// `super(x)` → `_super.call(this, x) || this`
    pub fn transform_super_call(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Transform super call to _super.call(this, ...)
        None
    }

    /// Transform a super property access
    /// `super.method()` → `_super.prototype.method.call(this)`
    pub fn transform_super_property(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Transform super.x to _super.prototype.x
        None
    }

    /// Transform a method declaration to prototype assignment
    /// `method() {}` → `Class.prototype.method = function() {}`
    pub fn transform_method_to_prototype(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        None
    }

    /// Transform a static method to function property
    /// `static create() {}` → `Class.create = function() {}`
    pub fn transform_static_method(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        None
    }

    /// Transform getter/setter to Object.defineProperty
    /// `get x() {}` → `Object.defineProperty(Class.prototype, "x", { get: function() {}, ... })`
    pub fn transform_accessor(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        None
    }

    /// Transform property initializer to constructor assignment
    /// `x = 1;` in class body → `this.x = 1;` in constructor
    pub fn transform_property_initializer(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        None
    }
}

/// Node kind for class-related transforms
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassNodeKind {
    ClassDeclaration,
    MethodDeclaration,
    PropertyDeclaration,
    GetAccessorDeclaration,
    SetAccessorDeclaration,
    ConstructorDeclaration,
    CallExpression,
    Other,
}

fn get_class_node_kind(node_idx: NodeIndex, ctx: &TransformContext) -> ClassNodeKind {
    match ctx.arena.get(node_idx) {
        Some(Node::ClassDeclaration(_)) => ClassNodeKind::ClassDeclaration,
        Some(Node::MethodDeclaration(_)) => ClassNodeKind::MethodDeclaration,
        Some(Node::PropertyDeclaration(_)) => ClassNodeKind::PropertyDeclaration,
        Some(Node::GetAccessorDeclaration(_)) => ClassNodeKind::GetAccessorDeclaration,
        Some(Node::SetAccessorDeclaration(_)) => ClassNodeKind::SetAccessorDeclaration,
        Some(Node::ConstructorDeclaration(_)) => ClassNodeKind::ConstructorDeclaration,
        Some(Node::CallExpression(_)) => ClassNodeKind::CallExpression,
        _ => ClassNodeKind::Other,
    }
}

impl Transformer for ClassTransformer {
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        let kind = get_class_node_kind(node_idx, ctx);

        match kind {
            ClassNodeKind::ClassDeclaration => {
                return self.transform_class_declaration(node_idx, ctx);
            }
            _ => {}
        }

        self.visit_children(node_idx, ctx);
        None
    }

    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) {
        // Collect children to avoid borrow issues
        let children = collect_class_children(node_idx, ctx);

        for child_idx in children {
            self.visit_node(child_idx, ctx);
        }
    }
}

fn collect_class_children(node_idx: NodeIndex, ctx: &TransformContext) -> Vec<NodeIndex> {
    let mut children = Vec::new();

    match ctx.arena.get(node_idx) {
        Some(Node::ClassDeclaration(class_decl)) => {
            for member in &class_decl.members.nodes {
                children.push(*member);
            }
        }
        Some(Node::MethodDeclaration(method)) => {
            if !method.body.is_none() {
                children.push(method.body);
            }
        }
        Some(Node::ConstructorDeclaration(ctor)) => {
            if !ctor.body.is_none() {
                children.push(ctor.body);
            }
        }
        Some(Node::GetAccessorDeclaration(getter)) => {
            if !getter.body.is_none() {
                children.push(getter.body);
            }
        }
        Some(Node::SetAccessorDeclaration(setter)) => {
            if !setter.body.is_none() {
                children.push(setter.body);
            }
        }
        Some(Node::Block(block)) => {
            for stmt in &block.statements.nodes {
                children.push(*stmt);
            }
        }
        Some(Node::SourceFile(sf)) => {
            for stmt in &sf.statements.nodes {
                children.push(*stmt);
            }
        }
        _ => {}
    }

    children
}

#[cfg(test)]
#[path = "class_tests.rs"]
mod class_tests;
