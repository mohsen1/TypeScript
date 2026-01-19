//! Type predicates and discriminated union narrowing.
//!
//! This module handles:
//! - User-defined type predicates (x is T)
//! - Discriminated union narrowing
//! - Control flow based type narrowing

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::type_defs::{Type, ObjectType, PropertySignature, CallSignature, ParameterSignature};
use super::type_guards::{TypeGuardEvaluator, TypeGuardResult};

/// A type predicate (x is T)
#[derive(Debug, Clone)]
pub struct TypePredicate {
    /// The kind of predicate
    pub kind: PredicateKind,
    /// The parameter name or 'this'
    pub parameter_name: String,
    /// The parameter index (None for 'this')
    pub parameter_index: Option<usize>,
    /// The predicate type
    pub type_: Arc<Type>,
}

/// Kind of type predicate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateKind {
    /// Regular parameter predicate (param is T)
    Identifier,
    /// This predicate (this is T)
    This,
}

impl TypePredicate {
    /// Create a new parameter predicate
    pub fn new(parameter_name: String, parameter_index: usize, type_: Arc<Type>) -> Self {
        Self {
            kind: PredicateKind::Identifier,
            parameter_name,
            parameter_index: Some(parameter_index),
            type_,
        }
    }

    /// Create a 'this' predicate
    pub fn this_predicate(type_: Arc<Type>) -> Self {
        Self {
            kind: PredicateKind::This,
            parameter_name: "this".to_string(),
            parameter_index: None,
            type_,
        }
    }
}

/// A function signature with a type predicate return type
#[derive(Debug, Clone)]
pub struct TypePredicateSignature {
    /// The base call signature
    pub signature: CallSignature,
    /// The type predicate
    pub predicate: TypePredicate,
}

impl TypePredicateSignature {
    /// Create a new type predicate signature
    pub fn new(signature: CallSignature, predicate: TypePredicate) -> Self {
        Self { signature, predicate }
    }

    /// Apply the type predicate to narrow a type
    pub fn apply(&self, argument_types: &[Arc<Type>], positive: bool) -> Option<TypeGuardResult> {
        match self.predicate.kind {
            PredicateKind::Identifier => {
                let idx = self.predicate.parameter_index?;
                let arg_type = argument_types.get(idx)?;
                Some(TypeGuardEvaluator::narrow_user_defined(
                    arg_type.as_ref(),
                    self.predicate.type_.as_ref(),
                    !positive,
                ))
            }
            PredicateKind::This => {
                // 'this' predicates need special handling from the caller
                None
            }
        }
    }
}

/// Discriminant property for discriminated unions
#[derive(Debug, Clone)]
pub struct DiscriminantProperty {
    /// The property name
    pub name: String,
    /// The literal values for each union member
    pub values: HashMap<String, Arc<Type>>,
}

/// Analyzer for discriminated unions
pub struct DiscriminatedUnionAnalyzer;

impl DiscriminatedUnionAnalyzer {
    /// Find discriminant properties in a union type
    pub fn find_discriminants(union: &[Arc<Type>]) -> Vec<DiscriminantProperty> {
        if union.len() < 2 {
            return Vec::new();
        }

        // Find common properties with literal types
        let first = &union[0];
        let common_props = Self::get_literal_properties(first);

        let mut discriminants = Vec::new();

        for prop_name in common_props {
            let mut values = HashMap::new();
            let mut is_discriminant = true;

            for member in union.iter() {
                if let Some(prop_type) = Self::get_property_type(member, &prop_name) {
                    if Self::is_unit_type(&prop_type) {
                        let key = Self::type_to_key(&prop_type);
                        if values.contains_key(&key) {
                            // Duplicate value, not a valid discriminant
                            is_discriminant = false;
                            break;
                        }
                        values.insert(key, member.clone());
                    } else {
                        is_discriminant = false;
                        break;
                    }
                } else {
                    is_discriminant = false;
                    break;
                }
            }

            if is_discriminant && values.len() == union.len() {
                discriminants.push(DiscriminantProperty {
                    name: prop_name,
                    values,
                });
            }
        }

        discriminants
    }

    /// Narrow a discriminated union based on a discriminant check
    pub fn narrow_discriminated_union(
        union_types: &[Arc<Type>],
        property_name: &str,
        compared_value: &Type,
        negate: bool,
    ) -> TypeGuardResult {
        let matching: Vec<Arc<Type>> = union_types
            .iter()
            .filter(|member| {
                if let Some(prop_type) = Self::get_property_type(member, property_name) {
                    let matches = Self::types_equal(&prop_type, compared_value);
                    if negate { !matches } else { matches }
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let excluded: Vec<Arc<Type>> = union_types
            .iter()
            .filter(|member| {
                if let Some(prop_type) = Self::get_property_type(member, property_name) {
                    let matches = Self::types_equal(&prop_type, compared_value);
                    if negate { matches } else { !matches }
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        TypeGuardResult {
            true_type: Self::create_union_or_never(matching),
            false_type: Self::create_union_or_never(excluded),
        }
    }

    /// Check if a property check is for a discriminant
    pub fn is_discriminant_check(
        union_types: &[Arc<Type>],
        property_name: &str,
    ) -> bool {
        let discriminants = Self::find_discriminants(union_types);
        discriminants.iter().any(|d| d.name == property_name)
    }

    /// Get property type from a type
    fn get_property_type(t: &Type, name: &str) -> Option<Type> {
        match t {
            Type::Object(obj) => obj.properties.get(name).map(|p| (*p.type_).clone()),
            _ => None,
        }
    }

    /// Get literal properties from a type
    fn get_literal_properties(t: &Type) -> HashSet<String> {
        let mut result = HashSet::new();

        if let Type::Object(obj) = t {
            for (name, prop) in &obj.properties {
                if Self::is_unit_type(&prop.type_) {
                    result.insert(name.clone());
                }
            }
        }

        result
    }

    /// Check if a type is a unit type (literal)
    fn is_unit_type(t: &Type) -> bool {
        matches!(
            t,
            Type::StringLiteral(_)
                | Type::NumberLiteral(_)
                | Type::BooleanLiteral(_)
                | Type::Null
                | Type::Undefined
        )
    }

    /// Convert a unit type to a key string
    fn type_to_key(t: &Type) -> String {
        match t {
            Type::StringLiteral(s) => format!("string:{}", s),
            Type::NumberLiteral(n) => format!("number:{}", n),
            Type::BooleanLiteral(b) => format!("boolean:{}", b),
            Type::Null => "null".to_string(),
            Type::Undefined => "undefined".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Check if two types are equal
    fn types_equal(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            _ => false,
        }
    }

    /// Create union or never from types
    fn create_union_or_never(types: Vec<Arc<Type>>) -> Arc<Type> {
        match types.len() {
            0 => Arc::new(Type::Never),
            1 => types.into_iter().next().unwrap(),
            _ => Arc::new(Type::Union(types)),
        }
    }
}

/// Control flow node for type narrowing
#[derive(Debug, Clone)]
pub struct FlowNode {
    /// The kind of flow node
    pub kind: FlowNodeKind,
    /// Previous flow nodes
    pub antecedents: Vec<Box<FlowNode>>,
}

/// Kind of control flow node
#[derive(Debug, Clone)]
pub enum FlowNodeKind {
    /// Start of flow
    Start,
    /// Assignment
    Assignment {
        symbol: String,
        type_: Arc<Type>,
    },
    /// Narrowing condition
    Condition {
        expression: NarrowingExpression,
        positive: bool,
    },
    /// Branch point (if/switch)
    Branch,
    /// Join point (after if/switch)
    Join,
    /// Loop back edge
    LoopBack,
    /// Unreachable code
    Unreachable,
}

/// An expression that causes narrowing
#[derive(Debug, Clone)]
pub enum NarrowingExpression {
    /// typeof check
    Typeof {
        symbol: String,
        type_string: String,
    },
    /// instanceof check
    Instanceof {
        symbol: String,
        type_: Arc<Type>,
    },
    /// Type predicate call
    TypePredicate {
        symbol: String,
        type_: Arc<Type>,
    },
    /// Strict equality
    StrictEquality {
        symbol: String,
        value: Arc<Type>,
    },
    /// In operator
    InOperator {
        property: String,
        symbol: String,
    },
    /// Truthiness
    Truthiness {
        symbol: String,
    },
    /// Discriminant property check
    Discriminant {
        symbol: String,
        property: String,
        value: Arc<Type>,
    },
}

/// Control flow analyzer for type narrowing
pub struct ControlFlowAnalyzer {
    /// Current type state for symbols
    type_state: HashMap<String, Arc<Type>>,
}

impl ControlFlowAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            type_state: HashMap::new(),
        }
    }

    /// Initialize a symbol's type
    pub fn set_type(&mut self, symbol: &str, type_: Arc<Type>) {
        self.type_state.insert(symbol.to_string(), type_);
    }

    /// Get current type for a symbol
    pub fn get_type(&self, symbol: &str) -> Option<&Arc<Type>> {
        self.type_state.get(symbol)
    }

    /// Apply a narrowing expression
    pub fn apply_narrowing(&mut self, expr: &NarrowingExpression, positive: bool) {
        match expr {
            NarrowingExpression::Typeof { symbol, type_string } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_typeof(&current, type_string, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::Instanceof { symbol, type_ } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_instanceof(&current, type_, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::TypePredicate { symbol, type_ } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_user_defined(&current, type_, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::StrictEquality { symbol, value } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_strict_equality(&current, value, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::InOperator { property, symbol } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_in_operator(&current, property, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::Truthiness { symbol } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    let result = TypeGuardEvaluator::narrow_truthiness(&current, !positive);
                    let narrowed = if positive { result.true_type } else { result.false_type };
                    self.type_state.insert(symbol.clone(), narrowed);
                }
            }
            NarrowingExpression::Discriminant { symbol, property, value } => {
                if let Some(current) = self.type_state.get(symbol).cloned() {
                    if let Type::Union(types) = current.as_ref() {
                        let result = DiscriminatedUnionAnalyzer::narrow_discriminated_union(
                            types,
                            property,
                            value,
                            !positive,
                        );
                        let narrowed = if positive { result.true_type } else { result.false_type };
                        self.type_state.insert(symbol.clone(), narrowed);
                    }
                }
            }
        }
    }

    /// Create a snapshot of current type state
    pub fn snapshot(&self) -> HashMap<String, Arc<Type>> {
        self.type_state.clone()
    }

    /// Restore type state from snapshot
    pub fn restore(&mut self, snapshot: HashMap<String, Arc<Type>>) {
        self.type_state = snapshot;
    }

    /// Merge two type states (for join points)
    pub fn merge(&mut self, other: &HashMap<String, Arc<Type>>) {
        for (symbol, other_type) in other {
            if let Some(current_type) = self.type_state.get(symbol) {
                // Create union of the two types
                let merged = Self::union_types(current_type, other_type);
                self.type_state.insert(symbol.clone(), merged);
            } else {
                self.type_state.insert(symbol.clone(), other_type.clone());
            }
        }
    }

    /// Union two types
    fn union_types(a: &Type, b: &Type) -> Arc<Type> {
        match (a, b) {
            (Type::Never, t) | (t, Type::Never) => Arc::new(t.clone()),
            (Type::Union(types_a), Type::Union(types_b)) => {
                let mut combined = types_a.clone();
                for t in types_b {
                    if !combined.iter().any(|existing| Self::same_type(existing, t)) {
                        combined.push(t.clone());
                    }
                }
                Arc::new(Type::Union(combined))
            }
            (Type::Union(types), other) | (other, Type::Union(types)) => {
                let mut combined = types.clone();
                let other_arc = Arc::new(other.clone());
                if !combined.iter().any(|existing| Self::same_type(existing, &other_arc)) {
                    combined.push(other_arc);
                }
                Arc::new(Type::Union(combined))
            }
            _ if Self::same_type(&Arc::new(a.clone()), &Arc::new(b.clone())) => {
                Arc::new(a.clone())
            }
            _ => Arc::new(Type::Union(vec![Arc::new(a.clone()), Arc::new(b.clone())])),
        }
    }

    /// Check if two types are the same
    fn same_type(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::String, Type::String) => true,
            (Type::Number, Type::Number) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            (Type::Never, Type::Never) => true,
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl Default for ControlFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_object_with_kind(kind_value: &str) -> Arc<Type> {
        let mut obj = ObjectType::default();
        obj.properties.insert(
            "kind".to_string(),
            PropertySignature {
                name: "kind".to_string(),
                type_: Arc::new(Type::StringLiteral(kind_value.to_string())),
                optional: false,
                readonly: false,
            },
        );
        Arc::new(Type::Object(obj))
    }

    #[test]
    fn test_find_discriminants() {
        let circle = make_object_with_kind("circle");
        let square = make_object_with_kind("square");

        let union = vec![circle, square];
        let discriminants = DiscriminatedUnionAnalyzer::find_discriminants(&union);

        assert_eq!(discriminants.len(), 1);
        assert_eq!(discriminants[0].name, "kind");
    }

    #[test]
    fn test_narrow_discriminated_union() {
        let circle = make_object_with_kind("circle");
        let square = make_object_with_kind("square");

        let union = vec![circle, square];
        let result = DiscriminatedUnionAnalyzer::narrow_discriminated_union(
            &union,
            "kind",
            &Type::StringLiteral("circle".to_string()),
            false,
        );

        // True type should be circle
        match result.true_type.as_ref() {
            Type::Object(obj) => {
                let kind_prop = obj.properties.get("kind").unwrap();
                match kind_prop.type_.as_ref() {
                    Type::StringLiteral(s) => assert_eq!(s, "circle"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected Object type"),
        }

        // False type should be square
        match result.false_type.as_ref() {
            Type::Object(obj) => {
                let kind_prop = obj.properties.get("kind").unwrap();
                match kind_prop.type_.as_ref() {
                    Type::StringLiteral(s) => assert_eq!(s, "square"),
                    _ => panic!("Expected StringLiteral"),
                }
            }
            _ => panic!("Expected Object type"),
        }
    }

    #[test]
    fn test_type_predicate() {
        let predicate = TypePredicate::new(
            "value".to_string(),
            0,
            Arc::new(Type::String),
        );

        let signature = TypePredicateSignature::new(
            CallSignature {
                type_parameters: Vec::new(),
                parameters: vec![ParameterSignature {
                    name: "value".to_string(),
                    type_: Arc::new(Type::Unknown),
                    optional: false,
                    rest: false,
                }],
                return_type: Arc::new(Type::Boolean),
            },
            predicate,
        );

        let args = vec![Arc::new(Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Number),
        ]))];

        let result = signature.apply(&args, true);
        assert!(result.is_some());

        let result = result.unwrap();
        match result.true_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type"),
        }
    }

    #[test]
    fn test_control_flow_narrowing() {
        let mut analyzer = ControlFlowAnalyzer::new();

        // Set initial type as string | number
        let union = Arc::new(Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Number),
        ]));
        analyzer.set_type("x", union);

        // Apply typeof === "string" narrowing
        analyzer.apply_narrowing(
            &NarrowingExpression::Typeof {
                symbol: "x".to_string(),
                type_string: "string".to_string(),
            },
            true,
        );

        // Type should now be string
        let narrowed = analyzer.get_type("x").unwrap();
        match narrowed.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type after narrowing"),
        }
    }

    #[test]
    fn test_control_flow_discriminant() {
        let mut analyzer = ControlFlowAnalyzer::new();

        let circle = make_object_with_kind("circle");
        let square = make_object_with_kind("square");

        let union = Arc::new(Type::Union(vec![circle, square]));
        analyzer.set_type("shape", union);

        // Apply discriminant narrowing
        analyzer.apply_narrowing(
            &NarrowingExpression::Discriminant {
                symbol: "shape".to_string(),
                property: "kind".to_string(),
                value: Arc::new(Type::StringLiteral("circle".to_string())),
            },
            true,
        );

        // Type should now be circle
        let narrowed = analyzer.get_type("shape").unwrap();
        match narrowed.as_ref() {
            Type::Object(obj) => {
                let kind_prop = obj.properties.get("kind").unwrap();
                match kind_prop.type_.as_ref() {
                    Type::StringLiteral(s) => assert_eq!(s, "circle"),
                    _ => panic!("Expected StringLiteral 'circle'"),
                }
            }
            _ => panic!("Expected Object type"),
        }
    }

    #[test]
    fn test_this_predicate() {
        let predicate = TypePredicate::this_predicate(Arc::new(Type::String));

        assert_eq!(predicate.kind, PredicateKind::This);
        assert_eq!(predicate.parameter_name, "this");
        assert!(predicate.parameter_index.is_none());
    }
}
