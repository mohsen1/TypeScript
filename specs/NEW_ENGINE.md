

### Part 1: The "Compiler as a Database" Paradigm

The most significant shift in moving to Phase 7.5 is changing your mental model of what the type checker **is**.

#### The Old World: The "Imperative Walk"
In the legacy TypeScript compiler (and our current Phase 5 checker), type checking is a **process**.
1.  You start at the top of a file.
2.  You walk down the AST.
3.  You maintain a massive, mutable `CheckerState` object.
4.  As you traverse, you flip flags, mutate symbol tables, and push/pop scopes.

**The Problem:** This is fragile and hard to parallelize. If Thread A is checking `function foo()` and modifies the global state, Thread B cannot safely check `function bar()` without complex locking.

#### The New World: The "Query Engine"
In the new architecture, the type checker is a **Database**. It doesn't "do" anything until you ask it a question.

Instead of "checking the file," the system waits for a query:
> *"Database, is the variable `user` assignable to interface `Admin`?"*

The Database (powered by **Salsa**) then works backward:
1.  **Dependency:** "To answer that, I first need to know the *structure* of `Admin`."
2.  **Lookup:** It looks up the `Admin` interface node.
3.  **Synthesis:** It converts that AST node into a mathematical Type Shape (Interning).
4.  **Logic:** It compares the shape of `user` to `Admin`.
5.  **Result:** It returns `true` or `false` and **caches the answer**.

#### Why This Changes Everything
*   **Laziness:** You never compute types for code that isn't queried.
*   **Parallelism:** Multiple threads can ask the Database questions simultaneously. Since the DB manages the dependencies, it knows exactly what work can be shared.
*   **Immutability:** The input (your `ThinNodeArena`) is read-only. The output (the answers) is cached. There is no "global mutable state" to corrupt.

**In Summary:** We are stopping the "script" that runs from top to bottom. We are building an Oracle that answers questions on demand.






### Part 2: The Universe of Shapes (Interning)

In the Old World (Phase 5), a "Type" is a heavy Rust struct or enum, often containing vectors of other types, pointers to symbols, and mutable flags. Comparing two types involves recursively chasing these pointers, which is slow and destroys CPU cache locality.

In the New World (Phase 7.5), **Types are just Integers (`u32`).**

#### The Concept: Structural Identity
TypeScript is a **structural** type system. This means that if two objects look the same, they *are* the same.
*   `interface A { x: number }`
*   `interface B { x: number }`
*   `const c = { x: 1 }`

To the compiler, these three things effectively map to the exact same "Shape."

#### How It Works: The Interning Engine
Instead of creating new objects for every type we encounter, we use an **Interner** (managed by Salsa or a specialized arena).

1.  **Lowering:** When the database encounters a type node (e.g., `ThinNode` for `{ x: number }`), it converts it into a **TypeKey**. This Key describes the structure purely:
    *   *Key:* `Object { props: [("x", TypeId::Number)] }`

2.  **Deduplication:** The Database checks its map: "Have I seen this Key before?"
    *   **Yes:** Return the existing `TypeId` (e.g., `1042`).
    *   **No:** Assign a new `TypeId` (e.g., `1043`), store the Key, and return the ID.

3.  **The Result:** `TypeId(1043)`
    Now, whenever you pass this type around your application, you are just passing the number `1043`.

#### Why This Wins
*   **O(1) Equality:** Checking if two types are *identical* becomes an integer comparison (`if id_a == id_b`). You don't need to traverse fields.
*   **Memory Efficiency:** We store the structure of `{ x: number }` exactly **once** in memory, even if it appears 10,000 times in your source code.
*   **Cache Locality:** The solver works with flat arrays of integers. This fits perfectly into the CPU's L1/L2 cache, unlike scattered heap objects.

**In Summary:** We convert the complex, messy graph of declared types into a compact, deduplicated universe of unique IDs.





### Part 3: The Solver & The Loop (Coinduction)

Now that we have Types as integers (Part 2), we need to answer the core question of TypeScript: **"Is Type A a subtype of Type B?"**

In a nominal language (like Java or Rust), this is easy: you just check the inheritance hierarchy declared by the user. In TypeScript’s **structural** system, it is much harder because shapes can be infinitely recursive.

#### The Problem: The Infinite Mirror
Consider a linked list:
```typescript
interface Node {
    next: Node;
}
```
If we want to check if `Node` is a subtype of itself (or a structurally identical `Node2`), the solver does this:
1.  Compare `Node` vs `Node2`.
2.  Check property `next`.
3.  Compare `Node.next` (which is `Node`) vs `Node2.next` (which is `Node2`).
4.  GOTO Step 1.

A naive recursive function will run until the stack overflows.

#### The Solution: Coinduction (The "Provisional Yes")
We switch from **Inductive** logic (prove it from base cases up) to **Coinductive** logic (assume it's true until proven false).

In our new Query Engine, the `solve_subtype(A, B)` function works like this:

1.  **The Stack Check:** Before doing any work, the solver looks at the current call stack.
    *   *Question:* "Am I already currently trying to prove that A is a subtype of B?"
    *   *Answer:* **YES.** (We found a cycle).
2.  **The Short-Circuit:** Instead of recurring again, we return a **"Provisional True"**.
    *   *Why?* In structural typing, if you trace a loop and find no contradictions (mismatched types) along the way, the types are considered compatible (Greatest Fixed Point semantics).
3.  **The Verification:** The solver pops back up the stack. If the rest of the structure matches, the "Provisional True" solidifies into a "Definite True."

#### Implementation in the Database
Because we are using **Salsa** (or a similar query system), we don't need to manually pass a `seen` set through thousands of functions.

We define `solve_subtype` as a query. Salsa detects when a query calls itself with the same arguments.
*   If we configure the recovery strategy correctly, Salsa detects the cycle.
*   We tell it: "If you see a cycle, assume the result is `true`."

**In Summary:** The Logic Engine is a state machine that walks the graph of your IDs. It doesn't just match shapes; it remembers its own path to handle the infinite recursion inherent in TypeScript, ensuring the compiler never hangs on complex nested types.






### Part 4: The Detective Work (Unification & Inference)

In Parts 1-3, we built a system that can answer: *"Is `String` a subtype of `Object`?"* This is like checking `5 < 10`—it's a static verification.

But TypeScript doesn't just verify types; it **invents** them.
When you write `const x = identity("hello")`, the compiler has to figure out that `T` is `String` and therefore `x` is `String`. The Solver can't just compare shapes; it has to solve for variables.

#### The Concept: Type Algebra
Think of Subtyping as **Arithmetic** (`5 < 10` -> True/False).
Think of Inference as **Algebra** (`x + 5 = 10` -> Solve for `x`).

In our architecture, we treat generic type parameters (like `<T>`) as **Inference Variables** (often denoted as `?0`, `?1`, etc.).

#### The Mechanism: Unification (Union-Find)
To solve this efficiently, we use a data structure called a **Unification Table** (implemented by the `ena` crate in our stack).

1.  **Instantiation:** When `identity<T>` is called, the solver replaces `T` with a fresh inference variable, `?0`.
2.  **Constraint Collection:** The compiler looks at the arguments.
    *   Parameter: `x: T` (which is now `?0`)
    *   Argument: `"hello"` (which is `Literal("hello")`)
    *   *Constraint:* `Literal("hello")` must be assignable to `?0`.
3.  **Unification:** The solver registers this fact in the table.
    *   *Action:* "Point `?0` to `Literal("hello")`."
    *   Now, effectively, `?0` *is* `"hello"`.
4.  **Resolution:** When we ask for the return type (which is `T`), the solver looks up `?0` in the table, follows the pointer, and returns `Literal("hello")`.

#### Handling Complexity: Backtracking
TypeScript is tricky. Sometimes you have multiple overloads:
```typescript
function foo(x: string): void;
function foo(x: number): void;
```
If you call `foo(42)`, the solver has to try the first signature, realize `Number` is not `String`, **undo** any constraints it assumed, and try the second signature.

Our inference engine supports **Snapshots**:
1.  Take a snapshot of the table.
2.  Try unification with Signature A.
3.  If it fails, **Rollback** to the snapshot (forgetting everything we just tried).
4.  Try unification with Signature B.

#### Why This Wins
*   **Speed:** Union-Find algorithms are nearly $O(1)$ (amortized constant time). Resolving generic chains is incredibly fast.
*   **Cleanliness:** We separate the "Global Immutable Types" (Part 2) from the "Local Mutable Inference" state. Inference happens locally within a function check, calculates the result, and then hands the final concrete type back to the Database.

**In Summary:** We combine the static "Database" of shapes with a local "Algebra Solver" that fills in the blanks for generic functions, enabling the magical type inference TypeScript developers expect.




### Part 5: The Bridge (Synthesis & Lowering)

We have a Database that answers questions (Part 1), a Universe of interned IDs (Part 2), a Logic Engine that handles loops (Part 3), and a Detective that solves generics (Part 4).

The final piece is the **Bridge**: How do we connect the raw text of your source code (the `ThinNode` AST) to this mathematical type system?

This process is called **Lowering** (or Synthesis). It is the translation layer that turns "Syntax" into "Semantics."

#### The Concept: Just-In-Time (JIT) Compilation for Types
In the Old World, the compiler would "bind" and "check" every node in the file, often calculating types that were never actually needed.

In the New World, Lowering is **Lazy**. We only convert AST nodes into Type IDs when a query specifically demands it.

#### The Workflow: From Text to ID
Let's trace a query: *"What is the type of the variable `config`?"*
Code: `interface Config { port: number }; const config: Config = ...;`

1.  **The Trigger:** The IDE or Compiler asks the Database: `type_of_symbol(SymbolId::Config)`.
2.  **The Lookup:** The Database looks at the **Binder** (Phase 4) to find where `config` is defined.
    *   *Result:* It points to a `VariableDeclaration` node in the `ThinNodeArena`.
3.  **The Analysis:** The Lowering engine inspects the AST node.
    *   It sees a type annotation: `: Config`.
    *   It resolves the name `Config` to a `SymbolId`.
4.  **Recursive Query:** The engine realizes it needs to know what `Config` is. It suspends the current task and fires a new query: `declared_type_of_symbol(SymbolId::Config)`.
5.  **The Synthesis (The Core Step):**
    *   The engine looks at the `interface Config` AST node.
    *   It iterates over the members (`port`).
    *   It resolves `number` to the Intrinsic ID for Number.
    *   It constructs the **Shape**: `TypeKey::Object( vec![ ("port", TypeId::Number) ] )`.
6.  **Interning:** It sends this Shape to the Interner (Part 2).
    *   *Result:* `TypeId(555)`.
7.  **The Return:** The queries unwind.
    *   `declared_type_of_symbol` returns `TypeId(555)`.
    *   `type_of_symbol` returns `TypeId(555)`.

#### Why This Wins
*   **Isolation:** The "Logic Engine" (Parts 3 & 4) never touches the raw AST. It only deals with clean, interned IDs. The "Lowering" layer is the only place that deals with the messy reality of parsing flag bits and syntax variations.
*   **Resilience:** If you make a syntax error in one function, it doesn't corrupt the Type IDs of the rest of the program. The Lowering engine just produces a `TypeKey::Error` for that specific spot, allowing the rest of the system to continue checking.
*   **Performance:** We don't allocate memory for types defined in functions that you aren't currently editing or checking.

### The Grand Summary

You are no longer building a "Compiler" in the traditional sense. You are building a **High-Performance Semantic Database**.

1.  **The Database (Salsa):** Orchestrates the work, manages dependencies, and enables parallelism.
2.  **The Data (Interning):** Compresses infinite shapes into simple integers (`u32`).
3.  **The Logic (Solver):** A cycle-aware state machine that verifies structural compatibility.
4.  **The Inference (Ena):** A snapshot-capable algebra solver for generics.
5.  **The Bridge (Lowering):** A lazy translation layer from your `ThinNode` AST to the Type system.

Th