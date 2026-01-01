# TypeScript Checker Implementation Reference

This document maps the type-theoretic concepts to actual TypeScript source code,
providing a practical guide for the Rust implementation.

---

## 1. Type Flags (from types.ts:6324)

The fundamental discriminator for types:

```rust
pub mod type_flags {
    // Primitive types
    pub const ANY: u32             = 1 << 0;
    pub const UNKNOWN: u32         = 1 << 1;
    pub const STRING: u32          = 1 << 2;
    pub const NUMBER: u32          = 1 << 3;
    pub const BOOLEAN: u32         = 1 << 4;
    pub const ENUM: u32            = 1 << 5;
    pub const BIG_INT: u32         = 1 << 6;

    // Literal types
    pub const STRING_LITERAL: u32  = 1 << 7;
    pub const NUMBER_LITERAL: u32  = 1 << 8;
    pub const BOOLEAN_LITERAL: u32 = 1 << 9;
    pub const ENUM_LITERAL: u32    = 1 << 10;
    pub const BIG_INT_LITERAL: u32 = 1 << 11;

    // Symbol types
    pub const ES_SYMBOL: u32       = 1 << 12;
    pub const UNIQUE_ES_SYMBOL: u32 = 1 << 13;

    // Special types
    pub const VOID: u32            = 1 << 14;
    pub const UNDEFINED: u32       = 1 << 15;
    pub const NULL: u32            = 1 << 16;
    pub const NEVER: u32           = 1 << 17;

    // Compound types
    pub const TYPE_PARAMETER: u32  = 1 << 18;
    pub const OBJECT: u32          = 1 << 19;
    pub const UNION: u32           = 1 << 20;
    pub const INTERSECTION: u32    = 1 << 21;

    // Type operators
    pub const INDEX: u32           = 1 << 22;  // keyof T
    pub const INDEXED_ACCESS: u32  = 1 << 23;  // T[K]
    pub const CONDITIONAL: u32     = 1 << 24;  // T extends U ? X : Y
    pub const SUBSTITUTION: u32    = 1 << 25;

    // Other
    pub const NON_PRIMITIVE: u32   = 1 << 26;  // object
    pub const TEMPLATE_LITERAL: u32 = 1 << 27;
    pub const STRING_MAPPING: u32  = 1 << 28;  // Uppercase<T>

    // Composite flags
    pub const NULLABLE: u32 = UNDEFINED | NULL;
    pub const LITERAL: u32 = STRING_LITERAL | NUMBER_LITERAL | BIG_INT_LITERAL | BOOLEAN_LITERAL;
    pub const STRING_LIKE: u32 = STRING | STRING_LITERAL | TEMPLATE_LITERAL | STRING_MAPPING;
    pub const NUMBER_LIKE: u32 = NUMBER | NUMBER_LITERAL | ENUM;
    pub const UNION_OR_INTERSECTION: u32 = UNION | INTERSECTION;
    pub const STRUCTURED_TYPE: u32 = OBJECT | UNION | INTERSECTION;
}
```

---

## 2. Object Flags (from types.ts:6443)

Additional flags for ObjectType variants:

```rust
pub mod object_flags {
    pub const CLASS: u32                = 1 << 0;
    pub const INTERFACE: u32            = 1 << 1;
    pub const REFERENCE: u32            = 1 << 2;   // TypeReference
    pub const TUPLE: u32                = 1 << 3;
    pub const ANONYMOUS: u32            = 1 << 4;   // Anonymous object type
    pub const MAPPED: u32               = 1 << 5;
    pub const INSTANTIATED: u32         = 1 << 6;
    pub const OBJECT_LITERAL: u32       = 1 << 7;
    pub const EVOLVING_ARRAY: u32       = 1 << 8;
    pub const OBJECT_LITERAL_PATTERN: u32 = 1 << 9;
    pub const FRESH_LITERAL: u32        = 1 << 10;  // Fresh object literal
    pub const ARRAY_LITERAL: u32        = 1 << 11;
    pub const PRIMITIVE_UNION: u32      = 1 << 12;
    pub const CONTAINS_SPREAD: u32      = 1 << 13;
    pub const REVERSE_MAPPED: u32       = 1 << 14;
    pub const JSX_ATTRIBUTES: u32       = 1 << 15;
    pub const MARKER: u32               = 1 << 16;
    pub const CLASS_OR_INTERFACE: u32   = CLASS | INTERFACE;
}
```

---

## 3. Type Structures

### 3.1 Base Type

```rust
#[derive(Clone, Debug)]
pub struct Type {
    pub flags: u32,
    pub id: TypeId,
    pub symbol: Option<SymbolId>,
    pub alias_symbol: Option<SymbolId>,
    pub alias_type_arguments: Vec<TypeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: TypeId = TypeId(u32::MAX);
}
```

### 3.2 Intrinsic Types (Singletons)

```rust
pub struct IntrinsicType {
    pub base: Type,
    pub intrinsic_name: String,  // "any", "unknown", "string", etc.
}

// Pre-allocated singleton types
lazy_static! {
    pub static ref ANY_TYPE: TypeId = ...;
    pub static ref UNKNOWN_TYPE: TypeId = ...;
    pub static ref STRING_TYPE: TypeId = ...;
    pub static ref NUMBER_TYPE: TypeId = ...;
    pub static ref BOOLEAN_TYPE: TypeId = ...;
    pub static ref VOID_TYPE: TypeId = ...;
    pub static ref UNDEFINED_TYPE: TypeId = ...;
    pub static ref NULL_TYPE: TypeId = ...;
    pub static ref NEVER_TYPE: TypeId = ...;
}
```

### 3.3 Literal Types

```rust
pub struct LiteralType {
    pub base: Type,
    pub value: LiteralValue,
    pub fresh_type: Option<TypeId>,   // Non-widening version
    pub regular_type: TypeId,         // Widening version
}

pub enum LiteralValue {
    String(String),
    Number(f64),
    BigInt(String),  // Store as string for precision
    Boolean(bool),
}
```

### 3.4 Object Types

```rust
pub struct ObjectType {
    pub base: Type,
    pub object_flags: u32,
    pub members: Option<SymbolTable>,
    pub properties: Vec<SymbolId>,
    pub call_signatures: Vec<Signature>,
    pub construct_signatures: Vec<Signature>,
    pub index_infos: Vec<IndexInfo>,
}

pub struct Signature {
    pub declaration: Option<NodeIndex>,
    pub type_parameters: Vec<TypeId>,
    pub parameters: Vec<SymbolId>,
    pub this_parameter: Option<SymbolId>,
    pub resolved_return_type: Option<TypeId>,
    pub min_argument_count: u32,
    pub flags: SignatureFlags,
}

pub struct IndexInfo {
    pub key_type: TypeId,    // string or number
    pub value_type: TypeId,
    pub is_readonly: bool,
}
```

### 3.5 Union and Intersection Types

```rust
pub struct UnionType {
    pub base: Type,
    pub object_flags: u32,
    pub types: Vec<TypeId>,          // Constituent types
    pub origin: Option<TypeId>,      // Original type before reduction
    pub property_cache: SymbolTable, // Cached common properties
}

pub struct IntersectionType {
    pub base: Type,
    pub object_flags: u32,
    pub types: Vec<TypeId>,          // Constituent types
    pub resolved_apparent_type: Option<TypeId>,
}
```

### 3.6 Type Parameters

```rust
pub struct TypeParameter {
    pub base: Type,
    pub symbol: SymbolId,
    pub constraint: Option<TypeId>,      // extends clause
    pub default: Option<TypeId>,         // default type
    pub target: Option<TypeId>,          // For substitution
    pub is_this_type: bool,              // "this" type
}
```

### 3.7 Conditional Types

```rust
pub struct ConditionalType {
    pub base: Type,
    pub check_type: TypeId,      // T in "T extends U"
    pub extends_type: TypeId,    // U in "T extends U"
    pub true_type: TypeId,       // X in "? X"
    pub false_type: TypeId,      // Y in ": Y"
    pub is_distributive: bool,   // Distributes over unions?
    pub infer_type_parameters: Vec<TypeId>,  // infer R types
}
```

### 3.8 Mapped Types

```rust
pub struct MappedType {
    pub base: Type,
    pub object_flags: u32,
    pub declaration: NodeIndex,
    pub type_parameter: TypeId,          // K in [K in ...]
    pub constraint_type: TypeId,         // keyof T or union
    pub name_type: Option<TypeId>,       // as clause
    pub template_type: Option<TypeId>,   // Property type template
    pub modifiers_type: Option<TypeId>,  // Source for +/- modifiers
}
```

---

## 4. Type Arena

```rust
pub struct TypeArena {
    types: Vec<TypeVariant>,
    // Singleton caches
    any_type: TypeId,
    unknown_type: TypeId,
    string_type: TypeId,
    // ... etc
}

pub enum TypeVariant {
    Intrinsic(IntrinsicType),
    Literal(LiteralType),
    Object(ObjectType),
    Interface(InterfaceType),
    TypeReference(TypeReferenceType),
    Union(UnionType),
    Intersection(IntersectionType),
    TypeParameter(TypeParameter),
    Conditional(ConditionalType),
    Mapped(MappedType),
    IndexedAccess(IndexedAccessType),
    Index(IndexType),
    TemplateLiteral(TemplateLiteralType),
}

impl TypeArena {
    pub fn new() -> Self {
        let mut arena = Self { types: Vec::new(), ... };
        // Pre-allocate singleton types
        arena.any_type = arena.create_intrinsic(type_flags::ANY, "any");
        arena.unknown_type = arena.create_intrinsic(type_flags::UNKNOWN, "unknown");
        // ... etc
        arena
    }

    pub fn get(&self, id: TypeId) -> Option<&TypeVariant> {
        if id.0 == u32::MAX { None } else { self.types.get(id.0 as usize) }
    }
}
```

---

## 5. Key Checker Functions

### 5.1 Type of Symbol

```rust
impl TypeChecker {
    /// Get the type of a symbol (cached)
    pub fn get_type_of_symbol(&mut self, symbol: SymbolId) -> TypeId {
        if let Some(cached) = self.symbol_type_cache.get(&symbol) {
            return *cached;
        }

        let type_id = self.get_type_of_symbol_worker(symbol);
        self.symbol_type_cache.insert(symbol, type_id);
        type_id
    }

    fn get_type_of_symbol_worker(&mut self, symbol: SymbolId) -> TypeId {
        let sym = self.symbols.get(symbol);
        let flags = sym.flags;

        if flags & symbol_flags::VARIABLE != 0 {
            self.get_type_of_variable_or_parameter(symbol)
        } else if flags & symbol_flags::FUNCTION != 0 {
            self.get_type_of_function(symbol)
        } else if flags & symbol_flags::CLASS != 0 {
            self.get_type_of_class(symbol)
        } else if flags & symbol_flags::INTERFACE != 0 {
            self.get_declared_type_of_symbol(symbol)
        } else if flags & symbol_flags::TYPE_ALIAS != 0 {
            self.get_declared_type_of_type_alias(symbol)
        } else {
            self.types.unknown_type
        }
    }
}
```

### 5.2 Assignability Check

```rust
impl TypeChecker {
    /// Check if source is assignable to target
    pub fn is_type_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        self.is_type_related_to(source, target, Relation::Assignable)
    }

    fn is_type_related_to(
        &mut self,
        source: TypeId,
        target: TypeId,
        relation: Relation,
    ) -> bool {
        // Check cache first
        let cache_key = (source, target, relation);
        if let Some(&result) = self.relation_cache.get(&cache_key) {
            return result;
        }

        let result = self.is_type_related_to_worker(source, target, relation);
        self.relation_cache.insert(cache_key, result);
        result
    }

    fn is_type_related_to_worker(
        &mut self,
        source: TypeId,
        target: TypeId,
        relation: Relation,
    ) -> bool {
        // Identity
        if source == target {
            return true;
        }

        let source_type = self.types.get(source);
        let target_type = self.types.get(target);

        // any is related to everything (except never in some cases)
        if source_type.flags & type_flags::ANY != 0 {
            return true;
        }
        if target_type.flags & type_flags::ANY != 0 {
            return true;
        }

        // unknown is top type - everything is assignable to it
        if target_type.flags & type_flags::UNKNOWN != 0 {
            return true;
        }

        // never is bottom type - assignable to everything
        if source_type.flags & type_flags::NEVER != 0 {
            return true;
        }

        // Union source: all members must be related
        if source_type.flags & type_flags::UNION != 0 {
            return self.each_type_related_to_type(source, target, relation);
        }

        // Union target: source must be related to some member
        if target_type.flags & type_flags::UNION != 0 {
            return self.type_related_to_some_type(source, target, relation);
        }

        // Object types: structural comparison
        if source_type.flags & type_flags::OBJECT != 0
            && target_type.flags & type_flags::OBJECT != 0
        {
            return self.object_type_related_to(source, target, relation);
        }

        // Primitive comparison
        self.primitive_types_related(source_type, target_type)
    }
}
```

### 5.3 Type Instantiation

```rust
impl TypeChecker {
    /// Instantiate a generic type with type arguments
    pub fn instantiate_type(&mut self, type_id: TypeId, mapper: &TypeMapper) -> TypeId {
        let cache_key = (type_id, mapper.id);
        if let Some(&cached) = self.instantiation_cache.get(&cache_key) {
            return cached;
        }

        let result = self.instantiate_type_worker(type_id, mapper);
        self.instantiation_cache.insert(cache_key, result);
        result
    }

    fn instantiate_type_worker(&mut self, type_id: TypeId, mapper: &TypeMapper) -> TypeId {
        let typ = self.types.get(type_id);

        // Type parameter: apply mapper
        if typ.flags & type_flags::TYPE_PARAMETER != 0 {
            return mapper.map(type_id).unwrap_or(type_id);
        }

        // Object type with type parameters
        if typ.flags & type_flags::OBJECT != 0 {
            if let TypeVariant::TypeReference(ref_type) = typ {
                let new_args: Vec<_> = ref_type.type_arguments
                    .iter()
                    .map(|&arg| self.instantiate_type(arg, mapper))
                    .collect();
                return self.create_type_reference(ref_type.target, new_args);
            }
        }

        // Union: instantiate each member
        if typ.flags & type_flags::UNION != 0 {
            if let TypeVariant::Union(union_type) = typ {
                let new_types: Vec<_> = union_type.types
                    .iter()
                    .map(|&t| self.instantiate_type(t, mapper))
                    .collect();
                return self.get_union_type(&new_types);
            }
        }

        // Conditional: may need to distribute
        if typ.flags & type_flags::CONDITIONAL != 0 {
            return self.instantiate_conditional_type(type_id, mapper);
        }

        type_id
    }
}
```

### 5.4 Type Inference

```rust
impl TypeChecker {
    /// Infer type arguments for a generic call
    pub fn infer_type_arguments(
        &mut self,
        signature: &Signature,
        args: &[NodeIndex],
        contextual_type: Option<TypeId>,
    ) -> Vec<TypeId> {
        let mut context = InferenceContext::new(
            &signature.type_parameters,
        );

        // Infer from arguments to parameters
        for (i, &arg) in args.iter().enumerate() {
            if i < signature.parameters.len() {
                let param_type = self.get_type_of_symbol(signature.parameters[i]);
                let arg_type = self.check_expression(arg);
                self.infer_types(&mut context, param_type, arg_type, Variance::Covariant);
            }
        }

        // Infer from return type to contextual type
        if let Some(ctx_type) = contextual_type {
            if let Some(return_type) = signature.resolved_return_type {
                self.infer_types(&mut context, return_type, ctx_type, Variance::Contravariant);
            }
        }

        context.get_inferred_types()
    }

    fn infer_types(
        &mut self,
        context: &mut InferenceContext,
        source: TypeId,      // Parameter type (has type params)
        target: TypeId,      // Argument type (concrete)
        variance: Variance,
    ) {
        let source_type = self.types.get(source);

        // If source is a type parameter we're inferring
        if source_type.flags & type_flags::TYPE_PARAMETER != 0 {
            context.add_inference(source, target, variance);
            return;
        }

        // Recurse into structure
        if source_type.flags & type_flags::OBJECT != 0 {
            let target_type = self.types.get(target);
            if target_type.flags & type_flags::OBJECT != 0 {
                self.infer_from_object_types(context, source, target, variance);
            }
        }

        // Union: try each constituent
        if source_type.flags & type_flags::UNION != 0 {
            // Complex: need to find matching constituents
        }
    }
}

struct InferenceContext {
    type_parameters: Vec<TypeId>,
    inferences: Vec<Vec<(TypeId, Variance)>>,  // Per type parameter
}

impl InferenceContext {
    fn add_inference(&mut self, type_param: TypeId, inferred: TypeId, variance: Variance) {
        if let Some(idx) = self.type_parameters.iter().position(|&p| p == type_param) {
            self.inferences[idx].push((inferred, variance));
        }
    }

    fn get_inferred_types(&self) -> Vec<TypeId> {
        self.inferences.iter().map(|candidates| {
            // Covariant: union of candidates
            // Contravariant: intersection of candidates
            // Both: best common type
            self.get_best_inference(candidates)
        }).collect()
    }
}
```

---

## 6. Type Narrowing

```rust
impl TypeChecker {
    /// Get narrowed type at a reference based on control flow
    pub fn get_flow_type_of_reference(
        &mut self,
        reference: NodeIndex,
        declared_type: TypeId,
        flow: FlowNodeId,
    ) -> TypeId {
        let cache_key = (reference, flow);
        if let Some(&cached) = self.flow_type_cache.get(&cache_key) {
            return cached;
        }

        let result = self.get_flow_type_worker(reference, declared_type, flow);
        self.flow_type_cache.insert(cache_key, result);
        result
    }

    fn get_flow_type_worker(
        &mut self,
        reference: NodeIndex,
        declared_type: TypeId,
        flow: FlowNodeId,
    ) -> TypeId {
        let flow_node = self.flow_nodes.get(flow);

        match flow_node.flags {
            // Start of flow: return declared type
            f if f & flow_flags::START != 0 => declared_type,

            // Assignment: use assigned type
            f if f & flow_flags::ASSIGNMENT != 0 => {
                self.get_assigned_type(flow_node, reference)
            }

            // Condition: narrow based on condition
            f if f & flow_flags::CONDITION != 0 => {
                let antecedent_type = self.get_flow_type_of_reference(
                    reference,
                    declared_type,
                    flow_node.antecedent[0],
                );
                self.narrow_type_by_condition(antecedent_type, flow_node)
            }

            // Branch label: union of antecedent types
            f if f & flow_flags::BRANCH_LABEL != 0 => {
                let types: Vec<_> = flow_node.antecedent
                    .iter()
                    .map(|&ant| self.get_flow_type_of_reference(reference, declared_type, ant))
                    .collect();
                self.get_union_type(&types)
            }

            _ => declared_type,
        }
    }

    fn narrow_type_by_condition(&mut self, type_id: TypeId, condition: &FlowNode) -> TypeId {
        // typeof x === "string" → narrow to string
        // x instanceof C → narrow to C
        // x === null → narrow to null
        // "prop" in x → narrow to types with prop
        // x (truthiness) → exclude null/undefined

        // Implementation depends on condition AST
        type_id
    }
}
```

---

## 7. Implementation Order

### Phase 5.1: Core Infrastructure
1. `type_flags` module
2. `TypeId`, `TypeArena`
3. Intrinsic singleton types
4. Basic `Type` enum/struct

### Phase 5.2: Symbol-to-Type
1. `get_type_of_symbol` for variables
2. `get_type_of_symbol` for functions
3. Return type annotation reading
4. Parameter type handling

### Phase 5.3: Assignability
1. `is_type_related_to` framework
2. Primitive type comparisons
3. Union type handling
4. Basic object type comparison

### Phase 5.4: Object Types
1. Property resolution
2. Call signature matching
3. Index signature checking
4. Structural comparison

### Phase 5.5: Generics
1. Type parameter handling
2. Type instantiation
3. Type argument inference
4. Constraint checking

### Phase 5.6: Advanced
1. Conditional types
2. Mapped types
3. Template literal types
4. Type narrowing

---

## 8. Testing Strategy

Each phase should have tests for:

1. **Unit tests** (Rust): Test individual functions
2. **Integration tests**: Parse + Bind + Check
3. **Comparison tests**: Compare with TypeScript checker output
4. **Error case tests**: Verify correct errors are produced

Example test structure:
```rust
#[test]
fn test_variable_type_inference() {
    let code = "const x = 42;";
    let checker = create_checker(code);
    let x_symbol = checker.get_symbol("x");
    let x_type = checker.get_type_of_symbol(x_symbol);
    assert!(checker.is_number_literal_type(x_type));
}

#[test]
fn test_assignability() {
    let checker = create_checker("");
    assert!(checker.is_type_assignable_to(
        checker.number_type,
        checker.number_type
    ));
    assert!(!checker.is_type_assignable_to(
        checker.string_type,
        checker.number_type
    ));
}
```
