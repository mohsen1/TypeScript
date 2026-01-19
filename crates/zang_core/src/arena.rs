//! Arena allocation for efficient memory management
//!
//! Uses bumpalo for fast, arena-based allocation of AST nodes and types.

use bumpalo::Bump;
use std::cell::RefCell;

/// Arena allocator for AST nodes and types
///
/// This provides a thread-local arena for allocating nodes that share a lifetime.
/// All allocations are freed together when the arena is dropped.
pub struct Arena {
    bump: RefCell<Bump>,
}

impl Arena {
    /// Creates a new arena
    pub fn new() -> Self {
        Self {
            bump: RefCell::new(Bump::new()),
        }
    }

    /// Creates a new arena with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: RefCell::new(Bump::with_capacity(capacity)),
        }
    }

    /// Allocates a value in the arena
    pub fn alloc<T>(&self, value: T) -> &T {
        // Safety: We're returning a reference with the arena's lifetime
        let bump = self.bump.borrow();
        let ptr = bump.alloc(value) as *const T;
        // Safety: The reference is valid for the lifetime of the arena
        unsafe { &*ptr }
    }

    /// Allocates a slice in the arena
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &[T] {
        let bump = self.bump.borrow();
        let ptr = bump.alloc_slice_copy(slice) as *const [T];
        unsafe { &*ptr }
    }

    /// Allocates a string in the arena
    pub fn alloc_str(&self, s: &str) -> &str {
        let bump = self.bump.borrow();
        let ptr = bump.alloc_str(s) as *const str;
        unsafe { &*ptr }
    }

    /// Returns the number of bytes allocated in this arena
    pub fn allocated_bytes(&self) -> usize {
        self.bump.borrow().allocated_bytes()
    }

    /// Resets the arena, deallocating all allocations
    pub fn reset(&self) {
        self.bump.borrow_mut().reset();
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let arena = Arena::new();
        let x = arena.alloc(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_arena_alloc_str() {
        let arena = Arena::new();
        let s = arena.alloc_str("hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_arena_alloc_slice() {
        let arena = Arena::new();
        let slice = arena.alloc_slice(&[1, 2, 3]);
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new();
        arena.alloc(42);
        let before = arena.allocated_bytes();
        arena.reset();
        let after = arena.allocated_bytes();
        assert!(after <= before);
    }
}
