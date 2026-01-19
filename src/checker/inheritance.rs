//! Class Inheritance Type Checking
//!
//! This module handles:
//! - Extends clause validation
//! - Implements clause validation
//! - Override checking
//! - Abstract method implementation checking

use super::modifiers::{ModifierFlags, AccessModifier};
use super::members::{ClassMember, MemberKind, MemberTable, TypeId, MemberId};
use std::collections::{HashMap, HashSet};

/// Represents a class type for inheritance checking
#[derive(Debug, Clone)]
pub struct ClassType {
    pub id: TypeId,
    pub name: String,
    pub is_abstract: bool,
    pub base_class: Option<TypeId>,
    pub implemented_interfaces: Vec<TypeId>,
    pub members: MemberTable,
    /// Type parameters
    pub type_parameters: Vec<TypeParameterInfo>,
}

impl ClassType {
    pub fn new(id: TypeId, name: impl Into<String>) -> Self {
        ClassType {
            id,
            name: name.into(),
            is_abstract: false,
            base_class: None,
            implemented_interfaces: Vec::new(),
            members: MemberTable::new(),
            type_parameters: Vec::new(),
        }
    }

    pub fn abstract_class(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    pub fn extends(mut self, base: TypeId) -> Self {
        self.base_class = Some(base);
        self
    }

    pub fn implements(mut self, interface: TypeId) -> Self {
        self.implemented_interfaces.push(interface);
        self
    }
}

/// Type parameter information
#[derive(Debug, Clone)]
pub struct TypeParameterInfo {
    pub name: String,
    pub constraint: Option<TypeId>,
    pub default: Option<TypeId>,
}

/// Represents an interface type
#[derive(Debug, Clone)]
pub struct InterfaceType {
    pub id: TypeId,
    pub name: String,
    pub members: MemberTable,
    pub extended_interfaces: Vec<TypeId>,
    pub type_parameters: Vec<TypeParameterInfo>,
}

impl InterfaceType {
    pub fn new(id: TypeId, name: impl Into<String>) -> Self {
        InterfaceType {
            id,
            name: name.into(),
            members: MemberTable::new(),
            extended_interfaces: Vec::new(),
            type_parameters: Vec::new(),
        }
    }
}

/// Registry for looking up types during inheritance checking
#[derive(Debug, Default)]
pub struct TypeRegistry {
    classes: HashMap<TypeId, ClassType>,
    interfaces: HashMap<TypeId, InterfaceType>,
    next_id: TypeId,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry {
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn register_class(&mut self, mut class: ClassType) -> TypeId {
        let id = self.next_id;
        self.next_id += 1;
        class.id = id;
        self.classes.insert(id, class);
        id
    }

    pub fn register_interface(&mut self, mut interface: InterfaceType) -> TypeId {
        let id = self.next_id;
        self.next_id += 1;
        interface.id = id;
        self.interfaces.insert(id, interface);
        id
    }

    pub fn get_class(&self, id: TypeId) -> Option<&ClassType> {
        self.classes.get(&id)
    }

    pub fn get_class_mut(&mut self, id: TypeId) -> Option<&mut ClassType> {
        self.classes.get_mut(&id)
    }

    pub fn get_interface(&self, id: TypeId) -> Option<&InterfaceType> {
        self.interfaces.get(&id)
    }

    /// Check if a type is a class
    pub fn is_class(&self, id: TypeId) -> bool {
        self.classes.contains_key(&id)
    }

    /// Check if a type is an interface
    pub fn is_interface(&self, id: TypeId) -> bool {
        self.interfaces.contains_key(&id)
    }
}

/// Diagnostic for inheritance errors
#[derive(Debug, Clone)]
pub struct InheritanceError {
    pub message: String,
    pub code: u32,
    pub class_id: Option<TypeId>,
    pub member_name: Option<String>,
}

/// Diagnostic codes for inheritance errors
pub mod error_codes {
    pub const EXTENDS_NON_CLASS: u32 = 2311;
    pub const EXTENDS_FINAL_CLASS: u32 = 2509;
    pub const IMPLEMENTS_CLASS: u32 = 2422;
    pub const CYCLIC_INHERITANCE: u32 = 2310;
    pub const ABSTRACT_NOT_IMPLEMENTED: u32 = 2515;
    pub const INTERFACE_NOT_IMPLEMENTED: u32 = 2420;
    pub const OVERRIDE_NOT_FOUND: u32 = 4113;
    pub const MISSING_OVERRIDE: u32 = 4114;
    pub const INCOMPATIBLE_OVERRIDE: u32 = 2416;
    pub const CANNOT_OVERRIDE_PRIVATE: u32 = 4115;
    pub const VISIBILITY_DECREASE: u32 = 2415;
    pub const STATIC_INSTANCE_MISMATCH: u32 = 2423;
    pub const CANNOT_EXTEND_INTERFACE: u32 = 2689;
}

/// Inheritance checker
#[derive(Debug)]
pub struct InheritanceChecker<'a> {
    registry: &'a TypeRegistry,
    /// Options for checking
    no_implicit_override: bool,
}

impl<'a> InheritanceChecker<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        InheritanceChecker {
            registry,
            no_implicit_override: false,
        }
    }

    /// Enable noImplicitOverride checking
    pub fn with_implicit_override_check(mut self) -> Self {
        self.no_implicit_override = true;
        self
    }

    /// Check a class for inheritance errors
    pub fn check_class(&self, class_id: TypeId) -> Vec<InheritanceError> {
        let mut errors = Vec::new();

        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return errors,
        };

        // Check for cyclic inheritance FIRST to avoid infinite recursion
        let has_cycle = self.has_cyclic_inheritance(class_id);
        if has_cycle {
            errors.push(InheritanceError {
                message: format!("Type '{}' recursively references itself as a base type", class.name),
                code: error_codes::CYCLIC_INHERITANCE,
                class_id: Some(class_id),
                member_name: None,
            });
            // Return early if there's a cycle - other checks might infinite loop
            return errors;
        }

        // Check extends clause
        if let Some(base_id) = class.base_class {
            errors.extend(self.check_extends(class, base_id));
        }

        // Check implements clauses
        for interface_id in &class.implemented_interfaces {
            errors.extend(self.check_implements(class, *interface_id));
        }

        // Check override modifiers
        errors.extend(self.check_overrides(class));

        // Check abstract method implementations (if not abstract)
        if !class.is_abstract {
            errors.extend(self.check_abstract_implementations(class));
        }

        errors
    }

    fn check_extends(&self, class: &ClassType, base_id: TypeId) -> Vec<InheritanceError> {
        let mut errors = Vec::new();

        // Must extend a class, not an interface
        if self.registry.is_interface(base_id) {
            errors.push(InheritanceError {
                message: "A class can only extend another class".to_string(),
                code: error_codes::CANNOT_EXTEND_INTERFACE,
                class_id: Some(class.id),
                member_name: None,
            });
            return errors;
        }

        let base = match self.registry.get_class(base_id) {
            Some(b) => b,
            None => {
                errors.push(InheritanceError {
                    message: "Base class not found".to_string(),
                    code: error_codes::EXTENDS_NON_CLASS,
                    class_id: Some(class.id),
                    member_name: None,
                });
                return errors;
            }
        };

        // Check member compatibility
        for member in class.members.instance_members_iter() {
            if let Some(base_members) = base.members.get_instance_members(&member.name) {
                for base_member_id in base_members {
                    if let Some(base_member) = base.members.get_member(*base_member_id) {
                        errors.extend(self.check_member_override(member, base_member, class));
                    }
                }
            }
        }

        errors
    }

    fn check_implements(&self, class: &ClassType, interface_id: TypeId) -> Vec<InheritanceError> {
        let mut errors = Vec::new();

        // Must implement an interface, not a class
        if self.registry.is_class(interface_id) {
            errors.push(InheritanceError {
                message: "A class can only implement an interface".to_string(),
                code: error_codes::IMPLEMENTS_CLASS,
                class_id: Some(class.id),
                member_name: None,
            });
            return errors;
        }

        let interface = match self.registry.get_interface(interface_id) {
            Some(i) => i,
            None => return errors,
        };

        // Check all interface members are implemented
        for interface_member in interface.members.instance_members_iter() {
            let implemented = class.members.has_member(&interface_member.name, false) ||
                self.find_inherited_member(class, &interface_member.name).is_some();

            if !implemented {
                errors.push(InheritanceError {
                    message: format!(
                        "Class '{}' incorrectly implements interface '{}'. Property '{}' is missing",
                        class.name, interface.name, interface_member.name
                    ),
                    code: error_codes::INTERFACE_NOT_IMPLEMENTED,
                    class_id: Some(class.id),
                    member_name: Some(interface_member.name.clone()),
                });
            }
        }

        errors
    }

    fn check_member_override(
        &self,
        member: &ClassMember,
        base_member: &ClassMember,
        class: &ClassType,
    ) -> Vec<InheritanceError> {
        let mut errors = Vec::new();

        // Cannot override private members
        if base_member.is_private() {
            errors.push(InheritanceError {
                message: format!(
                    "Class '{}' defines instance member property '{}', but extended class '{}' defines it as private",
                    class.name, member.name, self.get_class_name(base_member.declaring_class)
                ),
                code: error_codes::CANNOT_OVERRIDE_PRIVATE,
                class_id: Some(class.id),
                member_name: Some(member.name.clone()),
            });
            return errors;
        }

        // Check visibility is not decreased
        if !self.is_visibility_compatible(member.access_modifier(), base_member.access_modifier()) {
            errors.push(InheritanceError {
                message: format!(
                    "Property '{}' has a more restrictive visibility than the base member",
                    member.name
                ),
                code: error_codes::VISIBILITY_DECREASE,
                class_id: Some(class.id),
                member_name: Some(member.name.clone()),
            });
        }

        // Check static/instance consistency
        if member.is_static() != base_member.is_static() {
            errors.push(InheritanceError {
                message: format!(
                    "Property '{}' is {} in base class but {} in derived class",
                    member.name,
                    if base_member.is_static() { "static" } else { "instance" },
                    if member.is_static() { "static" } else { "instance" }
                ),
                code: error_codes::STATIC_INSTANCE_MISMATCH,
                class_id: Some(class.id),
                member_name: Some(member.name.clone()),
            });
        }

        // Check noImplicitOverride
        if self.no_implicit_override && !member.is_override() {
            errors.push(InheritanceError {
                message: format!(
                    "This member must have an 'override' modifier because it overrides a member in the base class '{}'",
                    self.get_class_name(base_member.declaring_class)
                ),
                code: error_codes::MISSING_OVERRIDE,
                class_id: Some(class.id),
                member_name: Some(member.name.clone()),
            });
        }

        errors
    }

    fn check_overrides(&self, class: &ClassType) -> Vec<InheritanceError> {
        let mut errors = Vec::new();

        for member in class.members.instance_members_iter() {
            if member.is_override() {
                // Must have a base class member to override
                let base_member = self.find_inherited_member(class, &member.name);

                if base_member.is_none() {
                    errors.push(InheritanceError {
                        message: format!(
                            "This member cannot have an 'override' modifier because it is not declared in the base class '{}'",
                            class.base_class.map(|id| self.get_class_name(id)).unwrap_or_default()
                        ),
                        code: error_codes::OVERRIDE_NOT_FOUND,
                        class_id: Some(class.id),
                        member_name: Some(member.name.clone()),
                    });
                }
            }
        }

        errors
    }

    fn check_abstract_implementations(&self, class: &ClassType) -> Vec<InheritanceError> {
        let mut errors = Vec::new();
        let mut unimplemented: Vec<(String, TypeId)> = Vec::new();

        // Collect all abstract members from base classes
        self.collect_abstract_members(class.base_class, &mut unimplemented);

        // Check each abstract member is implemented
        for (name, declaring_class) in unimplemented {
            let implemented = class.members.has_member(&name, false);

            if !implemented {
                errors.push(InheritanceError {
                    message: format!(
                        "Non-abstract class '{}' does not implement inherited abstract member '{}' from class '{}'",
                        class.name, name, self.get_class_name(declaring_class)
                    ),
                    code: error_codes::ABSTRACT_NOT_IMPLEMENTED,
                    class_id: Some(class.id),
                    member_name: Some(name),
                });
            }
        }

        errors
    }

    fn collect_abstract_members(&self, class_id: Option<TypeId>, result: &mut Vec<(String, TypeId)>) {
        let mut visited = HashSet::new();
        self.collect_abstract_members_impl(class_id, result, &mut visited);
    }

    fn collect_abstract_members_impl(
        &self,
        class_id: Option<TypeId>,
        result: &mut Vec<(String, TypeId)>,
        visited: &mut HashSet<TypeId>,
    ) {
        let class_id = match class_id {
            Some(id) => id,
            None => return,
        };

        // Prevent infinite recursion on cyclic inheritance
        if visited.contains(&class_id) {
            return;
        }
        visited.insert(class_id);

        let class = match self.registry.get_class(class_id) {
            Some(c) => c,
            None => return,
        };

        // Recurse to base class first
        self.collect_abstract_members_impl(class.base_class, result, visited);

        // Add abstract members from this class
        for member in class.members.instance_members_iter() {
            if member.is_abstract() {
                result.push((member.name.clone(), class_id));
            }
        }

        // Remove implemented members
        result.retain(|(name, _)| {
            !class.members.instance_members_iter()
                .any(|m| &m.name == name && !m.is_abstract())
        });
    }

    fn has_cyclic_inheritance(&self, class_id: TypeId) -> bool {
        let mut visited = HashSet::new();
        let mut current = Some(class_id);

        while let Some(id) = current {
            if visited.contains(&id) {
                return true;
            }
            visited.insert(id);

            current = self.registry.get_class(id).and_then(|c| c.base_class);
        }

        false
    }

    fn find_inherited_member(&self, class: &ClassType, name: &str) -> Option<&ClassMember> {
        let mut current_base = class.base_class;
        let mut visited = HashSet::new();

        while let Some(base_id) = current_base {
            // Prevent infinite loop on cyclic inheritance
            if visited.contains(&base_id) {
                break;
            }
            visited.insert(base_id);

            if let Some(base) = self.registry.get_class(base_id) {
                if let Some(member_ids) = base.members.get_instance_members(name) {
                    if let Some(member_id) = member_ids.first() {
                        return base.members.get_member(*member_id);
                    }
                }
                current_base = base.base_class;
            } else {
                break;
            }
        }

        None
    }

    fn is_visibility_compatible(&self, derived: AccessModifier, base: AccessModifier) -> bool {
        match base {
            AccessModifier::Public => matches!(derived, AccessModifier::Public),
            AccessModifier::Protected => matches!(derived, AccessModifier::Public | AccessModifier::Protected),
            AccessModifier::Private => true, // Private cannot be overridden anyway
        }
    }

    fn get_class_name(&self, id: TypeId) -> String {
        self.registry.get_class(id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> TypeRegistry {
        TypeRegistry::new()
    }

    #[test]
    fn test_simple_extends() {
        let mut registry = create_test_registry();

        let base = ClassType::new(0, "Base");
        let base_id = registry.register_class(base);

        let derived = ClassType::new(0, "Derived").extends(base_id);
        let derived_id = registry.register_class(derived);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(derived_id);

        assert!(errors.is_empty());
    }

    #[test]
    fn test_extend_interface_error() {
        let mut registry = create_test_registry();

        let interface = InterfaceType::new(0, "IFoo");
        let interface_id = registry.register_interface(interface);

        let class = ClassType::new(0, "Foo").extends(interface_id);
        let class_id = registry.register_class(class);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(class_id);

        assert!(errors.iter().any(|e| e.code == error_codes::CANNOT_EXTEND_INTERFACE));
    }

    #[test]
    fn test_implement_class_error() {
        let mut registry = create_test_registry();

        let other_class = ClassType::new(0, "OtherClass");
        let other_id = registry.register_class(other_class);

        let class = ClassType::new(0, "MyClass").implements(other_id);
        let class_id = registry.register_class(class);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(class_id);

        assert!(errors.iter().any(|e| e.code == error_codes::IMPLEMENTS_CLASS));
    }

    #[test]
    fn test_interface_implementation_missing() {
        let mut registry = create_test_registry();

        let mut interface = InterfaceType::new(0, "IFoo");
        interface.members.add_member(ClassMember::new(
            0, "bar", MemberKind::Method, ModifierFlags::PUBLIC
        ));
        let interface_id = registry.register_interface(interface);

        let class = ClassType::new(0, "Foo").implements(interface_id);
        let class_id = registry.register_class(class);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(class_id);

        assert!(errors.iter().any(|e| e.code == error_codes::INTERFACE_NOT_IMPLEMENTED));
    }

    #[test]
    fn test_interface_implementation_complete() {
        let mut registry = create_test_registry();

        let mut interface = InterfaceType::new(0, "IFoo");
        interface.members.add_member(ClassMember::new(
            0, "bar", MemberKind::Method, ModifierFlags::PUBLIC
        ));
        let interface_id = registry.register_interface(interface);

        let mut class = ClassType::new(0, "Foo").implements(interface_id);
        class.members.add_member(ClassMember::new(
            0, "bar", MemberKind::Method, ModifierFlags::PUBLIC
        ));
        let class_id = registry.register_class(class);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(class_id);

        assert!(errors.is_empty());
    }

    #[test]
    fn test_cyclic_inheritance() {
        let mut registry = create_test_registry();

        // Create a cycle: A extends B, B extends A
        let a = ClassType::new(0, "A");
        let a_id = registry.register_class(a);

        let b = ClassType::new(0, "B").extends(a_id);
        let b_id = registry.register_class(b);

        // Update A to extend B (creating cycle)
        registry.get_class_mut(a_id).unwrap().base_class = Some(b_id);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(a_id);

        assert!(errors.iter().any(|e| e.code == error_codes::CYCLIC_INHERITANCE));
    }

    #[test]
    fn test_abstract_not_implemented() {
        let mut registry = create_test_registry();

        let mut base = ClassType::new(0, "Base").abstract_class();
        let mut method = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::ABSTRACT);
        method.declaring_class = 1; // Will be base_id
        base.members.add_member(method);
        let base_id = registry.register_class(base);

        // Update declaring_class
        registry.get_class_mut(base_id).unwrap()
            .members.instance_members_iter()
            .for_each(|_| {});

        let derived = ClassType::new(0, "Derived").extends(base_id);
        let derived_id = registry.register_class(derived);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(derived_id);

        assert!(errors.iter().any(|e| e.code == error_codes::ABSTRACT_NOT_IMPLEMENTED));
    }

    #[test]
    fn test_override_not_found() {
        let mut registry = create_test_registry();

        let base = ClassType::new(0, "Base");
        let base_id = registry.register_class(base);

        let mut derived = ClassType::new(0, "Derived").extends(base_id);
        derived.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::OVERRIDE
        ));
        let derived_id = registry.register_class(derived);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(derived_id);

        assert!(errors.iter().any(|e| e.code == error_codes::OVERRIDE_NOT_FOUND));
    }

    #[test]
    fn test_visibility_decrease_error() {
        let mut registry = create_test_registry();

        let mut base = ClassType::new(0, "Base");
        let mut base_method = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::PUBLIC);
        base_method.declaring_class = 1;
        base.members.add_member(base_method);
        let base_id = registry.register_class(base);

        let mut derived = ClassType::new(0, "Derived").extends(base_id);
        derived.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::PROTECTED
        ));
        let derived_id = registry.register_class(derived);

        let checker = InheritanceChecker::new(&registry);
        let errors = checker.check_class(derived_id);

        assert!(errors.iter().any(|e| e.code == error_codes::VISIBILITY_DECREASE));
    }

    #[test]
    fn test_no_implicit_override() {
        let mut registry = create_test_registry();

        let mut base = ClassType::new(0, "Base");
        let mut base_method = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::PUBLIC);
        base_method.declaring_class = 1;
        base.members.add_member(base_method);
        let base_id = registry.register_class(base);

        let mut derived = ClassType::new(0, "Derived").extends(base_id);
        derived.members.add_member(ClassMember::new(
            0, "foo", MemberKind::Method, ModifierFlags::PUBLIC // No OVERRIDE
        ));
        let derived_id = registry.register_class(derived);

        let checker = InheritanceChecker::new(&registry).with_implicit_override_check();
        let errors = checker.check_class(derived_id);

        assert!(errors.iter().any(|e| e.code == error_codes::MISSING_OVERRIDE));
    }
}
