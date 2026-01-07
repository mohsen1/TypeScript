//! Type database abstraction for the solver.
//!
//! This trait isolates solver logic from concrete storage so we can
//! swap in a query system (e.g., Salsa) without touching core logic.

use crate::interner::Atom;
use crate::solver::intern::TypeInterner;
use crate::solver::types::{
    CallableShape, FunctionShape, ObjectShape, PropertyInfo, SymbolRef, TupleElement, TypeId,
    TypeKey,
};

/// Query interface for the solver.
///
/// This keeps solver components generic and prevents them from reaching
/// into concrete storage structures directly.
pub trait TypeDatabase {
    fn intern(&self, key: TypeKey) -> TypeId;
    fn lookup(&self, id: TypeId) -> Option<TypeKey>;
    fn intern_string(&self, s: &str) -> Atom;
    fn resolve_atom(&self, atom: Atom) -> String;

    fn literal_string(&self, value: &str) -> TypeId;
    fn literal_number(&self, value: f64) -> TypeId;
    fn literal_boolean(&self, value: bool) -> TypeId;
    fn literal_bigint(&self, value: &str) -> TypeId;
    fn literal_bigint_with_sign(&self, negative: bool, digits: &str) -> TypeId;

    fn union(&self, members: Vec<TypeId>) -> TypeId;
    fn intersection(&self, members: Vec<TypeId>) -> TypeId;
    fn array(&self, element: TypeId) -> TypeId;
    fn tuple(&self, elements: Vec<TupleElement>) -> TypeId;
    fn object(&self, properties: Vec<PropertyInfo>) -> TypeId;
    fn object_with_index(&self, shape: ObjectShape) -> TypeId;
    fn function(&self, shape: FunctionShape) -> TypeId;
    fn callable(&self, shape: CallableShape) -> TypeId;
    fn reference(&self, symbol: SymbolRef) -> TypeId;
    fn application(&self, base: TypeId, args: Vec<TypeId>) -> TypeId;
}

impl TypeDatabase for TypeInterner {
    fn intern(&self, key: TypeKey) -> TypeId {
        TypeInterner::intern(self, key)
    }

    fn lookup(&self, id: TypeId) -> Option<TypeKey> {
        TypeInterner::lookup(self, id)
    }

    fn intern_string(&self, s: &str) -> Atom {
        TypeInterner::intern_string(self, s)
    }

    fn resolve_atom(&self, atom: Atom) -> String {
        TypeInterner::resolve_atom(self, atom)
    }

    fn literal_string(&self, value: &str) -> TypeId {
        TypeInterner::literal_string(self, value)
    }

    fn literal_number(&self, value: f64) -> TypeId {
        TypeInterner::literal_number(self, value)
    }

    fn literal_boolean(&self, value: bool) -> TypeId {
        TypeInterner::literal_boolean(self, value)
    }

    fn literal_bigint(&self, value: &str) -> TypeId {
        TypeInterner::literal_bigint(self, value)
    }

    fn literal_bigint_with_sign(&self, negative: bool, digits: &str) -> TypeId {
        TypeInterner::literal_bigint_with_sign(self, negative, digits)
    }

    fn union(&self, members: Vec<TypeId>) -> TypeId {
        TypeInterner::union(self, members)
    }

    fn intersection(&self, members: Vec<TypeId>) -> TypeId {
        TypeInterner::intersection(self, members)
    }

    fn array(&self, element: TypeId) -> TypeId {
        TypeInterner::array(self, element)
    }

    fn tuple(&self, elements: Vec<TupleElement>) -> TypeId {
        TypeInterner::tuple(self, elements)
    }

    fn object(&self, properties: Vec<PropertyInfo>) -> TypeId {
        TypeInterner::object(self, properties)
    }

    fn object_with_index(&self, shape: ObjectShape) -> TypeId {
        TypeInterner::object_with_index(self, shape)
    }

    fn function(&self, shape: FunctionShape) -> TypeId {
        TypeInterner::function(self, shape)
    }

    fn callable(&self, shape: CallableShape) -> TypeId {
        TypeInterner::callable(self, shape)
    }

    fn reference(&self, symbol: SymbolRef) -> TypeId {
        TypeInterner::reference(self, symbol)
    }

    fn application(&self, base: TypeId, args: Vec<TypeId>) -> TypeId {
        TypeInterner::application(self, base, args)
    }
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
