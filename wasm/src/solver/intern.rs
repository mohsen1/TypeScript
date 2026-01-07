//! Type interning for structural deduplication.
//!
//! This module implements the type interning engine that converts
//! TypeKey structures into lightweight TypeId handles.
//!
//! Benefits:
//! - O(1) type equality (just compare TypeId values)
//! - Memory efficient (each unique structure stored once)
//! - Cache-friendly (work with u32 arrays instead of heap objects)

use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use rustc_hash::{FxHashMap, FxHasher};
use crate::solver::types::*;
use crate::interner::{Atom, Interner};

const SHARD_BITS: u32 = 6;
const SHARD_COUNT: usize = 1 << SHARD_BITS; // 64 shards
const SHARD_MASK: u32 = (SHARD_COUNT as u32) - 1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PrimitiveClass {
    String,
    Number,
    Boolean,
    Bigint,
    Symbol,
    Null,
    Undefined,
}

struct TypeShard {
    key_to_index: RwLock<FxHashMap<TypeKey, u32>>,
    index_to_key: RwLock<Vec<TypeKey>>,
}

impl TypeShard {
    fn new() -> Self {
        TypeShard {
            key_to_index: RwLock::new(FxHashMap::default()),
            index_to_key: RwLock::new(Vec::new()),
        }
    }
}

/// Type interning table.
/// Thread-safe via RwLock for concurrent access.
pub struct TypeInterner {
    /// Sharded storage for user-defined types
    shards: [TypeShard; SHARD_COUNT],
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

        TypeInterner {
            shards: std::array::from_fn(|_| TypeShard::new()),
            string_interner: RwLock::new(string_interner),
        }
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

    /// Intern a type key and return its TypeId.
    /// If the key already exists, returns the existing TypeId.
    /// Otherwise, creates a new TypeId and stores the key.
    pub fn intern(&self, key: TypeKey) -> TypeId {
        if let Some(id) = self.get_intrinsic_id(&key) {
            return id;
        }

        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        let shard_idx = (hasher.finish() as usize) & (SHARD_COUNT - 1);
        let shard = &self.shards[shard_idx];

        {
            let map = shard.key_to_index.read().unwrap();
            if let Some(&local_index) = map.get(&key) {
                return self.make_id(local_index, shard_idx as u32);
            }
        }

        let mut map = shard.key_to_index.write().unwrap();
        let mut storage = shard.index_to_key.write().unwrap();

        if let Some(&local_index) = map.get(&key) {
            return self.make_id(local_index, shard_idx as u32);
        }

        let local_index = storage.len() as u32;
        if local_index > (u32::MAX >> SHARD_BITS) {
            panic!("TypeInterner shard {} overflow", shard_idx);
        }

        storage.push(key.clone());
        map.insert(key, local_index);

        self.make_id(local_index, shard_idx as u32)
    }

    /// Look up the TypeKey for a given TypeId
    pub fn lookup(&self, id: TypeId) -> Option<TypeKey> {
        if id.is_intrinsic() || id.is_error() {
            return self.get_intrinsic_key(id);
        }

        let raw_val = id.0.checked_sub(TypeId::FIRST_USER)?;
        let shard_idx = (raw_val & SHARD_MASK) as usize;
        let local_index = raw_val >> SHARD_BITS;

        let shard = self.shards.get(shard_idx)?;
        let storage = shard.index_to_key.read().unwrap();
        storage.get(local_index as usize).cloned()
    }

    /// Get the number of interned types
    pub fn len(&self) -> usize {
        let mut total = TypeId::FIRST_USER as usize;
        for shard in &self.shards {
            total += shard.index_to_key.read().unwrap().len();
        }
        total
    }

    /// Check if the interner is empty (only has intrinsics)
    pub fn is_empty(&self) -> bool {
        self.len() <= TypeId::FIRST_USER as usize
    }

    #[inline]
    fn make_id(&self, local_index: u32, shard_idx: u32) -> TypeId {
        let raw_val = (local_index << SHARD_BITS) | (shard_idx & SHARD_MASK);
        TypeId(TypeId::FIRST_USER + raw_val)
    }

    fn get_intrinsic_id(&self, key: &TypeKey) -> Option<TypeId> {
        match key {
            TypeKey::Intrinsic(kind) => Some(kind.to_type_id()),
            TypeKey::Error => Some(TypeId::ERROR),
            _ => None,
        }
    }

    fn get_intrinsic_key(&self, id: TypeId) -> Option<TypeKey> {
        match id {
            TypeId::NONE => Some(TypeKey::Error),
            TypeId::ERROR => Some(TypeKey::Error),
            TypeId::NEVER => Some(TypeKey::Intrinsic(IntrinsicKind::Never)),
            TypeId::UNKNOWN => Some(TypeKey::Intrinsic(IntrinsicKind::Unknown)),
            TypeId::ANY => Some(TypeKey::Intrinsic(IntrinsicKind::Any)),
            TypeId::VOID => Some(TypeKey::Intrinsic(IntrinsicKind::Void)),
            TypeId::UNDEFINED => Some(TypeKey::Intrinsic(IntrinsicKind::Undefined)),
            TypeId::NULL => Some(TypeKey::Intrinsic(IntrinsicKind::Null)),
            TypeId::BOOLEAN => Some(TypeKey::Intrinsic(IntrinsicKind::Boolean)),
            TypeId::NUMBER => Some(TypeKey::Intrinsic(IntrinsicKind::Number)),
            TypeId::STRING => Some(TypeKey::Intrinsic(IntrinsicKind::String)),
            TypeId::BIGINT => Some(TypeKey::Intrinsic(IntrinsicKind::Bigint)),
            TypeId::SYMBOL => Some(TypeKey::Intrinsic(IntrinsicKind::Symbol)),
            TypeId::OBJECT => Some(TypeKey::Intrinsic(IntrinsicKind::Object)),
            _ => None,
        }
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

    /// Intern a literal bigint type
    pub fn literal_bigint(&self, value: &str) -> TypeId {
        let atom = self.intern_string(value);
        self.intern(TypeKey::Literal(LiteralValue::BigInt(atom)))
    }

    /// Intern a literal bigint type, allowing a sign prefix without extra clones.
    pub fn literal_bigint_with_sign(&self, negative: bool, digits: &str) -> TypeId {
        if !negative {
            return self.literal_bigint(digits);
        }

        let mut value = String::with_capacity(digits.len() + 1);
        value.push('-');
        value.push_str(digits);
        let atom = self.string_interner.write().unwrap().intern_owned(value);
        self.intern(TypeKey::Literal(LiteralValue::BigInt(atom)))
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
        if flat.contains(&TypeId::ERROR) {
            return TypeId::ERROR;
        }
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
        if flat.contains(&TypeId::ERROR) {
            return TypeId::ERROR;
        }
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
        // If any member is `any`, the intersection is `any`
        if flat.contains(&TypeId::ANY) {
            return TypeId::ANY;
        }
        // Remove `unknown` from intersections (identity element)
        flat.retain(|&id| id != TypeId::UNKNOWN);
        if self.intersection_has_disjoint_primitives(&flat) {
            return TypeId::NEVER;
        }
        if flat.is_empty() {
            return TypeId::UNKNOWN;
        }
        if flat.len() == 1 {
            return flat[0];
        }

        self.intern(TypeKey::Intersection(flat))
    }

    fn intersection_has_disjoint_primitives(&self, members: &[TypeId]) -> bool {
        let mut class: Option<PrimitiveClass> = None;

        for &member in members {
            let Some(member_class) = self.primitive_class_for(member) else {
                continue;
            };
            if let Some(existing) = class {
                if existing != member_class {
                    return true;
                }
            } else {
                class = Some(member_class);
            }
        }

        false
    }

    fn primitive_class_for(&self, type_id: TypeId) -> Option<PrimitiveClass> {
        match type_id {
            TypeId::STRING => return Some(PrimitiveClass::String),
            TypeId::NUMBER => return Some(PrimitiveClass::Number),
            TypeId::BOOLEAN => return Some(PrimitiveClass::Boolean),
            TypeId::BIGINT => return Some(PrimitiveClass::Bigint),
            TypeId::SYMBOL => return Some(PrimitiveClass::Symbol),
            TypeId::NULL => return Some(PrimitiveClass::Null),
            TypeId::UNDEFINED | TypeId::VOID => return Some(PrimitiveClass::Undefined),
            _ => {}
        }

        let key = self.lookup(type_id)?;

        match key {
            TypeKey::Intrinsic(kind) => match kind {
                IntrinsicKind::String => Some(PrimitiveClass::String),
                IntrinsicKind::Number => Some(PrimitiveClass::Number),
                IntrinsicKind::Boolean => Some(PrimitiveClass::Boolean),
                IntrinsicKind::Bigint => Some(PrimitiveClass::Bigint),
                IntrinsicKind::Symbol => Some(PrimitiveClass::Symbol),
                IntrinsicKind::Null => Some(PrimitiveClass::Null),
                IntrinsicKind::Undefined | IntrinsicKind::Void => Some(PrimitiveClass::Undefined),
                _ => None,
            },
            TypeKey::Literal(literal) => match literal {
                LiteralValue::String(_) => Some(PrimitiveClass::String),
                LiteralValue::Number(_) => Some(PrimitiveClass::Number),
                LiteralValue::Boolean(_) => Some(PrimitiveClass::Boolean),
                LiteralValue::BigInt(_) => Some(PrimitiveClass::Bigint),
            },
            TypeKey::UniqueSymbol(_) => Some(PrimitiveClass::Symbol),
            TypeKey::TemplateLiteral(_) => Some(PrimitiveClass::String),
            _ => None,
        }
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

    /// Intern a generic type application
    pub fn application(&self, base: TypeId, args: Vec<TypeId>) -> TypeId {
        self.intern(TypeKey::Application(TypeApplication { base, args }))
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
