//! String interning for efficient symbol handling
//!
//! Provides a concurrent string interner that deduplicates strings and
//! returns cheap-to-copy handles.

use dashmap::DashMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::sync::atomic::{AtomicU32, Ordering};

/// A handle to an interned string
///
/// This is a cheap-to-copy identifier that can be used to look up the
/// original string in the interner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct InternedString(u32);

impl InternedString {
    /// Returns the raw ID of this interned string
    pub fn as_u32(self) -> u32 {
        self.0
    }

    /// Creates an InternedString from a raw ID
    ///
    /// # Safety
    /// The caller must ensure the ID was obtained from a valid interning operation.
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }
}

type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;

/// A concurrent string interner
///
/// This deduplicates strings and returns handles that can be used to
/// look up the original strings. Thread-safe for concurrent access.
pub struct StringInterner {
    /// Map from string content to interned ID
    string_to_id: FxDashMap<String, InternedString>,
    /// Map from interned ID to string content
    id_to_string: DashMap<u32, String, BuildHasherDefault<FxHasher>>,
    /// Next ID to assign
    next_id: AtomicU32,
}

impl StringInterner {
    /// Creates a new string interner
    pub fn new() -> Self {
        Self {
            string_to_id: FxDashMap::default(),
            id_to_string: DashMap::default(),
            next_id: AtomicU32::new(0),
        }
    }

    /// Interns a string, returning a handle to it
    ///
    /// If the string was already interned, returns the existing handle.
    pub fn intern(&self, s: &str) -> InternedString {
        // Check if already interned
        if let Some(id) = self.string_to_id.get(s) {
            return *id;
        }

        // Allocate new ID
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let interned = InternedString(id);

        // Insert into both maps
        let owned = s.to_owned();
        self.string_to_id.insert(owned.clone(), interned);
        self.id_to_string.insert(id, owned);

        interned
    }

    /// Looks up an interned string by its handle
    ///
    /// Returns None if the handle is invalid.
    pub fn lookup(&self, id: InternedString) -> Option<String> {
        self.id_to_string.get(&id.0).map(|r| r.clone())
    }

    /// Returns the number of interned strings
    pub fn len(&self) -> usize {
        self.string_to_id.len()
    }

    /// Returns true if no strings have been interned
    pub fn is_empty(&self) -> bool {
        self.string_to_id.is_empty()
    }

    /// Pre-interns common TypeScript keywords for efficiency
    pub fn intern_keywords(&self) -> KeywordIds {
        KeywordIds {
            r#abstract: self.intern("abstract"),
            r#any: self.intern("any"),
            r#as: self.intern("as"),
            r#async: self.intern("async"),
            r#await: self.intern("await"),
            r#boolean: self.intern("boolean"),
            r#break: self.intern("break"),
            r#case: self.intern("case"),
            r#catch: self.intern("catch"),
            r#class: self.intern("class"),
            r#const: self.intern("const"),
            r#continue: self.intern("continue"),
            r#debugger: self.intern("debugger"),
            r#declare: self.intern("declare"),
            r#default: self.intern("default"),
            r#delete: self.intern("delete"),
            r#do: self.intern("do"),
            r#else: self.intern("else"),
            r#enum: self.intern("enum"),
            r#export: self.intern("export"),
            r#extends: self.intern("extends"),
            r#false: self.intern("false"),
            r#finally: self.intern("finally"),
            r#for: self.intern("for"),
            r#from: self.intern("from"),
            r#function: self.intern("function"),
            r#get: self.intern("get"),
            r#if: self.intern("if"),
            r#implements: self.intern("implements"),
            r#import: self.intern("import"),
            r#in: self.intern("in"),
            r#infer: self.intern("infer"),
            r#instanceof: self.intern("instanceof"),
            r#interface: self.intern("interface"),
            r#is: self.intern("is"),
            r#keyof: self.intern("keyof"),
            r#let: self.intern("let"),
            r#module: self.intern("module"),
            r#namespace: self.intern("namespace"),
            r#never: self.intern("never"),
            r#new: self.intern("new"),
            r#null: self.intern("null"),
            r#number: self.intern("number"),
            r#object: self.intern("object"),
            r#of: self.intern("of"),
            r#package: self.intern("package"),
            r#private: self.intern("private"),
            r#protected: self.intern("protected"),
            r#public: self.intern("public"),
            r#readonly: self.intern("readonly"),
            r#require: self.intern("require"),
            r#return: self.intern("return"),
            r#set: self.intern("set"),
            r#static: self.intern("static"),
            r#string: self.intern("string"),
            super_kw: self.intern("super"),
            r#switch: self.intern("switch"),
            r#symbol: self.intern("symbol"),
            r#this: self.intern("this"),
            r#throw: self.intern("throw"),
            r#true: self.intern("true"),
            r#try: self.intern("try"),
            r#type: self.intern("type"),
            r#typeof: self.intern("typeof"),
            r#undefined: self.intern("undefined"),
            r#unique: self.intern("unique"),
            r#unknown: self.intern("unknown"),
            r#var: self.intern("var"),
            r#void: self.intern("void"),
            r#while: self.intern("while"),
            r#with: self.intern("with"),
            r#yield: self.intern("yield"),
        }
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-interned keyword IDs for fast lookup
#[derive(Clone, Copy, Debug)]
pub struct KeywordIds {
    pub r#abstract: InternedString,
    pub r#any: InternedString,
    pub r#as: InternedString,
    pub r#async: InternedString,
    pub r#await: InternedString,
    pub r#boolean: InternedString,
    pub r#break: InternedString,
    pub r#case: InternedString,
    pub r#catch: InternedString,
    pub r#class: InternedString,
    pub r#const: InternedString,
    pub r#continue: InternedString,
    pub r#debugger: InternedString,
    pub r#declare: InternedString,
    pub r#default: InternedString,
    pub r#delete: InternedString,
    pub r#do: InternedString,
    pub r#else: InternedString,
    pub r#enum: InternedString,
    pub r#export: InternedString,
    pub r#extends: InternedString,
    pub r#false: InternedString,
    pub r#finally: InternedString,
    pub r#for: InternedString,
    pub r#from: InternedString,
    pub r#function: InternedString,
    pub r#get: InternedString,
    pub r#if: InternedString,
    pub r#implements: InternedString,
    pub r#import: InternedString,
    pub r#in: InternedString,
    pub r#infer: InternedString,
    pub r#instanceof: InternedString,
    pub r#interface: InternedString,
    pub r#is: InternedString,
    pub r#keyof: InternedString,
    pub r#let: InternedString,
    pub r#module: InternedString,
    pub r#namespace: InternedString,
    pub r#never: InternedString,
    pub r#new: InternedString,
    pub r#null: InternedString,
    pub r#number: InternedString,
    pub r#object: InternedString,
    pub r#of: InternedString,
    pub r#package: InternedString,
    pub r#private: InternedString,
    pub r#protected: InternedString,
    pub r#public: InternedString,
    pub r#readonly: InternedString,
    pub r#require: InternedString,
    pub r#return: InternedString,
    pub r#set: InternedString,
    pub r#static: InternedString,
    pub r#string: InternedString,
    pub super_kw: InternedString,
    pub r#switch: InternedString,
    pub r#symbol: InternedString,
    pub r#this: InternedString,
    pub r#throw: InternedString,
    pub r#true: InternedString,
    pub r#try: InternedString,
    pub r#type: InternedString,
    pub r#typeof: InternedString,
    pub r#undefined: InternedString,
    pub r#unique: InternedString,
    pub r#unknown: InternedString,
    pub r#var: InternedString,
    pub r#void: InternedString,
    pub r#while: InternedString,
    pub r#with: InternedString,
    pub r#yield: InternedString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_basic() {
        let interner = StringInterner::new();
        let id1 = interner.intern("hello");
        let id2 = interner.intern("hello");
        let id3 = interner.intern("world");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_lookup() {
        let interner = StringInterner::new();
        let id = interner.intern("test");
        let s = interner.lookup(id);
        assert_eq!(s, Some("test".to_string()));
    }

    #[test]
    fn test_len() {
        let interner = StringInterner::new();
        assert_eq!(interner.len(), 0);
        interner.intern("a");
        assert_eq!(interner.len(), 1);
        interner.intern("b");
        assert_eq!(interner.len(), 2);
        interner.intern("a"); // duplicate
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_keywords() {
        let interner = StringInterner::new();
        let keywords = interner.intern_keywords();

        assert_eq!(interner.lookup(keywords.r#function), Some("function".to_string()));
        assert_eq!(interner.lookup(keywords.r#const), Some("const".to_string()));
    }
}
