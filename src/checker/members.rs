//! Class Member Type Checking
//!
//! This module handles type checking of class members including:
//! - Properties (instance and static)
//! - Methods (instance and static)
//! - Accessors (get/set)
//! - Constructor parameter properties

use super::modifiers::{ModifierFlags, AccessModifier};
use std::collections::HashMap;

/// Unique identifier for types
pub type TypeId = u32;

/// Unique identifier for members
pub type MemberId = u32;

/// Member kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberKind {
    Property,
    Method,
    GetAccessor,
    SetAccessor,
    Constructor,
    IndexSignature,
}

/// Represents a class member
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub id: MemberId,
    pub name: String,
    pub kind: MemberKind,
    pub modifiers: ModifierFlags,
    pub type_id: Option<TypeId>,
    /// For methods, the parameter types
    pub parameters: Vec<ParameterInfo>,
    /// For methods, the return type
    pub return_type: Option<TypeId>,
    /// Whether the member is optional
    pub optional: bool,
    /// For accessors, the corresponding accessor
    pub paired_accessor: Option<MemberId>,
    /// The declaring class
    pub declaring_class: TypeId,
}

impl ClassMember {
    pub fn new(
        id: MemberId,
        name: impl Into<String>,
        kind: MemberKind,
        modifiers: ModifierFlags,
    ) -> Self {
        ClassMember {
            id,
            name: name.into(),
            kind,
            modifiers,
            type_id: None,
            parameters: Vec::new(),
            return_type: None,
            optional: false,
            paired_accessor: None,
            declaring_class: 0,
        }
    }

    pub fn is_static(&self) -> bool {
        self.modifiers.is_static()
    }

    pub fn is_readonly(&self) -> bool {
        self.modifiers.is_readonly()
    }

    pub fn is_abstract(&self) -> bool {
        self.modifiers.is_abstract()
    }

    pub fn is_override(&self) -> bool {
        self.modifiers.is_override()
    }

    pub fn access_modifier(&self) -> AccessModifier {
        self.modifiers.access_modifier()
    }

    pub fn is_private(&self) -> bool {
        matches!(self.access_modifier(), AccessModifier::Private)
    }

    pub fn is_protected(&self) -> bool {
        matches!(self.access_modifier(), AccessModifier::Protected)
    }

    pub fn is_public(&self) -> bool {
        matches!(self.access_modifier(), AccessModifier::Public)
    }

    pub fn is_accessor(&self) -> bool {
        matches!(self.kind, MemberKind::GetAccessor | MemberKind::SetAccessor)
    }

    pub fn is_method(&self) -> bool {
        matches!(self.kind, MemberKind::Method)
    }

    pub fn is_property(&self) -> bool {
        matches!(self.kind, MemberKind::Property)
    }
}

/// Parameter information for methods and constructors
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub type_id: Option<TypeId>,
    pub optional: bool,
    pub rest: bool,
    /// For constructor parameter properties
    pub is_parameter_property: bool,
    pub modifiers: ModifierFlags,
}

impl ParameterInfo {
    pub fn new(name: impl Into<String>) -> Self {
        ParameterInfo {
            name: name.into(),
            type_id: None,
            optional: false,
            rest: false,
            is_parameter_property: false,
            modifiers: ModifierFlags::NONE,
        }
    }

    pub fn with_type(mut self, type_id: TypeId) -> Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn rest(mut self) -> Self {
        self.rest = true;
        self
    }

    pub fn as_property(mut self, modifiers: ModifierFlags) -> Self {
        self.is_parameter_property = true;
        self.modifiers = modifiers;
        self
    }
}

/// Table of class members, keyed by name
#[derive(Debug, Clone, Default)]
pub struct MemberTable {
    /// Instance members
    instance_members: HashMap<String, Vec<MemberId>>,
    /// Static members
    static_members: HashMap<String, Vec<MemberId>>,
    /// Constructor
    constructor: Option<MemberId>,
    /// All members by ID
    members: HashMap<MemberId, ClassMember>,
    /// Next member ID
    next_id: MemberId,
}

impl MemberTable {
    pub fn new() -> Self {
        MemberTable {
            instance_members: HashMap::new(),
            static_members: HashMap::new(),
            constructor: None,
            members: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a member to the table
    pub fn add_member(&mut self, mut member: ClassMember) -> MemberId {
        let id = self.next_id;
        self.next_id += 1;
        member.id = id;

        let name = member.name.clone();
        let is_static = member.is_static();

        if member.kind == MemberKind::Constructor {
            self.constructor = Some(id);
        } else if is_static {
            self.static_members.entry(name).or_default().push(id);
        } else {
            self.instance_members.entry(name).or_default().push(id);
        }

        self.members.insert(id, member);
        id
    }

    /// Get a member by ID
    pub fn get_member(&self, id: MemberId) -> Option<&ClassMember> {
        self.members.get(&id)
    }

    /// Get a member by ID mutably
    pub fn get_member_mut(&mut self, id: MemberId) -> Option<&mut ClassMember> {
        self.members.get_mut(&id)
    }

    /// Get instance members by name
    pub fn get_instance_members(&self, name: &str) -> Option<&Vec<MemberId>> {
        self.instance_members.get(name)
    }

    /// Get static members by name
    pub fn get_static_members(&self, name: &str) -> Option<&Vec<MemberId>> {
        self.static_members.get(name)
    }

    /// Get members by name (static or instance based on is_static)
    pub fn get_members_by_name(&self, name: &str, is_static: bool) -> Option<&Vec<MemberId>> {
        if is_static {
            self.static_members.get(name)
        } else {
            self.instance_members.get(name)
        }
    }

    /// Check if a member exists
    pub fn has_member(&self, name: &str, is_static: bool) -> bool {
        if is_static {
            self.static_members.contains_key(name)
        } else {
            self.instance_members.contains_key(name)
        }
    }

    /// Get the constructor
    pub fn get_constructor(&self) -> Option<&ClassMember> {
        self.constructor.and_then(|id| self.members.get(&id))
    }

    /// Iterate over all instance members
    pub fn instance_members_iter(&self) -> impl Iterator<Item = &ClassMember> {
        self.instance_members.values()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.members.get(id))
    }

    /// Iterate over all static members
    pub fn static_members_iter(&self) -> impl Iterator<Item = &ClassMember> {
        self.static_members.values()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.members.get(id))
    }

    /// Get all member names
    pub fn member_names(&self, is_static: bool) -> impl Iterator<Item = &String> {
        if is_static {
            self.static_members.keys()
        } else {
            self.instance_members.keys()
        }
    }
}

/// Diagnostic for member errors
#[derive(Debug, Clone)]
pub struct MemberError {
    pub message: String,
    pub code: u32,
    pub member_id: Option<MemberId>,
}

/// Diagnostic codes for member errors
pub mod error_codes {
    pub const DUPLICATE_MEMBER: u32 = 2300;
    pub const PROPERTY_NOT_ASSIGNABLE: u32 = 2322;
    pub const ACCESSOR_TYPE_MISMATCH: u32 = 2380;
    pub const GET_ACCESSOR_CANNOT_HAVE_PARAMETERS: u32 = 1054;
    pub const SET_ACCESSOR_MUST_HAVE_ONE_PARAMETER: u32 = 1049;
    pub const SET_ACCESSOR_CANNOT_HAVE_RETURN_TYPE: u32 = 1095;
    pub const SET_ACCESSOR_CANNOT_HAVE_OPTIONAL: u32 = 1051;
    pub const READONLY_CANNOT_HAVE_SETTER: u32 = 2540;
    pub const ABSTRACT_METHOD_CANNOT_HAVE_IMPLEMENTATION: u32 = 1245;
    pub const NON_ABSTRACT_METHOD_MUST_HAVE_IMPLEMENTATION: u32 = 1246;
    pub const PRIVATE_IDENTIFIER_IN_INTERFACE: u32 = 18010;
}

/// Check accessor pair consistency
pub fn check_accessor_pair(
    getter: Option<&ClassMember>,
    setter: Option<&ClassMember>,
) -> Vec<MemberError> {
    let mut errors = Vec::new();

    match (getter, setter) {
        (Some(get), Some(set)) => {
            // Check accessibility matches
            if get.access_modifier() != set.access_modifier() {
                errors.push(MemberError {
                    message: "'get' and 'set' accessors must have the same accessibility".to_string(),
                    code: error_codes::ACCESSOR_TYPE_MISMATCH,
                    member_id: Some(set.id),
                });
            }

            // Check static matches
            if get.is_static() != set.is_static() {
                errors.push(MemberError {
                    message: "'get' and 'set' accessors must both be static or instance".to_string(),
                    code: error_codes::ACCESSOR_TYPE_MISMATCH,
                    member_id: Some(set.id),
                });
            }

            // Setter should have exactly one parameter
            if set.parameters.len() != 1 {
                errors.push(MemberError {
                    message: "A 'set' accessor must have exactly one parameter".to_string(),
                    code: error_codes::SET_ACCESSOR_MUST_HAVE_ONE_PARAMETER,
                    member_id: Some(set.id),
                });
            }

            // Setter parameter should not be optional
            if set.parameters.first().map_or(false, |p| p.optional) {
                errors.push(MemberError {
                    message: "A 'set' accessor parameter cannot be optional".to_string(),
                    code: error_codes::SET_ACCESSOR_CANNOT_HAVE_OPTIONAL,
                    member_id: Some(set.id),
                });
            }
        }
        (Some(get), None) => {
            // Getter parameters should be empty
            if !get.parameters.is_empty() {
                errors.push(MemberError {
                    message: "A 'get' accessor cannot have parameters".to_string(),
                    code: error_codes::GET_ACCESSOR_CANNOT_HAVE_PARAMETERS,
                    member_id: Some(get.id),
                });
            }

            // Readonly getter is fine without setter
        }
        (None, Some(set)) => {
            // Setter without getter - check parameter
            if set.parameters.len() != 1 {
                errors.push(MemberError {
                    message: "A 'set' accessor must have exactly one parameter".to_string(),
                    code: error_codes::SET_ACCESSOR_MUST_HAVE_ONE_PARAMETER,
                    member_id: Some(set.id),
                });
            }

            // Setter cannot have explicit return type
            if set.return_type.is_some() {
                errors.push(MemberError {
                    message: "A 'set' accessor cannot have a return type annotation".to_string(),
                    code: error_codes::SET_ACCESSOR_CANNOT_HAVE_RETURN_TYPE,
                    member_id: Some(set.id),
                });
            }
        }
        (None, None) => {}
    }

    errors
}

/// Check for duplicate members
pub fn check_duplicate_members(table: &MemberTable) -> Vec<MemberError> {
    let mut errors = Vec::new();

    // Check instance members
    for (name, ids) in &table.instance_members {
        check_member_duplicates(name, ids, table, &mut errors);
    }

    // Check static members
    for (name, ids) in &table.static_members {
        check_member_duplicates(name, ids, table, &mut errors);
    }

    errors
}

fn check_member_duplicates(
    name: &str,
    ids: &[MemberId],
    table: &MemberTable,
    errors: &mut Vec<MemberError>,
) {
    if ids.len() <= 1 {
        return;
    }

    let members: Vec<_> = ids.iter()
        .filter_map(|id| table.get_member(*id))
        .collect();

    // Separate by kind
    let mut properties = Vec::new();
    let mut methods = Vec::new();
    let mut getters = Vec::new();
    let mut setters = Vec::new();

    for member in &members {
        match member.kind {
            MemberKind::Property => properties.push(member),
            MemberKind::Method => methods.push(member),
            MemberKind::GetAccessor => getters.push(member),
            MemberKind::SetAccessor => setters.push(member),
            _ => {}
        }
    }

    // Properties cannot be duplicated
    if properties.len() > 1 {
        for prop in &properties[1..] {
            errors.push(MemberError {
                message: format!("Duplicate property '{}'", name),
                code: error_codes::DUPLICATE_MEMBER,
                member_id: Some(prop.id),
            });
        }
    }

    // Property cannot coexist with method/accessor
    if !properties.is_empty() && (!methods.is_empty() || !getters.is_empty() || !setters.is_empty()) {
        errors.push(MemberError {
            message: format!("Duplicate identifier '{}'", name),
            code: error_codes::DUPLICATE_MEMBER,
            member_id: properties.first().map(|m| m.id),
        });
    }

    // Multiple getters are not allowed
    if getters.len() > 1 {
        for getter in &getters[1..] {
            errors.push(MemberError {
                message: format!("Duplicate getter '{}'", name),
                code: error_codes::DUPLICATE_MEMBER,
                member_id: Some(getter.id),
            });
        }
    }

    // Multiple setters are not allowed
    if setters.len() > 1 {
        for setter in &setters[1..] {
            errors.push(MemberError {
                message: format!("Duplicate setter '{}'", name),
                code: error_codes::DUPLICATE_MEMBER,
                member_id: Some(setter.id),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_member_creation() {
        let member = ClassMember::new(1, "foo", MemberKind::Property, ModifierFlags::PUBLIC);
        assert_eq!(member.name, "foo");
        assert!(member.is_public());
        assert!(!member.is_static());
    }

    #[test]
    fn test_static_member() {
        let member = ClassMember::new(1, "bar", MemberKind::Method, ModifierFlags::PUBLIC | ModifierFlags::STATIC);
        assert!(member.is_static());
        assert!(member.is_method());
    }

    #[test]
    fn test_member_table() {
        let mut table = MemberTable::new();

        let prop = ClassMember::new(0, "name", MemberKind::Property, ModifierFlags::PUBLIC);
        let prop_id = table.add_member(prop);

        let method = ClassMember::new(0, "getName", MemberKind::Method, ModifierFlags::PUBLIC);
        let method_id = table.add_member(method);

        assert!(table.has_member("name", false));
        assert!(table.has_member("getName", false));
        assert!(!table.has_member("name", true)); // Not static

        assert!(table.get_member(prop_id).is_some());
        assert!(table.get_member(method_id).is_some());
    }

    #[test]
    fn test_static_vs_instance_members() {
        let mut table = MemberTable::new();

        // Instance method
        let instance = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::PUBLIC);
        table.add_member(instance);

        // Static method with same name
        let static_method = ClassMember::new(0, "foo", MemberKind::Method, ModifierFlags::PUBLIC | ModifierFlags::STATIC);
        table.add_member(static_method);

        assert!(table.has_member("foo", false));
        assert!(table.has_member("foo", true));
    }

    #[test]
    fn test_accessor_pair_valid() {
        let getter = ClassMember::new(1, "value", MemberKind::GetAccessor, ModifierFlags::PUBLIC);
        let mut setter = ClassMember::new(2, "value", MemberKind::SetAccessor, ModifierFlags::PUBLIC);
        setter.parameters.push(ParameterInfo::new("val"));

        let errors = check_accessor_pair(Some(&getter), Some(&setter));
        assert!(errors.is_empty());
    }

    #[test]
    fn test_accessor_pair_mismatched_accessibility() {
        let getter = ClassMember::new(1, "value", MemberKind::GetAccessor, ModifierFlags::PUBLIC);
        let mut setter = ClassMember::new(2, "value", MemberKind::SetAccessor, ModifierFlags::PRIVATE);
        setter.parameters.push(ParameterInfo::new("val"));

        let errors = check_accessor_pair(Some(&getter), Some(&setter));
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == error_codes::ACCESSOR_TYPE_MISMATCH));
    }

    #[test]
    fn test_getter_with_parameters_error() {
        let mut getter = ClassMember::new(1, "value", MemberKind::GetAccessor, ModifierFlags::PUBLIC);
        getter.parameters.push(ParameterInfo::new("x"));

        let errors = check_accessor_pair(Some(&getter), None);
        assert!(errors.iter().any(|e| e.code == error_codes::GET_ACCESSOR_CANNOT_HAVE_PARAMETERS));
    }

    #[test]
    fn test_setter_without_parameter_error() {
        let setter = ClassMember::new(1, "value", MemberKind::SetAccessor, ModifierFlags::PUBLIC);

        let errors = check_accessor_pair(None, Some(&setter));
        assert!(errors.iter().any(|e| e.code == error_codes::SET_ACCESSOR_MUST_HAVE_ONE_PARAMETER));
    }

    #[test]
    fn test_duplicate_property_detection() {
        let mut table = MemberTable::new();

        table.add_member(ClassMember::new(0, "x", MemberKind::Property, ModifierFlags::PUBLIC));
        table.add_member(ClassMember::new(0, "x", MemberKind::Property, ModifierFlags::PUBLIC));

        let errors = check_duplicate_members(&table);
        assert!(errors.iter().any(|e| e.code == error_codes::DUPLICATE_MEMBER));
    }

    #[test]
    fn test_constructor_parameter_property() {
        let param = ParameterInfo::new("name")
            .with_type(1)
            .as_property(ModifierFlags::PUBLIC | ModifierFlags::READONLY);

        assert!(param.is_parameter_property);
        assert!(param.modifiers.is_readonly());
    }
}
