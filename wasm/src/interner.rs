//! String Interner for identifier deduplication.
//!
//! PERFORMANCE OPTIMIZATION: Intern strings into a global pool and pass around
//! u32 indices (Atoms). This eliminates duplicate string allocations for common
//! identifiers like "id", "value", "length", etc.
//!
//! Comparisons become integer comparisons (atom_a == atom_b) instead of string
//! comparisons, which is significantly faster.

use rustc_hash::{FxHashMap, FxHasher};
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

/// An interned string identifier.
///
/// Atoms are cheap to copy (just a u32) and can be compared with == in O(1).
/// To get the actual string, use `Interner::resolve(atom)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Default, PartialOrd, Ord)]
pub struct Atom(pub u32);

impl Atom {
    /// A sentinel value representing no atom / empty string.
    pub const NONE: Atom = Atom(0);

    /// Check if this is the empty/none atom.
    #[inline]
    pub fn is_none(self) -> bool {
        self.0 == 0
    }

    /// Get the raw index value.
    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }
}

const SHARD_BITS: u32 = 6;
const SHARD_COUNT: usize = 1 << SHARD_BITS;
const SHARD_MASK: u32 = (SHARD_COUNT as u32) - 1;
const COMMON_STRINGS: &[&str] = &[
    // Keywords
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
    "with",
    "as",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "any",
    "boolean",
    "number",
    "string",
    "symbol",
    "type",
    "from",
    "of",
    "async",
    "await",
    // Common identifiers
    "id",
    "name",
    "value",
    "length",
    "key",
    "index",
    "item",
    "data",
    "error",
    "result",
    "response",
    "request",
    "options",
    "config",
    "props",
    "state",
    "children",
    "onClick",
    "onChange",
    "onSubmit",
    "constructor",
    "prototype",
    "toString",
    "valueOf",
    "hasOwnProperty",
    "Array",
    "Object",
    "String",
    "Number",
    "Boolean",
    "Function",
    "Promise",
    "Map",
    "Set",
    "Date",
    "RegExp",
    "Error",
    "Symbol",
    "console",
    "log",
    "warn",
    "error",
    "info",
    "debug",
    "document",
    "window",
    "global",
    "process",
    "module",
    "exports",
    "require",
    "define",
    "__dirname",
    "__filename",
];

/// String interner that deduplicates strings and returns Atom handles.
///
/// # Example
/// ```
/// use wasm::interner::Interner;
/// let mut interner = Interner::new();
/// let a1 = interner.intern("hello");
/// let a2 = interner.intern("hello");
/// assert_eq!(a1, a2); // Same atom for same string
/// assert_eq!(interner.resolve(a1), "hello");
/// ```
#[derive(Default)]
pub struct Interner {
    /// Map from string to atom index
    map: FxHashMap<String, Atom>,
    /// Vector of all interned strings (index 0 is empty string)
    strings: Vec<String>,
}

impl Interner {
    /// Create a new interner with the empty string pre-interned at index 0.
    pub fn new() -> Self {
        let mut interner = Interner {
            map: FxHashMap::default(),
            strings: Vec::with_capacity(1024), // Pre-allocate for common case
        };
        // Index 0 is reserved for empty/none
        interner.strings.push(String::new());
        interner.map.insert(String::new(), Atom::NONE);
        interner
    }

    /// Intern a string, returning its Atom handle.
    /// If the string was already interned, returns the existing Atom.
    #[inline]
    pub fn intern(&mut self, s: &str) -> Atom {
        if let Some(&atom) = self.map.get(s) {
            return atom;
        }
        let atom = Atom(self.strings.len() as u32);
        let owned = s.to_string();
        self.strings.push(owned.clone());
        self.map.insert(owned, atom);
        atom
    }

    /// Intern an owned String, avoiding allocation if possible.
    #[inline]
    pub fn intern_owned(&mut self, s: String) -> Atom {
        if let Some(&atom) = self.map.get(&s) {
            return atom;
        }
        let atom = Atom(self.strings.len() as u32);
        self.strings.push(s.clone());
        self.map.insert(s, atom);
        atom
    }

    /// Resolve an Atom back to its string value.
    /// Returns empty string if atom is out of bounds (safety for error recovery).
    #[inline]
    pub fn resolve(&self, atom: Atom) -> &str {
        self.strings
            .get(atom.0 as usize)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Try to resolve an Atom, returning None if invalid.
    #[inline]
    pub fn try_resolve(&self, atom: Atom) -> Option<&str> {
        self.strings.get(atom.0 as usize).map(|s| s.as_str())
    }

    /// Get the number of interned strings.
    #[inline]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Check if the interner is empty (only has the empty string).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.strings.len() <= 1
    }

    /// Pre-intern common TypeScript keywords and identifiers.
    /// Call this after creating the interner for better cache locality.
    pub fn intern_common(&mut self) {
        for s in COMMON_STRINGS {
            self.intern(s);
        }
    }
}

#[derive(Default)]
struct ShardState {
    map: FxHashMap<Arc<str>, Atom>,
    strings: Vec<Arc<str>>,
}

struct InternerShard {
    state: RwLock<ShardState>,
}

impl InternerShard {
    fn new() -> Self {
        InternerShard {
            state: RwLock::new(ShardState::default()),
        }
    }
}

/// Sharded string interner for concurrent use.
///
/// Uses fixed buckets to reduce lock contention while keeping Atom lookups O(1).
pub struct ShardedInterner {
    shards: [InternerShard; SHARD_COUNT],
}

impl ShardedInterner {
    /// Create a new sharded interner with the empty string pre-interned at index 0.
    pub fn new() -> Self {
        let shards = std::array::from_fn(|_| InternerShard::new());
        {
            let mut state = shards[0].state.write().unwrap();
            let empty: Arc<str> = Arc::from("");
            state.strings.push(empty.clone());
            state.map.insert(empty, Atom::NONE);
        }
        ShardedInterner { shards }
    }

    /// Intern a string, returning its Atom handle.
    /// If the string was already interned, returns the existing Atom.
    #[inline]
    pub fn intern(&self, s: &str) -> Atom {
        if s.is_empty() {
            return Atom::NONE;
        }

        let shard_idx = Self::shard_for(s);
        let shard = &self.shards[shard_idx];
        let mut state = shard.state.write().unwrap();

        if let Some(&atom) = state.map.get(s) {
            return atom;
        }

        let local_index = state.strings.len() as u32;
        if local_index > (u32::MAX >> SHARD_BITS) {
            panic!("ShardedInterner shard {} overflow", shard_idx);
        }

        let atom = Self::make_atom(local_index, shard_idx as u32);
        let owned: Arc<str> = Arc::from(s);
        state.strings.push(owned.clone());
        state.map.insert(owned, atom);
        atom
    }

    /// Intern an owned String, avoiding allocation if possible.
    #[inline]
    pub fn intern_owned(&self, s: String) -> Atom {
        if s.is_empty() {
            return Atom::NONE;
        }

        let shard_idx = Self::shard_for(&s);
        let shard = &self.shards[shard_idx];
        let mut state = shard.state.write().unwrap();

        if let Some(&atom) = state.map.get(s.as_str()) {
            return atom;
        }

        let local_index = state.strings.len() as u32;
        if local_index > (u32::MAX >> SHARD_BITS) {
            panic!("ShardedInterner shard {} overflow", shard_idx);
        }

        let atom = Self::make_atom(local_index, shard_idx as u32);
        let owned: Arc<str> = Arc::from(s);
        state.strings.push(owned.clone());
        state.map.insert(owned, atom);
        atom
    }

    /// Resolve an Atom back to its string value.
    /// Returns empty string if atom is out of bounds (safety for error recovery).
    #[inline]
    pub fn resolve(&self, atom: Atom) -> Arc<str> {
        self.try_resolve(atom).unwrap_or_else(|| Arc::from(""))
    }

    /// Try to resolve an Atom, returning None if invalid.
    #[inline]
    pub fn try_resolve(&self, atom: Atom) -> Option<Arc<str>> {
        let (shard_idx, local_index) = Self::split_atom(atom)?;
        let shard = self.shards.get(shard_idx)?;
        let state = shard.state.read().unwrap();
        state.strings.get(local_index).cloned()
    }

    /// Get the number of interned strings.
    #[inline]
    pub fn len(&self) -> usize {
        self.shards
            .iter()
            .map(|shard| shard.state.read().unwrap().strings.len())
            .sum()
    }

    /// Check if the interner is empty (only has the empty string).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() <= 1
    }

    /// Pre-intern common TypeScript keywords and identifiers.
    /// Call this after creating the interner for better cache locality.
    pub fn intern_common(&self) {
        for s in COMMON_STRINGS {
            self.intern(s);
        }
    }

    #[inline]
    fn shard_for(s: &str) -> usize {
        let mut hasher = FxHasher::default();
        s.hash(&mut hasher);
        (hasher.finish() as usize) & (SHARD_COUNT - 1)
    }

    #[inline]
    fn make_atom(local_index: u32, shard_idx: u32) -> Atom {
        Atom((local_index << SHARD_BITS) | (shard_idx & SHARD_MASK))
    }

    #[inline]
    fn split_atom(atom: Atom) -> Option<(usize, usize)> {
        if atom == Atom::NONE {
            return Some((0, 0));
        }

        let raw = atom.0;
        let shard_idx = (raw & SHARD_MASK) as usize;
        let local_index = (raw >> SHARD_BITS) as usize;
        Some((shard_idx, local_index))
    }
}

impl Default for ShardedInterner {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Arena-backed Interner with DashMap for High-Performance Concurrent Access
// =============================================================================

use ahash::AHasher;
use bumpalo::Bump;
use dashmap::DashMap;
use parking_lot::RwLock as ParkingLotRwLock;

/// High-performance arena-backed string interner with concurrent access support.
///
/// This interner uses:
/// - Bumpalo arena for efficient memory allocation (strings are never deallocated)
/// - DashMap (sharded concurrent hash map) for O(1) concurrent lookups
/// - Zero-allocation lookup path for already-interned strings
///
/// # Performance Characteristics
/// - Interning: O(1) amortized, single allocation for new strings
/// - Lookup: O(1), zero allocations
/// - Resolve: O(1), zero allocations
/// - Memory: Strings stored contiguously in arena with excellent cache locality
///
/// # Thread Safety
/// This interner is fully thread-safe. Multiple threads can concurrently:
/// - Look up existing strings (zero-allocation fast path)
/// - Intern new strings (lock-free via DashMap sharding)
/// - Resolve atoms to strings
pub struct ArenaInterner {
    /// DashMap for concurrent string->atom lookups
    /// Key: hash of string, Value: atom
    map: DashMap<u64, Atom, ahash::RandomState>,

    /// Arena allocator for string storage
    /// Protected by RwLock for rare growth operations
    arena: ParkingLotRwLock<Bump>,

    /// Vector of interned string pointers (atom index -> &str)
    /// Protected by RwLock for thread-safe growth
    strings: ParkingLotRwLock<Vec<(*const u8, usize)>>,

    /// Counter for generating unique atom IDs
    next_id: std::sync::atomic::AtomicU32,
}

// SAFETY: The arena contains only string data which is immutable after allocation.
// The pointers in `strings` are valid for the lifetime of the arena.
unsafe impl Send for ArenaInterner {}
unsafe impl Sync for ArenaInterner {}

impl ArenaInterner {
    /// Create a new arena-backed interner.
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    /// Create a new arena-backed interner with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let interner = ArenaInterner {
            map: DashMap::with_capacity_and_hasher(capacity, ahash::RandomState::new()),
            arena: ParkingLotRwLock::new(Bump::with_capacity(capacity * 16)), // ~16 bytes per string average
            strings: ParkingLotRwLock::new(Vec::with_capacity(capacity)),
            next_id: std::sync::atomic::AtomicU32::new(1), // Start at 1, 0 is reserved for NONE
        };

        // Pre-intern empty string at index 0
        {
            let arena = interner.arena.write();
            let empty_str: &str = arena.alloc_str("");
            let ptr = empty_str.as_ptr();
            let len = empty_str.len();
            drop(arena);

            let mut strings = interner.strings.write();
            strings.push((ptr, len));
        }

        // Pre-intern common strings for better cache locality
        for s in COMMON_STRINGS {
            let _ = interner.intern(s);
        }

        interner
    }

    /// Compute a hash for the string using ahash.
    #[inline]
    fn hash_string(s: &str) -> u64 {
        let mut hasher = AHasher::default();
        hasher.write(s.as_bytes());
        hasher.finish()
    }

    /// Intern a string, returning its Atom handle.
    ///
    /// If the string is already interned, returns the existing Atom without allocation.
    /// This is the zero-allocation fast path that makes repeated interning efficient.
    #[inline]
    pub fn intern(&self, s: &str) -> Atom {
        if s.is_empty() {
            return Atom::NONE;
        }

        let hash = Self::hash_string(s);

        // Fast path: check if already interned (zero allocation)
        if let Some(entry) = self.map.get(&hash) {
            let atom = *entry;
            // Verify string match to handle hash collisions
            if self.resolve_unchecked(atom) == s {
                return atom;
            }
        }

        // Slow path: need to intern the string
        self.intern_slow(s, hash)
    }

    /// Slow path for interning a new string.
    #[cold]
    fn intern_slow(&self, s: &str, hash: u64) -> Atom {
        // Use entry API for atomic insert-or-get
        let entry = self.map.entry(hash);

        match entry {
            dashmap::mapref::entry::Entry::Occupied(e) => {
                let atom = *e.get();
                // Verify string match (hash collision case)
                if self.resolve_unchecked(atom) == s {
                    return atom;
                }
                // Hash collision - use secondary probing
                self.intern_with_collision(s, hash)
            }
            dashmap::mapref::entry::Entry::Vacant(e) => {
                let atom = self.allocate_string(s);
                e.insert(atom);
                atom
            }
        }
    }

    /// Handle hash collisions with linear probing.
    fn intern_with_collision(&self, s: &str, original_hash: u64) -> Atom {
        let mut probe = original_hash.wrapping_add(1);
        loop {
            match self.map.entry(probe) {
                dashmap::mapref::entry::Entry::Occupied(e) => {
                    let atom = *e.get();
                    if self.resolve_unchecked(atom) == s {
                        return atom;
                    }
                    probe = probe.wrapping_add(1);
                }
                dashmap::mapref::entry::Entry::Vacant(e) => {
                    let atom = self.allocate_string(s);
                    e.insert(atom);
                    return atom;
                }
            }
        }
    }

    /// Allocate a string in the arena and return its Atom.
    fn allocate_string(&self, s: &str) -> Atom {
        // Allocate in arena
        let arena = self.arena.write();
        let allocated: &str = arena.alloc_str(s);
        let ptr = allocated.as_ptr();
        let len = allocated.len();
        drop(arena);

        // Get next atom ID
        let index = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Store pointer
        let mut strings = self.strings.write();
        if strings.len() <= index as usize {
            strings.resize(index as usize + 1, (std::ptr::null(), 0));
        }
        strings[index as usize] = (ptr, len);

        Atom(index)
    }

    /// Resolve an Atom to its string slice without bounds checking.
    #[inline]
    fn resolve_unchecked(&self, atom: Atom) -> &str {
        let strings = self.strings.read();
        let (ptr, len) = strings[atom.0 as usize];
        // SAFETY: ptr and len came from a valid &str allocated in our arena
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
    }

    /// Resolve an Atom back to its string value.
    ///
    /// Returns empty string if atom is out of bounds (safety for error recovery).
    #[inline]
    pub fn resolve(&self, atom: Atom) -> &str {
        if atom.is_none() {
            return "";
        }
        let strings = self.strings.read();
        if (atom.0 as usize) >= strings.len() {
            return "";
        }
        let (ptr, len) = strings[atom.0 as usize];
        if ptr.is_null() {
            return "";
        }
        // SAFETY: ptr and len came from a valid &str allocated in our arena
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) }
    }

    /// Look up a string without interning it (zero allocation).
    ///
    /// Returns `Some(Atom)` if the string is already interned, `None` otherwise.
    /// This never allocates memory.
    #[inline]
    pub fn lookup(&self, s: &str) -> Option<Atom> {
        if s.is_empty() {
            return Some(Atom::NONE);
        }

        let hash = Self::hash_string(s);

        if let Some(entry) = self.map.get(&hash) {
            let atom = *entry;
            if self.resolve_unchecked(atom) == s {
                return Some(atom);
            }
        }

        // Check collision chain
        let mut probe = hash.wrapping_add(1);
        for _ in 0..16 {
            // Limit probing to avoid infinite loops
            if let Some(entry) = self.map.get(&probe) {
                let atom = *entry;
                if self.resolve_unchecked(atom) == s {
                    return Some(atom);
                }
                probe = probe.wrapping_add(1);
            } else {
                break;
            }
        }

        None
    }

    /// Get the number of interned strings (including empty string).
    pub fn len(&self) -> usize {
        self.next_id
            .load(std::sync::atomic::Ordering::Relaxed) as usize
    }

    /// Check if the interner is empty (only has the empty string).
    pub fn is_empty(&self) -> bool {
        self.len() <= 1
    }

    /// Pre-intern common TypeScript keywords and identifiers.
    pub fn intern_common(&self) {
        for s in COMMON_STRINGS {
            self.intern(s);
        }
    }
}

impl Default for ArenaInterner {
    fn default() -> Self {
        Self::new()
    }
}
