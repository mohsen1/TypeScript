This benchmark result is a **smoking gun**.

It tells us exactly where the architecture is failing vs. where it is succeeding.

### 1. The Good News: The "ThinNode" Architecture Works (Emitter)
**500 MB/s** for the emitter is **world-class**.
*   It proves that your `ThinNode` memory layout (16-byte headers + side tables) is extremely cache-efficient for traversal.
*   The CPU is flying through the arrays of nodes without cache misses.
*   **Takeaway:** Do not change the `ThinNode` structure. It is validated.

### 2. The Bad News: The Parser is allocating memory like it's Java (Parser)
**50 MB/s** is unacceptably slow for a Rust parser. A well-written Rust parser should easily hit **150-300 MB/s**.
*   **The Diagnosis:** Your parser is "Zero-Copy" in name only. It is actually **"Copy-Everything"**.
*   **The Bottleneck:** `String` allocations.

I looked at your `src/scanner_impl.rs` and `src/parser/thin_node.rs`. Here is the exact line killing your performance:

```rust
// src/scanner_impl.rs

fn scan_identifier(&mut self) {
    // ... scanning logic ...
    
    // 1. You create a slice (Good)
    let text_slice = &self.source[start..self.pos];

    // 2. You Intern it (Good, O(1) comparison later)
    self.token_atom = self.interner.intern(text_slice);

    // 3. THE KILLER: You allocate a heap string for EVERY identifier immediately
    self.token_value = text_slice.to_string(); 
}
```

And in `src/parser/thin_node.rs`:
```rust
pub struct IdentifierData {
    pub escaped_text: String, // <--- 24 bytes + Heap Allocation
    // ...
}
```

**Why this destroys performance:**
1.  **Allocator Pressure:** For a 1MB source file, you might be doing 50,000 small heap allocations. `malloc` is slow.
2.  **Cache Thrashing:** Even though your *Nodes* are packed tightly, the *Data* they point to (`String`) is scattered randomly on the heap.
3.  **Redundancy:** You already have the source text in memory. `IdentifierData` is just duplicating bytes that already exist in `self.source_text`.

### 3. The Fix: True Zero-Copy (Lazy Text)

You need to switch to **View-Based** strings. Only allocate a `String` if the text actually differs from the source (e.g., contains escape sequences like `\u0061`).

#### Step 1: Remove `String` from the Hot Path

Change your data structures to store **Ranges** or **Atoms**, not Strings.

```rust
// src/parser/thin_node.rs

pub struct IdentifierData {
    // REPLACED: pub escaped_text: String,
    
    // OPTION A: Pure Range (Fastest, requires reference to SourceText to read)
    // You already have pos/end in the ThinNode! You might not need this struct AT ALL.
    
    // OPTION B: Interned Atom (Fast comparison, lookup cost)
    pub text: Atom, 
}
```

#### Step 2: Fix the Scanner

The scanner should not allocate `token_value` eagerly.

```rust
// src/scanner_impl.rs

pub struct ScannerState {
    // REMOVE: token_value: String
    // ADD:
    token_has_escapes: bool,
}

impl ScannerState {
    pub fn get_token_text<'a>(&'a self) -> &'a str {
        // Return slice of source directly
        &self.source[self.token_start..self.pos]
    }
}
```

#### Step 3: Lazy Evaluation in Parser

When parsing an identifier:
1.  Get the slice from scanner: `&str`.
2.  **Intern it** to get an `Atom` (u32).
3.  Store the `Atom` in `IdentifierData`.
4.  **Do not store the String.**

If you need the text later (e.g., for the Emitter), you have two options:
1.  **Resolve the Atom:** `interner.resolve(id.atom)`
2.  **Slice the Source:** `source_text[node.pos..node.end]` (This is what your 500MB/s emitter should use!)

### Summary of "What it gives"

1.  **Confirmation:** Your `ThinNode` architecture is a success (Emitter speed proves it).
2.  **Directive:** You must refactor `Scanner` and `Parser` to stop allocating `String`s.
    *   Target: **Identifier** and **Literal** parsing.
    *   Goal: `IdentifierData` should be `Copy` (just `u32` Atom).
3.  **Prediction:** Once you remove `String` allocations from the parse loop, your parser throughput should jump from **50 MB/s** to **>200 MB/s**.


You are asking a very deep architectural question: **"Where does the Logic live?"**

You are **not** misunderstanding. In fact, you have hit on the central tension in modern compiler design: **Imperative Algorithms** (Hand-written code) vs. **Declarative Constraints** (Logic Solvers).

Here is the breakdown of what parts of the "Judge" can be expressed by a generic Solver, and what parts must remain hand-written Rust code.

### 1. The "Solver" (The Engine)
The **Solver** is the machinery. It doesn't know what a "TypeScript Class" is. It only understands abstract mathematical relationships.

**Yes, the Solver SHOULD handle:**
1.  **The Graph Traversal (Cycles):** You should never manually write "if recursion depth > 100". The Solver engine handles Coinduction (checking if we've seen this pair `(A, B)` before).
2.  **Unification (Generics):** When you see `foo<T>(x: T)`, finding what `T` is equals solving a system of equations ($T = \text{string}$). The `ena` crate (Union-Find) is exactly this—a "mini-solver" that handles the math so you don't have to.
3.  **Cache Management:** The Solver owns the memoization table. The Judge shouldn't care about caching.

### 2. The "Judge" (The Rules)
The **Judge** defines the specific business logic of TypeScript.

**No, the Solver CANNOT easily express this (Keep it in Rust):**
TypeScript has rules that are mathematically "weird" and very hard to encode in a pure logic solver (like Prolog or Datalog) without destroying performance:
*   **Object Shapes:** "Type A is a subtype of B if A has every key in B, unless B is an index signature, or unless B is `weak`..." -> This is an iterative loop over properties. It's fastest to write this as a `for` loop in Rust.
*   **The Unsoundness Catalog:** Rules like "Bivariant function arguments" or "Void returns can swallow anything" are arbitrary hacks. Trying to teach a formal logic solver about these hacks makes the solver incredibly complex and slow.

### The Sweet Spot: "Logic-Driven Code"

You should aim for a **Hybrid Approach**.

#### Bad Architecture (Spaghetti)
The logic is mixed with the mechanics.
```rust
fn check_subtype(a, b) {
    // MECHANICS (Solver)
    if recursion_depth > 100 { return error; } 
    
    // LOGIC (Judge)
    if is_object(a) && is_object(b) { ... } 
}
```

#### Good Architecture (Separated)
The **Solver** runs the loop; the **Judge** answers simple questions.

**1. The Solver (Generic Engine):**
```rust
// src/solver/engine.rs
impl Solver {
    fn solve_relation(&mut self, a: TypeId, b: TypeId) -> bool {
        if let Some(res) = self.check_cycle(a, b) { return res; }
        
        // Ask the Judge for the rule
        let result = Judge::compare(a, b, self); 
        
        self.cache(a, b, result);
        result
    }
}
```

**2. The Judge (Pure Rules):**
```rust
// src/solver/judge.rs
impl Judge {
    fn compare(sub: TypeId, sup: TypeId, solver: &mut Solver) -> bool {
        match (sub.kind, sup.kind) {
            (Object, Object) => {
                // Pure logic: iterate keys
                // Delegate child checks back to solver
                solver.solve_relation(sub.prop, sup.prop) 
            }
            (Union, _) => {
                 // Pure logic: all members must match
            }
        }
    }
}
```

### Answer to your question

**"Do you think some of the 'judge' can be expressed by solver?"**

**YES.** Specifically, **Inference**.

When you are inferring types for generics (`<T>`), you **are** creating a constraint satisfaction problem.
*   *Constraint 1:* `T` must be a subtype of `Animal`.
*   *Constraint 2:* `Dog` must be a subtype of `T`.
*   *Solution:* `T` is `Dog` (or `Animal`).

You **should not** write imperative code to solve `T`. You should simply **register these constraints** with the `InferenceContext` (backed by `ena` or a similar unifier), and let the "Solver" mathematically compute the value of `T`.

### Summary

1.  **Subtyping (Checking):** Keep this as **Rust Code**. It's too quirky and performance-critical for a generic solver.
2.  **Inference (Guessing):** Use a **Solver** (Unification/Union-Find). This is pure math.
3.  **Recursion/Caching:** Use a **Solver** (The Engine). Never handle recursion manually in the rule logic.




This is a solved problem in the Rust compiler ecosystem. The architecture you are describing—separating the **logic rules** from the **execution engine**—is exactly how modern Rust compilers (like `rustc`'s new trait solver and `chalk`) are built.

Here are the concrete examples and patterns you can copy.

### 1. The Reference Architecture: "Chalk"
The Rust team built a separate library called **Chalk** to solve this exact problem.
*   **The Judge ("Lowering"):** Chalk defines "program clauses" (logic rules) that map Rust source code into logic predicates (e.g., `Implements(T, Trait)`).
*   **The Solver ("Engine"):** A separate logic engine (based on Prolog/Datalog) takes these rules and solves them. It handles the recursion, caching, and cycle detection.

**How to copy this:**
You don't need a full Prolog engine, but you should adopt the **Constraint** pattern for your Inference engine.

### 2. Implementation: Using `ena` for Inference
The `ena` crate is what `rustc` uses for its "Solver". It implements efficient Union-Find (Disjoint Set Union) with backtracking.

**Don't write this:**
```rust
// BAD: Imperative "Judge" trying to do "Solver" work
fn unify(a: Type, b: Type) {
    if let Type::Var(v) = a {
        v.set(b); // Mutating state directly in the rules
    }
}
```

**Write this (The "Constraint" Pattern):**
The Judge generates constraints; the Solver applies them.

```rust
use ena::unify::{InPlaceUnificationTable, UnifyKey, UnifyValue, NoError};

// 1. The Solver State (The Engine)
struct Solver {
    unification_table: InPlaceUnificationTable<TypeVar>,
}

// 2. Define the "Atom" for the Solver
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct TypeVar(u32);

impl UnifyKey for TypeVar {
    type Value = Option<TypeId>; // What does this variable point to?
    fn index(&self) -> u32 { self.0 }
    fn from_index(u: u32) -> Self { TypeVar(u) }
    fn tag() -> &'static str { "TypeVar" }
}

// 3. The Judge (Constraint Generator)
// Just registers requirements. Doesn't know "how" to solve them.
impl InferenceContext {
    pub fn constrain_subtype(&mut self, sub: TypeId, sup: TypeId) {
        // Just record the constraint. The Solver processes it later.
        self.constraints.push(Constraint::Subtype(sub, sup));
    }
    
    pub fn solve(&mut self) {
        // The Solver Engine runs here
        while let Some(constraint) = self.constraints.pop() {
            match constraint {
                 Constraint::Eq(a, b) => self.solver.unify_var_var(a, b),
                 // ...
            }
        }
    }
}
```

### 3. Implementation: Coinduction (The "Seen Set")
For recursive types, you asked if the Solver can handle the "cycle detection". Yes. This is called **Coinductive Solving**.

Instead of passing `depth: u32` everywhere, your Solver maintains a `CycleStack`.

**The Pattern:**
```rust
struct SubtypeSolver {
    // The "Seen Set" (Coinductive Assumption)
    // If we see (A, B) again while it's in this stack, we return TRUE.
    stack: Vec<(TypeId, TypeId)>,
}

impl SubtypeSolver {
    fn is_subtype(&mut self, sub: TypeId, sup: TypeId) -> bool {
        // 1. Solver Logic: Cycle Detection
        if self.stack.contains(&(sub, sup)) {
            return true; // Coinductive assumption: "Assume yes until proven no"
        }
        self.stack.push((sub, sup));

        // 2. Judge Logic: The actual rules
        let result = self.check_structure(sub, sup);

        // 3. Solver Logic: Cleanup
        self.stack.pop();
        result
    }

    fn check_structure(&mut self, sub: TypeId, sup: TypeId) -> bool {
        // The Judge just checks the structure.
        // It blindly recurses, trusting the Solver to catch the cycle.
        match (sub, sup) {
             (Object(a), Object(b)) => {
                 a.props.all(|p| self.is_subtype(p.ty, b.get(p.name)))
             }
             // ...
        }
    }
}
```

### Summary of Examples
1.  **For Inference:** Look at `rustc_infer` (or the `ena` crate examples). They strictly separate "registering obligations" (Judge) from "fulfilling obligations" (Solver).
2.  **For Subtyping:** Look at `chalk-solve`. It uses a "Goal" system. `Goal: Subtype(A, B)` is pushed onto a stack. If the engine sees the same Goal again, it applies coinductive logic automatically.

**Recommendation:**
Refactor your `InferenceContext` to wrap `ena::InPlaceUnificationTable`. This gives you the "Solver" part for free (transactional rollback, path compression, O(1) unification) so you only have to write the "Judge" part (traversing the AST and adding constraints).

Yes, you **are** using `ena` (the Unification Solver). It is correctly integrated in `src/solver/infer.rs`.

However, you are manually writing "Solver Mechanics" (graph traversal, cycle detection) inside your "Judge" (Type Checker) logic in at least two major places.

### 1. The Subtype Checker (Manual Cycle Detection)
**File:** `src/solver/subtype.rs`

You are manually implementing Coinduction (Cycle Detection) inside the business logic.

**Current Code (Mixed Mechanics + Logic):**
```rust
// src/solver/subtype.rs

fn check_subtype(&mut self, source: TypeId, target: TypeId) -> SubtypeResult {
    // --- MECHANICS (The Solver's Job) ---
    // 1. Stack Overflow Protection
    if self.depth > 100 { return SubtypeResult::Provisional; }

    // 2. Cycle Detection (Coinduction)
    let pair = (source, target);
    if self.in_progress.contains(&pair) {
        return SubtypeResult::Provisional;
    }
    self.in_progress.insert(pair);
    self.depth += 1;

    // --- LOGIC (The Judge's Job) ---
    let result = self.check_subtype_inner(source, target);

    // --- MECHANICS CLEANUP ---
    self.depth -= 1;
    self.in_progress.remove(&pair);
    result
}
```

**How it should look (Separated):**
The `SubtypeChecker` should only contain the `match` statement from `check_subtype_inner`. The cycle detection should be a generic wrapper or a method on the `Interner`/`Database`.

### 2. The Instantiator (Manual Cycle Detection)
**File:** `src/solver/instantiate.rs`

When replacing `<T>` with a concrete type, you are manually tracking visited nodes to prevent infinite recursion in recursive types.

**Current Code:**
```rust
// src/solver/instantiate.rs

pub struct TypeInstantiator<'a> {
    // ...
    // MECHANICS: Manual "Visited" set for graph traversal
    visiting: HashMap<TypeId, TypeId>, 
}

impl<'a> TypeInstantiator<'a> {
    pub fn instantiate(&mut self, type_id: TypeId) -> TypeId {
        // MECHANICS: Manual Cycle Check
        if let Some(&cached) = self.visiting.get(&type_id) {
            return cached;
        }
        
        // MECHANICS: Cycle Prevention (Placeholder)
        self.visiting.insert(type_id, type_id); 
        
        // LOGIC: The actual substitution rule
        let result = self.instantiate_key(&key);
        
        // MECHANICS: Update cache
        self.visiting.insert(type_id, result);
        result
    }
}
```

**Why this is bad:**
You have implemented "Graph Traversal with Cycle Detection" twice. Once in `subtype.rs` and once in `instantiate.rs`. If you add a "Type Printer" or "Type Normalizer", you will likely implement it a third time.

### 3. The "Good Example" (Inference)
**File:** `src/solver/infer.rs`

This file is actually doing it right! You are using `ena` to handle the "Solver" part (Unification).

```rust
// src/solver/infer.rs

// THE JUDGE (You just state the facts/constraints)
pub fn add_lower_bound(&mut self, var: InferenceVar, ty: TypeId) {
    // ... logic to record constraint ...
}

// THE SOLVER (ena does the math)
// You don't write the "find" or "union" algorithms manually.
// You delegate to self.table (InPlaceUnificationTable).
pub fn unify_vars(&mut self, a: InferenceVar, b: InferenceVar) -> Result<(), InferenceError> {
    self.table.unify_var_var(a, b) // <--- ena handles the mechanics
}
```

### Recommendation

Refactor `SubtypeChecker` and `TypeInstantiator` to use a shared **`GraphWalker`** or **`CycleDetector`** abstraction.

```rust
// Example Abstraction
struct CycleDetector<K, V> {
    stack: Vec<K>,
    visited: HashMap<K, V>,
}

impl<K, V> CycleDetector<K, V> {
    fn run<F>(&mut self, key: K, compute: F) -> V 
    where F: FnOnce(&mut Self) -> V {
        if let Some(v) = self.check_cycle(&key) { return v; }
        self.enter(key);
        let result = compute(self);
        self.exit(key, result);
        result
    }
}
```

This removes the "Mechanism" code from your "Business Logic" files (`subtype.rs`, `instantiate.rs`), making them purely about TypeScript rules.