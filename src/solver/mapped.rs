//! Mapped type resolution.
//!
//! This module implements TypeScript's mapped types:
//! `{ [P in K]: T }`
//!
//! Features:
//! - Property iteration over key types
//! - Optional and readonly modifiers
//! - Key remapping with `as` clause

use std::collections::HashMap;

use super::{
    generics::{GenericInstantiator, TypeSubstitution},
    IndexKeyType, IndexSignature, ObjectType, PrimitiveKind, Property, Type,
};

/// Mapped type representation.
#[derive(Debug, Clone, PartialEq)]
pub struct MappedType {
    /// The type parameter name (P in `[P in K]`)
    pub type_parameter: String,
    /// The constraint/source of keys (K in `[P in K]`)
    pub constraint: Type,
    /// The template type for values (T in `: T`)
    pub template_type: Type,
    /// Optional modifier (+?, -?, or none)
    pub optional_modifier: OptionalModifier,
    /// Readonly modifier (+readonly, -readonly, or none)
    pub readonly_modifier: ReadonlyModifier,
    /// Key remapping type (the `as N` clause)
    pub name_type: Option<Type>,
}

/// Optional modifier for mapped types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptionalModifier {
    /// No change to optionality
    #[default]
    None,
    /// Add optional (?)
    Add,
    /// Remove optional (-?)
    Remove,
}

/// Readonly modifier for mapped types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadonlyModifier {
    /// No change to readonly
    #[default]
    None,
    /// Add readonly
    Add,
    /// Remove readonly (-readonly)
    Remove,
}

impl MappedType {
    /// Create a new mapped type.
    pub fn new(type_parameter: &str, constraint: Type, template_type: Type) -> Self {
        Self {
            type_parameter: type_parameter.to_string(),
            constraint,
            template_type,
            optional_modifier: OptionalModifier::None,
            readonly_modifier: ReadonlyModifier::None,
            name_type: None,
        }
    }

    /// Add optional modifier.
    pub fn with_optional(mut self, modifier: OptionalModifier) -> Self {
        self.optional_modifier = modifier;
        self
    }

    /// Add readonly modifier.
    pub fn with_readonly(mut self, modifier: ReadonlyModifier) -> Self {
        self.readonly_modifier = modifier;
        self
    }

    /// Add key remapping.
    pub fn with_name_type(mut self, name_type: Type) -> Self {
        self.name_type = Some(name_type);
        self
    }
}

/// Mapped type resolver.
pub struct MappedTypeResolver<F>
where
    F: Fn(&Type) -> Vec<String>,
{
    /// Function to get the keys of a type.
    get_keys: F,
}

impl<F> MappedTypeResolver<F>
where
    F: Fn(&Type) -> Vec<String>,
{
    /// Create a new resolver.
    pub fn new(get_keys: F) -> Self {
        Self { get_keys }
    }

    /// Resolve a mapped type to an object type.
    pub fn resolve(&self, mapped: &MappedType, source_type: Option<&Type>) -> Type {
        // Get the keys to iterate over
        let keys = self.get_constraint_keys(&mapped.constraint, source_type);

        if keys.is_empty() {
            // No keys - return empty object or handle special cases
            return self.handle_empty_mapped(mapped);
        }

        // Build the resulting object type
        let mut properties = HashMap::new();
        let mut index_signatures = Vec::new();

        for key in keys {
            // Check for index signature case
            if key == "__index_string__" {
                let value_type = self.resolve_template(&mapped.template_type, &key, source_type);
                index_signatures.push(IndexSignature {
                    key_type: IndexKeyType::String,
                    value_type,
                    readonly: self.should_be_readonly(mapped, false),
                });
                continue;
            }
            if key == "__index_number__" {
                let value_type = self.resolve_template(&mapped.template_type, &key, source_type);
                index_signatures.push(IndexSignature {
                    key_type: IndexKeyType::Number,
                    value_type,
                    readonly: self.should_be_readonly(mapped, false),
                });
                continue;
            }

            // Apply key remapping if present
            let final_key = if let Some(ref name_type) = mapped.name_type {
                self.remap_key(&key, name_type)
            } else {
                Some(key.clone())
            };

            if let Some(final_key) = final_key {
                let value_type = self.resolve_template(&mapped.template_type, &key, source_type);

                // Determine optionality
                let is_optional = self.determine_optionality(mapped, &key, source_type);

                // Determine readonly
                let is_readonly = self.determine_readonly(mapped, &key, source_type);

                properties.insert(
                    final_key.clone(),
                    Property {
                        name: final_key,
                        ty: value_type,
                        optional: is_optional,
                        readonly: is_readonly,
                    },
                );
            }
        }

        Type::Object(ObjectType {
            properties,
            index_signatures,
            ..Default::default()
        })
    }

    /// Get keys from the constraint type.
    fn get_constraint_keys(&self, constraint: &Type, source_type: Option<&Type>) -> Vec<String> {
        match constraint {
            // Union of literal strings
            Type::Union(members) => members
                .iter()
                .filter_map(|m| {
                    if let Type::Literal(super::LiteralType::String(s)) = m {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect(),

            // Single literal string
            Type::Literal(super::LiteralType::String(s)) => vec![s.clone()],

            // keyof T
            Type::Keyof(inner) => (self.get_keys)(inner),

            // Type parameter that should be resolved
            Type::TypeParameter(_) => {
                if let Some(source) = source_type {
                    (self.get_keys)(source)
                } else {
                    vec![]
                }
            }

            // string (index signature)
            Type::Primitive(PrimitiveKind::String) => vec!["__index_string__".to_string()],

            // number (index signature)
            Type::Primitive(PrimitiveKind::Number) => vec!["__index_number__".to_string()],

            _ => vec![],
        }
    }

    /// Resolve the template type for a specific key.
    fn resolve_template(&self, template: &Type, key: &str, source_type: Option<&Type>) -> Type {
        // Create a substitution with P = key literal
        let mut subst = TypeSubstitution::new();
        subst.insert(
            "P".to_string(),
            Type::Literal(super::LiteralType::String(key.to_string())),
        );

        // If we have a source type and the template references it, also handle T[P]
        if let Some(source) = source_type {
            if let Type::IndexedAccess(_, _) = template {
                // Resolve the indexed access
                return self.resolve_indexed_access(source, key);
            }
        }

        let instantiator = GenericInstantiator::new(subst);
        instantiator.instantiate(template)
    }

    /// Resolve an indexed access T[P] where P is a literal key.
    fn resolve_indexed_access(&self, object_type: &Type, key: &str) -> Type {
        if let Type::Object(obj) = object_type {
            if let Some(prop) = obj.properties.get(key) {
                return prop.ty.clone();
            }
            // Check index signatures
            for sig in &obj.index_signatures {
                if sig.key_type == IndexKeyType::String {
                    return sig.value_type.clone();
                }
            }
        }
        Type::Unknown
    }

    /// Remap a key using the name type.
    fn remap_key(&self, key: &str, name_type: &Type) -> Option<String> {
        // Substitute the key into the name type
        let mut subst = TypeSubstitution::new();
        subst.insert(
            "P".to_string(),
            Type::Literal(super::LiteralType::String(key.to_string())),
        );

        let instantiator = GenericInstantiator::new(subst);
        let remapped = instantiator.instantiate(name_type);

        match remapped {
            Type::Literal(super::LiteralType::String(s)) => Some(s),
            Type::Never => None, // Filter out this key
            _ => Some(key.to_string()), // Keep original if can't resolve
        }
    }

    /// Determine if a property should be optional.
    fn determine_optionality(
        &self,
        mapped: &MappedType,
        key: &str,
        source_type: Option<&Type>,
    ) -> bool {
        let source_optional = if let Some(Type::Object(obj)) = source_type {
            obj.properties.get(key).map(|p| p.optional).unwrap_or(false)
        } else {
            false
        };

        match mapped.optional_modifier {
            OptionalModifier::None => source_optional,
            OptionalModifier::Add => true,
            OptionalModifier::Remove => false,
        }
    }

    /// Check if should be readonly based on modifier.
    fn should_be_readonly(&self, mapped: &MappedType, source_readonly: bool) -> bool {
        match mapped.readonly_modifier {
            ReadonlyModifier::None => source_readonly,
            ReadonlyModifier::Add => true,
            ReadonlyModifier::Remove => false,
        }
    }

    /// Determine if a property should be readonly.
    fn determine_readonly(
        &self,
        mapped: &MappedType,
        key: &str,
        source_type: Option<&Type>,
    ) -> bool {
        let source_readonly = if let Some(Type::Object(obj)) = source_type {
            obj.properties.get(key).map(|p| p.readonly).unwrap_or(false)
        } else {
            false
        };

        self.should_be_readonly(mapped, source_readonly)
    }

    /// Handle mapped type with no keys.
    fn handle_empty_mapped(&self, _mapped: &MappedType) -> Type {
        // Return empty object
        Type::Object(ObjectType::default())
    }
}

/// Get keys from an object type.
pub fn get_object_keys(ty: &Type) -> Vec<String> {
    match ty {
        Type::Object(obj) => obj.properties.keys().cloned().collect(),
        Type::Union(members) => {
            // Intersection of keys (only keys present in all members)
            let mut result: Option<Vec<String>> = None;
            for member in members {
                let keys = get_object_keys(member);
                result = Some(match result {
                    Some(existing) => existing
                        .into_iter()
                        .filter(|k| keys.contains(k))
                        .collect(),
                    None => keys,
                });
            }
            result.unwrap_or_default()
        }
        Type::Intersection(members) => {
            // Union of keys (keys present in any member)
            let mut result = Vec::new();
            for member in members {
                for key in get_object_keys(member) {
                    if !result.contains(&key) {
                        result.push(key);
                    }
                }
            }
            result
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::TypeParameter;

    fn create_object_with_props(props: Vec<(&str, Type, bool, bool)>) -> Type {
        let mut properties = HashMap::new();
        for (name, ty, optional, readonly) in props {
            properties.insert(
                name.to_string(),
                Property {
                    name: name.to_string(),
                    ty,
                    optional,
                    readonly,
                },
            );
        }
        Type::Object(ObjectType {
            properties,
            ..Default::default()
        })
    }

    #[test]
    fn test_simple_mapped_type() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { [P in 'a' | 'b']: number }
        let mapped = MappedType::new(
            "P",
            Type::Union(vec![
                Type::Literal(super::super::LiteralType::String("a".to_string())),
                Type::Literal(super::super::LiteralType::String("b".to_string())),
            ]),
            Type::number(),
        );

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.properties.contains_key("a"));
            assert!(obj.properties.contains_key("b"));
            assert!(matches!(
                obj.properties.get("a").unwrap().ty,
                Type::Primitive(PrimitiveKind::Number)
            ));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_mapped_type_with_optional() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { [P in 'a' | 'b']?: number }
        let mapped = MappedType::new(
            "P",
            Type::Union(vec![
                Type::Literal(super::super::LiteralType::String("a".to_string())),
                Type::Literal(super::super::LiteralType::String("b".to_string())),
            ]),
            Type::number(),
        )
        .with_optional(OptionalModifier::Add);

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            assert!(obj.properties.get("a").unwrap().optional);
            assert!(obj.properties.get("b").unwrap().optional);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_mapped_type_with_readonly() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { readonly [P in 'a' | 'b']: number }
        let mapped = MappedType::new(
            "P",
            Type::Union(vec![
                Type::Literal(super::super::LiteralType::String("a".to_string())),
                Type::Literal(super::super::LiteralType::String("b".to_string())),
            ]),
            Type::number(),
        )
        .with_readonly(ReadonlyModifier::Add);

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            assert!(obj.properties.get("a").unwrap().readonly);
            assert!(obj.properties.get("b").unwrap().readonly);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_mapped_type_remove_optional() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // Source: { a?: number; b?: string }
        let source = create_object_with_props(vec![
            ("a", Type::number(), true, false),
            ("b", Type::string(), true, false),
        ]);

        // { [P in keyof T]-?: T[P] }
        let mapped = MappedType::new(
            "P",
            Type::Keyof(Box::new(source.clone())),
            Type::IndexedAccess(
                Box::new(Type::TypeParameter(TypeParameter {
                    name: "T".to_string(),
                    constraint: None,
                    default: None,
                })),
                Box::new(Type::TypeParameter(TypeParameter {
                    name: "P".to_string(),
                    constraint: None,
                    default: None,
                })),
            ),
        )
        .with_optional(OptionalModifier::Remove);

        let result = resolver.resolve(&mapped, Some(&source));

        if let Type::Object(obj) = result {
            assert!(!obj.properties.get("a").unwrap().optional);
            assert!(!obj.properties.get("b").unwrap().optional);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_keyof_constraint() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // Source: { x: number; y: string }
        let source = create_object_with_props(vec![
            ("x", Type::number(), false, false),
            ("y", Type::string(), false, false),
        ]);

        // { [P in keyof Source]: boolean }
        let mapped = MappedType::new(
            "P",
            Type::Keyof(Box::new(source.clone())),
            Type::boolean(),
        );

        let result = resolver.resolve(&mapped, Some(&source));

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.properties.contains_key("x"));
            assert!(obj.properties.contains_key("y"));
            assert!(matches!(
                obj.properties.get("x").unwrap().ty,
                Type::Primitive(PrimitiveKind::Boolean)
            ));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_key_remapping() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { [P in 'a' | 'b' as `get${Capitalize<P>}`]: number }
        // Simplified: { [P in 'a' | 'b' as P]: number } (identity remapping)
        let mapped = MappedType::new(
            "P",
            Type::Union(vec![
                Type::Literal(super::super::LiteralType::String("a".to_string())),
                Type::Literal(super::super::LiteralType::String("b".to_string())),
            ]),
            Type::number(),
        )
        .with_name_type(Type::TypeParameter(TypeParameter {
            name: "P".to_string(),
            constraint: None,
            default: None,
        }));

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.properties.contains_key("a"));
            assert!(obj.properties.contains_key("b"));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_key_filtering() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { [P in 'a' | 'b' as P extends 'a' ? P : never]: number }
        // Should only include 'a'
        let mapped = MappedType::new(
            "P",
            Type::Union(vec![
                Type::Literal(super::super::LiteralType::String("a".to_string())),
                Type::Literal(super::super::LiteralType::String("b".to_string())),
            ]),
            Type::number(),
        )
        .with_name_type(Type::Literal(super::super::LiteralType::String(
            "a".to_string(),
        )));

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            // Both keys map to "a", so we only get one property
            assert!(obj.properties.contains_key("a"));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_get_object_keys() {
        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
        ]);

        let keys = get_object_keys(&obj);
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn test_index_signature_mapped() {
        let resolver = MappedTypeResolver::new(get_object_keys);

        // { [P in string]: number }
        let mapped = MappedType::new(
            "P",
            Type::Primitive(PrimitiveKind::String),
            Type::number(),
        );

        let result = resolver.resolve(&mapped, None);

        if let Type::Object(obj) = result {
            assert!(obj.properties.is_empty());
            assert_eq!(obj.index_signatures.len(), 1);
            assert_eq!(obj.index_signatures[0].key_type, IndexKeyType::String);
        } else {
            panic!("Expected object type");
        }
    }
}
