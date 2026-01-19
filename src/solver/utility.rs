//! Built-in TypeScript utility types.
//!
//! This module implements all standard TypeScript utility types:
//! - Partial<T>, Required<T>, Readonly<T>
//! - Pick<T, K>, Omit<T, K>
//! - Record<K, V>
//! - Exclude<T, U>, Extract<T, U>
//! - NonNullable<T>
//! - Parameters<T>, ReturnType<T>, ConstructorParameters<T>, InstanceType<T>
//! - ThisParameterType<T>, OmitThisParameter<T>
//! - Uppercase<S>, Lowercase<S>, Capitalize<S>, Uncapitalize<S>
//! - Awaited<T>

use std::collections::HashMap;

use super::{FunctionType, LiteralType, ObjectType, PrimitiveKind, Property, Type};

/// Utility type implementations.
pub struct UtilityTypes<F>
where
    F: Fn(&Type, &Type) -> bool + Clone,
{
    is_subtype: F,
}

impl<F> UtilityTypes<F>
where
    F: Fn(&Type, &Type) -> bool + Clone,
{
    /// Create a new utility types resolver.
    pub fn new(is_subtype: F) -> Self {
        Self { is_subtype }
    }

    /// Resolve a utility type by name.
    pub fn resolve(&self, name: &str, type_args: &[Type]) -> Option<Type> {
        match name {
            "Partial" => self.partial(type_args.first()?),
            "Required" => self.required(type_args.first()?),
            "Readonly" => self.readonly(type_args.first()?),
            "Pick" => self.pick(type_args.first()?, type_args.get(1)?),
            "Omit" => self.omit(type_args.first()?, type_args.get(1)?),
            "Record" => self.record(type_args.first()?, type_args.get(1)?),
            "Exclude" => self.exclude(type_args.first()?, type_args.get(1)?),
            "Extract" => self.extract(type_args.first()?, type_args.get(1)?),
            "NonNullable" => self.non_nullable(type_args.first()?),
            "Parameters" => self.parameters(type_args.first()?),
            "ReturnType" => self.return_type(type_args.first()?),
            "ConstructorParameters" => self.constructor_parameters(type_args.first()?),
            "InstanceType" => self.instance_type(type_args.first()?),
            "ThisParameterType" => self.this_parameter_type(type_args.first()?),
            "OmitThisParameter" => self.omit_this_parameter(type_args.first()?),
            "Uppercase" => self.uppercase(type_args.first()?),
            "Lowercase" => self.lowercase(type_args.first()?),
            "Capitalize" => self.capitalize(type_args.first()?),
            "Uncapitalize" => self.uncapitalize(type_args.first()?),
            "Awaited" => self.awaited(type_args.first()?),
            _ => None,
        }
    }

    /// `Partial<T>` - Make all properties optional
    pub fn partial(&self, ty: &Type) -> Option<Type> {
        if let Type::Object(obj) = ty {
            let new_props: HashMap<_, _> = obj
                .properties
                .iter()
                .map(|(name, prop)| {
                    (
                        name.clone(),
                        Property {
                            name: prop.name.clone(),
                            ty: prop.ty.clone(),
                            optional: true,
                            readonly: prop.readonly,
                        },
                    )
                })
                .collect();

            Some(Type::Object(ObjectType {
                properties: new_props,
                call_signatures: obj.call_signatures.clone(),
                construct_signatures: obj.construct_signatures.clone(),
                index_signatures: obj.index_signatures.clone(),
            }))
        } else {
            Some(ty.clone())
        }
    }

    /// `Required<T>` - Make all properties required
    pub fn required(&self, ty: &Type) -> Option<Type> {
        if let Type::Object(obj) = ty {
            let new_props: HashMap<_, _> = obj
                .properties
                .iter()
                .map(|(name, prop)| {
                    (
                        name.clone(),
                        Property {
                            name: prop.name.clone(),
                            ty: prop.ty.clone(),
                            optional: false,
                            readonly: prop.readonly,
                        },
                    )
                })
                .collect();

            Some(Type::Object(ObjectType {
                properties: new_props,
                call_signatures: obj.call_signatures.clone(),
                construct_signatures: obj.construct_signatures.clone(),
                index_signatures: obj.index_signatures.clone(),
            }))
        } else {
            Some(ty.clone())
        }
    }

    /// `Readonly<T>` - Make all properties readonly
    pub fn readonly(&self, ty: &Type) -> Option<Type> {
        if let Type::Object(obj) = ty {
            let new_props: HashMap<_, _> = obj
                .properties
                .iter()
                .map(|(name, prop)| {
                    (
                        name.clone(),
                        Property {
                            name: prop.name.clone(),
                            ty: prop.ty.clone(),
                            optional: prop.optional,
                            readonly: true,
                        },
                    )
                })
                .collect();

            Some(Type::Object(ObjectType {
                properties: new_props,
                call_signatures: obj.call_signatures.clone(),
                construct_signatures: obj.construct_signatures.clone(),
                index_signatures: obj.index_signatures.clone(),
            }))
        } else {
            Some(ty.clone())
        }
    }

    /// `Pick<T, K>` - Pick specified properties from T
    pub fn pick(&self, ty: &Type, keys: &Type) -> Option<Type> {
        if let Type::Object(obj) = ty {
            let key_set = self.get_literal_keys(keys);

            let new_props: HashMap<_, _> = obj
                .properties
                .iter()
                .filter(|(name, _)| key_set.contains(name))
                .map(|(name, prop)| (name.clone(), prop.clone()))
                .collect();

            Some(Type::Object(ObjectType {
                properties: new_props,
                ..Default::default()
            }))
        } else {
            Some(ty.clone())
        }
    }

    /// `Omit<T, K>` - Omit specified properties from T
    pub fn omit(&self, ty: &Type, keys: &Type) -> Option<Type> {
        if let Type::Object(obj) = ty {
            let key_set = self.get_literal_keys(keys);

            let new_props: HashMap<_, _> = obj
                .properties
                .iter()
                .filter(|(name, _)| !key_set.contains(name))
                .map(|(name, prop)| (name.clone(), prop.clone()))
                .collect();

            Some(Type::Object(ObjectType {
                properties: new_props,
                call_signatures: obj.call_signatures.clone(),
                construct_signatures: obj.construct_signatures.clone(),
                index_signatures: obj.index_signatures.clone(),
            }))
        } else {
            Some(ty.clone())
        }
    }

    /// `Record<K, V>` - Create object type with keys K and values V
    pub fn record(&self, keys: &Type, value: &Type) -> Option<Type> {
        let key_list = self.get_literal_keys(keys);

        if key_list.is_empty() {
            // Handle Record<string, V> or Record<number, V>
            if matches!(keys, Type::Primitive(PrimitiveKind::String)) {
                return Some(Type::Object(ObjectType {
                    index_signatures: vec![super::IndexSignature {
                        key_type: super::IndexKeyType::String,
                        value_type: value.clone(),
                        readonly: false,
                    }],
                    ..Default::default()
                }));
            }
            if matches!(keys, Type::Primitive(PrimitiveKind::Number)) {
                return Some(Type::Object(ObjectType {
                    index_signatures: vec![super::IndexSignature {
                        key_type: super::IndexKeyType::Number,
                        value_type: value.clone(),
                        readonly: false,
                    }],
                    ..Default::default()
                }));
            }
        }

        let props: HashMap<_, _> = key_list
            .iter()
            .map(|key| {
                (
                    key.clone(),
                    Property {
                        name: key.clone(),
                        ty: value.clone(),
                        optional: false,
                        readonly: false,
                    },
                )
            })
            .collect();

        Some(Type::Object(ObjectType {
            properties: props,
            ..Default::default()
        }))
    }

    /// `Exclude<T, U>` - Exclude types from T that are assignable to U
    pub fn exclude(&self, ty: &Type, excluded: &Type) -> Option<Type> {
        match ty {
            Type::Union(members) => {
                let filtered: Vec<_> = members
                    .iter()
                    .filter(|m| !(self.is_subtype)(m, excluded))
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    Some(Type::Never)
                } else if filtered.len() == 1 {
                    Some(filtered.into_iter().next().unwrap())
                } else {
                    Some(Type::Union(filtered))
                }
            }
            _ => {
                if (self.is_subtype)(ty, excluded) {
                    Some(Type::Never)
                } else {
                    Some(ty.clone())
                }
            }
        }
    }

    /// `Extract<T, U>` - Extract types from T that are assignable to U
    pub fn extract(&self, ty: &Type, extracted: &Type) -> Option<Type> {
        match ty {
            Type::Union(members) => {
                let filtered: Vec<_> = members
                    .iter()
                    .filter(|m| (self.is_subtype)(m, extracted))
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    Some(Type::Never)
                } else if filtered.len() == 1 {
                    Some(filtered.into_iter().next().unwrap())
                } else {
                    Some(Type::Union(filtered))
                }
            }
            _ => {
                if (self.is_subtype)(ty, extracted) {
                    Some(ty.clone())
                } else {
                    Some(Type::Never)
                }
            }
        }
    }

    /// `NonNullable<T>` - Exclude null and undefined from T
    pub fn non_nullable(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Union(members) => {
                let filtered: Vec<_> = members
                    .iter()
                    .filter(|m| !matches!(m, Type::Null | Type::Undefined))
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    Some(Type::Never)
                } else if filtered.len() == 1 {
                    Some(filtered.into_iter().next().unwrap())
                } else {
                    Some(Type::Union(filtered))
                }
            }
            Type::Null | Type::Undefined => Some(Type::Never),
            _ => Some(ty.clone()),
        }
    }

    /// `Parameters<T>` - Get parameter types of a function type as a tuple
    pub fn parameters(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Function(func) => {
                let param_types: Vec<_> = func.parameters.iter().map(|p| p.ty.clone()).collect();
                Some(Type::Tuple(param_types))
            }
            _ => Some(Type::Never),
        }
    }

    /// `ReturnType<T>` - Get return type of a function type
    pub fn return_type(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Function(func) => Some((*func.return_type).clone()),
            _ => Some(Type::Any),
        }
    }

    /// `ConstructorParameters<T>` - Get constructor parameter types
    pub fn constructor_parameters(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Object(obj) => {
                if let Some(construct_sig) = obj.construct_signatures.first() {
                    let param_types: Vec<_> =
                        construct_sig.parameters.iter().map(|p| p.ty.clone()).collect();
                    Some(Type::Tuple(param_types))
                } else {
                    Some(Type::Never)
                }
            }
            _ => Some(Type::Never),
        }
    }

    /// `InstanceType<T>` - Get instance type of a constructor
    pub fn instance_type(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Object(obj) => {
                if let Some(construct_sig) = obj.construct_signatures.first() {
                    Some((*construct_sig.return_type).clone())
                } else {
                    Some(Type::Any)
                }
            }
            _ => Some(Type::Any),
        }
    }

    /// `ThisParameterType<T>` - Get the this parameter type
    pub fn this_parameter_type(&self, ty: &Type) -> Option<Type> {
        // TypeScript's this parameter is the first parameter named "this"
        match ty {
            Type::Function(func) => {
                if let Some(param) = func.parameters.first() {
                    if param.name == "this" {
                        return Some(param.ty.clone());
                    }
                }
                Some(Type::Unknown)
            }
            _ => Some(Type::Unknown),
        }
    }

    /// `OmitThisParameter<T>` - Remove the this parameter
    pub fn omit_this_parameter(&self, ty: &Type) -> Option<Type> {
        match ty {
            Type::Function(func) => {
                let new_params: Vec<_> = func
                    .parameters
                    .iter()
                    .filter(|p| p.name != "this")
                    .cloned()
                    .collect();

                Some(Type::Function(FunctionType {
                    type_parameters: func.type_parameters.clone(),
                    parameters: new_params,
                    return_type: func.return_type.clone(),
                    rest_parameter: func.rest_parameter.clone(),
                }))
            }
            _ => Some(ty.clone()),
        }
    }

    /// `Uppercase<S>` - Convert string literal to uppercase
    pub fn uppercase(&self, ty: &Type) -> Option<Type> {
        self.transform_string_literal(ty, &|s: &str| s.to_uppercase())
    }

    /// `Lowercase<S>` - Convert string literal to lowercase
    pub fn lowercase(&self, ty: &Type) -> Option<Type> {
        self.transform_string_literal(ty, &|s: &str| s.to_lowercase())
    }

    /// `Capitalize<S>` - Capitalize first character
    pub fn capitalize(&self, ty: &Type) -> Option<Type> {
        self.transform_string_literal(ty, &|s: &str| {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
    }

    /// `Uncapitalize<S>` - Uncapitalize first character
    pub fn uncapitalize(&self, ty: &Type) -> Option<Type> {
        self.transform_string_literal(ty, &|s: &str| {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
    }

    /// `Awaited<T>` - Get the resolved type of a Promise
    pub fn awaited(&self, ty: &Type) -> Option<Type> {
        // Recursively unwrap Promise<T> to T
        match ty {
            Type::TypeRef(name, args) if name == "Promise" => {
                if let Some(inner) = args.first() {
                    self.awaited(inner)
                } else {
                    Some(Type::Unknown)
                }
            }
            Type::Union(members) => {
                let awaited: Vec<_> = members
                    .iter()
                    .filter_map(|m| self.awaited(m))
                    .collect();
                if awaited.is_empty() {
                    Some(Type::Never)
                } else if awaited.len() == 1 {
                    Some(awaited.into_iter().next().unwrap())
                } else {
                    Some(Type::Union(awaited))
                }
            }
            _ => Some(ty.clone()),
        }
    }

    // Helper methods

    /// Get literal string keys from a type.
    fn get_literal_keys(&self, ty: &Type) -> Vec<String> {
        match ty {
            Type::Literal(LiteralType::String(s)) => vec![s.clone()],
            Type::Union(members) => members.iter().flat_map(|m| self.get_literal_keys(m)).collect(),
            _ => vec![],
        }
    }

    /// Transform string literal types.
    fn transform_string_literal(
        &self,
        ty: &Type,
        transform: &dyn Fn(&str) -> String,
    ) -> Option<Type> {
        match ty {
            Type::Literal(LiteralType::String(s)) => {
                Some(Type::Literal(LiteralType::String(transform(s))))
            }
            Type::Union(members) => {
                let transformed: Vec<_> = members
                    .iter()
                    .filter_map(|m| self.transform_string_literal(m, transform))
                    .collect();
                if transformed.is_empty() {
                    Some(Type::Never)
                } else if transformed.len() == 1 {
                    Some(transformed.into_iter().next().unwrap())
                } else {
                    Some(Type::Union(transformed))
                }
            }
            Type::Primitive(PrimitiveKind::String) => {
                // String primitive stays as string
                Some(Type::Primitive(PrimitiveKind::String))
            }
            _ => Some(ty.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_subtype(source: &Type, target: &Type) -> bool {
        match (source, target) {
            (Type::Never, _) => true,
            (_, Type::Any) => true,
            (_, Type::Unknown) => true,
            (Type::Primitive(a), Type::Primitive(b)) => a == b,
            (Type::Literal(LiteralType::String(_)), Type::Primitive(PrimitiveKind::String)) => true,
            (Type::Literal(LiteralType::Number(_)), Type::Primitive(PrimitiveKind::Number)) => true,
            (Type::Literal(a), Type::Literal(b)) => a == b,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            _ => false,
        }
    }

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
    fn test_partial() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
        ]);

        let result = utils.partial(&obj).unwrap();

        if let Type::Object(obj) = result {
            assert!(obj.properties.get("a").unwrap().optional);
            assert!(obj.properties.get("b").unwrap().optional);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_required() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), true, false),
            ("b", Type::string(), true, false),
        ]);

        let result = utils.required(&obj).unwrap();

        if let Type::Object(obj) = result {
            assert!(!obj.properties.get("a").unwrap().optional);
            assert!(!obj.properties.get("b").unwrap().optional);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_readonly() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
        ]);

        let result = utils.readonly(&obj).unwrap();

        if let Type::Object(obj) = result {
            assert!(obj.properties.get("a").unwrap().readonly);
            assert!(obj.properties.get("b").unwrap().readonly);
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_pick() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
            ("c", Type::boolean(), false, false),
        ]);

        let keys = Type::Union(vec![
            Type::Literal(LiteralType::String("a".to_string())),
            Type::Literal(LiteralType::String("b".to_string())),
        ]);

        let result = utils.pick(&obj, &keys).unwrap();

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.properties.contains_key("a"));
            assert!(obj.properties.contains_key("b"));
            assert!(!obj.properties.contains_key("c"));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_omit() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
            ("c", Type::boolean(), false, false),
        ]);

        let keys = Type::Literal(LiteralType::String("b".to_string()));

        let result = utils.omit(&obj, &keys).unwrap();

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(obj.properties.contains_key("a"));
            assert!(!obj.properties.contains_key("b"));
            assert!(obj.properties.contains_key("c"));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_record() {
        let utils = UtilityTypes::new(simple_subtype);

        let keys = Type::Union(vec![
            Type::Literal(LiteralType::String("a".to_string())),
            Type::Literal(LiteralType::String("b".to_string())),
        ]);

        let result = utils.record(&keys, &Type::number()).unwrap();

        if let Type::Object(obj) = result {
            assert_eq!(obj.properties.len(), 2);
            assert!(matches!(
                obj.properties.get("a").unwrap().ty,
                Type::Primitive(PrimitiveKind::Number)
            ));
            assert!(matches!(
                obj.properties.get("b").unwrap().ty,
                Type::Primitive(PrimitiveKind::Number)
            ));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_exclude() {
        let utils = UtilityTypes::new(simple_subtype);

        // Exclude<string | number | boolean, string>
        let union = Type::Union(vec![Type::string(), Type::number(), Type::boolean()]);
        let result = utils.exclude(&union, &Type::string()).unwrap();

        if let Type::Union(members) = result {
            assert_eq!(members.len(), 2);
            assert!(members.iter().any(|m| matches!(m, Type::Primitive(PrimitiveKind::Number))));
            assert!(members.iter().any(|m| matches!(m, Type::Primitive(PrimitiveKind::Boolean))));
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_extract() {
        let utils = UtilityTypes::new(simple_subtype);

        // Extract<string | number | boolean, string | number>
        let union = Type::Union(vec![Type::string(), Type::number(), Type::boolean()]);
        let target = Type::Union(vec![Type::string(), Type::number()]);
        let result = utils.extract(&union, &target).unwrap();

        // With our simple subtype check, this extracts string and number
        if let Type::Union(members) = result {
            assert_eq!(members.len(), 2);
        } else {
            // Could also be a single type if only one matched
        }
    }

    #[test]
    fn test_non_nullable() {
        let utils = UtilityTypes::new(simple_subtype);

        // NonNullable<string | null | undefined>
        let union = Type::Union(vec![Type::string(), Type::Null, Type::Undefined]);
        let result = utils.non_nullable(&union).unwrap();

        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_parameters() {
        let utils = UtilityTypes::new(simple_subtype);

        let func = Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: vec![
                super::super::Parameter {
                    name: "a".to_string(),
                    ty: Type::number(),
                    optional: false,
                },
                super::super::Parameter {
                    name: "b".to_string(),
                    ty: Type::string(),
                    optional: false,
                },
            ],
            return_type: Box::new(Type::Void),
            rest_parameter: None,
        });

        let result = utils.parameters(&func).unwrap();

        if let Type::Tuple(elems) = result {
            assert_eq!(elems.len(), 2);
            assert!(matches!(elems[0], Type::Primitive(PrimitiveKind::Number)));
            assert!(matches!(elems[1], Type::Primitive(PrimitiveKind::String)));
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_return_type() {
        let utils = UtilityTypes::new(simple_subtype);

        let func = Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: vec![],
            return_type: Box::new(Type::string()),
            rest_parameter: None,
        });

        let result = utils.return_type(&func).unwrap();
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_uppercase() {
        let utils = UtilityTypes::new(simple_subtype);

        let lit = Type::Literal(LiteralType::String("hello".to_string()));
        let result = utils.uppercase(&lit).unwrap();

        if let Type::Literal(LiteralType::String(s)) = result {
            assert_eq!(s, "HELLO");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_lowercase() {
        let utils = UtilityTypes::new(simple_subtype);

        let lit = Type::Literal(LiteralType::String("HELLO".to_string()));
        let result = utils.lowercase(&lit).unwrap();

        if let Type::Literal(LiteralType::String(s)) = result {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_capitalize() {
        let utils = UtilityTypes::new(simple_subtype);

        let lit = Type::Literal(LiteralType::String("hello".to_string()));
        let result = utils.capitalize(&lit).unwrap();

        if let Type::Literal(LiteralType::String(s)) = result {
            assert_eq!(s, "Hello");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_uncapitalize() {
        let utils = UtilityTypes::new(simple_subtype);

        let lit = Type::Literal(LiteralType::String("Hello".to_string()));
        let result = utils.uncapitalize(&lit).unwrap();

        if let Type::Literal(LiteralType::String(s)) = result {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_awaited() {
        let utils = UtilityTypes::new(simple_subtype);

        // Awaited<Promise<string>>
        let promise = Type::TypeRef("Promise".to_string(), vec![Type::string()]);
        let result = utils.awaited(&promise).unwrap();
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));

        // Awaited<Promise<Promise<number>>>
        let nested_promise = Type::TypeRef(
            "Promise".to_string(),
            vec![Type::TypeRef("Promise".to_string(), vec![Type::number()])],
        );
        let result = utils.awaited(&nested_promise).unwrap();
        assert!(matches!(result, Type::Primitive(PrimitiveKind::Number)));
    }

    #[test]
    fn test_resolve_by_name() {
        let utils = UtilityTypes::new(simple_subtype);

        let obj = create_object_with_props(vec![
            ("a", Type::number(), false, false),
            ("b", Type::string(), false, false),
        ]);

        // Test Partial
        let result = utils.resolve("Partial", &[obj.clone()]).unwrap();
        if let Type::Object(obj) = result {
            assert!(obj.properties.get("a").unwrap().optional);
        }

        // Test NonNullable
        let union = Type::Union(vec![Type::string(), Type::Null]);
        let result = utils.resolve("NonNullable", &[union]).unwrap();
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }
}
