//! Arena-based memory allocation for symbols
//!
//! This module provides an arena allocator for efficient symbol storage.
//! All symbols created during binding are stored in a single arena,
//! which allows for:
//! - Fast allocation (just bump a pointer)
//! - Cache-friendly iteration
//! - Simple deallocation (free the whole arena at once)

use std::cell::RefCell;

/// A simple arena allocator for storing values of type T
pub struct Arena<T> {
    chunks: RefCell<Vec<Vec<T>>>,
    chunk_size: usize,
}

impl<T> Arena<T> {
    /// Create a new arena with the default chunk size
    pub fn new() -> Self {
        Self::with_chunk_size(1024)
    }

    /// Create a new arena with a specific chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Arena {
            chunks: RefCell::new(vec![Vec::with_capacity(chunk_size)]),
            chunk_size,
        }
    }

    /// Allocate a new value in the arena and return its index
    pub fn alloc(&self, value: T) -> ArenaId {
        let mut chunks = self.chunks.borrow_mut();
        let last_chunk_idx = chunks.len() - 1;

        // Check if we need a new chunk
        if chunks[last_chunk_idx].len() >= self.chunk_size {
            chunks.push(Vec::with_capacity(self.chunk_size));
        }

        let chunk_idx = chunks.len() - 1;
        let item_idx = chunks[chunk_idx].len();
        chunks[chunk_idx].push(value);

        ArenaId {
            chunk: chunk_idx as u32,
            index: item_idx as u32,
        }
    }

    /// Get a reference to a value by its arena ID
    pub fn get(&self, id: ArenaId) -> Option<std::cell::Ref<'_, T>> {
        let chunks = self.chunks.borrow();
        if (id.chunk as usize) < chunks.len() && (id.index as usize) < chunks[id.chunk as usize].len()
        {
            Some(std::cell::Ref::map(chunks, |c| {
                &c[id.chunk as usize][id.index as usize]
            }))
        } else {
            None
        }
    }

    /// Get a mutable reference to a value by its arena ID
    pub fn get_mut(&self, id: ArenaId) -> Option<std::cell::RefMut<'_, T>> {
        let chunks = self.chunks.borrow_mut();
        if (id.chunk as usize) < chunks.len() && (id.index as usize) < chunks[id.chunk as usize].len()
        {
            Some(std::cell::RefMut::map(chunks, |c| {
                &mut c[id.chunk as usize][id.index as usize]
            }))
        } else {
            None
        }
    }

    /// Get the total number of items in the arena
    pub fn len(&self) -> usize {
        let chunks = self.chunks.borrow();
        if chunks.is_empty() {
            0
        } else {
            (chunks.len() - 1) * self.chunk_size + chunks.last().map_or(0, |c| c.len())
        }
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items from the arena
    pub fn clear(&self) {
        let mut chunks = self.chunks.borrow_mut();
        chunks.clear();
        chunks.push(Vec::with_capacity(self.chunk_size));
    }

    /// Iterate over all items in the arena
    pub fn iter(&self) -> ArenaIter<'_, T> {
        ArenaIter {
            chunks: self.chunks.borrow(),
            chunk_idx: 0,
            item_idx: 0,
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// An identifier for an item in the arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaId {
    chunk: u32,
    index: u32,
}

impl ArenaId {
    /// Create an invalid/null arena ID
    pub const fn null() -> Self {
        ArenaId {
            chunk: u32::MAX,
            index: u32::MAX,
        }
    }

    /// Check if this is a null/invalid ID
    pub fn is_null(&self) -> bool {
        self.chunk == u32::MAX && self.index == u32::MAX
    }

    /// Convert to a raw index (for use as a unique identifier)
    pub fn as_raw(&self) -> u64 {
        ((self.chunk as u64) << 32) | (self.index as u64)
    }
}

/// Iterator over items in an arena
pub struct ArenaIter<'a, T> {
    chunks: std::cell::Ref<'a, Vec<Vec<T>>>,
    chunk_idx: usize,
    item_idx: usize,
}

impl<'a, T: Clone> Iterator for ArenaIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.chunk_idx < self.chunks.len() {
            if self.item_idx < self.chunks[self.chunk_idx].len() {
                let item = self.chunks[self.chunk_idx][self.item_idx].clone();
                self.item_idx += 1;
                return Some(item);
            }
            self.chunk_idx += 1;
            self.item_idx = 0;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc_and_get() {
        let arena: Arena<i32> = Arena::new();
        let id1 = arena.alloc(42);
        let id2 = arena.alloc(100);

        assert_eq!(*arena.get(id1).unwrap(), 42);
        assert_eq!(*arena.get(id2).unwrap(), 100);
    }

    #[test]
    fn test_arena_len() {
        let arena: Arena<i32> = Arena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());

        arena.alloc(1);
        arena.alloc(2);
        arena.alloc(3);

        assert_eq!(arena.len(), 3);
        assert!(!arena.is_empty());
    }

    #[test]
    fn test_arena_id_null() {
        let id = ArenaId::null();
        assert!(id.is_null());

        let id2 = ArenaId { chunk: 0, index: 0 };
        assert!(!id2.is_null());
    }

    #[test]
    fn test_arena_clear() {
        let arena: Arena<i32> = Arena::new();
        arena.alloc(1);
        arena.alloc(2);

        assert_eq!(arena.len(), 2);

        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }
}
