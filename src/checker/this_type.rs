//! This Type Checking
//!
//! Integrates 'this' type checking with the class type checker:
//! - Validates this type usage in class methods
//! - Handles polymorphic this for fluent APIs
//! - Checks this parameter in function signatures
//! - Validates this access in static contexts
//! - Supports ThisType<T> utility type

use super::members::{ClassMember, MemberKind, TypeId, MemberId};
use super::modifiers::ModifierFlags;
use super::inheritance::{ClassType, InterfaceType, TypeRegistry};
use std::collections::HashMap;

/// Diagnostic for this type errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThisTypeDiagnostic {
    pub message: String,
    pub code: u32,
    pub class_id: Option<TypeId>,
    pub member_name: Option<String>,
}

/// Diagnostic codes for this type errors
pub mod error_codes {
    /// 'this' cannot be referenced in a static property initializer
    pub const THIS_IN_STATIC_PROPERTY: u32 = 2334;
    /// 'this' cannot be referenced in current location
    pub const THIS_NOT_ALLOWED: u32 = 2331;
    /// 'this' implicitly has type 'any' because it does not have a type annotation
    pub const THIS_IMPLICIT_ANY: u32 = 2683;
    /// The 'this' context is not assignable to method's 'this'
    pub const THIS_CONTEXT_MISMATCH: u32 = 2684;
    /// A 'this' type is available only in a non-static member
    pub const THIS_TYPE_NOT_AVAILABLE: u32 = 2526;
    /// 'this' types are incompatible between signatures
    pub const INCOMPATIBLE_THIS_TYPES: u32 = 2686;
    /// Cannot use 'this' as a type in static member
    pub const THIS_IN_STATIC_TYPE: u32 = 2335;
}

/// Represents the kind of this binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisBindingKind {
    /// Lexical this (arrow function)
    Lexical,
    /// Dynamic this (regular function/method)
    Dynamic,
    /// Static context this (constructor type)
    Static,
    /// No this binding
    None,
}

/// Context for this type checking
#[derive(Debug, Clone)]
pub struct ThisCheckContext {
    /// Current class (if any)
    pub current_class: Option<TypeId>,
    /// Current interface (if any)
    pub current_interface: Option<TypeId>,
    /// Whether we're in a static context
    pub is_static: bool,
    /// Whether we're in an arrow function
    pub is_arrow: bool,
    /// The kind of this binding
    pub binding_kind: ThisBindingKind,
    /// Explicit this parameter type (if any)
    pub explicit_this: Option<TypeId>,
}

impl ThisCheckContext {
    pub fn global() -> Self {
        ThisCheckContext {
            current_class: None,
            current_interface: None,
            is_static: false,
            is_arrow: false,
            binding_kind: ThisBindingKind::None,
            explicit_this: None,
        }
    }

    pub fn class_instance(class_id: TypeId) -> Self {
        ThisCheckContext {
            current_class: Some(class_id),
            current_interface: None,
            is_static: false,
            is_arrow: false,
            binding_kind: ThisBindingKind::Dynamic,
            explicit_this: None,
        }
    }

    pub fn class_static(class_id: TypeId) -> Self {
        ThisCheckContext {
            current_class: Some(class_id),
            current_interface: None,
            is_static: true,
            is_arrow: false,
            binding_kind: ThisBindingKind::Static,
            explicit_this: None,
        }
    }

    pub fn interface_method(interface_id: TypeId) -> Self {
        ThisCheckContext {
            current_class: None,
            current_interface: Some(interface_id),
            is_static: false,
            is_arrow: false,
            binding_kind: ThisBindingKind::Dynamic,
            explicit_this: None,
        }
    }

    pub fn arrow_function(parent: &ThisCheckContext) -> Self {
        ThisCheckContext {
            current_class: parent.current_class,
            current_interface: parent.current_interface,
            is_static: parent.is_static,
            is_arrow: true,
            binding_kind: ThisBindingKind::Lexical,
            explicit_this: parent.explicit_this,
        }
    }

    pub fn with_explicit_this(mut self, this_type: TypeId) -> Self {
        self.explicit_this = Some(this_type);
        self
    }
}

/// Method with this type information
#[derive(Debug, Clone)]
pub struct MethodThisInfo {
    pub member_id: MemberId,
    pub name: String,
    /// Whether the method returns 'this' type
    pub returns_this: bool,
    /// Explicit return type (if not returning this)
    pub return_type: Option<TypeId>,
    /// This parameter (if declared)
    pub this_parameter: Option<ThisParameterInfo>,
    /// Whether this is a static method
    pub is_static: bool,
}

impl MethodThisInfo {
    pub fn new(member_id: MemberId, name: impl Into<String>) -> Self {
        MethodThisInfo {
            member_id,
            name: name.into(),
            returns_this: false,
            return_type: None,
            this_parameter: None,
            is_static: false,
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

    pub fn with_this_parameter(mut self, param: ThisParameterInfo) -> Self {
        self.this_parameter = Some(param);
        self
    }

    pub fn static_method(mut self) -> Self {
        self.is_static = true;
        self
    }
}

/// Information about a this parameter
#[derive(Debug, Clone)]
pub struct ThisParameterInfo {
    /// The declared type of 'this'
    pub this_type: ThisParameterType,
}

/// Type of this parameter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThisParameterType {
    /// Explicit type ID
    Explicit(TypeId),
    /// void (this: void)
    Void,
    /// Polymorphic this
    This,
}

impl ThisParameterInfo {
    pub fn explicit(type_id: TypeId) -> Self {
        ThisParameterInfo {
            this_type: ThisParameterType::Explicit(type_id),
        }
    }

    pub fn void() -> Self {
        ThisParameterInfo {
            this_type: ThisParameterType::Void,
        }
    }

    pub fn polymorphic_this() -> Self {
        ThisParameterInfo {
            this_type: ThisParameterType::This,
        }
    }
}

/// This type checker
#[derive(Debug)]
pub struct ThisTypeChecker<'a> {
    registry: &'a TypeRegistry,
    /// Collected diagnostics
    diagnostics: Vec<ThisTypeDiagnostic>,
    /// Methods with this type info
    method_info: HashMap<MemberId, MethodThisInfo>,
    /// Whether to check noImplicitThis
    no_implicit_this: bool,
}

impl<'a> ThisTypeChecker<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        ThisTypeChecker {
            registry,
            diagnostics: Vec::new(),
            method_info: HashMap::new(),
            no_implicit_this: false,
        }
    }

    /// Enable noImplicitThis checking
    pub fn with_no_implicit_this(mut self) -> Self {
        self.no_implicit_this = true;
        self
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &[ThisTypeDiagnostic] {
        &self.diagnostics
    }

    /// Clear diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Register method this info
    pub fn register_method(&mut self, info: MethodThisInfo) {
        self.method_info.insert(info.member_id, info);
    }

    /// Check if 'this' type annotation is valid in context
    pub fn check_this_type_annotation(&mut self, context: &ThisCheckContext) -> bool {
        if context.is_static {
            self.diagnostics.push(ThisTypeDiagnostic {
                message: "A 'this' type is available only in a non-static member of a class or interface".to_string(),
                code: error_codes::THIS_TYPE_NOT_AVAILABLE,
                class_id: context.current_class,
                member_name: None,
            });
            return false;
        }

        if context.current_class.is_none() && context.current_interface.is_none() {
            self.diagnostics.push(ThisTypeDiagnostic {
                message: "A 'this' type is available only in a non-static member of a class or interface".to_string(),
                code: error_codes::THIS_TYPE_NOT_AVAILABLE,
                class_id: None,
                member_name: None,
            });
            return false;
        }

        true
    }

    /// Check 'this' access in the given context
    pub fn check_this_access(&mut self, context: &ThisCheckContext, member_name: Option<&str>) -> bool {
        // In static property initializer, 'this' is not allowed (unless in arrow)
        if context.is_static && !context.is_arrow {
            self.diagnostics.push(ThisTypeDiagnostic {
                message: "'this' cannot be referenced in a static property initializer".to_string(),
                code: error_codes::THIS_IN_STATIC_PROPERTY,
                class_id: context.current_class,
                member_name: member_name.map(String::from),
            });
            return false;
        }

        // Check noImplicitThis
        if self.no_implicit_this
            && context.current_class.is_none()
            && context.current_interface.is_none()
            && context.explicit_this.is_none()
            && !context.is_arrow
        {
            self.diagnostics.push(ThisTypeDiagnostic {
                message: "'this' implicitly has type 'any' because it does not have a type annotation".to_string(),
                code: error_codes::THIS_IMPLICIT_ANY,
                class_id: None,
                member_name: member_name.map(String::from),
            });
            return false;
        }

        true
    }

    /// Check this parameter compatibility for a method call
    pub fn check_this_parameter_call(
        &mut self,
        method_id: MemberId,
        call_context: &ThisCheckContext,
    ) -> bool {
        let method = match self.method_info.get(&method_id) {
            Some(m) => m.clone(),
            None => return true, // No this info, assume OK
        };

        let this_param = match &method.this_parameter {
            Some(p) => p,
            None => return true, // No this parameter, assume OK
        };

        // Check compatibility
        match &this_param.this_type {
            ThisParameterType::Void => {
                // Method requires no this context
                true
            }
            ThisParameterType::Explicit(required_type) => {
                // Check if call context provides compatible this
                if let Some(class_id) = call_context.current_class {
                    if class_id == *required_type || self.is_subtype(class_id, *required_type) {
                        true
                    } else {
                        self.diagnostics.push(ThisTypeDiagnostic {
                            message: format!(
                                "The 'this' context of type '{}' is not assignable to method's 'this' of type '{}'",
                                self.get_type_name(class_id),
                                self.get_type_name(*required_type)
                            ),
                            code: error_codes::THIS_CONTEXT_MISMATCH,
                            class_id: Some(class_id),
                            member_name: Some(method.name.clone()),
                        });
                        false
                    }
                } else {
                    // No class context
                    if call_context.explicit_this == Some(*required_type) {
                        true
                    } else {
                        self.diagnostics.push(ThisTypeDiagnostic {
                            message: "The 'this' context is not assignable to method's 'this' type".to_string(),
                            code: error_codes::THIS_CONTEXT_MISMATCH,
                            class_id: None,
                            member_name: Some(method.name.clone()),
                        });
                        false
                    }
                }
            }
            ThisParameterType::This => {
                // Polymorphic this - compatible if we have a class/interface context
                call_context.current_class.is_some() || call_context.current_interface.is_some()
            }
        }
    }

    /// Resolve method return type considering polymorphic this
    pub fn resolve_return_type(
        &self,
        method_id: MemberId,
        call_site_class: TypeId,
    ) -> Option<TypeId> {
        let method = self.method_info.get(&method_id)?;

        if method.returns_this {
            // Substitute 'this' with actual class at call site
            Some(call_site_class)
        } else {
            method.return_type
        }
    }

    /// Check all methods in a class for this-related issues
    pub fn check_class_methods(&mut self, class_id: TypeId) -> Vec<ThisTypeDiagnostic> {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut errors = Vec::new();

        for member in class.members.instance_members_iter() {
            if matches!(member.kind, MemberKind::Method | MemberKind::GetAccessor | MemberKind::SetAccessor) {
                let context = ThisCheckContext::class_instance(class_id);

                if let Some(info) = self.method_info.get(&member.id) {
                    // Check this parameter if present
                    if let Some(this_param) = &info.this_parameter {
                        if let ThisParameterType::Explicit(this_type) = this_param.this_type {
                            // This parameter type must be compatible with class
                            if !self.is_subtype(class_id, this_type) && class_id != this_type {
                                errors.push(ThisTypeDiagnostic {
                                    message: format!(
                                        "The 'this' parameter's type '{}' is not compatible with class '{}'",
                                        self.get_type_name(this_type),
                                        self.get_type_name(class_id)
                                    ),
                                    code: error_codes::THIS_CONTEXT_MISMATCH,
                                    class_id: Some(class_id),
                                    member_name: Some(member.name.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Check static members
        for member in class.members.static_members_iter() {
            if matches!(member.kind, MemberKind::Method) {
                // Static methods shouldn't return 'this' type (unless explicit)
                if let Some(info) = self.method_info.get(&member.id) {
                    if info.returns_this && !info.is_static {
                        errors.push(ThisTypeDiagnostic {
                            message: "Static method cannot return polymorphic 'this' type".to_string(),
                            code: error_codes::THIS_IN_STATIC_TYPE,
                            class_id: Some(class_id),
                            member_name: Some(member.name.clone()),
                        });
                    }
                }
            }
        }

        errors
    }

    /// Check interface methods for this type usage
    pub fn check_interface_methods(&mut self, interface_id: TypeId) -> Vec<ThisTypeDiagnostic> {
        let interface = match self.registry.get_interface(interface_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        let errors = Vec::new();
        // Interface methods can freely use 'this' type
        // Just validate that this parameters are sensible

        errors
    }

    /// Check if type_a is a subtype of type_b (for this compatibility)
    fn is_subtype(&self, type_a: TypeId, type_b: TypeId) -> bool {
        if type_a == type_b {
            return true;
        }

        // Check class inheritance
        if let Some(class) = self.registry.get_class(type_a) {
            if let Some(base) = class.base_class {
                return self.is_subtype(base, type_b);
            }
        }

        false
    }

    /// Get type name for error messages
    fn get_type_name(&self, type_id: TypeId) -> String {
        if let Some(class) = self.registry.get_class(type_id) {
            return class.name.clone();
        }
        if let Some(interface) = self.registry.get_interface(type_id) {
            return interface.name.clone();
        }
        format!("type#{}", type_id)
    }
}

/// Fluent API support for method chaining
#[derive(Debug)]
pub struct FluentApiChecker {
    /// Classes that support fluent API
    fluent_classes: HashMap<TypeId, Vec<String>>,
}

impl FluentApiChecker {
    pub fn new() -> Self {
        FluentApiChecker {
            fluent_classes: HashMap::new(),
        }
    }

    /// Register a class method as returning 'this' for fluent chaining
    pub fn register_fluent_method(&mut self, class_id: TypeId, method_name: impl Into<String>) {
        self.fluent_classes
            .entry(class_id)
            .or_insert_with(Vec::new)
            .push(method_name.into());
    }

    /// Check if a method is a fluent method
    pub fn is_fluent_method(&self, class_id: TypeId, method_name: &str) -> bool {
        self.fluent_classes
            .get(&class_id)
            .map(|methods| methods.iter().any(|m| m == method_name))
            .unwrap_or(false)
    }

    /// Get the return type for a method call in a chain
    pub fn get_chain_return_type(
        &self,
        class_id: TypeId,
        method_name: &str,
        actual_class: TypeId,
    ) -> Option<TypeId> {
        if self.is_fluent_method(class_id, method_name) {
            // Fluent method returns the actual class type (polymorphic this)
            Some(actual_class)
        } else {
            None
        }
    }

    /// Validate a method chain
    pub fn validate_chain(
        &self,
        initial_class: TypeId,
        method_names: &[&str],
    ) -> Result<TypeId, String> {
        let mut current_type = initial_class;

        for method_name in method_names {
            if let Some(return_type) = self.get_chain_return_type(current_type, method_name, current_type) {
                current_type = return_type;
            } else {
                // For non-fluent methods, we'd need more info about return types
                // For now, just check if it's a known fluent method
                if !self.is_fluent_method(current_type, method_name) {
                    return Err(format!("Method '{}' is not a fluent method", method_name));
                }
            }
        }

        Ok(current_type)
    }
}

impl Default for FluentApiChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> TypeRegistry {
        TypeRegistry::new()
    }

    #[test]
    fn test_this_check_context_creation() {
        let ctx = ThisCheckContext::global();
        assert!(ctx.current_class.is_none());
        assert!(!ctx.is_static);
        assert!(!ctx.is_arrow);

        let ctx = ThisCheckContext::class_instance(1);
        assert_eq!(ctx.current_class, Some(1));
        assert!(!ctx.is_static);
        assert_eq!(ctx.binding_kind, ThisBindingKind::Dynamic);

        let ctx = ThisCheckContext::class_static(1);
        assert!(ctx.is_static);
        assert_eq!(ctx.binding_kind, ThisBindingKind::Static);
    }

    #[test]
    fn test_arrow_function_context() {
        let parent = ThisCheckContext::class_instance(1);
        let arrow = ThisCheckContext::arrow_function(&parent);

        assert_eq!(arrow.current_class, Some(1));
        assert!(arrow.is_arrow);
        assert_eq!(arrow.binding_kind, ThisBindingKind::Lexical);
    }

    #[test]
    fn test_method_this_info() {
        let method = MethodThisInfo::new(1, "setName")
            .returning_this();
        assert!(method.returns_this);
        assert!(method.return_type.is_none());

        let method = MethodThisInfo::new(2, "getName")
            .with_return_type(100);
        assert!(!method.returns_this);
        assert_eq!(method.return_type, Some(100));
    }

    #[test]
    fn test_this_parameter_info() {
        let param = ThisParameterInfo::explicit(42);
        assert_eq!(param.this_type, ThisParameterType::Explicit(42));

        let param = ThisParameterInfo::void();
        assert_eq!(param.this_type, ThisParameterType::Void);

        let param = ThisParameterInfo::polymorphic_this();
        assert_eq!(param.this_type, ThisParameterType::This);
    }

    #[test]
    fn test_this_type_not_in_static() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let static_ctx = ThisCheckContext::class_static(1);
        let result = checker.check_this_type_annotation(&static_ctx);

        assert!(!result);
        assert!(checker.diagnostics().iter().any(|d| d.code == error_codes::THIS_TYPE_NOT_AVAILABLE));
    }

    #[test]
    fn test_this_type_not_in_global() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let global_ctx = ThisCheckContext::global();
        let result = checker.check_this_type_annotation(&global_ctx);

        assert!(!result);
        assert!(checker.diagnostics().iter().any(|d| d.code == error_codes::THIS_TYPE_NOT_AVAILABLE));
    }

    #[test]
    fn test_this_type_valid_in_instance() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let instance_ctx = ThisCheckContext::class_instance(1);
        let result = checker.check_this_type_annotation(&instance_ctx);

        assert!(result);
        assert!(checker.diagnostics().is_empty());
    }

    #[test]
    fn test_this_access_in_static_property() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let static_ctx = ThisCheckContext::class_static(1);
        let result = checker.check_this_access(&static_ctx, Some("prop"));

        assert!(!result);
        assert!(checker.diagnostics().iter().any(|d| d.code == error_codes::THIS_IN_STATIC_PROPERTY));
    }

    #[test]
    fn test_this_access_in_static_arrow() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let static_ctx = ThisCheckContext::class_static(1);
        let arrow_ctx = ThisCheckContext::arrow_function(&static_ctx);
        let result = checker.check_this_access(&arrow_ctx, None);

        // Arrow in static captures outer this - this is allowed
        assert!(result);
    }

    #[test]
    fn test_no_implicit_this() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry).with_no_implicit_this();

        let global_ctx = ThisCheckContext::global();
        let result = checker.check_this_access(&global_ctx, None);

        assert!(!result);
        assert!(checker.diagnostics().iter().any(|d| d.code == error_codes::THIS_IMPLICIT_ANY));
    }

    #[test]
    fn test_resolve_return_type_this() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let method = MethodThisInfo::new(1, "setName").returning_this();
        checker.register_method(method);

        // When called on class 42, returns 42
        let result = checker.resolve_return_type(1, 42);
        assert_eq!(result, Some(42));

        // Different class gets different result
        let result = checker.resolve_return_type(1, 100);
        assert_eq!(result, Some(100));
    }

    #[test]
    fn test_resolve_return_type_explicit() {
        let registry = create_test_registry();
        let mut checker = ThisTypeChecker::new(&registry);

        let method = MethodThisInfo::new(1, "getName").with_return_type(200);
        checker.register_method(method);

        // Returns explicit type regardless of call site
        let result = checker.resolve_return_type(1, 42);
        assert_eq!(result, Some(200));
    }

    #[test]
    fn test_fluent_api_checker() {
        let mut checker = FluentApiChecker::new();

        checker.register_fluent_method(1, "setName");
        checker.register_fluent_method(1, "setAge");

        assert!(checker.is_fluent_method(1, "setName"));
        assert!(checker.is_fluent_method(1, "setAge"));
        assert!(!checker.is_fluent_method(1, "build"));
        assert!(!checker.is_fluent_method(2, "setName"));
    }

    #[test]
    fn test_fluent_chain_return_type() {
        let mut checker = FluentApiChecker::new();
        checker.register_fluent_method(1, "setName");

        // Base class method called on derived class returns derived
        let result = checker.get_chain_return_type(1, "setName", 42);
        assert_eq!(result, Some(42));

        // Non-fluent method returns None
        let result = checker.get_chain_return_type(1, "build", 42);
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_fluent_chain() {
        let mut checker = FluentApiChecker::new();
        checker.register_fluent_method(1, "setName");
        checker.register_fluent_method(1, "setAge");

        // Valid chain
        let result = checker.validate_chain(1, &["setName", "setAge"]);
        assert_eq!(result, Ok(1));

        // Invalid chain
        let result = checker.validate_chain(1, &["setName", "unknown"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_interface_this_context() {
        let ctx = ThisCheckContext::interface_method(5);
        assert_eq!(ctx.current_interface, Some(5));
        assert!(ctx.current_class.is_none());
        assert!(!ctx.is_static);
    }

    #[test]
    fn test_explicit_this_context() {
        let ctx = ThisCheckContext::class_instance(1)
            .with_explicit_this(42);
        assert_eq!(ctx.explicit_this, Some(42));
    }
}
