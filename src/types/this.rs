//! TypeScript 'this' Type Implementation
//!
//! This module provides polymorphic 'this' type support for TypeScript classes and interfaces.
//! The 'this' type refers to the type of the current class instance and supports:
//!
//! - Polymorphic this in class/interface method return types
//! - ThisType<T> utility type for object literals
//! - this parameter in functions
//! - Correct this binding in arrow vs regular functions

use std::collections::HashMap;

/// Unique identifier for types
pub type TypeId = u64;

/// Represents the polymorphic 'this' type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThisType {
    /// The enclosing class or interface type
    pub containing_type: TypeId,
    /// Whether this is from an interface (affects subtyping)
    pub is_interface_this: bool,
    /// Constraint on this type (if any)
    pub constraint: Option<TypeId>,
}

impl ThisType {
    pub fn new(containing_type: TypeId) -> Self {
        ThisType {
            containing_type,
            is_interface_this: false,
            constraint: None,
        }
    }

    pub fn interface_this(containing_type: TypeId) -> Self {
        ThisType {
            containing_type,
            is_interface_this: true,
            constraint: None,
        }
    }

    pub fn with_constraint(mut self, constraint: TypeId) -> Self {
        self.constraint = Some(constraint);
        self
    }
}

/// ThisType<T> utility type - used for typing object literals with methods
/// that reference 'this'
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThisTypeUtility {
    /// The type argument T in ThisType<T>
    pub type_argument: TypeId,
}

impl ThisTypeUtility {
    pub fn new(type_argument: TypeId) -> Self {
        ThisTypeUtility { type_argument }
    }
}

/// Represents a 'this' parameter in a function signature
/// e.g., function foo(this: SomeType, arg: string) {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThisParameter {
    /// The explicit type of 'this'
    pub this_type: ThisTypeValue,
    /// Whether this is a void this (this: void)
    pub is_void: bool,
}

impl ThisParameter {
    pub fn new(this_type: ThisTypeValue) -> Self {
        ThisParameter {
            this_type,
            is_void: false,
        }
    }

    pub fn void_this() -> Self {
        ThisParameter {
            this_type: ThisTypeValue::Void,
            is_void: true,
        }
    }
}

/// The value of a 'this' type in different contexts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThisTypeValue {
    /// Polymorphic 'this' type
    This(ThisType),
    /// Explicit type annotation
    Explicit(TypeId),
    /// ThisType<T> utility
    ThisTypeUtility(ThisTypeUtility),
    /// void (for callbacks that shouldn't use this)
    Void,
    /// Unknown (for unresolved this)
    Unknown,
    /// Any (for loose checking)
    Any,
}

impl ThisTypeValue {
    /// Check if this type is assignable to target type
    pub fn is_assignable_to(&self, target: &ThisTypeValue) -> bool {
        match (self, target) {
            // Any is assignable to anything (bidirectional)
            (ThisTypeValue::Any, _) | (_, ThisTypeValue::Any) => true,
            // Unknown is only assignable to unknown
            (ThisTypeValue::Unknown, ThisTypeValue::Unknown) => true,
            // Void is only assignable to void
            (ThisTypeValue::Void, ThisTypeValue::Void) => true,
            // Same polymorphic this types
            (ThisTypeValue::This(a), ThisTypeValue::This(b)) => {
                a.containing_type == b.containing_type
            }
            // Same explicit types (simplified - would need full type checking)
            (ThisTypeValue::Explicit(a), ThisTypeValue::Explicit(b)) => a == b,
            // ThisType<T> assignability
            (ThisTypeValue::ThisTypeUtility(a), ThisTypeValue::ThisTypeUtility(b)) => {
                a.type_argument == b.type_argument
            }
            // Other cases are not assignable
            _ => false,
        }
    }
}

/// Context kind for 'this' binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisContextKind {
    /// Global scope - 'this' is globalThis or undefined
    Global,
    /// Module scope - 'this' is undefined
    Module,
    /// Class instance method - 'this' is the class instance
    ClassInstance,
    /// Class static method - 'this' is the class constructor
    ClassStatic,
    /// Class constructor - 'this' is the class instance being created
    ClassConstructor,
    /// Regular function - 'this' depends on call site
    Function,
    /// Arrow function - 'this' is lexically bound
    ArrowFunction,
    /// Object literal method - 'this' is the object
    ObjectMethod,
    /// Interface method - 'this' is polymorphic
    InterfaceMethod,
    /// Callback - 'this' may be void or explicit
    Callback,
}

/// Represents the 'this' context at a specific location in code
#[derive(Debug, Clone)]
pub struct ThisContext {
    /// The kind of context
    pub kind: ThisContextKind,
    /// The type of 'this' in this context
    pub this_type: ThisTypeValue,
    /// The enclosing class (if any)
    pub enclosing_class: Option<TypeId>,
    /// The enclosing interface (if any)
    pub enclosing_interface: Option<TypeId>,
    /// Whether this context is inside an arrow function
    pub in_arrow_function: bool,
    /// Parent context (for lexical scoping)
    pub parent: Option<Box<ThisContext>>,
}

impl ThisContext {
    pub fn global() -> Self {
        ThisContext {
            kind: ThisContextKind::Global,
            this_type: ThisTypeValue::Any, // globalThis
            enclosing_class: None,
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn module() -> Self {
        ThisContext {
            kind: ThisContextKind::Module,
            this_type: ThisTypeValue::Void, // undefined in modules
            enclosing_class: None,
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn class_instance(class_id: TypeId) -> Self {
        ThisContext {
            kind: ThisContextKind::ClassInstance,
            this_type: ThisTypeValue::This(ThisType::new(class_id)),
            enclosing_class: Some(class_id),
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn class_static(class_id: TypeId, constructor_type: TypeId) -> Self {
        ThisContext {
            kind: ThisContextKind::ClassStatic,
            this_type: ThisTypeValue::Explicit(constructor_type),
            enclosing_class: Some(class_id),
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn class_constructor(class_id: TypeId) -> Self {
        ThisContext {
            kind: ThisContextKind::ClassConstructor,
            this_type: ThisTypeValue::This(ThisType::new(class_id)),
            enclosing_class: Some(class_id),
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn interface_method(interface_id: TypeId) -> Self {
        ThisContext {
            kind: ThisContextKind::InterfaceMethod,
            this_type: ThisTypeValue::This(ThisType::interface_this(interface_id)),
            enclosing_class: None,
            enclosing_interface: Some(interface_id),
            in_arrow_function: false,
            parent: None,
        }
    }

    pub fn arrow_function(parent: ThisContext) -> Self {
        // Arrow functions inherit 'this' from parent context
        ThisContext {
            kind: ThisContextKind::ArrowFunction,
            this_type: parent.this_type.clone(),
            enclosing_class: parent.enclosing_class,
            enclosing_interface: parent.enclosing_interface,
            in_arrow_function: true,
            parent: Some(Box::new(parent)),
        }
    }

    pub fn regular_function(parent: Option<ThisContext>) -> Self {
        // Regular functions have their own 'this' that depends on call site
        ThisContext {
            kind: ThisContextKind::Function,
            this_type: ThisTypeValue::Unknown,
            enclosing_class: None,
            enclosing_interface: None,
            in_arrow_function: false,
            parent: parent.map(Box::new),
        }
    }

    pub fn object_method(object_type: TypeId) -> Self {
        ThisContext {
            kind: ThisContextKind::ObjectMethod,
            this_type: ThisTypeValue::Explicit(object_type),
            enclosing_class: None,
            enclosing_interface: None,
            in_arrow_function: false,
            parent: None,
        }
    }

    /// Get the effective 'this' type considering arrow function lexical binding
    pub fn effective_this_type(&self) -> &ThisTypeValue {
        if self.in_arrow_function {
            // For arrow functions, walk up to find the non-arrow parent
            if let Some(parent) = &self.parent {
                parent.effective_this_type()
            } else {
                &self.this_type
            }
        } else {
            &self.this_type
        }
    }

    /// Check if we're inside a static context
    pub fn is_static_context(&self) -> bool {
        matches!(self.kind, ThisContextKind::ClassStatic)
    }

    /// Check if 'this' access is valid in this context
    pub fn can_access_this(&self) -> bool {
        !matches!(
            self.kind,
            ThisContextKind::Global | ThisContextKind::Module
        ) || self.in_arrow_function && self.parent.is_some()
    }
}

/// Method signature with 'this' return type for fluent APIs
#[derive(Debug, Clone)]
pub struct FluentMethod {
    pub name: String,
    /// Returns 'this' type for method chaining
    pub returns_this: bool,
    /// Explicit return type (if not returning this)
    pub return_type: Option<TypeId>,
    /// Optional this parameter
    pub this_parameter: Option<ThisParameter>,
}

impl FluentMethod {
    pub fn new(name: impl Into<String>) -> Self {
        FluentMethod {
            name: name.into(),
            returns_this: false,
            return_type: None,
            this_parameter: None,
        }
    }

    pub fn returning_this(mut self) -> Self {
        self.returns_this = true;
        self
    }

    pub fn with_return_type(mut self, type_id: TypeId) -> Self {
        self.return_type = Some(type_id);
        self.returns_this = false;
        self
    }

    pub fn with_this_parameter(mut self, param: ThisParameter) -> Self {
        self.this_parameter = Some(param);
        self
    }
}

/// Registry for managing 'this' types across the type system
#[derive(Debug, Default)]
pub struct ThisTypeRegistry {
    /// Cached this type substitutions for instantiated classes
    substitutions: HashMap<(TypeId, TypeId), TypeId>,
    /// ThisType<T> instances
    this_type_utilities: HashMap<TypeId, ThisTypeUtility>,
    /// Next type ID for generated types
    next_id: TypeId,
}

impl ThisTypeRegistry {
    pub fn new() -> Self {
        ThisTypeRegistry {
            substitutions: HashMap::new(),
            this_type_utilities: HashMap::new(),
            next_id: 10000, // Start high to avoid conflicts
        }
    }

    /// Substitute 'this' type with concrete class type
    /// Used when a derived class inherits a method returning 'this'
    pub fn substitute_this(&mut self, original_this: TypeId, concrete_class: TypeId) -> TypeId {
        let key = (original_this, concrete_class);
        if let Some(&cached) = self.substitutions.get(&key) {
            return cached;
        }

        // Create a new substituted type
        let substituted_id = self.next_id;
        self.next_id += 1;
        self.substitutions.insert(key, substituted_id);
        substituted_id
    }

    /// Register a ThisType<T> utility type
    pub fn register_this_type_utility(&mut self, utility: ThisTypeUtility) -> TypeId {
        let id = self.next_id;
        self.next_id += 1;
        self.this_type_utilities.insert(id, utility);
        id
    }

    /// Get a ThisType<T> utility by ID
    pub fn get_this_type_utility(&self, id: TypeId) -> Option<&ThisTypeUtility> {
        self.this_type_utilities.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_this_type_creation() {
        let this_type = ThisType::new(1);
        assert_eq!(this_type.containing_type, 1);
        assert!(!this_type.is_interface_this);
        assert!(this_type.constraint.is_none());
    }

    #[test]
    fn test_interface_this_type() {
        let this_type = ThisType::interface_this(2);
        assert_eq!(this_type.containing_type, 2);
        assert!(this_type.is_interface_this);
    }

    #[test]
    fn test_this_type_with_constraint() {
        let this_type = ThisType::new(1).with_constraint(100);
        assert_eq!(this_type.constraint, Some(100));
    }

    #[test]
    fn test_this_type_utility() {
        let utility = ThisTypeUtility::new(42);
        assert_eq!(utility.type_argument, 42);
    }

    #[test]
    fn test_this_parameter() {
        let param = ThisParameter::new(ThisTypeValue::Explicit(1));
        assert!(!param.is_void);

        let void_param = ThisParameter::void_this();
        assert!(void_param.is_void);
    }

    #[test]
    fn test_this_type_value_assignability() {
        // Any is assignable to anything
        assert!(ThisTypeValue::Any.is_assignable_to(&ThisTypeValue::Void));
        assert!(ThisTypeValue::Any.is_assignable_to(&ThisTypeValue::Unknown));

        // Void is only assignable to void or any
        assert!(ThisTypeValue::Void.is_assignable_to(&ThisTypeValue::Void));
        assert!(ThisTypeValue::Void.is_assignable_to(&ThisTypeValue::Any));
        assert!(!ThisTypeValue::Void.is_assignable_to(&ThisTypeValue::Unknown));

        // Same this types
        let this_a = ThisTypeValue::This(ThisType::new(1));
        let this_b = ThisTypeValue::This(ThisType::new(1));
        let this_c = ThisTypeValue::This(ThisType::new(2));
        assert!(this_a.is_assignable_to(&this_b));
        assert!(!this_a.is_assignable_to(&this_c));
    }

    #[test]
    fn test_this_context_global() {
        let ctx = ThisContext::global();
        assert_eq!(ctx.kind, ThisContextKind::Global);
        assert!(!ctx.can_access_this());
    }

    #[test]
    fn test_this_context_class_instance() {
        let ctx = ThisContext::class_instance(1);
        assert_eq!(ctx.kind, ThisContextKind::ClassInstance);
        assert_eq!(ctx.enclosing_class, Some(1));
        assert!(ctx.can_access_this());
    }

    #[test]
    fn test_this_context_arrow_function() {
        let parent = ThisContext::class_instance(1);
        let arrow = ThisContext::arrow_function(parent);

        assert_eq!(arrow.kind, ThisContextKind::ArrowFunction);
        assert!(arrow.in_arrow_function);

        // Arrow function inherits 'this' from parent
        match arrow.effective_this_type() {
            ThisTypeValue::This(t) => assert_eq!(t.containing_type, 1),
            _ => panic!("Expected This type"),
        }
    }

    #[test]
    fn test_this_context_static() {
        let ctx = ThisContext::class_static(1, 100);
        assert!(ctx.is_static_context());
        assert_eq!(ctx.kind, ThisContextKind::ClassStatic);
    }

    #[test]
    fn test_fluent_method() {
        let method = FluentMethod::new("setName")
            .returning_this();
        assert!(method.returns_this);
        assert!(method.return_type.is_none());

        let method_with_type = FluentMethod::new("getName")
            .with_return_type(42);
        assert!(!method_with_type.returns_this);
        assert_eq!(method_with_type.return_type, Some(42));
    }

    #[test]
    fn test_this_type_registry_substitution() {
        let mut registry = ThisTypeRegistry::new();

        let sub1 = registry.substitute_this(1, 10);
        let sub2 = registry.substitute_this(1, 10);

        // Same substitution should return same ID
        assert_eq!(sub1, sub2);

        // Different substitution should return different ID
        let sub3 = registry.substitute_this(1, 20);
        assert_ne!(sub1, sub3);
    }

    #[test]
    fn test_this_type_registry_utility() {
        let mut registry = ThisTypeRegistry::new();

        let utility = ThisTypeUtility::new(42);
        let id = registry.register_this_type_utility(utility);

        let retrieved = registry.get_this_type_utility(id).unwrap();
        assert_eq!(retrieved.type_argument, 42);
    }
}
