//! Generic instantiation and type parameter inference.
//!
//! This module handles:
//! - Instantiation of generic types with type arguments
//! - Type parameter inference from usage
//! - Type argument validation against constraints

use std::collections::HashMap;

use super::{
    ConditionalType, FunctionType, IndexSignature, MappedType, ObjectType, Parameter, Property,
    TemplateLiteralSpan, Type, TypeParameter,
};

/// Type substitution map for generic instantiation.
pub type TypeSubstitution = HashMap<String, Type>;

/// Result of type inference.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Inferred type arguments.
    pub type_args: TypeSubstitution,
    /// Whether inference was successful.
    pub success: bool,
    /// Error message if inference failed.
    pub error: Option<String>,
}

impl InferenceResult {
    pub fn success(type_args: TypeSubstitution) -> Self {
        Self {
            type_args,
            success: true,
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            type_args: HashMap::new(),
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Generic type instantiator.
pub struct GenericInstantiator {
    /// Type substitution for type parameters.
    substitution: TypeSubstitution,
}

impl GenericInstantiator {
    /// Create a new instantiator with the given substitution.
    pub fn new(substitution: TypeSubstitution) -> Self {
        Self { substitution }
    }

    /// Create an instantiator from type parameters and arguments.
    pub fn from_params_and_args(params: &[TypeParameter], args: &[Type]) -> Self {
        let mut substitution = HashMap::new();
        for (param, arg) in params.iter().zip(args.iter()) {
            substitution.insert(param.name.clone(), arg.clone());
        }
        // Fill in defaults for missing arguments
        for param in params.iter().skip(args.len()) {
            if let Some(default) = &param.default {
                substitution.insert(param.name.clone(), (**default).clone());
            }
        }
        Self { substitution }
    }

    /// Instantiate a type with the current substitution.
    pub fn instantiate(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeParameter(tp) => {
                if let Some(substituted) = self.substitution.get(&tp.name) {
                    substituted.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Array(elem) => Type::Array(Box::new(self.instantiate(elem))),
            Type::Tuple(elements) => {
                Type::Tuple(elements.iter().map(|e| self.instantiate(e)).collect())
            }
            Type::Union(members) => {
                let instantiated: Vec<_> = members.iter().map(|m| self.instantiate(m)).collect();
                Type::union(instantiated)
            }
            Type::Intersection(members) => {
                let instantiated: Vec<_> = members.iter().map(|m| self.instantiate(m)).collect();
                Type::intersection(instantiated)
            }
            Type::Object(obj) => Type::Object(self.instantiate_object(obj)),
            Type::Function(func) => Type::Function(self.instantiate_function(func)),
            Type::Conditional(cond) => {
                Type::Conditional(Box::new(self.instantiate_conditional(cond)))
            }
            Type::Mapped(mapped) => Type::Mapped(Box::new(self.instantiate_mapped(mapped))),
            Type::IndexedAccess(obj_type, index_type) => Type::IndexedAccess(
                Box::new(self.instantiate(obj_type)),
                Box::new(self.instantiate(index_type)),
            ),
            Type::Keyof(inner) => Type::Keyof(Box::new(self.instantiate(inner))),
            Type::TypeRef(name, args) => {
                let new_args: Vec<_> = args.iter().map(|a| self.instantiate(a)).collect();
                Type::TypeRef(name.clone(), new_args)
            }
            Type::TemplateLiteral(spans) => {
                let new_spans: Vec<_> = spans
                    .iter()
                    .map(|span| match span {
                        TemplateLiteralSpan::Text(t) => TemplateLiteralSpan::Text(t.clone()),
                        TemplateLiteralSpan::Type(t) => {
                            TemplateLiteralSpan::Type(self.instantiate(t))
                        }
                    })
                    .collect();
                Type::TemplateLiteral(new_spans)
            }
            // Primitive types and other leaf types don't need instantiation
            Type::Any
            | Type::Unknown
            | Type::Never
            | Type::Void
            | Type::Null
            | Type::Undefined
            | Type::Primitive(_)
            | Type::Literal(_)
            | Type::Infer(_) => ty.clone(),
        }
    }

    /// Instantiate an object type.
    fn instantiate_object(&self, obj: &ObjectType) -> ObjectType {
        ObjectType {
            properties: obj
                .properties
                .iter()
                .map(|(name, prop)| {
                    (
                        name.clone(),
                        Property {
                            name: prop.name.clone(),
                            ty: self.instantiate(&prop.ty),
                            optional: prop.optional,
                            readonly: prop.readonly,
                        },
                    )
                })
                .collect(),
            call_signatures: obj
                .call_signatures
                .iter()
                .map(|sig| self.instantiate_function(sig))
                .collect(),
            construct_signatures: obj
                .construct_signatures
                .iter()
                .map(|sig| self.instantiate_function(sig))
                .collect(),
            index_signatures: obj
                .index_signatures
                .iter()
                .map(|sig| IndexSignature {
                    key_type: sig.key_type,
                    value_type: self.instantiate(&sig.value_type),
                    readonly: sig.readonly,
                })
                .collect(),
        }
    }

    /// Instantiate a function type.
    fn instantiate_function(&self, func: &FunctionType) -> FunctionType {
        // Don't substitute type parameters that are shadowed by this function's own type params
        let mut inner_subst = self.substitution.clone();
        for tp in &func.type_parameters {
            inner_subst.remove(&tp.name);
        }
        let inner_instantiator = GenericInstantiator::new(inner_subst);

        FunctionType {
            type_parameters: func.type_parameters.clone(),
            parameters: func
                .parameters
                .iter()
                .map(|p| Parameter {
                    name: p.name.clone(),
                    ty: inner_instantiator.instantiate(&p.ty),
                    optional: p.optional,
                })
                .collect(),
            return_type: Box::new(inner_instantiator.instantiate(&func.return_type)),
            rest_parameter: func.rest_parameter.as_ref().map(|p| {
                Box::new(Parameter {
                    name: p.name.clone(),
                    ty: inner_instantiator.instantiate(&p.ty),
                    optional: p.optional,
                })
            }),
        }
    }

    /// Instantiate a conditional type.
    fn instantiate_conditional(&self, cond: &ConditionalType) -> ConditionalType {
        ConditionalType {
            check_type: self.instantiate(&cond.check_type),
            extends_type: self.instantiate(&cond.extends_type),
            true_type: self.instantiate(&cond.true_type),
            false_type: self.instantiate(&cond.false_type),
        }
    }

    /// Instantiate a mapped type.
    fn instantiate_mapped(&self, mapped: &MappedType) -> MappedType {
        // The type parameter is local to the mapped type, don't substitute it
        let mut inner_subst = self.substitution.clone();
        inner_subst.remove(&mapped.type_parameter);

        let inner_instantiator = GenericInstantiator::new(inner_subst);

        MappedType {
            type_parameter: mapped.type_parameter.clone(),
            constraint: inner_instantiator.instantiate(&mapped.constraint),
            template_type: inner_instantiator.instantiate(&mapped.template_type),
            optional_modifier: mapped.optional_modifier,
            readonly_modifier: mapped.readonly_modifier,
            name_type: mapped
                .name_type
                .as_ref()
                .map(|t| inner_instantiator.instantiate(t)),
        }
    }
}

/// Type inference engine for inferring type arguments from usage.
pub struct TypeInferrer {
    /// Inferred type arguments.
    inferences: HashMap<String, Vec<Type>>,
    /// Type parameters being inferred.
    type_parameters: Vec<String>,
}

impl TypeInferrer {
    /// Create a new inferrer for the given type parameters.
    pub fn new(type_parameters: &[TypeParameter]) -> Self {
        Self {
            inferences: HashMap::new(),
            type_parameters: type_parameters.iter().map(|p| p.name.clone()).collect(),
        }
    }

    /// Infer type arguments by comparing a source type to a target type pattern.
    pub fn infer(&mut self, source: &Type, target: &Type) {
        match (source, target) {
            // If target is a type parameter we're inferring, record the source
            (_, Type::TypeParameter(tp)) if self.type_parameters.contains(&tp.name) => {
                self.inferences
                    .entry(tp.name.clone())
                    .or_default()
                    .push(source.clone());
            }

            // For Infer types in conditional types
            (_, Type::Infer(name)) if self.type_parameters.contains(name) => {
                self.inferences
                    .entry(name.clone())
                    .or_default()
                    .push(source.clone());
            }

            // Recurse into array types
            (Type::Array(source_elem), Type::Array(target_elem)) => {
                self.infer(source_elem, target_elem);
            }

            // Recurse into tuple types
            (Type::Tuple(source_elems), Type::Tuple(target_elems)) => {
                for (s, t) in source_elems.iter().zip(target_elems.iter()) {
                    self.infer(s, t);
                }
            }

            // Recurse into union types
            (Type::Union(source_members), Type::Union(target_members)) => {
                // Simple heuristic: match by position
                for (s, t) in source_members.iter().zip(target_members.iter()) {
                    self.infer(s, t);
                }
            }

            // Recurse into function types
            (Type::Function(source_func), Type::Function(target_func)) => {
                // Infer from parameters (contravariant)
                for (s_param, t_param) in source_func
                    .parameters
                    .iter()
                    .zip(target_func.parameters.iter())
                {
                    self.infer(&t_param.ty, &s_param.ty);
                }
                // Infer from return type (covariant)
                self.infer(&source_func.return_type, &target_func.return_type);
            }

            // Recurse into object types
            (Type::Object(source_obj), Type::Object(target_obj)) => {
                for (name, target_prop) in &target_obj.properties {
                    if let Some(source_prop) = source_obj.properties.get(name) {
                        self.infer(&source_prop.ty, &target_prop.ty);
                    }
                }
            }

            // Recurse into type references
            (Type::TypeRef(source_name, source_args), Type::TypeRef(target_name, target_args))
                if source_name == target_name =>
            {
                for (s, t) in source_args.iter().zip(target_args.iter()) {
                    self.infer(s, t);
                }
            }

            // Recurse into indexed access
            (Type::IndexedAccess(s_obj, s_idx), Type::IndexedAccess(t_obj, t_idx)) => {
                self.infer(s_obj, t_obj);
                self.infer(s_idx, t_idx);
            }

            // Other cases don't contribute to inference
            _ => {}
        }
    }

    /// Get the final inferred type arguments.
    pub fn get_inferred_types(&self) -> InferenceResult {
        let mut type_args = HashMap::new();

        for param_name in &self.type_parameters {
            if let Some(candidates) = self.inferences.get(param_name) {
                if candidates.is_empty() {
                    // No inference - will use default or error
                    continue;
                }
                // Use the first candidate (or we could do union of candidates)
                // In a real implementation, we'd pick the best common supertype
                type_args.insert(param_name.clone(), candidates[0].clone());
            }
        }

        InferenceResult::success(type_args)
    }
}

/// Validate type arguments against type parameter constraints.
pub fn validate_type_args(
    params: &[TypeParameter],
    args: &[Type],
    is_subtype: impl Fn(&Type, &Type) -> bool,
) -> Result<(), String> {
    if args.len() > params.len() {
        return Err(format!(
            "Expected {} type arguments, got {}",
            params.len(),
            args.len()
        ));
    }

    for (i, (param, arg)) in params.iter().zip(args.iter()).enumerate() {
        if let Some(constraint) = &param.constraint {
            // Instantiate the constraint with previous type args
            let mut subst = HashMap::new();
            for (p, a) in params.iter().zip(args.iter()).take(i) {
                subst.insert(p.name.clone(), a.clone());
            }
            let instantiator = GenericInstantiator::new(subst);
            let resolved_constraint = instantiator.instantiate(constraint);

            if !is_subtype(arg, &resolved_constraint) {
                return Err(format!(
                    "Type argument for '{}' does not satisfy constraint",
                    param.name
                ));
            }
        }
    }

    // Check for missing required type arguments
    for param in params.iter().skip(args.len()) {
        if param.default.is_none() {
            return Err(format!("Missing type argument for '{}'", param.name));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PrimitiveKind;

    fn type_param(name: &str) -> Type {
        Type::TypeParameter(TypeParameter {
            name: name.to_string(),
            constraint: None,
            default: None,
        })
    }

    #[test]
    fn test_simple_instantiation() {
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), Type::number());

        let instantiator = GenericInstantiator::new(subst);

        // T should become number
        let result = instantiator.instantiate(&type_param("T"));
        assert!(matches!(result, Type::Primitive(PrimitiveKind::Number)));

        // Array<T> should become Array<number>
        let array_t = Type::Array(Box::new(type_param("T")));
        let result = instantiator.instantiate(&array_t);
        if let Type::Array(elem) = result {
            assert!(matches!(*elem, Type::Primitive(PrimitiveKind::Number)));
        } else {
            panic!("Expected array type");
        }
    }

    #[test]
    fn test_tuple_instantiation() {
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), Type::number());
        subst.insert("U".to_string(), Type::string());

        let instantiator = GenericInstantiator::new(subst);

        let tuple = Type::Tuple(vec![type_param("T"), type_param("U")]);
        let result = instantiator.instantiate(&tuple);

        if let Type::Tuple(elems) = result {
            assert_eq!(elems.len(), 2);
            assert!(matches!(elems[0], Type::Primitive(PrimitiveKind::Number)));
            assert!(matches!(elems[1], Type::Primitive(PrimitiveKind::String)));
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_union_instantiation() {
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), Type::number());

        let instantiator = GenericInstantiator::new(subst);

        let union = Type::Union(vec![type_param("T"), Type::Null]);
        let result = instantiator.instantiate(&union);

        if let Type::Union(members) = result {
            assert_eq!(members.len(), 2);
            assert!(matches!(members[0], Type::Primitive(PrimitiveKind::Number)));
            assert!(matches!(members[1], Type::Null));
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_object_instantiation() {
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), Type::number());

        let instantiator = GenericInstantiator::new(subst);

        let mut props = HashMap::new();
        props.insert(
            "value".to_string(),
            Property {
                name: "value".to_string(),
                ty: type_param("T"),
                optional: false,
                readonly: false,
            },
        );
        let obj = Type::Object(ObjectType {
            properties: props,
            ..Default::default()
        });

        let result = instantiator.instantiate(&obj);

        if let Type::Object(obj) = result {
            let prop = obj.properties.get("value").unwrap();
            assert!(matches!(prop.ty, Type::Primitive(PrimitiveKind::Number)));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_function_instantiation() {
        let mut subst = HashMap::new();
        subst.insert("T".to_string(), Type::number());

        let instantiator = GenericInstantiator::new(subst);

        let func = Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: vec![Parameter {
                name: "x".to_string(),
                ty: type_param("T"),
                optional: false,
            }],
            return_type: Box::new(type_param("T")),
            rest_parameter: None,
        });

        let result = instantiator.instantiate(&func);

        if let Type::Function(f) = result {
            assert!(matches!(
                f.parameters[0].ty,
                Type::Primitive(PrimitiveKind::Number)
            ));
            assert!(matches!(*f.return_type, Type::Primitive(PrimitiveKind::Number)));
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_inference_simple() {
        let params = vec![TypeParameter {
            name: "T".to_string(),
            constraint: None,
            default: None,
        }];

        let mut inferrer = TypeInferrer::new(&params);

        // Infer T from: Array<number> matches Array<T>
        let source = Type::Array(Box::new(Type::number()));
        let target = Type::Array(Box::new(type_param("T")));

        inferrer.infer(&source, &target);

        let result = inferrer.get_inferred_types();
        assert!(result.success);
        assert!(matches!(
            result.type_args.get("T"),
            Some(Type::Primitive(PrimitiveKind::Number))
        ));
    }

    #[test]
    fn test_inference_multiple() {
        let params = vec![
            TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            },
            TypeParameter {
                name: "U".to_string(),
                constraint: None,
                default: None,
            },
        ];

        let mut inferrer = TypeInferrer::new(&params);

        // Infer T and U from: [number, string] matches [T, U]
        let source = Type::Tuple(vec![Type::number(), Type::string()]);
        let target = Type::Tuple(vec![type_param("T"), type_param("U")]);

        inferrer.infer(&source, &target);

        let result = inferrer.get_inferred_types();
        assert!(result.success);
        assert!(matches!(
            result.type_args.get("T"),
            Some(Type::Primitive(PrimitiveKind::Number))
        ));
        assert!(matches!(
            result.type_args.get("U"),
            Some(Type::Primitive(PrimitiveKind::String))
        ));
    }

    #[test]
    fn test_validate_type_args() {
        let params = vec![
            TypeParameter {
                name: "T".to_string(),
                constraint: Some(Box::new(Type::Object(ObjectType::default()))),
                default: None,
            },
        ];

        // This should succeed with an object type
        let args = vec![Type::Object(ObjectType::default())];
        let result = validate_type_args(&params, &args, |sub, sup| {
            // Simple check: object is subtype of object
            matches!((sub, sup), (Type::Object(_), Type::Object(_)))
        });
        assert!(result.is_ok());

        // This should fail with a number type
        let args = vec![Type::number()];
        let result = validate_type_args(&params, &args, |sub, sup| {
            matches!((sub, sup), (Type::Object(_), Type::Object(_)))
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_instantiate_from_params_and_args() {
        let params = vec![
            TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            },
            TypeParameter {
                name: "U".to_string(),
                constraint: None,
                default: Some(Box::new(Type::string())),
            },
        ];

        // Only provide T, U should use default
        let args = vec![Type::number()];
        let instantiator = GenericInstantiator::from_params_and_args(&params, &args);

        assert!(matches!(
            instantiator.instantiate(&type_param("T")),
            Type::Primitive(PrimitiveKind::Number)
        ));
        assert!(matches!(
            instantiator.instantiate(&type_param("U")),
            Type::Primitive(PrimitiveKind::String)
        ));
    }
}
