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

/// Type interning table.
/// Thread-safe via RwLock for concurrent access.
pub struct TypeInterner {
    /// Map from TypeKey to TypeId for deduplication
    key_to_id: RwLock<HashMap<TypeKey, TypeId>>,
    /// Reverse map from TypeId to TypeKey for lookup
    id_to_key: RwLock<Vec<TypeKey>>,
    /// Next available TypeId
    next_id: RwLock<u32>,
}

impl TypeInterner {
    /// Create a new type interner with pre-registered intrinsics
    pub fn new() -> Self {
        let interner = TypeInterner {
            key_to_id: RwLock::new(HashMap::new()),
            id_to_key: RwLock::new(Vec::new()),
            next_id: RwLock::new(TypeId::FIRST_USER),
        };

        // Pre-register intrinsic types
        interner.register_intrinsics();
        interner
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
        self.intern(TypeKey::Literal(LiteralValue::String(value.into())))
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

    /// Intern a function type
    pub fn function(&self, shape: FunctionShape) -> TypeId {
        self.intern(TypeKey::Function(shape))
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
mod tests {
    use super::*;

    #[test]
    fn test_interner_intrinsics() {
        let interner = TypeInterner::new();

        // Intrinsics should be pre-registered
        assert!(interner.lookup(TypeId::STRING).is_some());
        assert!(interner.lookup(TypeId::NUMBER).is_some());
        assert!(interner.lookup(TypeId::ANY).is_some());
    }

    #[test]
    fn test_interner_deduplication() {
        let interner = TypeInterner::new();

        // Same structure should get same TypeId
        let id1 = interner.literal_string("hello");
        let id2 = interner.literal_string("hello");
        let id3 = interner.literal_string("world");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_interner_union_normalization() {
        let interner = TypeInterner::new();

        // Union with single member should return that member
        let single = interner.union(vec![TypeId::STRING]);
        assert_eq!(single, TypeId::STRING);

        // Union with `any` should be `any`
        let with_any = interner.union(vec![TypeId::STRING, TypeId::ANY]);
        assert_eq!(with_any, TypeId::ANY);

        // Union with `never` should exclude `never`
        let with_never = interner.union(vec![TypeId::STRING, TypeId::NEVER]);
        assert_eq!(with_never, TypeId::STRING);

        // Empty union is `never`
        let empty = interner.union(vec![]);
        assert_eq!(empty, TypeId::NEVER);
    }

    #[test]
    fn test_interner_intersection_normalization() {
        let interner = TypeInterner::new();

        // Intersection with single member should return that member
        let single = interner.intersection(vec![TypeId::STRING]);
        assert_eq!(single, TypeId::STRING);

        // Intersection with `never` should be `never`
        let with_never = interner.intersection(vec![TypeId::STRING, TypeId::NEVER]);
        assert_eq!(with_never, TypeId::NEVER);

        // Empty intersection is `unknown`
        let empty = interner.intersection(vec![]);
        assert_eq!(empty, TypeId::UNKNOWN);
    }

    #[test]
    fn test_interner_object_sorting() {
        let interner = TypeInterner::new();
        use std::sync::Arc;

        // Properties in different order should produce same TypeId
        let props1 = vec![
            PropertyInfo { name: Arc::from("a"), type_id: TypeId::STRING, optional: false, readonly: false },
            PropertyInfo { name: Arc::from("b"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        ];
        let props2 = vec![
            PropertyInfo { name: Arc::from("b"), type_id: TypeId::NUMBER, optional: false, readonly: false },
            PropertyInfo { name: Arc::from("a"), type_id: TypeId::STRING, optional: false, readonly: false },
        ];

        let id1 = interner.object(props1);
        let id2 = interner.object(props2);

        assert_eq!(id1, id2);
    }
}
