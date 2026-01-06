//! Type interning for structural deduplication.
//!
//! This module implements the type interning engine that converts
//! TypeKey structures into lightweight TypeId handles.
//!
//! Benefits:
//! - O(1) type equality (just compare TypeId values)
//! - Memory efficient (each unique structure stored once)
//! - Cache-friendly (work with u32 arrays instead of heap objects)

use std::collections::HashMap;
use std::sync::RwLock;
use crate::solver::types::*;
use crate::interner::{Atom, Interner};

/// Type interning table.
/// Thread-safe via RwLock for concurrent access.
pub struct TypeInterner {
    /// Map from TypeKey to TypeId for deduplication
    key_to_id: RwLock<HashMap<TypeKey, TypeId>>,
    /// Reverse map from TypeId to TypeKey for lookup
    id_to_key: RwLock<Vec<TypeKey>>,
    /// Next available TypeId
    next_id: RwLock<u32>,
    /// String interner for property names and string literals
    /// Thread-safe for concurrent access during type construction
    pub string_interner: RwLock<Interner>,
}

impl TypeInterner {
    /// Create a new type interner with pre-registered intrinsics
    pub fn new() -> Self {
        let mut string_interner = Interner::new();
        // Pre-intern common TypeScript identifiers for better performance
        string_interner.intern_common();

        let interner = TypeInterner {
            key_to_id: RwLock::new(HashMap::new()),
            id_to_key: RwLock::new(Vec::new()),
            next_id: RwLock::new(TypeId::FIRST_USER),
            string_interner: RwLock::new(string_interner),
        };

        // Pre-register intrinsic types
        interner.register_intrinsics();
        interner
    }

    /// Intern a string into an Atom.
    /// This is used when constructing types with property names or string literals.
    pub fn intern_string(&self, s: &str) -> Atom {
        self.string_interner.write().unwrap().intern(s)
    }

    /// Resolve an Atom back to its string value.
    /// This is used when formatting types for error messages.
    pub fn resolve_atom(&self, atom: Atom) -> String {
        self.string_interner.read().unwrap().resolve(atom).to_string()
    }

    /// Register all intrinsic types at their fixed positions
    fn register_intrinsics(&self) {
        let mut key_to_id = self.key_to_id.write().unwrap();
        let mut id_to_key = self.id_to_key.write().unwrap();

        // Ensure we have enough space for intrinsics
        id_to_key.resize(TypeId::FIRST_USER as usize, TypeKey::Error);

        let intrinsics = [
            (TypeId::NONE, TypeKey::Error),
            (TypeId::ERROR, TypeKey::Error),
            (TypeId::NEVER, TypeKey::Intrinsic(IntrinsicKind::Never)),
            (TypeId::UNKNOWN, TypeKey::Intrinsic(IntrinsicKind::Unknown)),
            (TypeId::ANY, TypeKey::Intrinsic(IntrinsicKind::Any)),
            (TypeId::VOID, TypeKey::Intrinsic(IntrinsicKind::Void)),
            (TypeId::UNDEFINED, TypeKey::Intrinsic(IntrinsicKind::Undefined)),
            (TypeId::NULL, TypeKey::Intrinsic(IntrinsicKind::Null)),
            (TypeId::BOOLEAN, TypeKey::Intrinsic(IntrinsicKind::Boolean)),
            (TypeId::NUMBER, TypeKey::Intrinsic(IntrinsicKind::Number)),
            (TypeId::STRING, TypeKey::Intrinsic(IntrinsicKind::String)),
            (TypeId::BIGINT, TypeKey::Intrinsic(IntrinsicKind::Bigint)),
            (TypeId::SYMBOL, TypeKey::Intrinsic(IntrinsicKind::Symbol)),
            (TypeId::OBJECT, TypeKey::Intrinsic(IntrinsicKind::Object)),
        ];

        for (id, key) in intrinsics {
            id_to_key[id.0 as usize] = key.clone();
            key_to_id.insert(key, id);
        }
    }

    /// Intern a type key and return its TypeId.
    /// If the key already exists, returns the existing TypeId.
    /// Otherwise, creates a new TypeId and stores the key.
    pub fn intern(&self, key: TypeKey) -> TypeId {
        // Fast path: check if already interned
        {
            let key_to_id = self.key_to_id.read().unwrap();
            if let Some(&id) = key_to_id.get(&key) {
                return id;
            }
        }

        // Slow path: need to insert
        let mut key_to_id = self.key_to_id.write().unwrap();
        let mut id_to_key = self.id_to_key.write().unwrap();
        let mut next_id = self.next_id.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(&id) = key_to_id.get(&key) {
            return id;
        }

        // Create new TypeId
        let id = TypeId(*next_id);
        *next_id += 1;

        // Store in both maps
        id_to_key.push(key.clone());
        key_to_id.insert(key, id);

        id
    }

    /// Look up the TypeKey for a given TypeId
    pub fn lookup(&self, id: TypeId) -> Option<TypeKey> {
        let id_to_key = self.id_to_key.read().unwrap();
        id_to_key.get(id.0 as usize).cloned()
    }

    /// Get the number of interned types
    pub fn len(&self) -> usize {
        self.id_to_key.read().unwrap().len()
    }

    /// Check if the interner is empty (only has intrinsics)
    pub fn is_empty(&self) -> bool {
        self.len() <= TypeId::FIRST_USER as usize
    }

    // =========================================================================
    // Convenience methods for common type constructions
    // =========================================================================

    /// Intern an intrinsic type
    pub fn intrinsic(&self, kind: IntrinsicKind) -> TypeId {
        kind.to_type_id()
    }

    /// Intern a literal string type
    pub fn literal_string(&self, value: &str) -> TypeId {
        let atom = self.intern_string(value);
        self.intern(TypeKey::Literal(LiteralValue::String(atom)))
    }

    /// Intern a literal number type
    pub fn literal_number(&self, value: f64) -> TypeId {
        self.intern(TypeKey::Literal(LiteralValue::Number(OrderedFloat(value))))
    }

    /// Intern a literal boolean type
    pub fn literal_boolean(&self, value: bool) -> TypeId {
        self.intern(TypeKey::Literal(LiteralValue::Boolean(value)))
    }

    /// Intern a union type, normalizing and deduplicating members
    pub fn union(&self, mut members: Vec<TypeId>) -> TypeId {
        // Flatten nested unions
        let mut flat: Vec<TypeId> = Vec::new();
        for member in members.drain(..) {
            if let Some(TypeKey::Union(inner)) = self.lookup(member) {
                flat.extend(inner);
            } else {
                flat.push(member);
            }
        }

        // Deduplicate and sort for consistent hashing
        flat.sort_by_key(|id| id.0);
        flat.dedup();

        // Handle special cases
        if flat.is_empty() {
            return TypeId::NEVER;
        }
        if flat.len() == 1 {
            return flat[0];
        }
        // If any member is `any`, the union is `any`
        if flat.contains(&TypeId::ANY) {
            return TypeId::ANY;
        }
        // If any member is `unknown`, the union is `unknown`
        if flat.contains(&TypeId::UNKNOWN) {
            return TypeId::UNKNOWN;
        }
        // Remove `never` from unions
        flat.retain(|&id| id != TypeId::NEVER);
        if flat.is_empty() {
            return TypeId::NEVER;
        }
        if flat.len() == 1 {
            return flat[0];
        }

        self.intern(TypeKey::Union(flat))
    }

    /// Intern an intersection type, normalizing and deduplicating members
    pub fn intersection(&self, mut members: Vec<TypeId>) -> TypeId {
        // Flatten nested intersections
        let mut flat: Vec<TypeId> = Vec::new();
        for member in members.drain(..) {
            if let Some(TypeKey::Intersection(inner)) = self.lookup(member) {
                flat.extend(inner);
            } else {
                flat.push(member);
            }
        }

        // Deduplicate and sort for consistent hashing
        flat.sort_by_key(|id| id.0);
        flat.dedup();

        // Handle special cases
        if flat.is_empty() {
            return TypeId::UNKNOWN;
        }
        if flat.len() == 1 {
            return flat[0];
        }
        // If any member is `never`, the intersection is `never`
        if flat.contains(&TypeId::NEVER) {
            return TypeId::NEVER;
        }
        // Remove `any` and `unknown` from intersections (they don't constrain)
        flat.retain(|&id| id != TypeId::ANY && id != TypeId::UNKNOWN);
        if flat.is_empty() {
            return TypeId::UNKNOWN;
        }
        if flat.len() == 1 {
            return flat[0];
        }

        self.intern(TypeKey::Intersection(flat))
    }

    /// Intern an array type
    pub fn array(&self, element: TypeId) -> TypeId {
        self.intern(TypeKey::Array(element))
    }

    /// Intern a tuple type
    pub fn tuple(&self, elements: Vec<TupleElement>) -> TypeId {
        self.intern(TypeKey::Tuple(elements))
    }

    /// Intern an object type with properties
    pub fn object(&self, mut properties: Vec<PropertyInfo>) -> TypeId {
        // Sort by property name for consistent hashing
        properties.sort_by(|a, b| a.name.cmp(&b.name));
        self.intern(TypeKey::Object(properties))
    }

    /// Intern an object type with index signatures
    pub fn object_with_index(&self, mut shape: ObjectShape) -> TypeId {
        // Sort properties by name for consistent hashing
        shape.properties.sort_by(|a, b| a.name.cmp(&b.name));
        self.intern(TypeKey::ObjectWithIndex(shape))
    }

    /// Intern a function type
    pub fn function(&self, shape: FunctionShape) -> TypeId {
        self.intern(TypeKey::Function(shape))
    }

    /// Intern a callable type with overloaded signatures
    pub fn callable(&self, shape: CallableShape) -> TypeId {
        self.intern(TypeKey::Callable(shape))
    }

    /// Intern a type reference
    pub fn reference(&self, symbol: SymbolRef) -> TypeId {
        self.intern(TypeKey::Ref(symbol))
    }
}

impl Default for TypeInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "intern_tests.rs"]
mod tests;
