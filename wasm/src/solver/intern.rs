//! Type interning for structural deduplication.
//!
//! This module implements the type interning engine that converts
//! TypeKey structures into lightweight TypeId handles.
//!
//! Benefits:
//! - O(1) type equality (just compare TypeId values)
//! - Memory efficient (each unique structure stored once)
//! - Cache-friendly (work with u32 arrays instead of heap objects)

use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};
use crate::solver::types::*;
use crate::interner::{Atom, ShardedInterner};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LiteralDomain {
    String,
    Number,
    Boolean,
    Bigint,
}

#[derive(Clone, Debug)]
struct LiteralSet {
    domain: LiteralDomain,
    values: FxHashSet<LiteralValue>,
}

impl LiteralSet {
    fn from_literal(literal: LiteralValue) -> Self {
        let domain = literal_domain(&literal);
        let mut values = FxHashSet::default();
        values.insert(literal);
        LiteralSet { domain, values }
    }
}

fn literal_domain(literal: &LiteralValue) -> LiteralDomain {
    match literal {
        LiteralValue::String(_) => LiteralDomain::String,
        LiteralValue::Number(_) => LiteralDomain::Number,
        LiteralValue::Boolean(_) => LiteralDomain::Boolean,
        LiteralValue::BigInt(_) => LiteralDomain::Bigint,
    }
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

struct SliceInterner<T> {
    items: Vec<Arc<[T]>>,
    map: FxHashMap<Arc<[T]>, u32>,
}

impl<T> SliceInterner<T>
where
    T: Eq + Hash,
{
    fn new() -> Self {
        let empty: Arc<[T]> = Arc::from(Vec::new());
        let mut map = FxHashMap::default();
        map.insert(empty.clone(), 0);
        SliceInterner {
            items: vec![empty],
            map,
        }
    }

    fn intern(&mut self, items: Vec<T>) -> u32 {
        if items.is_empty() {
            return 0;
        }

        if let Some(&id) = self.map.get(items.as_slice()) {
            return id;
        }

        let arc: Arc<[T]> = items.into();
        let id = self.items.len() as u32;
        self.items.push(arc.clone());
        self.map.insert(arc, id);
        id
    }

    fn get(&self, id: u32) -> Option<Arc<[T]>> {
        self.items.get(id as usize).cloned()
    }

    fn empty(&self) -> Arc<[T]> {
        self.items[0].clone()
    }
}

struct ValueInterner<T> {
    items: Vec<Arc<T>>,
    map: FxHashMap<Arc<T>, u32>,
}

impl<T> ValueInterner<T>
where
    T: Eq + Hash,
{
    fn new() -> Self {
        ValueInterner {
            items: Vec::new(),
            map: FxHashMap::default(),
        }
    }

    fn intern(&mut self, value: T) -> u32 {
        if let Some(&id) = self.map.get(&value) {
            return id;
        }

        let arc = Arc::new(value);
        let id = self.items.len() as u32;
        self.items.push(arc.clone());
        self.map.insert(arc, id);
        id
    }

    fn get(&self, id: u32) -> Option<Arc<T>> {
        self.items.get(id as usize).cloned()
    }
}

/// Type interning table.
/// Thread-safe via RwLock for concurrent access.
pub struct TypeInterner {
    /// Sharded storage for user-defined types
    shards: [TypeShard; SHARD_COUNT],
    /// String interner for property names and string literals
    /// Thread-safe for concurrent access during type construction
    pub string_interner: ShardedInterner,
    type_lists: RwLock<SliceInterner<TypeId>>,
    tuple_lists: RwLock<SliceInterner<TupleElement>>,
    template_lists: RwLock<SliceInterner<TemplateSpan>>,
    object_shapes: RwLock<ValueInterner<ObjectShape>>,
    function_shapes: RwLock<ValueInterner<FunctionShape>>,
    callable_shapes: RwLock<ValueInterner<CallableShape>>,
    applications: RwLock<ValueInterner<TypeApplication>>,
}

impl TypeInterner {
    /// Create a new type interner with pre-registered intrinsics
    pub fn new() -> Self {
        TypeInterner {
            shards: std::array::from_fn(|_| TypeShard::new()),
            string_interner: {
                let interner = ShardedInterner::new();
                interner.intern_common();
                interner
            },
            type_lists: RwLock::new(SliceInterner::new()),
            tuple_lists: RwLock::new(SliceInterner::new()),
            template_lists: RwLock::new(SliceInterner::new()),
            object_shapes: RwLock::new(ValueInterner::new()),
            function_shapes: RwLock::new(ValueInterner::new()),
            callable_shapes: RwLock::new(ValueInterner::new()),
            applications: RwLock::new(ValueInterner::new()),
        }
    }

    /// Intern a string into an Atom.
    /// This is used when constructing types with property names or string literals.
    pub fn intern_string(&self, s: &str) -> Atom {
        self.string_interner.intern(s)
    }

    /// Resolve an Atom back to its string value.
    /// This is used when formatting types for error messages.
    pub fn resolve_atom(&self, atom: Atom) -> String {
        self.string_interner.resolve(atom)
    }

    pub fn type_list(&self, id: TypeListId) -> Arc<[TypeId]> {
        let lists = self.type_lists.read().unwrap();
        lists.get(id.0).unwrap_or_else(|| lists.empty())
    }

    pub fn tuple_list(&self, id: TupleListId) -> Arc<[TupleElement]> {
        let lists = self.tuple_lists.read().unwrap();
        lists.get(id.0).unwrap_or_else(|| lists.empty())
    }

    pub fn template_list(&self, id: TemplateLiteralId) -> Arc<[TemplateSpan]> {
        let lists = self.template_lists.read().unwrap();
        lists.get(id.0).unwrap_or_else(|| lists.empty())
    }

    pub fn object_shape(&self, id: ObjectShapeId) -> Arc<ObjectShape> {
        self.object_shapes
            .read()
            .unwrap()
            .get(id.0)
            .unwrap_or_else(|| Arc::new(ObjectShape {
                properties: Vec::new(),
                string_index: None,
                number_index: None,
            }))
    }

    pub fn function_shape(&self, id: FunctionShapeId) -> Arc<FunctionShape> {
        self.function_shapes
            .read()
            .unwrap()
            .get(id.0)
            .unwrap_or_else(|| Arc::new(FunctionShape {
                type_params: Vec::new(),
                params: Vec::new(),
                this_type: None,
                return_type: TypeId::ERROR,
                type_predicate: None,
                is_constructor: false,
            }))
    }

    pub fn callable_shape(&self, id: CallableShapeId) -> Arc<CallableShape> {
        self.callable_shapes
            .read()
            .unwrap()
            .get(id.0)
            .unwrap_or_else(|| Arc::new(CallableShape {
                call_signatures: Vec::new(),
                construct_signatures: Vec::new(),
                properties: Vec::new(),
            }))
    }

    pub fn type_application(&self, id: TypeApplicationId) -> Arc<TypeApplication> {
        self.applications
            .read()
            .unwrap()
            .get(id.0)
            .unwrap_or_else(|| Arc::new(TypeApplication {
                base: TypeId::ERROR,
                args: Vec::new(),
            }))
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

    fn intern_type_list(&self, members: Vec<TypeId>) -> TypeListId {
        let mut lists = self.type_lists.write().unwrap();
        TypeListId(lists.intern(members))
    }

    fn intern_tuple_list(&self, elements: Vec<TupleElement>) -> TupleListId {
        let mut lists = self.tuple_lists.write().unwrap();
        TupleListId(lists.intern(elements))
    }

    fn intern_template_list(&self, spans: Vec<TemplateSpan>) -> TemplateLiteralId {
        let mut lists = self.template_lists.write().unwrap();
        TemplateLiteralId(lists.intern(spans))
    }

    fn intern_object_shape(&self, shape: ObjectShape) -> ObjectShapeId {
        let mut shapes = self.object_shapes.write().unwrap();
        ObjectShapeId(shapes.intern(shape))
    }

    fn intern_function_shape(&self, shape: FunctionShape) -> FunctionShapeId {
        let mut shapes = self.function_shapes.write().unwrap();
        FunctionShapeId(shapes.intern(shape))
    }

    fn intern_callable_shape(&self, shape: CallableShape) -> CallableShapeId {
        let mut shapes = self.callable_shapes.write().unwrap();
        CallableShapeId(shapes.intern(shape))
    }

    fn intern_application(&self, application: TypeApplication) -> TypeApplicationId {
        let mut apps = self.applications.write().unwrap();
        TypeApplicationId(apps.intern(application))
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
        let atom = self.string_interner.intern_owned(value);
        self.intern(TypeKey::Literal(LiteralValue::BigInt(atom)))
    }

    /// Intern a union type, normalizing and deduplicating members
    pub fn union(&self, mut members: Vec<TypeId>) -> TypeId {
        // Flatten nested unions
        let mut flat: Vec<TypeId> = Vec::new();
        for member in members.drain(..) {
            if let Some(TypeKey::Union(inner)) = self.lookup(member) {
                let members = self.type_list(inner);
                flat.extend(members.iter().copied());
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

        let list_id = self.intern_type_list(flat);
        self.intern(TypeKey::Union(list_id))
    }

    /// Intern an intersection type, normalizing and deduplicating members
    pub fn intersection(&self, mut members: Vec<TypeId>) -> TypeId {
        // Flatten nested intersections
        let mut flat: Vec<TypeId> = Vec::new();
        for member in members.drain(..) {
            if let Some(TypeKey::Intersection(inner)) = self.lookup(member) {
                let members = self.type_list(inner);
                flat.extend(members.iter().copied());
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
        if self.intersection_has_disjoint_object_literals(&flat) {
            return TypeId::NEVER;
        }
        if flat.is_empty() {
            return TypeId::UNKNOWN;
        }
        if flat.len() == 1 {
            return flat[0];
        }

        let list_id = self.intern_type_list(flat);
        self.intern(TypeKey::Intersection(list_id))
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

    fn intersection_has_disjoint_object_literals(&self, members: &[TypeId]) -> bool {
        let mut objects: Vec<Arc<ObjectShape>> = Vec::new();

        for &member in members {
            let Some(key) = self.lookup(member) else {
                continue;
            };
            match key {
                TypeKey::Object(shape_id) | TypeKey::ObjectWithIndex(shape_id) => {
                    objects.push(self.object_shape(shape_id));
                }
                _ => {}
            }
        }

        if objects.len() < 2 {
            return false;
        }

        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                if self.object_literals_disjoint(
                    objects[i].properties.as_slice(),
                    objects[j].properties.as_slice(),
                ) {
                    return true;
                }
            }
        }

        false
    }

    fn object_literals_disjoint(&self, left: &[PropertyInfo], right: &[PropertyInfo]) -> bool {
        let (small, large) = if left.len() <= right.len() {
            (left, right)
        } else {
            (right, left)
        };

        for prop in small {
            if prop.optional {
                continue;
            }
            let Some(left_set) = self.literal_set_from_type(prop.type_id) else {
                continue;
            };
            let Some(other) = Self::find_property(large, prop.name) else {
                continue;
            };
            if other.optional {
                continue;
            }
            let Some(right_set) = self.literal_set_from_type(other.type_id) else {
                continue;
            };
            if self.literal_sets_disjoint(&left_set, &right_set) {
                return true;
            }
        }

        false
    }

    fn literal_sets_disjoint(&self, left: &LiteralSet, right: &LiteralSet) -> bool {
        if left.domain != right.domain {
            return true;
        }
        !left.values.iter().any(|value| right.values.contains(value))
    }

    fn literal_set_from_type(&self, type_id: TypeId) -> Option<LiteralSet> {
        let key = self.lookup(type_id)?;
        match key {
            TypeKey::Literal(literal) => Some(LiteralSet::from_literal(literal)),
            TypeKey::Union(members) => {
                let members = self.type_list(members);
                let mut domain: Option<LiteralDomain> = None;
                let mut values = FxHashSet::default();
                for &member in members.iter() {
                    let Some(TypeKey::Literal(literal)) = self.lookup(member) else {
                        return None;
                    };
                    let literal_domain = literal_domain(&literal);
                    if let Some(existing) = domain {
                        if existing != literal_domain {
                            return None;
                        }
                    } else {
                        domain = Some(literal_domain);
                    }
                    values.insert(literal);
                }
                domain.map(|domain| LiteralSet { domain, values })
            }
            _ => None,
        }
    }

    fn find_property(props: &[PropertyInfo], name: Atom) -> Option<&PropertyInfo> {
        props
            .binary_search_by(|prop| prop.name.cmp(&name))
            .ok()
            .map(|idx| &props[idx])
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
        let list_id = self.intern_tuple_list(elements);
        self.intern(TypeKey::Tuple(list_id))
    }

    /// Intern an object type with properties
    pub fn object(&self, mut properties: Vec<PropertyInfo>) -> TypeId {
        // Sort by property name for consistent hashing
        properties.sort_by(|a, b| a.name.cmp(&b.name));
        let shape_id = self.intern_object_shape(ObjectShape {
            properties,
            string_index: None,
            number_index: None,
        });
        self.intern(TypeKey::Object(shape_id))
    }

    /// Intern an object type with index signatures
    pub fn object_with_index(&self, mut shape: ObjectShape) -> TypeId {
        // Sort properties by name for consistent hashing
        shape.properties.sort_by(|a, b| a.name.cmp(&b.name));
        let shape_id = self.intern_object_shape(shape);
        self.intern(TypeKey::ObjectWithIndex(shape_id))
    }

    /// Intern a function type
    pub fn function(&self, shape: FunctionShape) -> TypeId {
        let shape_id = self.intern_function_shape(shape);
        self.intern(TypeKey::Function(shape_id))
    }

    /// Intern a callable type with overloaded signatures
    pub fn callable(&self, shape: CallableShape) -> TypeId {
        let shape_id = self.intern_callable_shape(shape);
        self.intern(TypeKey::Callable(shape_id))
    }

    /// Intern a template literal type
    pub fn template_literal(&self, spans: Vec<TemplateSpan>) -> TypeId {
        let list_id = self.intern_template_list(spans);
        self.intern(TypeKey::TemplateLiteral(list_id))
    }

    /// Intern a type reference
    pub fn reference(&self, symbol: SymbolRef) -> TypeId {
        self.intern(TypeKey::Ref(symbol))
    }

    /// Intern a generic type application
    pub fn application(&self, base: TypeId, args: Vec<TypeId>) -> TypeId {
        let app_id = self.intern_application(TypeApplication { base, args });
        self.intern(TypeKey::Application(app_id))
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
