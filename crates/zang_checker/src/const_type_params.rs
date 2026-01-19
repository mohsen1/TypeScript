//! Const Type Parameters
//!
//! This module handles const type parameter checking:
//! - Parse const modifier on type parameters
//! - Check const modifier placement validity
//! - Handle const type parameters in generic functions
//! - Handle const type parameters in generic classes
//! - Prevent widening for const type parameter arguments

use std::collections::HashMap;
use zang_core::{InternedString, Span, StringInterner};
use zang_parser::{
    TypeParameterNode, FunctionDeclaration, ClassDeclaration, InterfaceDeclaration,
    TypeAliasDeclaration, ArrowFunction, FunctionExpression, MethodDeclaration,
};

use crate::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
use crate::types::{
    ResolvedType, TypeParameterType, FunctionType, ObjectType, LiteralType,
    PropertySignature, ParameterType,
};

/// Result of const type parameter checking
#[derive(Debug, Clone)]
pub struct ConstTypeParamCheckResult {
    /// Whether the check passed
    pub success: bool,
    /// Diagnostics generated during checking
    pub diagnostics: Vec<Diagnostic>,
}

/// Const type parameter checker
pub struct ConstTypeParamChecker<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Map of type parameter names to their const status
    const_type_params: HashMap<InternedString, bool>,
    /// Diagnostics collected during checking
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ConstTypeParamChecker<'a> {
    /// Creates a new const type parameter checker
    pub fn new(interner: &'a StringInterner) -> Self {
        Self {
            interner,
            const_type_params: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Checks if a type parameter is const
    pub fn is_const_type_param(&self, name: &InternedString) -> bool {
        self.const_type_params.get(name).copied().unwrap_or(false)
    }

    /// Registers a const type parameter
    pub fn register_const_type_param(&mut self, name: InternedString) {
        self.const_type_params.insert(name, true);
    }

    /// Clears registered const type parameters (for scope management)
    pub fn clear_const_type_params(&mut self) {
        self.const_type_params.clear();
    }

    /// Gets collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Takes collected diagnostics, leaving empty vec
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Validates const modifier placement on type parameters
    pub fn check_const_modifier_placement(
        &mut self,
        type_params: &[TypeParameterNode],
        context: ConstModifierContext,
    ) -> ConstTypeParamCheckResult {
        let mut diagnostics = Vec::new();

        for param in type_params {
            if param.is_const {
                // Register this const type parameter
                self.const_type_params.insert(param.name.name, true);

                // Check if const is allowed in this context
                if !context.allows_const() {
                    diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::ConstModifierNotAllowed,
                        message: format!(
                            "'const' modifier cannot be used on a type parameter of a {}",
                            context.context_name()
                        ),
                        span: param.span,
                        severity: DiagnosticSeverity::Error,
                    });
                }

                // Check if const type parameter has constraint that's compatible
                if let Some(ref constraint) = param.constraint {
                    // const type parameters can only have constraints that are
                    // compatible with literal types (string, number, boolean, object, array, etc.)
                    if !self.is_valid_const_constraint(constraint) {
                        diagnostics.push(Diagnostic {
                            kind: DiagnosticKind::InvalidConstConstraint,
                            message: "Const type parameter constraint must be a type that can contain literal types".to_string(),
                            span: param.span,
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
            }
        }

        ConstTypeParamCheckResult {
            success: diagnostics.is_empty(),
            diagnostics,
        }
    }

    /// Checks if a constraint is valid for a const type parameter
    fn is_valid_const_constraint(&self, _constraint: &zang_parser::TypeNode) -> bool {
        // Most constraints are valid for const type parameters
        // Invalid: function types, void, never (in most cases)
        // For now, we'll allow all constraints and let inference handle it
        true
    }

    /// Checks a function declaration for const type parameters
    pub fn check_function_declaration(&mut self, func: &FunctionDeclaration) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = func.type_parameters {
            self.check_const_modifier_placement(type_params, ConstModifierContext::Function)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks a class declaration for const type parameters
    pub fn check_class_declaration(&mut self, class: &ClassDeclaration) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = class.type_parameters {
            // Const type parameters are NOT allowed on class declarations (only methods)
            self.check_const_modifier_placement(type_params, ConstModifierContext::Class)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks an interface declaration for const type parameters
    pub fn check_interface_declaration(&mut self, iface: &InterfaceDeclaration) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = iface.type_parameters {
            // Const type parameters are NOT allowed on interface declarations
            self.check_const_modifier_placement(type_params, ConstModifierContext::Interface)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks a type alias declaration for const type parameters
    pub fn check_type_alias_declaration(&mut self, alias: &TypeAliasDeclaration) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = alias.type_parameters {
            // Const type parameters are NOT allowed on type alias declarations
            self.check_const_modifier_placement(type_params, ConstModifierContext::TypeAlias)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks an arrow function for const type parameters
    pub fn check_arrow_function(&mut self, arrow: &ArrowFunction) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = arrow.type_parameters {
            self.check_const_modifier_placement(type_params, ConstModifierContext::ArrowFunction)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks a function expression for const type parameters
    pub fn check_function_expression(&mut self, func: &FunctionExpression) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = func.type_parameters {
            self.check_const_modifier_placement(type_params, ConstModifierContext::Function)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Checks a method declaration for const type parameters
    pub fn check_method_declaration(&mut self, method: &MethodDeclaration) -> ConstTypeParamCheckResult {
        if let Some(ref type_params) = method.type_parameters {
            self.check_const_modifier_placement(type_params, ConstModifierContext::Method)
        } else {
            ConstTypeParamCheckResult {
                success: true,
                diagnostics: Vec::new(),
            }
        }
    }

    /// Creates TypeParameterType from TypeParameterNode, preserving const status
    pub fn create_type_parameter(&self, param: &TypeParameterNode) -> TypeParameterType {
        TypeParameterType {
            name: param.name.name,
            constraint: None, // Would be resolved from param.constraint
            default: None,    // Would be resolved from param.default
            is_const: param.is_const,
        }
    }

    /// Creates TypeParameterType list from TypeParameterNode list
    pub fn create_type_parameters(&self, params: &[TypeParameterNode]) -> Vec<TypeParameterType> {
        params.iter().map(|p| self.create_type_parameter(p)).collect()
    }
}

/// Context where const modifier is being used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstModifierContext {
    /// Function declaration
    Function,
    /// Arrow function
    ArrowFunction,
    /// Method in a class
    Method,
    /// Class declaration
    Class,
    /// Interface declaration
    Interface,
    /// Type alias declaration
    TypeAlias,
    /// Call signature
    CallSignature,
    /// Construct signature
    ConstructSignature,
}

impl ConstModifierContext {
    /// Returns true if const modifier is allowed in this context
    pub fn allows_const(&self) -> bool {
        match self {
            // Const is allowed on functions, arrow functions, methods, and call signatures
            ConstModifierContext::Function
            | ConstModifierContext::ArrowFunction
            | ConstModifierContext::Method
            | ConstModifierContext::CallSignature => true,
            // Const is NOT allowed on classes, interfaces, type aliases, or construct signatures
            ConstModifierContext::Class
            | ConstModifierContext::Interface
            | ConstModifierContext::TypeAlias
            | ConstModifierContext::ConstructSignature => false,
        }
    }

    /// Returns the name of this context for error messages
    pub fn context_name(&self) -> &'static str {
        match self {
            ConstModifierContext::Function => "function",
            ConstModifierContext::ArrowFunction => "arrow function",
            ConstModifierContext::Method => "method",
            ConstModifierContext::Class => "class",
            ConstModifierContext::Interface => "interface",
            ConstModifierContext::TypeAlias => "type alias",
            ConstModifierContext::CallSignature => "call signature",
            ConstModifierContext::ConstructSignature => "construct signature",
        }
    }
}

/// Utility functions for working with const type parameters
pub mod utils {
    use super::*;

    /// Checks if a type should be treated as readonly due to const inference
    pub fn should_be_readonly(ty: &ResolvedType, is_const_context: bool) -> bool {
        if !is_const_context {
            return false;
        }

        match ty {
            ResolvedType::Object(_) | ResolvedType::Array(_) | ResolvedType::Tuple(_) => true,
            _ => false,
        }
    }

    /// Makes an object type readonly by marking all properties as readonly
    pub fn make_readonly_object(obj: &ObjectType) -> ObjectType {
        ObjectType {
            properties: obj
                .properties
                .iter()
                .map(|p| PropertySignature {
                    name: p.name,
                    ty: p.ty.clone(),
                    optional: p.optional,
                    readonly: true,
                })
                .collect(),
            call_signatures: obj.call_signatures.clone(),
            construct_signatures: obj.construct_signatures.clone(),
            index_signatures: obj
                .index_signatures
                .iter()
                .map(|s| crate::types::IndexSignature {
                    key_type: s.key_type.clone(),
                    value_type: s.value_type.clone(),
                    readonly: true,
                })
                .collect(),
        }
    }

    /// Recursively applies readonly to nested types
    pub fn make_deeply_readonly(ty: &ResolvedType) -> ResolvedType {
        match ty {
            ResolvedType::Object(obj) => {
                let readonly_props: Vec<PropertySignature> = obj
                    .properties
                    .iter()
                    .map(|p| PropertySignature {
                        name: p.name,
                        ty: Box::new(make_deeply_readonly(&p.ty)),
                        optional: p.optional,
                        readonly: true,
                    })
                    .collect();

                ResolvedType::Object(ObjectType {
                    properties: readonly_props,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: obj
                        .index_signatures
                        .iter()
                        .map(|s| crate::types::IndexSignature {
                            key_type: s.key_type.clone(),
                            value_type: Box::new(make_deeply_readonly(&s.value_type)),
                            readonly: true,
                        })
                        .collect(),
                })
            }
            ResolvedType::Array(elem) => {
                // Convert array to readonly tuple or readonly array
                ResolvedType::Array(Box::new(make_deeply_readonly(elem)))
            }
            ResolvedType::Tuple(elems) => {
                ResolvedType::Tuple(elems.iter().map(make_deeply_readonly).collect())
            }
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zang_core::StringInterner;

    #[test]
    fn test_const_type_param_checker_creation() {
        let interner = StringInterner::new();
        let checker = ConstTypeParamChecker::new(&interner);
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_const_modifier_context_allows_const() {
        assert!(ConstModifierContext::Function.allows_const());
        assert!(ConstModifierContext::ArrowFunction.allows_const());
        assert!(ConstModifierContext::Method.allows_const());
        assert!(ConstModifierContext::CallSignature.allows_const());

        assert!(!ConstModifierContext::Class.allows_const());
        assert!(!ConstModifierContext::Interface.allows_const());
        assert!(!ConstModifierContext::TypeAlias.allows_const());
        assert!(!ConstModifierContext::ConstructSignature.allows_const());
    }

    #[test]
    fn test_const_modifier_context_name() {
        assert_eq!(ConstModifierContext::Function.context_name(), "function");
        assert_eq!(ConstModifierContext::Class.context_name(), "class");
        assert_eq!(ConstModifierContext::Interface.context_name(), "interface");
    }

    #[test]
    fn test_register_const_type_param() {
        let interner = StringInterner::new();
        let mut checker = ConstTypeParamChecker::new(&interner);
        let name = interner.intern("T");

        assert!(!checker.is_const_type_param(&name));
        checker.register_const_type_param(name);
        assert!(checker.is_const_type_param(&name));
    }

    #[test]
    fn test_clear_const_type_params() {
        let interner = StringInterner::new();
        let mut checker = ConstTypeParamChecker::new(&interner);
        let name = interner.intern("T");

        checker.register_const_type_param(name);
        assert!(checker.is_const_type_param(&name));

        checker.clear_const_type_params();
        assert!(!checker.is_const_type_param(&name));
    }

    #[test]
    fn test_make_readonly_object() {
        let obj = ObjectType {
            properties: vec![PropertySignature {
                name: zang_core::InternedString::from_raw(1),
                ty: Box::new(ResolvedType::String),
                optional: false,
                readonly: false,
            }],
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        };

        let readonly_obj = utils::make_readonly_object(&obj);
        assert!(readonly_obj.properties[0].readonly);
    }

    #[test]
    fn test_make_deeply_readonly_object() {
        let nested_obj = ObjectType {
            properties: vec![PropertySignature {
                name: zang_core::InternedString::from_raw(1),
                ty: Box::new(ResolvedType::String),
                optional: false,
                readonly: false,
            }],
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        };

        let outer_obj = ObjectType {
            properties: vec![PropertySignature {
                name: zang_core::InternedString::from_raw(2),
                ty: Box::new(ResolvedType::Object(nested_obj)),
                optional: false,
                readonly: false,
            }],
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        };

        let readonly = utils::make_deeply_readonly(&ResolvedType::Object(outer_obj));

        if let ResolvedType::Object(obj) = readonly {
            assert!(obj.properties[0].readonly);
            if let ResolvedType::Object(inner) = &*obj.properties[0].ty {
                assert!(inner.properties[0].readonly);
            } else {
                panic!("Expected inner object");
            }
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_should_be_readonly() {
        let obj_type = ResolvedType::Object(ObjectType {
            properties: Vec::new(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        });

        assert!(utils::should_be_readonly(&obj_type, true));
        assert!(!utils::should_be_readonly(&obj_type, false));
        assert!(!utils::should_be_readonly(&ResolvedType::String, true));
    }
}
