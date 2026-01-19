//! Tests for interner.rs

use crate::interner::*;

#[test]
fn test_intern_basic() {
    let mut interner = Interner::new();
    let a1 = interner.intern("hello");
    let a2 = interner.intern("hello");
    let a3 = interner.intern("world");

    assert_eq!(a1, a2, "Same string should return same atom");
    assert_ne!(a1, a3, "Different strings should return different atoms");
    assert_eq!(interner.resolve(a1), "hello");
    assert_eq!(interner.resolve(a3), "world");
}

#[test]
fn test_empty_string() {
    let mut interner = Interner::new();
    let empty = interner.intern("");
    assert_eq!(empty, Atom::NONE);
    assert!(empty.is_none());
    assert_eq!(interner.resolve(empty), "");
}

#[test]
fn test_intern_common() {
    let mut interner = Interner::new();
    interner.intern_common();

    // Common keywords should be interned
    let const_atom = interner.intern("const");
    let let_atom = interner.intern("let");
    assert_ne!(const_atom, let_atom);

    // Should be able to resolve them
    assert_eq!(interner.resolve(const_atom), "const");
    assert_eq!(interner.resolve(let_atom), "let");
}

#[test]
fn test_atom_copy() {
    let mut interner = Interner::new();
    let a1 = interner.intern("test");
    let a2 = a1; // Copy
    assert_eq!(a1, a2);
}

#[test]
fn test_sharded_interner_basic() {
    let interner = ShardedInterner::new();
    let a1 = interner.intern("hello");
    let a2 = interner.intern("hello");
    let a3 = interner.intern("world");

    assert_eq!(a1, a2, "Same string should return same atom");
    assert_ne!(a1, a3, "Different strings should return different atoms");
    assert_eq!(interner.resolve(a1).as_ref(), "hello");
    assert_eq!(interner.resolve(a3).as_ref(), "world");
}

#[test]
fn test_sharded_interner_empty_string() {
    let interner = ShardedInterner::new();
    let empty = interner.intern("");
    assert_eq!(empty, Atom::NONE);
    assert!(empty.is_none());
    assert_eq!(interner.resolve(empty).as_ref(), "");
    assert_eq!(interner.try_resolve(empty).as_deref(), Some(""));
}

#[test]
fn test_sharded_interner_concurrent() {
    use std::sync::Arc;
    use std::thread;

    let interner = Arc::new(ShardedInterner::new());
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let interner = Arc::clone(&interner);
            thread::spawn(move || interner.intern("parallel"))
        })
        .collect();

    let atoms: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread failed"))
        .collect();

    assert!(!atoms.is_empty());
    assert!(atoms.iter().all(|&atom| atom == atoms[0]));
    assert_eq!(interner.resolve(atoms[0]).as_ref(), "parallel");
}

#[test]
fn test_sharded_interner_try_resolve_invalid() {
    let interner = ShardedInterner::new();
    assert_eq!(interner.try_resolve(Atom(u32::MAX)), None);
}

// =============================================================================
// ArenaInterner Tests
// =============================================================================

#[test]
fn test_arena_interner_basic() {
    let interner = ArenaInterner::new();
    let a1 = interner.intern("hello");
    let a2 = interner.intern("hello");
    let a3 = interner.intern("world");

    assert_eq!(a1, a2, "Same string should return same atom");
    assert_ne!(a1, a3, "Different strings should return different atoms");
    assert_eq!(interner.resolve(a1), "hello");
    assert_eq!(interner.resolve(a3), "world");
}

#[test]
fn test_arena_interner_empty_string() {
    let interner = ArenaInterner::new();
    let empty = interner.intern("");
    assert_eq!(empty, Atom::NONE);
    assert!(empty.is_none());
    assert_eq!(interner.resolve(empty), "");
}

#[test]
fn test_arena_interner_zero_allocation_lookup() {
    let interner = ArenaInterner::new();

    // First intern
    let id1 = interner.intern("test_string");

    // Lookup should find it without allocation
    let found = interner.lookup("test_string");
    assert_eq!(found, Some(id1));

    // Lookup for non-existent string
    let not_found = interner.lookup("nonexistent_xyz_12345");
    assert_eq!(not_found, None);
}

#[test]
fn test_arena_interner_unicode() {
    let interner = ArenaInterner::new();

    let a1 = interner.intern("héllo");
    let a2 = interner.intern("世界");
    let a3 = interner.intern("🦀");

    assert_eq!(interner.resolve(a1), "héllo");
    assert_eq!(interner.resolve(a2), "世界");
    assert_eq!(interner.resolve(a3), "🦀");
}

#[test]
fn test_arena_interner_concurrent_basic() {
    use std::sync::Arc;
    use std::thread;

    let interner = Arc::new(ArenaInterner::new());
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let interner = Arc::clone(&interner);
            thread::spawn(move || interner.intern("concurrent_test"))
        })
        .collect();

    let atoms: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("thread failed"))
        .collect();

    // All threads should get the same atom for the same string
    assert!(!atoms.is_empty());
    assert!(atoms.iter().all(|&atom| atom == atoms[0]));
    assert_eq!(interner.resolve(atoms[0]), "concurrent_test");
}

#[test]
fn test_arena_interner_concurrent_many_strings() {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    let interner = Arc::new(ArenaInterner::new());
    let mut handles = vec![];

    // Spawn multiple threads that intern overlapping strings
    for thread_id in 0..8 {
        let interner = Arc::clone(&interner);
        let handle = thread::spawn(move || {
            let mut results = vec![];
            for i in 0..1000 {
                // Use overlapping string patterns to test concurrent access
                let s = format!("string_{}_{}", thread_id % 4, i % 100);
                let atom = interner.intern(&s);
                results.push((s, atom));
            }
            results
        });
        handles.push(handle);
    }

    // Collect all results
    let all_results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify consistency: same string should have same atom across all threads
    let mut string_to_atom: HashMap<String, Atom> = HashMap::new();
    for results in all_results {
        for (s, atom) in results {
            if let Some(&existing_atom) = string_to_atom.get(&s) {
                assert_eq!(
                    existing_atom, atom,
                    "Same string got different atoms: {}",
                    s
                );
            } else {
                string_to_atom.insert(s.clone(), atom);
            }

            // Also verify resolution works
            assert_eq!(interner.resolve(atom), s);
        }
    }
}

#[test]
fn test_arena_interner_concurrent_read_heavy() {
    use std::sync::Arc;
    use std::thread;

    let interner = Arc::new(ArenaInterner::new());

    // Pre-populate with some strings
    let mut pre_interned = vec![];
    for i in 0..100 {
        let s = format!("preinterned_{}", i);
        let atom = interner.intern(&s);
        pre_interned.push((s, atom));
    }

    let mut handles = vec![];

    // Spawn reader threads (zero-allocation lookups)
    for _ in 0..4 {
        let interner = Arc::clone(&interner);
        let pre_interned = pre_interned.clone();
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                for (s, expected_atom) in &pre_interned {
                    // Zero-allocation lookup
                    let atom = interner.lookup(s);
                    assert_eq!(atom, Some(*expected_atom));
                }
            }
        });
        handles.push(handle);
    }

    // Spawn writer threads (interning new strings)
    for thread_id in 0..2 {
        let interner = Arc::clone(&interner);
        let handle = thread::spawn(move || {
            for i in 0..500 {
                let s = format!("new_string_{}_{}", thread_id, i);
                let atom = interner.intern(&s);
                assert_eq!(interner.resolve(atom), s);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_arena_interner_len() {
    let interner = ArenaInterner::new();
    let initial_len = interner.len();

    // Intern some new strings (not already in common strings)
    interner.intern("unique_test_string_a");
    assert_eq!(interner.len(), initial_len + 1);

    interner.intern("unique_test_string_b");
    assert_eq!(interner.len(), initial_len + 2);

    // Duplicate should not increase length
    interner.intern("unique_test_string_a");
    assert_eq!(interner.len(), initial_len + 2);
}
