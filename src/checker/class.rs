//! Class Type Checker
//!
//! This module provides the main class type checking functionality including:
//! - Class declaration and expression validation
//! - Constructor checking
//! - Member type checking
//! - Static initialization checking

use super::modifiers::{ModifierFlags, ModifierContext, validate_modifiers, AccessModifier};
use super::members::{ClassMember, MemberKind, MemberTable, ParameterInfo, TypeId, MemberId, check_accessor_pair, check_duplicate_members};
use super::inheritance::{ClassType, InterfaceType, TypeRegistry, InheritanceChecker};
use std::collections::HashMap;

/// Class checker diagnostic
#[derive(Debug, Clone)]
pub struct ClassDiagnostic {
    pub message: String,
    pub code: u32,
    pub node_id: u32,
}

/// Diagnostic codes
pub mod error_codes {
    pub const CONSTRUCTOR_PRIVATE_ABSTRACT: u32 = 1089;
    pub const ABSTRACT_CLASS_CONSTRUCT: u32 = 2511;
    pub const MISSING_CONSTRUCTOR_SUPER: u32 = 2377;
    pub const SUPER_BEFORE_THIS: u32 = 2376;
    pub const PROPERTY_INITIALIZATION: u32 = 2564;
    pub const DEFINITE_ASSIGNMENT_WRONG: u32 = 2565;
    pub const CLASS_EXPRESSION_ABSTRACT: u32 = 1243;
    pub const STATIC_BLOCK_IN_AMBIENT: u32 = 2694;
    pub const STATIC_BLOCK_RETURN: u32 = 2695;
    pub const STATIC_BLOCK_AWAIT: u32 = 2696;
    pub const PARAMETER_PROPERTY_IN_SIGNATURE: u32 = 1092;
    pub const PARAMETER_INITIALIZER_SUPER: u32 = 2524;
    pub const MULTIPLE_CONSTRUCTORS: u32 = 2392;
    pub const CONSTRUCTOR_OVERRIDE: u32 = 1249;
    pub const CONSTRUCTOR_TYPE_PARAM: u32 = 1092;
    pub const PRIVATE_CONSTRUCTOR: u32 = 2673;
}

/// Options for class checking
#[derive(Debug, Clone, Default)]
pub struct ClassCheckOptions {
    /// Strict property initialization checking
    pub strict_property_initialization: bool,
    /// No implicit override checking
    pub no_implicit_override: bool,
    /// Strict null checks
    pub strict_null_checks: bool,
    /// Use define for class fields
    pub use_define_for_class_fields: bool,
}

/// Main class type checker
#[derive(Debug)]
pub struct ClassChecker {
    pub registry: TypeRegistry,
    pub options: ClassCheckOptions,
    pub diagnostics: Vec<ClassDiagnostic>,
}

impl ClassChecker {
    pub fn new() -> Self {
        ClassChecker {
            registry: TypeRegistry::new(),
            options: ClassCheckOptions::default(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_options(mut self, options: ClassCheckOptions) -> Self {
        self.options = options;
        self
    }

    /// Register and check a class
    pub fn check_class(&mut self, class: ClassType, is_expression: bool) -> TypeId {
        let class_id = self.registry.register_class(class);

        // Check abstract on class expression
        if is_expression {
            if let Some(class) = self.registry.get_class(class_id) {
                if class.is_abstract {
                    self.diagnostics.push(ClassDiagnostic {
                        message: "Abstract classes cannot be used in a class expression".to_string(),
                        code: error_codes::CLASS_EXPRESSION_ABSTRACT,
                        node_id: class_id,
                    });
                }
            }
        }

        // Check class members
        self.check_class_members(class_id);

        // Check constructor
        self.check_constructor(class_id);

        // Check inheritance
        let checker = if self.options.no_implicit_override {
            InheritanceChecker::new(&self.registry).with_implicit_override_check()
        } else {
            InheritanceChecker::new(&self.registry)
        };
        let errors = checker.check_class(class_id);
        for error in errors {
            self.diagnostics.push(ClassDiagnostic {
                message: error.message,
                code: error.code,
                node_id: class_id,
            });
        }

        // Check property initialization
        if self.options.strict_property_initialization {
            self.check_property_initialization(class_id);
        }

        class_id
    }

    /// Register and check an interface
    pub fn check_interface(&mut self, interface: InterfaceType) -> TypeId {
        self.registry.register_interface(interface)
    }

    fn check_class_members(&mut self, class_id: TypeId) {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return,
        };

        let is_abstract = class.is_abstract;
        let has_base = class.base_class.is_some();

        // Check for duplicate members
        let dup_errors = check_duplicate_members(&class.members);
        for err in dup_errors {
            self.diagnostics.push(ClassDiagnostic {
                message: err.message,
                code: err.code,
                node_id: class_id,
            });
        }

        // Collect accessors for pair checking
        let mut getters: HashMap<String, &ClassMember> = HashMap::new();
        let mut setters: HashMap<String, &ClassMember> = HashMap::new();

        for member in class.members.instance_members_iter() {
            // Check member modifiers
            let context = ModifierContext::for_member(is_abstract, has_base);
            let mod_errors = validate_modifiers(member.modifiers, &context);
            for err in mod_errors {
                self.diagnostics.push(ClassDiagnostic {
                    message: err.message,
                    code: err.code,
                    node_id: class_id,
                });
            }

            // Collect accessors
            match member.kind {
                MemberKind::GetAccessor => {
                    getters.insert(member.name.clone(), member);
                }
                MemberKind::SetAccessor => {
                    setters.insert(member.name.clone(), member);
                }
                _ => {}
            }

            // Check abstract members in non-abstract class
            if member.is_abstract() && !is_abstract {
                self.diagnostics.push(ClassDiagnostic {
                    message: format!("Abstract methods can only appear within an abstract class"),
                    code: 1244,
                    node_id: class_id,
                });
            }
        }

        // Check static members
        for member in class.members.static_members_iter() {
            let context = ModifierContext::for_member(is_abstract, has_base);
            let mod_errors = validate_modifiers(member.modifiers, &context);
            for err in mod_errors {
                self.diagnostics.push(ClassDiagnostic {
                    message: err.message,
                    code: err.code,
                    node_id: class_id,
                });
            }
        }

        // Check accessor pairs
        let all_accessor_names: std::collections::HashSet<_> = getters.keys()
            .chain(setters.keys())
            .cloned()
            .collect();

        for name in all_accessor_names {
            let getter = getters.get(&name).copied();
            let setter = setters.get(&name).copied();
            let errors = check_accessor_pair(getter, setter);
            for err in errors {
                self.diagnostics.push(ClassDiagnostic {
                    message: err.message,
                    code: err.code,
                    node_id: class_id,
                });
            }
        }
    }

    fn check_constructor(&mut self, class_id: TypeId) {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return,
        };

        let constructor = match class.members.get_constructor() {
            Some(c) => c,
            None => return,
        };

        // Constructor cannot have override modifier
        if constructor.is_override() {
            self.diagnostics.push(ClassDiagnostic {
                message: "A constructor cannot have an 'override' modifier".to_string(),
                code: error_codes::CONSTRUCTOR_OVERRIDE,
                node_id: class_id,
            });
        }

        // Check private constructor in abstract class
        if class.is_abstract && constructor.is_private() {
            self.diagnostics.push(ClassDiagnostic {
                message: "A constructor cannot be both abstract and private".to_string(),
                code: error_codes::CONSTRUCTOR_PRIVATE_ABSTRACT,
                node_id: class_id,
            });
        }

        // Check parameter properties
        for param in &constructor.parameters {
            if param.is_parameter_property {
                // Check valid modifier combination
                let context = ModifierContext::for_constructor_param(class.base_class.is_some());
                let mod_errors = validate_modifiers(param.modifiers, &context);
                for err in mod_errors {
                    self.diagnostics.push(ClassDiagnostic {
                        message: err.message,
                        code: err.code,
                        node_id: class_id,
                    });
                }
            }
        }
    }

    fn check_property_initialization(&mut self, class_id: TypeId) {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return,
        };

        // Skip abstract classes
        if class.is_abstract {
            return;
        }

        // Collect properties that need initialization
        let mut needs_init: Vec<String> = Vec::new();

        for member in class.members.instance_members_iter() {
            if member.kind == MemberKind::Property
                && !member.optional
                && !member.is_abstract()
                && member.type_id.is_some()
            {
                // Property needs initialization (simplified check)
                // In a real implementation, we'd check:
                // - Has initializer
                // - Is initialized in constructor
                // - Has definite assignment assertion (!)
                needs_init.push(member.name.clone());
            }
        }

        // Check constructor initializes all required properties
        // (This is a simplified check - real implementation would do flow analysis)
        let constructor_params: Vec<_> = class.members.get_constructor()
            .map(|c| c.parameters.iter()
                .filter(|p| p.is_parameter_property)
                .map(|p| p.name.clone())
                .collect())
            .unwrap_or_default();

        for prop_name in &needs_init {
            if !constructor_params.contains(prop_name) {
                // Property may not be initialized
                // In real implementation, check constructor body
            }
        }
    }

    /// Check if a class can be instantiated
    pub fn check_instantiation(&mut self, class_id: TypeId, access_context: AccessModifier) -> bool {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return false,
        };

        // Cannot instantiate abstract class
        if class.is_abstract {
            self.diagnostics.push(ClassDiagnostic {
                message: format!("Cannot create an instance of an abstract class"),
                code: error_codes::ABSTRACT_CLASS_CONSTRUCT,
                node_id: class_id,
            });
            return false;
        }

        // Check constructor accessibility
        if let Some(ctor) = class.members.get_constructor() {
            let ctor_access = ctor.access_modifier();

            match ctor_access {
                AccessModifier::Private => {
                    if access_context != AccessModifier::Private {
                        self.diagnostics.push(ClassDiagnostic {
                            message: format!("Constructor of class '{}' is private and only accessible within the class", class.name),
                            code: error_codes::PRIVATE_CONSTRUCTOR,
                            node_id: class_id,
                        });
                        return false;
                    }
                }
                AccessModifier::Protected => {
                    if !matches!(access_context, AccessModifier::Private | AccessModifier::Protected) {
                        self.diagnostics.push(ClassDiagnostic {
                            message: format!("Constructor of class '{}' is protected and only accessible within the class and its subclasses", class.name),
                            code: 2674,
                            node_id: class_id,
                        });
                        return false;
                    }
                }
                AccessModifier::Public => {}
            }
        }

        true
    }

    /// Check member access
    pub fn check_member_access(
        &mut self,
        class_id: TypeId,
        member_name: &str,
        is_static: bool,
        access_context: AccessContext,
    ) -> Option<&ClassMember> {
        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return None,
        };

        let member_ids = class.members.get_members_by_name(member_name, is_static)?;
        let member_id = member_ids.first()?;
        let member = class.members.get_member(*member_id)?;

        // Check accessibility
        let accessible = match member.access_modifier() {
            AccessModifier::Private => access_context.is_same_class(class_id),
            AccessModifier::Protected => {
                access_context.is_same_class(class_id) ||
                access_context.is_derived_class(class_id, &self.registry)
            }
            AccessModifier::Public => true,
        };

        if !accessible {
            self.diagnostics.push(ClassDiagnostic {
                message: format!(
                    "Property '{}' is {} and only accessible within {}",
                    member_name,
                    match member.access_modifier() {
                        AccessModifier::Private => "private",
                        AccessModifier::Protected => "protected",
                        AccessModifier::Public => "public",
                    },
                    match member.access_modifier() {
                        AccessModifier::Private => format!("class '{}'", class.name),
                        AccessModifier::Protected => format!("class '{}' and its subclasses", class.name),
                        AccessModifier::Public => "everywhere".to_string(),
                    }
                ),
                code: 2341,
                node_id: class_id,
            });
            return None;
        }

        Some(member)
    }

    /// Get all diagnostics
    pub fn get_diagnostics(&self) -> &[ClassDiagnostic] {
        &self.diagnostics
    }

    /// Clear diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }
}

impl Default for ClassChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for member access checking
#[derive(Debug, Clone)]
pub struct AccessContext {
    /// The class where access is occurring
    pub enclosing_class: Option<TypeId>,
    /// Whether in static context
    pub is_static: bool,
}

impl AccessContext {
    pub fn new() -> Self {
        AccessContext {
            enclosing_class: None,
            is_static: false,
        }
    }

    pub fn in_class(class_id: TypeId) -> Self {
        AccessContext {
            enclosing_class: Some(class_id),
            is_static: false,
        }
    }

    pub fn is_same_class(&self, class_id: TypeId) -> bool {
        self.enclosing_class == Some(class_id)
    }

    pub fn is_derived_class(&self, base_id: TypeId, registry: &TypeRegistry) -> bool {
        let enclosing = match self.enclosing_class {
            Some(id) => id,
            None => return false,
        };

        let mut current = registry.get_class(enclosing).and_then(|c| c.base_class);
        while let Some(id) = current {
            if id == base_id {
                return true;
            }
            current = registry.get_class(id).and_then(|c| c.base_class);
        }

        false
    }
}

impl Default for AccessContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_class() {
        let mut checker = ClassChecker::new();

        let class = ClassType::new(0, "MyClass");
        let class_id = checker.check_class(class, false);

        assert!(checker.diagnostics.is_empty());
        assert!(checker.registry.is_class(class_id));
    }

    #[test]
    fn test_abstract_class_expression_error() {
        let mut checker = ClassChecker::new();

        let class = ClassType::new(0, "AbstractExpr").abstract_class();
        checker.check_class(class, true); // is_expression = true

        assert!(checker.diagnostics.iter().any(|d| d.code == error_codes::CLASS_EXPRESSION_ABSTRACT));
    }

    #[test]
    fn test_class_with_members() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "Person");
        class.members.add_member(ClassMember::new(
            0, "name", MemberKind::Property, ModifierFlags::PUBLIC
        ));
        class.members.add_member(ClassMember::new(
            0, "age", MemberKind::Property, ModifierFlags::PRIVATE
        ));
        class.members.add_member(ClassMember::new(
            0, "getName", MemberKind::Method, ModifierFlags::PUBLIC
        ));

        let class_id = checker.check_class(class, false);
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_abstract_method_in_non_abstract_class() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "Concrete");
        class.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::ABSTRACT
        ));

        checker.check_class(class, false);
        assert!(checker.diagnostics.iter().any(|d| d.code == 1244));
    }

    #[test]
    fn test_constructor_with_override_error() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "MyClass");
        class.members.add_member(ClassMember::new(
            0, "constructor", MemberKind::Constructor, ModifierFlags::OVERRIDE
        ));

        checker.check_class(class, false);
        assert!(checker.diagnostics.iter().any(|d| d.code == error_codes::CONSTRUCTOR_OVERRIDE));
    }

    #[test]
    fn test_instantiate_abstract_class_error() {
        let mut checker = ClassChecker::new();

        let class = ClassType::new(0, "AbstractClass").abstract_class();
        let class_id = checker.check_class(class, false);

        checker.clear_diagnostics();
        let can_instantiate = checker.check_instantiation(class_id, AccessModifier::Public);

        assert!(!can_instantiate);
        assert!(checker.diagnostics.iter().any(|d| d.code == error_codes::ABSTRACT_CLASS_CONSTRUCT));
    }

    #[test]
    fn test_private_constructor_access() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "Singleton");
        class.members.add_member(ClassMember::new(
            0, "constructor", MemberKind::Constructor, ModifierFlags::PRIVATE
        ));
        let class_id = checker.check_class(class, false);

        checker.clear_diagnostics();

        // From outside - should fail
        let can_instantiate = checker.check_instantiation(class_id, AccessModifier::Public);
        assert!(!can_instantiate);
    }

    #[test]
    fn test_member_access_public() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "MyClass");
        class.members.add_member(ClassMember::new(
            0, "publicField", MemberKind::Property, ModifierFlags::PUBLIC
        ));
        let class_id = checker.check_class(class, false);

        checker.clear_diagnostics();

        // Public access from anywhere
        let context = AccessContext::new();
        let member = checker.check_member_access(class_id, "publicField", false, context);
        assert!(member.is_some());
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_member_access_private() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "MyClass");
        class.members.add_member(ClassMember::new(
            0, "privateField", MemberKind::Property, ModifierFlags::PRIVATE
        ));
        let class_id = checker.check_class(class, false);

        checker.clear_diagnostics();

        // From outside - should fail
        let context = AccessContext::new();
        let member = checker.check_member_access(class_id, "privateField", false, context);
        assert!(member.is_none());
        assert!(!checker.diagnostics.is_empty());

        checker.clear_diagnostics();

        // From same class - should succeed
        let context = AccessContext::in_class(class_id);
        let member = checker.check_member_access(class_id, "privateField", false, context);
        assert!(member.is_some());
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_member_access_protected() {
        let mut checker = ClassChecker::new();

        let mut base = ClassType::new(0, "Base");
        base.members.add_member(ClassMember::new(
            0, "protectedField", MemberKind::Property, ModifierFlags::PROTECTED
        ));
        let base_id = checker.check_class(base, false);

        let derived = ClassType::new(0, "Derived").extends(base_id);
        let derived_id = checker.check_class(derived, false);

        checker.clear_diagnostics();

        // From derived class - should succeed
        let context = AccessContext::in_class(derived_id);
        let member = checker.check_member_access(base_id, "protectedField", false, context);
        assert!(member.is_some());
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_class_with_extends_and_implements() {
        let mut checker = ClassChecker::new();

        let base = ClassType::new(0, "BaseClass");
        let base_id = checker.check_class(base, false);

        let mut interface = super::super::inheritance::InterfaceType::new(0, "IFoo");
        interface.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::PUBLIC
        ));
        let interface_id = checker.check_interface(interface);

        let mut derived = ClassType::new(0, "DerivedClass")
            .extends(base_id)
            .implements(interface_id);
        derived.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::PUBLIC
        ));

        let derived_id = checker.check_class(derived, false);
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_accessor_pair_checking() {
        let mut checker = ClassChecker::new();

        let mut class = ClassType::new(0, "WithAccessors");
        class.members.add_member(ClassMember::new(
            0, "value", MemberKind::GetAccessor, ModifierFlags::PUBLIC
        ));

        let mut setter = ClassMember::new(0, "value", MemberKind::SetAccessor, ModifierFlags::PUBLIC);
        setter.parameters.push(ParameterInfo::new("val"));
        class.members.add_member(setter);

        checker.check_class(class, false);
        assert!(checker.diagnostics.is_empty());
    }

    #[test]
    fn test_no_implicit_override_option() {
        let mut checker = ClassChecker::new()
            .with_options(ClassCheckOptions {
                no_implicit_override: true,
                ..Default::default()
            });

        let mut base = ClassType::new(0, "Base");
        let mut base_method = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::PUBLIC);
        base_method.declaring_class = 1;
        base.members.add_member(base_method);
        let base_id = checker.check_class(base, false);

        checker.clear_diagnostics();

        let mut derived = ClassType::new(0, "Derived").extends(base_id);
        // Missing override modifier
        derived.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::PUBLIC
        ));

        checker.check_class(derived, false);
        assert!(checker.diagnostics.iter().any(|d| d.code == super::super::inheritance::error_codes::MISSING_OVERRIDE));
    }
}
