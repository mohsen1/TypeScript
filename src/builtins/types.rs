//! Core type definitions for built-in types
//!
//! This module defines the Rust structs used to represent TypeScript's
//! built-in type declarations without heap allocations where possible.

/// Represents a TypeScript type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Primitive types
    Any,
    Unknown,
    Number,
    String,
    Boolean,
    Void,
    Undefined,
    Null,
    Never,
    Symbol,
    BigInt,
    Object,

    /// Named type reference (e.g., Array<T>, Promise<T>)
    TypeReference {
        name: &'static str,
        type_arguments: Vec<Type>,
    },

    /// Array type (T[])
    Array(Box<Type>),

    /// Tuple type ([T, U, ...])
    Tuple(Vec<Type>),

    /// Function type ((a: T, b: U) => R)
    Function(FunctionType),

    /// Object literal type ({ prop: T })
    ObjectLiteral(Vec<PropertySignature>),

    /// Union type (T | U)
    Union(Vec<Type>),

    /// Intersection type (T & U)
    Intersection(Vec<Type>),

    /// Literal type ("literal" | 42 | true)
    StringLiteral(&'static str),
    NumericLiteral(f64),
    BooleanLiteral(bool),

    /// Conditional type (T extends U ? X : Y)
    Conditional {
        check_type: Box<Type>,
        extends_type: Box<Type>,
        true_type: Box<Type>,
        false_type: Box<Type>,
    },

    /// Indexed access type (T[K])
    IndexedAccess {
        object_type: Box<Type>,
        index_type: Box<Type>,
    },

    /// Mapped type ({ [K in T]: U })
    Mapped {
        type_parameter: &'static str,
        constraint: Box<Type>,
        template_type: Box<Type>,
        readonly_modifier: Option<MappedModifier>,
        optional_modifier: Option<MappedModifier>,
    },

    /// Type parameter (T)
    TypeParameter {
        name: &'static str,
        constraint: Option<Box<Type>>,
        default: Option<Box<Type>>,
    },

    /// Keyof type (keyof T)
    KeyOf(Box<Type>),

    /// Typeof type (typeof x)
    TypeOf(&'static str),

    /// Template literal type (`${T}`)
    TemplateLiteral {
        head: &'static str,
        spans: Vec<TemplateLiteralSpan>,
    },

    /// This type
    This,

    /// Intrinsic type (for built-in type manipulation)
    Intrinsic(&'static str),
}

/// Mapped type modifier (+/- readonly, +/- optional)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedModifier {
    Add,
    Remove,
    Preserve,
}

/// Template literal span
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteralSpan {
    pub type_: Type,
    pub literal: &'static str,
}

/// Function type representation
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterDeclaration>,
    pub return_type: Box<Type>,
    pub is_constructor: bool,
}

/// Type parameter declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: &'static str,
    pub constraint: Option<Type>,
    pub default: Option<Type>,
}

/// Parameter declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDeclaration {
    pub name: &'static str,
    pub type_: Type,
    pub optional: bool,
    pub rest: bool,
}

/// Property signature in an interface/object type
#[derive(Debug, Clone, PartialEq)]
pub struct PropertySignature {
    pub name: &'static str,
    pub type_: Type,
    pub optional: bool,
    pub readonly: bool,
}

/// Method signature in an interface
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub name: &'static str,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterDeclaration>,
    pub return_type: Type,
    pub optional: bool,
}

/// Index signature ([key: K]: V)
#[derive(Debug, Clone, PartialEq)]
pub struct IndexSignature {
    pub key_name: &'static str,
    pub key_type: Type,
    pub value_type: Type,
    pub readonly: bool,
}

/// Call signature (for callable objects)
#[derive(Debug, Clone, PartialEq)]
pub struct CallSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterDeclaration>,
    pub return_type: Type,
}

/// Construct signature (for new-able objects)
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterDeclaration>,
    pub return_type: Type,
}

/// Interface declaration
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDeclaration {
    pub name: &'static str,
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Vec<Type>,
    pub properties: Vec<PropertySignature>,
    pub methods: Vec<MethodSignature>,
    pub call_signatures: Vec<CallSignature>,
    pub construct_signatures: Vec<ConstructSignature>,
    pub index_signatures: Vec<IndexSignature>,
}

/// Type alias declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDeclaration {
    pub name: &'static str,
    pub type_parameters: Vec<TypeParameter>,
    pub type_: Type,
}

/// Variable declaration (for global variables like `undefined`, `NaN`, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub name: &'static str,
    pub type_: Type,
    pub readonly: bool,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: &'static str,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterDeclaration>,
    pub return_type: Type,
}

/// Class declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    pub name: &'static str,
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Option<Type>,
    pub implements: Vec<Type>,
    pub constructor: Option<ConstructSignature>,
    pub properties: Vec<PropertySignature>,
    pub methods: Vec<MethodSignature>,
    pub static_properties: Vec<PropertySignature>,
    pub static_methods: Vec<MethodSignature>,
}

/// Enum member
#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: &'static str,
    pub value: Option<EnumValue>,
}

/// Enum value (string or number)
#[derive(Debug, Clone, PartialEq)]
pub enum EnumValue {
    Number(f64),
    String(&'static str),
}

/// Enum declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration {
    pub name: &'static str,
    pub members: Vec<EnumMember>,
    pub is_const: bool,
}

/// Namespace/Module declaration
#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDeclaration {
    pub name: &'static str,
    pub interfaces: Vec<InterfaceDeclaration>,
    pub type_aliases: Vec<TypeAliasDeclaration>,
    pub variables: Vec<VariableDeclaration>,
    pub functions: Vec<FunctionDeclaration>,
    pub classes: Vec<ClassDeclaration>,
    pub enums: Vec<EnumDeclaration>,
    pub namespaces: Vec<NamespaceDeclaration>,
}

/// A complete lib.d.ts file representation
#[derive(Debug, Clone, Default)]
pub struct LibDeclarations {
    pub interfaces: Vec<InterfaceDeclaration>,
    pub type_aliases: Vec<TypeAliasDeclaration>,
    pub variables: Vec<VariableDeclaration>,
    pub functions: Vec<FunctionDeclaration>,
    pub classes: Vec<ClassDeclaration>,
    pub enums: Vec<EnumDeclaration>,
    pub namespaces: Vec<NamespaceDeclaration>,
}

impl LibDeclarations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: LibDeclarations) {
        self.interfaces.extend(other.interfaces);
        self.type_aliases.extend(other.type_aliases);
        self.variables.extend(other.variables);
        self.functions.extend(other.functions);
        self.classes.extend(other.classes);
        self.enums.extend(other.enums);
        self.namespaces.extend(other.namespaces);
    }

    /// Find an interface by name
    pub fn find_interface(&self, name: &str) -> Option<&InterfaceDeclaration> {
        self.interfaces.iter().find(|i| i.name == name)
    }

    /// Find a type alias by name
    pub fn find_type_alias(&self, name: &str) -> Option<&TypeAliasDeclaration> {
        self.type_aliases.iter().find(|t| t.name == name)
    }

    /// Find a variable by name
    pub fn find_variable(&self, name: &str) -> Option<&VariableDeclaration> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// Find a function by name
    pub fn find_function(&self, name: &str) -> Option<&FunctionDeclaration> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Find a class by name
    pub fn find_class(&self, name: &str) -> Option<&ClassDeclaration> {
        self.classes.iter().find(|c| c.name == name)
    }
}

// Helper constructors for common type patterns

impl Type {
    /// Create a simple type reference without type arguments
    pub fn reference(name: &'static str) -> Self {
        Type::TypeReference {
            name,
            type_arguments: vec![],
        }
    }

    /// Create a type reference with one type argument
    pub fn reference1(name: &'static str, arg: Type) -> Self {
        Type::TypeReference {
            name,
            type_arguments: vec![arg],
        }
    }

    /// Create a type reference with two type arguments
    pub fn reference2(name: &'static str, arg1: Type, arg2: Type) -> Self {
        Type::TypeReference {
            name,
            type_arguments: vec![arg1, arg2],
        }
    }

    /// Create a union type
    pub fn union(types: Vec<Type>) -> Self {
        Type::Union(types)
    }

    /// Create an intersection type
    pub fn intersection(types: Vec<Type>) -> Self {
        Type::Intersection(types)
    }

    /// Create an array type
    pub fn array(element_type: Type) -> Self {
        Type::Array(Box::new(element_type))
    }

    /// Create a simple function type
    pub fn func(params: Vec<ParameterDeclaration>, return_type: Type) -> Self {
        Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: params,
            return_type: Box::new(return_type),
            is_constructor: false,
        })
    }

    /// Create a nullable type (T | null)
    pub fn nullable(inner: Type) -> Self {
        Type::Union(vec![inner, Type::Null])
    }

    /// Create an optional type (T | undefined)
    pub fn optional(inner: Type) -> Self {
        Type::Union(vec![inner, Type::Undefined])
    }

    /// Create a type parameter reference
    pub fn type_param(name: &'static str) -> Self {
        Type::TypeParameter {
            name,
            constraint: None,
            default: None,
        }
    }
}

impl ParameterDeclaration {
    pub fn new(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: false,
            rest: false,
        }
    }

    pub fn optional(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: true,
            rest: false,
        }
    }

    pub fn rest(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: false,
            rest: true,
        }
    }
}

impl PropertySignature {
    pub fn new(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: false,
            readonly: false,
        }
    }

    pub fn optional(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: true,
            readonly: false,
        }
    }

    pub fn readonly(name: &'static str, type_: Type) -> Self {
        Self {
            name,
            type_,
            optional: false,
            readonly: true,
        }
    }
}

impl MethodSignature {
    pub fn new(name: &'static str, params: Vec<ParameterDeclaration>, return_type: Type) -> Self {
        Self {
            name,
            type_parameters: vec![],
            parameters: params,
            return_type,
            optional: false,
        }
    }

    pub fn generic(
        name: &'static str,
        type_params: Vec<TypeParameter>,
        params: Vec<ParameterDeclaration>,
        return_type: Type,
    ) -> Self {
        Self {
            name,
            type_parameters: type_params,
            parameters: params,
            return_type,
            optional: false,
        }
    }
}

impl TypeParameter {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            constraint: None,
            default: None,
        }
    }

    pub fn with_constraint(name: &'static str, constraint: Type) -> Self {
        Self {
            name,
            constraint: Some(constraint),
            default: None,
        }
    }

    pub fn with_default(name: &'static str, default: Type) -> Self {
        Self {
            name,
            constraint: None,
            default: Some(default),
        }
    }
}

impl InterfaceDeclaration {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            type_parameters: vec![],
            extends: vec![],
            properties: vec![],
            methods: vec![],
            call_signatures: vec![],
            construct_signatures: vec![],
            index_signatures: vec![],
        }
    }
}
