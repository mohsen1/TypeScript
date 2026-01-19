//! Promise Type Utilities
//!
//! This module handles Promise-related type operations:
//! - Creating Promise<T> types
//! - Implementing Awaited<T> type resolution
//! - Checking thenable compatibility
//! - Handling Promise.all/race/allSettled type inference

use zang_core::{InternedString, StringInterner};

use crate::types::{
    FunctionType, ObjectType, ParameterType, ResolvedType, TypeReference,
};

/// Promise type utilities
pub struct PromiseChecker<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Cached Promise type name
    promise_name: InternedString,
    /// Cached PromiseLike type name
    promise_like_name: InternedString,
    /// Cached Awaited type name
    awaited_name: InternedString,
    /// Recursion depth for awaited type resolution
    awaited_depth: u32,
    /// Maximum recursion depth
    max_awaited_depth: u32,
}

/// Information about a Promise type
#[derive(Debug, Clone)]
pub struct PromiseType {
    /// The inner type T in Promise<T>
    pub inner_type: ResolvedType,
    /// Whether this is a native Promise
    pub is_native: bool,
    /// Whether this is a PromiseLike/thenable
    pub is_thenable: bool,
}

/// Result of checking Promise.all/race/allSettled
#[derive(Debug, Clone)]
pub struct PromiseCombinatorResult {
    /// The result type
    pub result_type: ResolvedType,
    /// The inner types from all arguments
    pub inner_types: Vec<ResolvedType>,
}

impl<'a> PromiseChecker<'a> {
    /// Creates a new Promise checker
    pub fn new(interner: &'a StringInterner) -> Self {
        Self {
            promise_name: interner.intern("Promise"),
            promise_like_name: interner.intern("PromiseLike"),
            awaited_name: interner.intern("Awaited"),
            interner,
            awaited_depth: 0,
            max_awaited_depth: 100,
        }
    }

    /// Creates a Promise<T> type
    pub fn create_promise_type(&self, inner_type: ResolvedType) -> ResolvedType {
        ResolvedType::Reference(TypeReference {
            name: self.promise_name,
            type_arguments: vec![inner_type],
            target: None,
        })
    }

    /// Creates a PromiseLike<T> type
    pub fn create_promise_like_type(&self, inner_type: ResolvedType) -> ResolvedType {
        ResolvedType::Reference(TypeReference {
            name: self.promise_like_name,
            type_arguments: vec![inner_type],
            target: None,
        })
    }

    /// Checks if a type is a Promise or PromiseLike
    pub fn is_promise_like(&self, ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Reference(type_ref) => {
                type_ref.name == self.promise_name || type_ref.name == self.promise_like_name
            }
            ResolvedType::Object(obj) => {
                // Check if it has a `then` method - this makes it a thenable
                self.has_then_method(obj)
            }
            ResolvedType::Union(types) => {
                // A union is promise-like if any member is promise-like
                types.iter().any(|t| self.is_promise_like(t))
            }
            ResolvedType::Any => true, // Any could be a promise
            _ => false,
        }
    }

    /// Checks if an object type has a `then` method (making it a thenable)
    fn has_then_method(&self, obj: &ObjectType) -> bool {
        let then_name = self.interner.intern("then");
        obj.properties.iter().any(|p| {
            p.name == then_name && matches!(p.ty.as_ref(), ResolvedType::Function(_))
        })
    }

    /// Gets the inner type from a Promise<T> or PromiseLike<T>
    pub fn get_promise_inner_type(&self, ty: &ResolvedType) -> Option<ResolvedType> {
        match ty {
            ResolvedType::Reference(type_ref) => {
                if type_ref.name == self.promise_name || type_ref.name == self.promise_like_name {
                    type_ref.type_arguments.first().cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Implements the Awaited<T> type - recursively unwraps promises/thenables
    pub fn get_awaited_type(&self, ty: &ResolvedType) -> ResolvedType {
        self.get_awaited_type_recursive(ty, 0)
    }

    /// Recursive implementation of Awaited<T>
    fn get_awaited_type_recursive(&self, ty: &ResolvedType, depth: u32) -> ResolvedType {
        if depth > self.max_awaited_depth {
            // Prevent infinite recursion
            return ty.clone();
        }

        match ty {
            // Promise<T> -> Awaited<T>
            ResolvedType::Reference(type_ref) => {
                if type_ref.name == self.promise_name || type_ref.name == self.promise_like_name {
                    if let Some(inner) = type_ref.type_arguments.first() {
                        // Recursively unwrap nested promises
                        return self.get_awaited_type_recursive(inner, depth + 1);
                    }
                }
                // Not a promise type, return as-is
                ty.clone()
            }

            // Union types: Awaited<T | U> = Awaited<T> | Awaited<U>
            ResolvedType::Union(types) => {
                let awaited_types: Vec<_> = types
                    .iter()
                    .map(|t| self.get_awaited_type_recursive(t, depth + 1))
                    .collect();

                // Deduplicate and simplify the union
                self.simplify_union(awaited_types)
            }

            // Intersection types: Awaited<T & U> = Awaited<T> & Awaited<U>
            ResolvedType::Intersection(types) => {
                let awaited_types: Vec<_> = types
                    .iter()
                    .map(|t| self.get_awaited_type_recursive(t, depth + 1))
                    .collect();
                ResolvedType::Intersection(awaited_types)
            }

            // Object types with `then` method are thenables
            ResolvedType::Object(obj) => {
                if let Some(then_return) = self.get_then_return_type(obj) {
                    return self.get_awaited_type_recursive(&then_return, depth + 1);
                }
                ty.clone()
            }

            // Any stays any
            ResolvedType::Any => ResolvedType::Any,

            // Never stays never
            ResolvedType::Never => ResolvedType::Never,

            // Unknown becomes unknown
            ResolvedType::Unknown => ResolvedType::Unknown,

            // Primitives and other types don't unwrap
            _ => ty.clone(),
        }
    }

    /// Gets the return type from a `then` method if present
    fn get_then_return_type(&self, obj: &ObjectType) -> Option<ResolvedType> {
        let then_name = self.interner.intern("then");

        for prop in &obj.properties {
            if prop.name == then_name {
                if let ResolvedType::Function(func) = prop.ty.as_ref() {
                    // The then callback's return type determines the awaited type
                    // then<T>(onfulfilled: (value: T) => U): Promise<U>
                    // We want to get U from the first parameter's return type
                    if let Some(first_param) = func.parameters.first() {
                        if let ResolvedType::Function(callback) = first_param.ty.as_ref() {
                            return Some(callback.return_type.as_ref().clone());
                        }
                    }
                }
            }
        }

        // Also check call signatures
        for sig in &obj.call_signatures {
            // Look for then-like signatures
            if let Some(first_param) = sig.parameters.first() {
                if let ResolvedType::Function(callback) = first_param.ty.as_ref() {
                    return Some(callback.return_type.as_ref().clone());
                }
            }
        }

        None
    }

    /// Simplifies a union type by removing duplicates
    fn simplify_union(&self, types: Vec<ResolvedType>) -> ResolvedType {
        let mut unique_types: Vec<ResolvedType> = Vec::new();

        for ty in types {
            // Simple deduplication - just checks if the same discriminant exists
            let dominated = unique_types.iter().any(|existing| {
                std::mem::discriminant(existing) == std::mem::discriminant(&ty)
            });

            if !dominated {
                unique_types.push(ty);
            }
        }

        if unique_types.len() == 1 {
            unique_types.pop().unwrap()
        } else if unique_types.is_empty() {
            ResolvedType::Never
        } else {
            ResolvedType::Union(unique_types)
        }
    }

    /// Infers the type of Promise.all(promises)
    pub fn infer_promise_all(&self, argument_types: &[ResolvedType]) -> ResolvedType {
        // Promise.all<T extends readonly unknown[] | []>(values: T): Promise<{ -readonly [P in keyof T]: Awaited<T[P]> }>
        // Simplified: Promise.all([Promise<A>, Promise<B>]) -> Promise<[A, B]>

        let awaited_types: Vec<_> = argument_types
            .iter()
            .map(|t| self.get_awaited_type(t))
            .collect();

        // If single array argument, extract element types
        if argument_types.len() == 1 {
            if let ResolvedType::Array(elem) = &argument_types[0] {
                let awaited_elem = self.get_awaited_type(elem);
                return self.create_promise_type(ResolvedType::Array(Box::new(awaited_elem)));
            }
            if let ResolvedType::Tuple(elems) = &argument_types[0] {
                let awaited_elems: Vec<_> = elems
                    .iter()
                    .map(|e| self.get_awaited_type(e))
                    .collect();
                return self.create_promise_type(ResolvedType::Tuple(awaited_elems));
            }
        }

        // Return Promise<[...awaited_types]>
        self.create_promise_type(ResolvedType::Tuple(awaited_types))
    }

    /// Infers the type of Promise.race(promises)
    pub fn infer_promise_race(&self, argument_types: &[ResolvedType]) -> ResolvedType {
        // Promise.race<T>(values: Iterable<T | PromiseLike<T>>): Promise<Awaited<T>>
        // The result is a promise of the union of all awaited types

        let awaited_types: Vec<_> = argument_types
            .iter()
            .map(|t| self.get_awaited_type(t))
            .collect();

        let result_type = if awaited_types.len() == 1 {
            awaited_types.into_iter().next().unwrap()
        } else {
            self.simplify_union(awaited_types)
        };

        self.create_promise_type(result_type)
    }

    /// Infers the type of Promise.allSettled(promises)
    pub fn infer_promise_all_settled(&self, argument_types: &[ResolvedType]) -> ResolvedType {
        // Promise.allSettled returns Promise<PromiseSettledResult<T>[]>
        // where PromiseSettledResult<T> = PromiseFulfilledResult<T> | PromiseRejectedResult

        let settled_result_name = self.interner.intern("PromiseSettledResult");

        let settled_types: Vec<_> = argument_types
            .iter()
            .map(|t| {
                let awaited = self.get_awaited_type(t);
                ResolvedType::Reference(TypeReference {
                    name: settled_result_name,
                    type_arguments: vec![awaited],
                    target: None,
                })
            })
            .collect();

        // Return Promise<PromiseSettledResult<T>[]>
        self.create_promise_type(ResolvedType::Tuple(settled_types))
    }

    /// Infers the type of Promise.any(promises)
    pub fn infer_promise_any(&self, argument_types: &[ResolvedType]) -> ResolvedType {
        // Promise.any<T>(values: Iterable<T | PromiseLike<T>>): Promise<Awaited<T>>
        // Same as race in terms of type
        self.infer_promise_race(argument_types)
    }

    /// Checks if a type is thenable (has a `then` method)
    pub fn is_thenable(&self, ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Reference(type_ref) => {
                type_ref.name == self.promise_name || type_ref.name == self.promise_like_name
            }
            ResolvedType::Object(obj) => self.has_then_method(obj),
            ResolvedType::Any => true,
            _ => false,
        }
    }

    /// Creates the type for the Promise constructor executor callback
    /// (resolve: (value: T) => void, reject: (reason?: any) => void) => void
    pub fn create_executor_type(&self, resolve_type: ResolvedType) -> ResolvedType {
        let resolve_name = self.interner.intern("resolve");
        let reject_name = self.interner.intern("reject");
        let value_name = self.interner.intern("value");
        let reason_name = self.interner.intern("reason");

        let resolve_param = ParameterType {
            name: resolve_name,
            ty: Box::new(ResolvedType::Function(FunctionType {
                type_parameters: Vec::new(),
                parameters: vec![ParameterType {
                    name: value_name,
                    ty: Box::new(resolve_type),
                    optional: false,
                    rest: false,
                }],
                return_type: Box::new(ResolvedType::Void),
            })),
            optional: false,
            rest: false,
        };

        let reject_param = ParameterType {
            name: reject_name,
            ty: Box::new(ResolvedType::Function(FunctionType {
                type_parameters: Vec::new(),
                parameters: vec![ParameterType {
                    name: reason_name,
                    ty: Box::new(ResolvedType::Any),
                    optional: true,
                    rest: false,
                }],
                return_type: Box::new(ResolvedType::Void),
            })),
            optional: false,
            rest: false,
        };

        ResolvedType::Function(FunctionType {
            type_parameters: Vec::new(),
            parameters: vec![resolve_param, reject_param],
            return_type: Box::new(ResolvedType::Void),
        })
    }
}

/// Checks if an expression is a call to a Promise combinator method
pub fn is_promise_combinator_call(method_name: &str) -> bool {
    matches!(method_name, "all" | "race" | "allSettled" | "any" | "resolve" | "reject")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_checker_creation() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        assert_eq!(
            interner.lookup(checker.promise_name).unwrap_or_default(),
            "Promise"
        );
    }

    #[test]
    fn test_create_promise_type() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let promise_number = checker.create_promise_type(ResolvedType::Number);

        assert!(matches!(promise_number, ResolvedType::Reference(_)));
        if let ResolvedType::Reference(ref_type) = promise_number {
            let name = interner.lookup(ref_type.name).unwrap_or_default();
            assert_eq!(name, "Promise");
            assert_eq!(ref_type.type_arguments.len(), 1);
            assert!(matches!(ref_type.type_arguments[0], ResolvedType::Number));
        }
    }

    #[test]
    fn test_is_promise_like() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let promise_type = checker.create_promise_type(ResolvedType::String);
        assert!(checker.is_promise_like(&promise_type));

        let string_type = ResolvedType::String;
        assert!(!checker.is_promise_like(&string_type));

        assert!(checker.is_promise_like(&ResolvedType::Any));
    }

    #[test]
    fn test_get_awaited_type_non_promise() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let string_type = ResolvedType::String;
        let awaited = checker.get_awaited_type(&string_type);

        assert!(matches!(awaited, ResolvedType::String));
    }

    #[test]
    fn test_get_awaited_type_promise() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let promise_number = checker.create_promise_type(ResolvedType::Number);
        let awaited = checker.get_awaited_type(&promise_number);

        assert!(matches!(awaited, ResolvedType::Number));
    }

    #[test]
    fn test_get_awaited_type_nested_promise() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        // Promise<Promise<string>>
        let inner_promise = checker.create_promise_type(ResolvedType::String);
        let outer_promise = checker.create_promise_type(inner_promise);

        let awaited = checker.get_awaited_type(&outer_promise);

        // Should unwrap to string
        assert!(matches!(awaited, ResolvedType::String));
    }

    #[test]
    fn test_infer_promise_all() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let promise_a = checker.create_promise_type(ResolvedType::Number);
        let promise_b = checker.create_promise_type(ResolvedType::String);

        let result = checker.infer_promise_all(&[promise_a, promise_b]);

        // Should be Promise<[number, string]>
        assert!(matches!(result, ResolvedType::Reference(_)));
        if let ResolvedType::Reference(ref_type) = result {
            if let Some(ResolvedType::Tuple(elems)) = ref_type.type_arguments.first() {
                assert_eq!(elems.len(), 2);
                assert!(matches!(elems[0], ResolvedType::Number));
                assert!(matches!(elems[1], ResolvedType::String));
            }
        }
    }

    #[test]
    fn test_infer_promise_race() {
        let interner = StringInterner::new();
        let checker = PromiseChecker::new(&interner);

        let promise_a = checker.create_promise_type(ResolvedType::Number);
        let promise_b = checker.create_promise_type(ResolvedType::String);

        let result = checker.infer_promise_race(&[promise_a, promise_b]);

        // Should be Promise<number | string>
        assert!(matches!(result, ResolvedType::Reference(_)));
    }

    #[test]
    fn test_is_promise_combinator_call() {
        assert!(is_promise_combinator_call("all"));
        assert!(is_promise_combinator_call("race"));
        assert!(is_promise_combinator_call("allSettled"));
        assert!(is_promise_combinator_call("any"));
        assert!(!is_promise_combinator_call("then"));
        assert!(!is_promise_combinator_call("catch"));
    }
}
