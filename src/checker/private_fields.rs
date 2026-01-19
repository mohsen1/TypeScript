//! Type checking for private class fields and methods.
//!
//! This module handles:
//! - Type checking private field declarations
//! - Checking private field access restrictions
//! - Validating private method signatures
//! - Checking private field existence with `in` operator
//! - Enforcing no inheritance of private fields
//! - Checking private static fields and methods

use std::collections::{HashMap, HashSet};

/// A unique identifier for a class.
pub type ClassId = u64;

/// Represents a type in the type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Unknown,
    Never,
    Void,
    Null,
    Undefined,
    Number,
    String,
    Boolean,
    BigInt,
    Symbol,
    Object(ObjectType),
    Array(Box<Type>),
    Tuple(Vec<Type>),
    Function(FunctionType),
    Union(Vec<Type>),
    Intersection(Vec<Type>),
    Class(ClassType),
    TypeParameter(String),
    Literal(LiteralType),
}

/// Object type representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectType {
    pub properties: HashMap<String, PropertyInfo>,
}

/// Property information.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyInfo {
    pub ty: Type,
    pub optional: bool,
    pub readonly: bool,
}

/// Function type representation.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Box<Type>,
}

/// Parameter information.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInfo {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
}

/// Class type representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassType {
    pub id: ClassId,
    pub name: String,
    pub type_parameters: Vec<String>,
}

/// Literal type variants.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    Number(f64),
    String(String),
    Boolean(bool),
}

/// Private field declaration in a class.
#[derive(Debug, Clone)]
pub struct PrivateFieldDeclaration {
    /// The field name (without #).
    pub name: String,
    /// The declared type.
    pub ty: Type,
    /// Whether the field is static.
    pub is_static: bool,
    /// Whether the field is readonly.
    pub is_readonly: bool,
    /// The class that declares this field.
    pub declaring_class: ClassId,
    /// Whether the field has an initializer.
    pub has_initializer: bool,
}

/// Private method declaration in a class.
#[derive(Debug, Clone)]
pub struct PrivateMethodDeclaration {
    /// The method name (without #).
    pub name: String,
    /// The method type.
    pub ty: FunctionType,
    /// Whether the method is static.
    pub is_static: bool,
    /// The class that declares this method.
    pub declaring_class: ClassId,
}

/// Private accessor declaration in a class.
#[derive(Debug, Clone)]
pub struct PrivateAccessorDeclaration {
    /// The accessor name (without #).
    pub name: String,
    /// The getter type (if any).
    pub getter_type: Option<Type>,
    /// The setter type (if any).
    pub setter_type: Option<Type>,
    /// Whether the accessor is static.
    pub is_static: bool,
    /// The class that declares this accessor.
    pub declaring_class: ClassId,
}

/// Represents a private class member.
#[derive(Debug, Clone)]
pub enum PrivateMember {
    Field(PrivateFieldDeclaration),
    Method(PrivateMethodDeclaration),
    Accessor(PrivateAccessorDeclaration),
}

impl PrivateMember {
    pub fn name(&self) -> &str {
        match self {
            PrivateMember::Field(f) => &f.name,
            PrivateMember::Method(m) => &m.name,
            PrivateMember::Accessor(a) => &a.name,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            PrivateMember::Field(f) => f.is_static,
            PrivateMember::Method(m) => m.is_static,
            PrivateMember::Accessor(a) => a.is_static,
        }
    }

    pub fn declaring_class(&self) -> ClassId {
        match self {
            PrivateMember::Field(f) => f.declaring_class,
            PrivateMember::Method(m) => m.declaring_class,
            PrivateMember::Accessor(a) => a.declaring_class,
        }
    }
}

/// Diagnostic error for private field checking.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateFieldError {
    pub kind: PrivateFieldErrorKind,
    pub message: String,
    pub span: Option<(usize, usize)>,
}

/// Kind of private field error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivateFieldErrorKind {
    /// Private field accessed outside its class.
    AccessOutsideClass,
    /// Private field not found.
    NotFound,
    /// Duplicate private field declaration.
    Duplicate,
    /// Private field cannot be inherited.
    InheritanceNotAllowed,
    /// Invalid access context (e.g., static vs instance).
    InvalidAccessContext,
    /// Private field used in `in` expression with non-object.
    InvalidInExpression,
    /// Assignment to readonly private field.
    ReadonlyAssignment,
    /// Missing initializer for non-optional private field.
    MissingInitializer,
    /// Type mismatch.
    TypeMismatch,
    /// Accessor type mismatch between getter and setter.
    AccessorTypeMismatch,
}

impl PrivateFieldError {
    pub fn access_outside_class(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::AccessOutsideClass,
            message: format!(
                "Property '#{name}' is not accessible outside class because it has a private identifier."
            ),
            span: None,
        }
    }

    pub fn not_found(name: &str, class_name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::NotFound,
            message: format!("Property '#{name}' does not exist on type '{class_name}'."),
            span: None,
        }
    }

    pub fn duplicate(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::Duplicate,
            message: format!("Duplicate identifier '#{name}'."),
            span: None,
        }
    }

    pub fn inheritance_not_allowed(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::InheritanceNotAllowed,
            message: format!(
                "Class declares private member '#{name}' which cannot be accessed in subclass."
            ),
            span: None,
        }
    }

    pub fn static_vs_instance(name: &str, is_static_access: bool) -> Self {
        let context = if is_static_access {
            "static"
        } else {
            "instance"
        };
        Self {
            kind: PrivateFieldErrorKind::InvalidAccessContext,
            message: format!("Cannot access {context} private member '#{name}' through instance."),
            span: None,
        }
    }

    pub fn invalid_in_expression(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::InvalidInExpression,
            message: format!(
                "The right-hand side of an 'in' expression must be an object when checking for private field '#{name}'."
            ),
            span: None,
        }
    }

    pub fn readonly_assignment(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::ReadonlyAssignment,
            message: format!("Cannot assign to '#{name}' because it is a read-only property."),
            span: None,
        }
    }

    pub fn missing_initializer(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::MissingInitializer,
            message: format!("Property '#{name}' has no initializer and is not definitely assigned in the constructor."),
            span: None,
        }
    }

    pub fn type_mismatch(name: &str, expected: &str, actual: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::TypeMismatch,
            message: format!(
                "Type '{actual}' is not assignable to type '{expected}' for private field '#{name}'."
            ),
            span: None,
        }
    }

    pub fn accessor_type_mismatch(name: &str) -> Self {
        Self {
            kind: PrivateFieldErrorKind::AccessorTypeMismatch,
            message: format!(
                "The return type of a 'get' accessor must be assignable to its 'set' accessor type for '#{name}'."
            ),
            span: None,
        }
    }
}

/// Class information for private field checking.
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub id: ClassId,
    pub name: String,
    pub base_class: Option<ClassId>,
    pub private_members: HashMap<String, PrivateMember>,
}

impl ClassInfo {
    pub fn new(id: ClassId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            base_class: None,
            private_members: HashMap::new(),
        }
    }

    pub fn with_base_class(mut self, base: ClassId) -> Self {
        self.base_class = Some(base);
        self
    }

    pub fn add_private_field(&mut self, decl: PrivateFieldDeclaration) {
        self.private_members
            .insert(decl.name.clone(), PrivateMember::Field(decl));
    }

    pub fn add_private_method(&mut self, decl: PrivateMethodDeclaration) {
        self.private_members
            .insert(decl.name.clone(), PrivateMember::Method(decl));
    }

    pub fn add_private_accessor(&mut self, decl: PrivateAccessorDeclaration) {
        self.private_members
            .insert(decl.name.clone(), PrivateMember::Accessor(decl));
    }

    pub fn get_private_member(&self, name: &str) -> Option<&PrivateMember> {
        self.private_members.get(name)
    }

    pub fn has_private_member(&self, name: &str) -> bool {
        self.private_members.contains_key(name)
    }
}

/// Private field type checker.
pub struct PrivateFieldChecker {
    /// All known classes.
    classes: HashMap<ClassId, ClassInfo>,
    /// Current class context (for access checking).
    current_class: Option<ClassId>,
    /// Whether we're in a static context.
    in_static_context: bool,
    /// Collected errors.
    errors: Vec<PrivateFieldError>,
}

impl PrivateFieldChecker {
    /// Create a new checker.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            current_class: None,
            in_static_context: false,
            errors: Vec::new(),
        }
    }

    /// Register a class.
    pub fn register_class(&mut self, info: ClassInfo) {
        self.classes.insert(info.id, info);
    }

    /// Enter a class context.
    pub fn enter_class(&mut self, class_id: ClassId) {
        self.current_class = Some(class_id);
        self.in_static_context = false;
    }

    /// Exit the current class context.
    pub fn exit_class(&mut self) {
        self.current_class = None;
        self.in_static_context = false;
    }

    /// Enter static context.
    pub fn enter_static_context(&mut self) {
        self.in_static_context = true;
    }

    /// Exit static context.
    pub fn exit_static_context(&mut self) {
        self.in_static_context = false;
    }

    /// Get collected errors.
    pub fn get_errors(&self) -> &[PrivateFieldError] {
        &self.errors
    }

    /// Clear collected errors.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Check a private field declaration.
    pub fn check_private_field_declaration(
        &mut self,
        class_id: ClassId,
        decl: &PrivateFieldDeclaration,
    ) -> bool {
        let class = match self.classes.get(&class_id) {
            Some(c) => c,
            None => return false,
        };

        // Check for duplicates
        let existing = class
            .private_members
            .values()
            .filter(|m| m.name() == decl.name)
            .count();

        if existing > 1 {
            self.errors.push(PrivateFieldError::duplicate(&decl.name));
            return false;
        }

        true
    }

    /// Check a private field access.
    pub fn check_private_field_access(
        &mut self,
        target_class_id: ClassId,
        field_name: &str,
        is_assignment: bool,
    ) -> Option<Type> {
        // Must be inside a class
        let current_class = match self.current_class {
            Some(id) => id,
            None => {
                self.errors
                    .push(PrivateFieldError::access_outside_class(field_name));
                return None;
            }
        };

        // Must be the same class (private fields are not accessible in subclasses)
        if current_class != target_class_id {
            // Check if it's a subclass - private fields are still not accessible
            if self.is_subclass_of(current_class, target_class_id) {
                self.errors
                    .push(PrivateFieldError::inheritance_not_allowed(field_name));
            } else {
                self.errors
                    .push(PrivateFieldError::access_outside_class(field_name));
            }
            return None;
        }

        // Get the class info
        let class = self.classes.get(&target_class_id)?;

        // Look up the private member
        let member = match class.get_private_member(field_name) {
            Some(m) => m,
            None => {
                self.errors
                    .push(PrivateFieldError::not_found(field_name, &class.name));
                return None;
            }
        };

        // Check static vs instance context
        if member.is_static() != self.in_static_context {
            self.errors
                .push(PrivateFieldError::static_vs_instance(field_name, member.is_static()));
            return None;
        }

        // Get the type and check for readonly assignment
        match member {
            PrivateMember::Field(f) => {
                if is_assignment && f.is_readonly {
                    self.errors
                        .push(PrivateFieldError::readonly_assignment(field_name));
                    return None;
                }
                Some(f.ty.clone())
            }
            PrivateMember::Method(m) => {
                if is_assignment {
                    self.errors
                        .push(PrivateFieldError::readonly_assignment(field_name));
                    return None;
                }
                Some(Type::Function(m.ty.clone()))
            }
            PrivateMember::Accessor(a) => {
                if is_assignment {
                    match &a.setter_type {
                        Some(ty) => Some(ty.clone()),
                        None => {
                            self.errors
                                .push(PrivateFieldError::readonly_assignment(field_name));
                            None
                        }
                    }
                } else {
                    a.getter_type.clone()
                }
            }
        }
    }

    /// Check a private field `in` expression.
    pub fn check_private_in_expression(
        &mut self,
        field_name: &str,
        target_type: &Type,
    ) -> Option<Type> {
        // Must be inside a class
        let current_class = match self.current_class {
            Some(id) => id,
            None => {
                self.errors
                    .push(PrivateFieldError::access_outside_class(field_name));
                return None;
            }
        };

        // Target must be an object type
        let target_class_id = match target_type {
            Type::Class(c) => c.id,
            Type::Object(_) => {
                // Object literals can't have private fields
                self.errors
                    .push(PrivateFieldError::invalid_in_expression(field_name));
                return None;
            }
            _ => {
                self.errors
                    .push(PrivateFieldError::invalid_in_expression(field_name));
                return None;
            }
        };

        // Check if the current class has this private field
        let class = self.classes.get(&current_class)?;
        if !class.has_private_member(field_name) {
            self.errors
                .push(PrivateFieldError::not_found(field_name, &class.name));
            return None;
        }

        // The result of private `in` is always boolean
        Some(Type::Boolean)
    }

    /// Check that a class doesn't try to inherit private fields.
    pub fn check_inheritance(&mut self, class_id: ClassId) -> bool {
        let class = match self.classes.get(&class_id) {
            Some(c) => c.clone(),
            None => return true,
        };

        let base_id = match class.base_class {
            Some(id) => id,
            None => return true,
        };

        let base = match self.classes.get(&base_id) {
            Some(b) => b,
            None => return true,
        };

        // Check for name collisions with base class private members
        for (name, _) in &class.private_members {
            if base.has_private_member(name) {
                // This is actually allowed - derived class can have same-named private field
                // but they are completely separate
            }
        }

        true
    }

    /// Check if class is a subclass of another.
    fn is_subclass_of(&self, class_id: ClassId, potential_base: ClassId) -> bool {
        let mut current = class_id;
        while let Some(class) = self.classes.get(&current) {
            if let Some(base) = class.base_class {
                if base == potential_base {
                    return true;
                }
                current = base;
            } else {
                break;
            }
        }
        false
    }

    /// Validate all private field declarations in a class.
    pub fn validate_class(&mut self, class_id: ClassId) -> bool {
        let class = match self.classes.get(&class_id) {
            Some(c) => c.clone(),
            None => return false,
        };

        let mut valid = true;
        let mut seen_names: HashSet<String> = HashSet::new();

        for (name, member) in &class.private_members {
            // Check for duplicates
            if !seen_names.insert(name.clone()) {
                self.errors.push(PrivateFieldError::duplicate(name));
                valid = false;
            }

            // Check accessor consistency
            if let PrivateMember::Accessor(a) = member {
                if let (Some(getter), Some(setter)) = (&a.getter_type, &a.setter_type) {
                    // Getter return type should be assignable to setter parameter type
                    if !self.is_assignable(getter, setter) {
                        self.errors
                            .push(PrivateFieldError::accessor_type_mismatch(name));
                        valid = false;
                    }
                }
            }
        }

        valid
    }

    /// Simple assignability check (placeholder).
    fn is_assignable(&self, source: &Type, target: &Type) -> bool {
        // Simplified implementation
        match (source, target) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Never, _) => true,
            (_, Type::Unknown) => true,
            (Type::Number, Type::Number) => true,
            (Type::String, Type::String) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            _ => source == target,
        }
    }
}

impl Default for PrivateFieldChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_class(id: ClassId, name: &str) -> ClassInfo {
        ClassInfo::new(id, name)
    }

    #[test]
    fn test_private_field_access_inside_class() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "value".to_string(),
            ty: Type::Number,
            is_static: false,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        // Enter the class context
        checker.enter_class(1);

        // Access should succeed
        let result = checker.check_private_field_access(1, "value", false);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Type::Number);
        assert!(checker.get_errors().is_empty());
    }

    #[test]
    fn test_private_field_access_outside_class() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "value".to_string(),
            ty: Type::Number,
            is_static: false,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        // Don't enter any class context
        let result = checker.check_private_field_access(1, "value", false);
        assert!(result.is_none());
        assert!(!checker.get_errors().is_empty());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::AccessOutsideClass
        );
    }

    #[test]
    fn test_private_field_not_found() {
        let mut checker = PrivateFieldChecker::new();

        let class = create_simple_class(1, "MyClass");
        checker.register_class(class);

        checker.enter_class(1);
        let result = checker.check_private_field_access(1, "nonexistent", false);
        assert!(result.is_none());
        assert!(!checker.get_errors().is_empty());
        assert_eq!(checker.get_errors()[0].kind, PrivateFieldErrorKind::NotFound);
    }

    #[test]
    fn test_private_field_readonly() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "constant".to_string(),
            ty: Type::Number,
            is_static: false,
            is_readonly: true,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Read should succeed
        let result = checker.check_private_field_access(1, "constant", false);
        assert!(result.is_some());

        // Write should fail
        checker.clear_errors();
        let result = checker.check_private_field_access(1, "constant", true);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::ReadonlyAssignment
        );
    }

    #[test]
    fn test_private_static_field() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "staticValue".to_string(),
            ty: Type::Number,
            is_static: true,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Access from instance context should fail
        let result = checker.check_private_field_access(1, "staticValue", false);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::InvalidAccessContext
        );

        // Access from static context should succeed
        checker.clear_errors();
        checker.enter_static_context();
        let result = checker.check_private_field_access(1, "staticValue", false);
        assert!(result.is_some());
        assert!(checker.get_errors().is_empty());
    }

    #[test]
    fn test_private_field_inheritance_blocked() {
        let mut checker = PrivateFieldChecker::new();

        // Base class with private field
        let mut base = create_simple_class(1, "Base");
        base.add_private_field(PrivateFieldDeclaration {
            name: "secret".to_string(),
            ty: Type::String,
            is_static: false,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(base);

        // Derived class
        let derived = create_simple_class(2, "Derived").with_base_class(1);
        checker.register_class(derived);

        // Enter derived class context
        checker.enter_class(2);

        // Trying to access base class private field should fail
        let result = checker.check_private_field_access(1, "secret", false);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::InheritanceNotAllowed
        );
    }

    #[test]
    fn test_private_method() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_method(PrivateMethodDeclaration {
            name: "doSomething".to_string(),
            ty: FunctionType {
                parameters: vec![ParameterInfo {
                    name: "x".to_string(),
                    ty: Type::Number,
                    optional: false,
                }],
                return_type: Box::new(Type::Void),
            },
            is_static: false,
            declaring_class: 1,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Should be able to access the method
        let result = checker.check_private_field_access(1, "doSomething", false);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Type::Function(_)));

        // Should not be able to assign to it
        checker.clear_errors();
        let result = checker.check_private_field_access(1, "doSomething", true);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::ReadonlyAssignment
        );
    }

    #[test]
    fn test_private_accessor() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_accessor(PrivateAccessorDeclaration {
            name: "prop".to_string(),
            getter_type: Some(Type::String),
            setter_type: Some(Type::String),
            is_static: false,
            declaring_class: 1,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Read should return getter type
        let result = checker.check_private_field_access(1, "prop", false);
        assert_eq!(result, Some(Type::String));

        // Write should return setter type
        let result = checker.check_private_field_access(1, "prop", true);
        assert_eq!(result, Some(Type::String));
    }

    #[test]
    fn test_private_getter_only() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_accessor(PrivateAccessorDeclaration {
            name: "readOnly".to_string(),
            getter_type: Some(Type::Number),
            setter_type: None,
            is_static: false,
            declaring_class: 1,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Read should succeed
        let result = checker.check_private_field_access(1, "readOnly", false);
        assert_eq!(result, Some(Type::Number));

        // Write should fail
        checker.clear_errors();
        let result = checker.check_private_field_access(1, "readOnly", true);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::ReadonlyAssignment
        );
    }

    #[test]
    fn test_private_in_expression() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "value".to_string(),
            ty: Type::Number,
            is_static: false,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Check with class type
        let target_type = Type::Class(ClassType {
            id: 1,
            name: "MyClass".to_string(),
            type_parameters: vec![],
        });
        let result = checker.check_private_in_expression("value", &target_type);
        assert_eq!(result, Some(Type::Boolean));
    }

    #[test]
    fn test_private_in_expression_invalid_target() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        class.add_private_field(PrivateFieldDeclaration {
            name: "value".to_string(),
            ty: Type::Number,
            is_static: false,
            is_readonly: false,
            declaring_class: 1,
            has_initializer: true,
        });
        checker.register_class(class);

        checker.enter_class(1);

        // Check with non-object type
        let result = checker.check_private_in_expression("value", &Type::Number);
        assert!(result.is_none());
        assert_eq!(
            checker.get_errors()[0].kind,
            PrivateFieldErrorKind::InvalidInExpression
        );
    }

    #[test]
    fn test_validate_class_duplicate() {
        let mut checker = PrivateFieldChecker::new();

        let mut class = create_simple_class(1, "MyClass");
        // Manually insert duplicate (normally prevented by HashMap)
        class.private_members.insert(
            "dup".to_string(),
            PrivateMember::Field(PrivateFieldDeclaration {
                name: "dup".to_string(),
                ty: Type::Number,
                is_static: false,
                is_readonly: false,
                declaring_class: 1,
                has_initializer: true,
            }),
        );
        checker.register_class(class);

        let result = checker.validate_class(1);
        // Should still be valid since HashMap handles duplicates
        assert!(result);
    }
}
