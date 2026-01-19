//! This Type Solver
//!
//! Resolves and infers 'this' types in various contexts including:
//! - Method return type resolution for fluent APIs
//! - this parameter checking
//! - Arrow function vs regular function this binding
//! - Static context this validation
//! - This type narrowing in type guards

use std::collections::HashMap;

/// Unique identifier for types
pub type TypeId = u64;

/// Diagnostic for this type errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThisTypeDiagnostic {
    pub message: String,
    pub code: u32,
    pub span: Option<TextSpan>,
}

/// Text span for error locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan {
    pub start: u32,
    pub end: u32,
}

/// Diagnostic codes for this-related errors
pub mod error_codes {
    /// 'this' cannot be referenced in a static property initializer
    pub const THIS_IN_STATIC_PROPERTY: u32 = 2334;
    /// 'this' cannot be referenced in current location
    pub const THIS_NOT_ALLOWED: u32 = 2331;
    /// The containing arrow function captures the outer 'this'
    pub const ARROW_CAPTURES_THIS: u32 = 2332;
    /// 'this' implicitly has type 'any' because it does not have a type annotation
    pub const THIS_IMPLICIT_ANY: u32 = 2683;
    /// The 'this' context of type X is not assignable to method's 'this' of type Y
    pub const THIS_CONTEXT_MISMATCH: u32 = 2684;
    /// A 'this' type is available only in a non-static member of a class or interface
    pub const THIS_TYPE_NOT_AVAILABLE: u32 = 2526;
    /// Cannot find name 'this'
    pub const CANNOT_FIND_THIS: u32 = 2355;
    /// Method's 'this' type is not assignable to class 'this'
    pub const INCOMPATIBLE_THIS_PARAMETER: u32 = 2685;
    /// The 'this' types of each signature are incompatible
    pub const INCOMPATIBLE_THIS_TYPES: u32 = 2686;
}

/// Represents a function context for this resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// Regular function declaration/expression
    Regular,
    /// Arrow function
    Arrow,
    /// Method in class/object
    Method,
    /// Constructor
    Constructor,
    /// Getter
    Getter,
    /// Setter
    Setter,
}

/// Information about a class for this resolution
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub id: TypeId,
    pub name: String,
    pub base_class: Option<TypeId>,
    pub is_abstract: bool,
    /// Constructor type (typeof Class)
    pub constructor_type: TypeId,
}

impl ClassInfo {
    pub fn new(id: TypeId, name: impl Into<String>) -> Self {
        ClassInfo {
            id,
            name: name.into(),
            base_class: None,
            is_abstract: false,
            constructor_type: id, // Default to same as class type
        }
    }

    pub fn with_constructor_type(mut self, constructor_type: TypeId) -> Self {
        self.constructor_type = constructor_type;
        self
    }

    pub fn extends(mut self, base: TypeId) -> Self {
        self.base_class = Some(base);
        self
    }
}

/// Information about an interface for this resolution
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub id: TypeId,
    pub name: String,
    pub extended_interfaces: Vec<TypeId>,
}

impl InterfaceInfo {
    pub fn new(id: TypeId, name: impl Into<String>) -> Self {
        InterfaceInfo {
            id,
            name: name.into(),
            extended_interfaces: Vec::new(),
        }
    }

    pub fn extends(mut self, interface: TypeId) -> Self {
        self.extended_interfaces.push(interface);
        self
    }
}

/// The resolved 'this' type value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedThis {
    /// Polymorphic 'this' bound to a class
    ClassThis(TypeId),
    /// Polymorphic 'this' bound to an interface
    InterfaceThis(TypeId),
    /// Explicit type from this parameter
    Explicit(TypeId),
    /// void (this: void)
    Void,
    /// any (loose mode or untyped)
    Any,
    /// unknown (strict mode unresolved)
    Unknown,
    /// Static context - this is the constructor type
    StaticThis(TypeId),
    /// Error - this not available
    Error(String),
}

impl ResolvedThis {
    /// Check if this type can be used for member access
    pub fn can_access_members(&self) -> bool {
        !matches!(self, ResolvedThis::Void | ResolvedThis::Error(_))
    }

    /// Get the underlying type ID if available
    pub fn type_id(&self) -> Option<TypeId> {
        match self {
            ResolvedThis::ClassThis(id) => Some(*id),
            ResolvedThis::InterfaceThis(id) => Some(*id),
            ResolvedThis::Explicit(id) => Some(*id),
            ResolvedThis::StaticThis(id) => Some(*id),
            _ => None,
        }
    }
}

/// Stack of scopes for tracking this binding
#[derive(Debug, Clone)]
pub struct ThisScope {
    pub kind: ThisScopeKind,
    pub this_type: ResolvedThis,
    /// Whether this is inside an arrow function
    pub is_arrow: bool,
}

/// Kind of scope for this tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisScopeKind {
    Global,
    Module,
    Class,
    Interface,
    Function,
    StaticMember,
    InstanceMember,
    Constructor,
}

/// This type solver - resolves 'this' in various contexts
#[derive(Debug)]
pub struct ThisTypeSolver {
    /// Registered classes
    classes: HashMap<TypeId, ClassInfo>,
    /// Registered interfaces
    interfaces: HashMap<TypeId, InterfaceInfo>,
    /// Current scope stack
    scope_stack: Vec<ThisScope>,
    /// Diagnostics collected during resolution
    diagnostics: Vec<ThisTypeDiagnostic>,
    /// Whether to use strict mode (noImplicitThis)
    strict_this: bool,
}

impl ThisTypeSolver {
    pub fn new() -> Self {
        ThisTypeSolver {
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            scope_stack: vec![ThisScope {
                kind: ThisScopeKind::Global,
                this_type: ResolvedThis::Any,
                is_arrow: false,
            }],
            diagnostics: Vec::new(),
            strict_this: false,
        }
    }

    /// Enable noImplicitThis checking
    pub fn with_strict_this(mut self) -> Self {
        self.strict_this = true;
        self
    }

    /// Register a class for this resolution
    pub fn register_class(&mut self, class: ClassInfo) {
        self.classes.insert(class.id, class);
    }

    /// Register an interface for this resolution
    pub fn register_interface(&mut self, interface: InterfaceInfo) {
        self.interfaces.insert(interface.id, interface);
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &[ThisTypeDiagnostic] {
        &self.diagnostics
    }

    /// Clear diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Enter a class scope
    pub fn enter_class(&mut self, class_id: TypeId) {
        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::Class,
            this_type: ResolvedThis::ClassThis(class_id),
            is_arrow: false,
        });
    }

    /// Enter an interface scope
    pub fn enter_interface(&mut self, interface_id: TypeId) {
        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::Interface,
            this_type: ResolvedThis::InterfaceThis(interface_id),
            is_arrow: false,
        });
    }

    /// Enter an instance member (method, property initializer)
    pub fn enter_instance_member(&mut self, class_id: TypeId) {
        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::InstanceMember,
            this_type: ResolvedThis::ClassThis(class_id),
            is_arrow: false,
        });
    }

    /// Enter a static member
    pub fn enter_static_member(&mut self, class_id: TypeId) {
        let constructor_type = self.classes.get(&class_id)
            .map(|c| c.constructor_type)
            .unwrap_or(class_id);

        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::StaticMember,
            this_type: ResolvedThis::StaticThis(constructor_type),
            is_arrow: false,
        });
    }

    /// Enter a constructor
    pub fn enter_constructor(&mut self, class_id: TypeId) {
        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::Constructor,
            this_type: ResolvedThis::ClassThis(class_id),
            is_arrow: false,
        });
    }

    /// Enter a function (regular or arrow)
    pub fn enter_function(&mut self, kind: FunctionKind, explicit_this: Option<TypeId>) {
        let (this_type, is_arrow) = match kind {
            FunctionKind::Arrow => {
                // Arrow functions capture lexical this
                let parent_this = self.current_this();
                (parent_this, true)
            }
            FunctionKind::Regular => {
                // Regular functions have their own this
                if let Some(explicit) = explicit_this {
                    (ResolvedThis::Explicit(explicit), false)
                } else if self.strict_this {
                    (ResolvedThis::Unknown, false)
                } else {
                    (ResolvedThis::Any, false)
                }
            }
            FunctionKind::Method | FunctionKind::Getter | FunctionKind::Setter => {
                // Methods use class/interface this
                if let Some(explicit) = explicit_this {
                    (ResolvedThis::Explicit(explicit), false)
                } else {
                    (self.current_this(), false)
                }
            }
            FunctionKind::Constructor => {
                // Constructor uses class this
                (self.current_this(), false)
            }
        };

        self.scope_stack.push(ThisScope {
            kind: ThisScopeKind::Function,
            this_type,
            is_arrow,
        });
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Get the current 'this' type
    pub fn current_this(&self) -> ResolvedThis {
        self.scope_stack.last()
            .map(|s| s.this_type.clone())
            .unwrap_or(ResolvedThis::Any)
    }

    /// Resolve 'this' at the current location
    pub fn resolve_this(&mut self, span: Option<TextSpan>) -> ResolvedThis {
        let current_scope = self.scope_stack.last();

        match current_scope {
            Some(scope) => {
                match scope.kind {
                    ThisScopeKind::Global | ThisScopeKind::Module => {
                        if self.strict_this {
                            self.diagnostics.push(ThisTypeDiagnostic {
                                message: "'this' implicitly has type 'any' because it does not have a type annotation".to_string(),
                                code: error_codes::THIS_IMPLICIT_ANY,
                                span,
                            });
                            ResolvedThis::Unknown
                        } else {
                            ResolvedThis::Any
                        }
                    }
                    ThisScopeKind::StaticMember => {
                        // In static context, 'this' refers to constructor
                        scope.this_type.clone()
                    }
                    _ => scope.this_type.clone(),
                }
            }
            None => ResolvedThis::Any,
        }
    }

    /// Check if 'this' type annotation is available at current location
    pub fn check_this_type_annotation(&mut self, span: Option<TextSpan>) -> bool {
        let current = self.scope_stack.last();

        match current {
            Some(scope) => {
                match scope.kind {
                    ThisScopeKind::Class | ThisScopeKind::Interface |
                    ThisScopeKind::InstanceMember | ThisScopeKind::Constructor => true,
                    ThisScopeKind::StaticMember => {
                        self.diagnostics.push(ThisTypeDiagnostic {
                            message: "A 'this' type is available only in a non-static member of a class or interface".to_string(),
                            code: error_codes::THIS_TYPE_NOT_AVAILABLE,
                            span,
                        });
                        false
                    }
                    ThisScopeKind::Global | ThisScopeKind::Module | ThisScopeKind::Function => {
                        // Check if we're inside a method through parent scopes
                        let in_class = self.scope_stack.iter()
                            .any(|s| matches!(s.kind, ThisScopeKind::Class | ThisScopeKind::Interface));
                        if !in_class {
                            self.diagnostics.push(ThisTypeDiagnostic {
                                message: "A 'this' type is available only in a non-static member of a class or interface".to_string(),
                                code: error_codes::THIS_TYPE_NOT_AVAILABLE,
                                span,
                            });
                        }
                        in_class
                    }
                }
            }
            None => {
                self.diagnostics.push(ThisTypeDiagnostic {
                    message: "A 'this' type is available only in a non-static member of a class or interface".to_string(),
                    code: error_codes::THIS_TYPE_NOT_AVAILABLE,
                    span,
                });
                false
            }
        }
    }

    /// Check this access in static context
    pub fn check_this_in_static(&mut self, span: Option<TextSpan>) -> bool {
        let in_static = self.scope_stack.iter()
            .any(|s| s.kind == ThisScopeKind::StaticMember);

        // Check if there's an arrow function that captures outer this
        if in_static {
            let arrow_captures = self.scope_stack.iter()
                .any(|s| s.is_arrow);

            if !arrow_captures {
                // Direct this access in static property initializer is error
                let in_static_initializer = self.scope_stack.last()
                    .map(|s| s.kind == ThisScopeKind::StaticMember)
                    .unwrap_or(false);

                if in_static_initializer {
                    self.diagnostics.push(ThisTypeDiagnostic {
                        message: "'this' cannot be referenced in a static property initializer".to_string(),
                        code: error_codes::THIS_IN_STATIC_PROPERTY,
                        span,
                    });
                    return false;
                }
            }
        }

        true
    }

    /// Check this parameter compatibility
    pub fn check_this_parameter(
        &mut self,
        required_this: &ResolvedThis,
        provided_this: &ResolvedThis,
        span: Option<TextSpan>,
    ) -> bool {
        let compatible = match (required_this, provided_this) {
            // Any is always compatible
            (_, ResolvedThis::Any) | (ResolvedThis::Any, _) => true,
            // Void requires void
            (ResolvedThis::Void, ResolvedThis::Void) => true,
            (ResolvedThis::Void, _) => false,
            // Same class this
            (ResolvedThis::ClassThis(a), ResolvedThis::ClassThis(b)) => {
                a == b || self.is_subclass(*b, *a)
            }
            // Same interface this
            (ResolvedThis::InterfaceThis(a), ResolvedThis::InterfaceThis(b)) => a == b,
            // Class implements interface
            (ResolvedThis::InterfaceThis(iface), ResolvedThis::ClassThis(class)) => {
                // Would need to check if class implements interface
                // Simplified: assume compatible for now
                true
            }
            // Explicit types
            (ResolvedThis::Explicit(a), ResolvedThis::Explicit(b)) => a == b,
            (ResolvedThis::Explicit(a), ResolvedThis::ClassThis(b)) => a == b,
            (ResolvedThis::ClassThis(a), ResolvedThis::Explicit(b)) => a == b,
            // Static this
            (ResolvedThis::StaticThis(a), ResolvedThis::StaticThis(b)) => a == b,
            // Error cases
            (ResolvedThis::Error(_), _) | (_, ResolvedThis::Error(_)) => false,
            // Unknown is not compatible with specific types
            (ResolvedThis::Unknown, _) | (_, ResolvedThis::Unknown) => false,
            // Other combinations
            _ => false,
        };

        if !compatible {
            self.diagnostics.push(ThisTypeDiagnostic {
                message: format!(
                    "The 'this' context of type '{}' is not assignable to method's 'this' of type '{}'",
                    self.format_this(provided_this),
                    self.format_this(required_this)
                ),
                code: error_codes::THIS_CONTEXT_MISMATCH,
                span,
            });
        }

        compatible
    }

    /// Check if class_id is a subclass of base_id
    fn is_subclass(&self, class_id: TypeId, base_id: TypeId) -> bool {
        if class_id == base_id {
            return true;
        }

        let class = match self.classes.get(&class_id) {
            Some(c) => c,
            None => return false,
        };

        if let Some(parent) = class.base_class {
            self.is_subclass(parent, base_id)
        } else {
            false
        }
    }

    /// Format a this type for error messages
    fn format_this(&self, this: &ResolvedThis) -> String {
        match this {
            ResolvedThis::ClassThis(id) => {
                self.classes.get(id)
                    .map(|c| format!("this (in class {})", c.name))
                    .unwrap_or_else(|| "this".to_string())
            }
            ResolvedThis::InterfaceThis(id) => {
                self.interfaces.get(id)
                    .map(|i| format!("this (in interface {})", i.name))
                    .unwrap_or_else(|| "this".to_string())
            }
            ResolvedThis::Explicit(id) => format!("type #{}", id),
            ResolvedThis::Void => "void".to_string(),
            ResolvedThis::Any => "any".to_string(),
            ResolvedThis::Unknown => "unknown".to_string(),
            ResolvedThis::StaticThis(id) => {
                self.classes.get(id)
                    .map(|c| format!("typeof {}", c.name))
                    .unwrap_or_else(|| "typeof <class>".to_string())
            }
            ResolvedThis::Error(msg) => format!("error: {}", msg),
        }
    }

    /// Resolve method return type considering polymorphic this
    pub fn resolve_method_return_type(
        &self,
        method_returns_this: bool,
        explicit_return_type: Option<TypeId>,
        call_site_class: TypeId,
    ) -> Option<TypeId> {
        if method_returns_this {
            // Method returns 'this', substitute with actual class
            Some(call_site_class)
        } else {
            explicit_return_type
        }
    }

    /// Narrow 'this' type based on type guard
    pub fn narrow_this(&self, guard_type: TypeId) -> ResolvedThis {
        // This narrowing happens when a type guard narrows 'this'
        // e.g., if (this instanceof DerivedClass) { ... }
        ResolvedThis::ClassThis(guard_type)
    }
}

impl Default for ThisTypeSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for fluent API method chains
#[derive(Debug)]
pub struct FluentApiBuilder {
    class_id: TypeId,
    methods: Vec<FluentMethodInfo>,
}

/// Information about a fluent method
#[derive(Debug, Clone)]
pub struct FluentMethodInfo {
    pub name: String,
    pub returns_this: bool,
    pub return_type: Option<TypeId>,
}

impl FluentApiBuilder {
    pub fn new(class_id: TypeId) -> Self {
        FluentApiBuilder {
            class_id,
            methods: Vec::new(),
        }
    }

    /// Add a method that returns 'this' for chaining
    pub fn add_fluent_method(mut self, name: impl Into<String>) -> Self {
        self.methods.push(FluentMethodInfo {
            name: name.into(),
            returns_this: true,
            return_type: None,
        });
        self
    }

    /// Add a method with explicit return type (breaks chain)
    pub fn add_terminal_method(mut self, name: impl Into<String>, return_type: TypeId) -> Self {
        self.methods.push(FluentMethodInfo {
            name: name.into(),
            returns_this: false,
            return_type: Some(return_type),
        });
        self
    }

    /// Get method return type at call site
    pub fn get_method_return_type(&self, method_name: &str, actual_class: TypeId) -> Option<TypeId> {
        for method in &self.methods {
            if method.name == method_name {
                if method.returns_this {
                    // Substitute 'this' with actual class for polymorphic return
                    return Some(actual_class);
                } else {
                    return method.return_type;
                }
            }
        }
        None
    }

    /// Check method chain validity
    pub fn check_chain(&self, chain: &[&str], solver: &mut ThisTypeSolver) -> Result<TypeId, String> {
        let mut current_type = self.class_id;

        for method_name in chain {
            if let Some(return_type) = self.get_method_return_type(method_name, current_type) {
                current_type = return_type;
            } else {
                return Err(format!("Method '{}' not found on type", method_name));
            }
        }

        Ok(current_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_this_solver_creation() {
        let solver = ThisTypeSolver::new();
        assert!(solver.diagnostics().is_empty());
        assert!(matches!(solver.current_this(), ResolvedThis::Any));
    }

    #[test]
    fn test_enter_class_scope() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_class(1);

        assert!(matches!(solver.current_this(), ResolvedThis::ClassThis(1)));
    }

    #[test]
    fn test_enter_interface_scope() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_interface(2);

        assert!(matches!(solver.current_this(), ResolvedThis::InterfaceThis(2)));
    }

    #[test]
    fn test_static_member_this() {
        let mut solver = ThisTypeSolver::new();
        solver.register_class(ClassInfo::new(1, "Foo").with_constructor_type(100));
        solver.enter_class(1);
        solver.enter_static_member(1);

        assert!(matches!(solver.current_this(), ResolvedThis::StaticThis(100)));
    }

    #[test]
    fn test_arrow_function_captures_this() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_class(1);
        solver.enter_instance_member(1);
        solver.enter_function(FunctionKind::Arrow, None);

        // Arrow function captures parent's this
        assert!(matches!(solver.current_this(), ResolvedThis::ClassThis(1)));
    }

    #[test]
    fn test_regular_function_has_own_this() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_class(1);
        solver.enter_instance_member(1);
        solver.enter_function(FunctionKind::Regular, None);

        // Regular function has its own this (any in non-strict)
        assert!(matches!(solver.current_this(), ResolvedThis::Any));
    }

    #[test]
    fn test_strict_this_mode() {
        let mut solver = ThisTypeSolver::new().with_strict_this();
        solver.enter_function(FunctionKind::Regular, None);

        // In strict mode, untyped this is unknown
        assert!(matches!(solver.current_this(), ResolvedThis::Unknown));
    }

    #[test]
    fn test_explicit_this_parameter() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_function(FunctionKind::Regular, Some(42));

        assert!(matches!(solver.current_this(), ResolvedThis::Explicit(42)));
    }

    #[test]
    fn test_resolve_this_in_global() {
        let mut solver = ThisTypeSolver::new().with_strict_this();
        solver.resolve_this(None);

        assert!(solver.diagnostics().iter().any(|d| d.code == error_codes::THIS_IMPLICIT_ANY));
    }

    #[test]
    fn test_this_type_not_in_static() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_class(1);
        solver.enter_static_member(1);

        let available = solver.check_this_type_annotation(None);
        assert!(!available);
        assert!(solver.diagnostics().iter().any(|d| d.code == error_codes::THIS_TYPE_NOT_AVAILABLE));
    }

    #[test]
    fn test_this_parameter_compatibility() {
        let mut solver = ThisTypeSolver::new();

        // Same class this is compatible
        let this_a = ResolvedThis::ClassThis(1);
        let this_b = ResolvedThis::ClassThis(1);
        assert!(solver.check_this_parameter(&this_a, &this_b, None));

        // Different class this is not compatible (without inheritance)
        solver.clear_diagnostics();
        let this_c = ResolvedThis::ClassThis(2);
        assert!(!solver.check_this_parameter(&this_a, &this_c, None));
        assert!(solver.diagnostics().iter().any(|d| d.code == error_codes::THIS_CONTEXT_MISMATCH));
    }

    #[test]
    fn test_subclass_this_compatibility() {
        let mut solver = ThisTypeSolver::new();

        // Register class hierarchy: Derived extends Base
        solver.register_class(ClassInfo::new(1, "Base"));
        solver.register_class(ClassInfo::new(2, "Derived").extends(1));

        // Derived this is compatible with Base this requirement
        let base_this = ResolvedThis::ClassThis(1);
        let derived_this = ResolvedThis::ClassThis(2);
        assert!(solver.check_this_parameter(&base_this, &derived_this, None));
    }

    #[test]
    fn test_fluent_api_builder() {
        let builder = FluentApiBuilder::new(1)
            .add_fluent_method("setName")
            .add_fluent_method("setAge")
            .add_terminal_method("build", 100);

        // Fluent methods return the actual class type
        assert_eq!(builder.get_method_return_type("setName", 2), Some(2));

        // Terminal method returns explicit type
        assert_eq!(builder.get_method_return_type("build", 2), Some(100));

        // Unknown method returns None
        assert_eq!(builder.get_method_return_type("unknown", 2), None);
    }

    #[test]
    fn test_fluent_chain_check() {
        let builder = FluentApiBuilder::new(1)
            .add_fluent_method("setName")
            .add_fluent_method("setAge")
            .add_terminal_method("build", 100);

        let mut solver = ThisTypeSolver::new();

        // Valid chain
        let result = builder.check_chain(&["setName", "setAge", "build"], &mut solver);
        assert_eq!(result, Ok(100));

        // Invalid chain (unknown method)
        let result = builder.check_chain(&["setName", "unknown"], &mut solver);
        assert!(result.is_err());
    }

    #[test]
    fn test_polymorphic_return_type() {
        let solver = ThisTypeSolver::new();

        // When base class method returns 'this', derived class gets derived type
        let return_type = solver.resolve_method_return_type(true, None, 42);
        assert_eq!(return_type, Some(42));

        // Explicit return type is preserved
        let return_type = solver.resolve_method_return_type(false, Some(100), 42);
        assert_eq!(return_type, Some(100));
    }

    #[test]
    fn test_narrow_this() {
        let solver = ThisTypeSolver::new();
        let narrowed = solver.narrow_this(42);
        assert!(matches!(narrowed, ResolvedThis::ClassThis(42)));
    }

    #[test]
    fn test_scope_exit() {
        let mut solver = ThisTypeSolver::new();
        solver.enter_class(1);
        solver.enter_instance_member(1);

        assert!(matches!(solver.current_this(), ResolvedThis::ClassThis(1)));

        solver.exit_scope();
        solver.exit_scope();

        // Back to global
        assert!(matches!(solver.current_this(), ResolvedThis::Any));
    }

    #[test]
    fn test_resolved_this_can_access_members() {
        assert!(ResolvedThis::ClassThis(1).can_access_members());
        assert!(ResolvedThis::InterfaceThis(1).can_access_members());
        assert!(ResolvedThis::Any.can_access_members());
        assert!(!ResolvedThis::Void.can_access_members());
        assert!(!ResolvedThis::Error("test".to_string()).can_access_members());
    }

    #[test]
    fn test_resolved_this_type_id() {
        assert_eq!(ResolvedThis::ClassThis(1).type_id(), Some(1));
        assert_eq!(ResolvedThis::InterfaceThis(2).type_id(), Some(2));
        assert_eq!(ResolvedThis::Explicit(3).type_id(), Some(3));
        assert_eq!(ResolvedThis::StaticThis(4).type_id(), Some(4));
        assert_eq!(ResolvedThis::Void.type_id(), None);
        assert_eq!(ResolvedThis::Any.type_id(), None);
    }
}
