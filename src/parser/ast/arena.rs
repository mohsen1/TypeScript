//! AST Arena - Bump allocator based storage for AST nodes
//!
//! Uses bumpalo for cache-efficient arena allocation with Structure of Arrays (SoA) layout.

use bumpalo::Bump;
use std::cell::Cell;

/// Compact node index - 4 bytes instead of 8-byte pointers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const NONE: NodeId = NodeId(u32::MAX);

    #[inline]
    pub const fn new(index: u32) -> Self {
        NodeId(index)
    }

    #[inline]
    pub const fn is_some(self) -> bool {
        self.0 != u32::MAX
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl From<u32> for NodeId {
    #[inline]
    fn from(value: u32) -> Self {
        NodeId(value)
    }
}

impl From<usize> for NodeId {
    #[inline]
    fn from(value: usize) -> Self {
        NodeId(value as u32)
    }
}

/// String interned ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct StringId(pub u32);

impl StringId {
    pub const EMPTY: StringId = StringId(0);

    #[inline]
    pub const fn new(index: u32) -> Self {
        StringId(index)
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Source span - 8 bytes total
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Span { start, end }
    }

    #[inline]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub const fn contains(self, pos: u32) -> bool {
        pos >= self.start && pos < self.end
    }
}

/// Children reference - points to contiguous children in arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ChildList {
    pub start: u32,
    pub len: u16,
    _pad: u16,
}

impl ChildList {
    pub const EMPTY: ChildList = ChildList { start: 0, len: 0, _pad: 0 };

    #[inline]
    pub const fn new(start: u32, len: u16) -> Self {
        ChildList { start, len, _pad: 0 }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }
}

/// String interner for deduplication
pub struct StringInterner {
    data: String,
    entries: Vec<(u32, u32)>, // (offset, len)
}

impl StringInterner {
    pub fn new() -> Self {
        let mut interner = StringInterner {
            data: String::new(),
            entries: Vec::new(),
        };
        // Reserve index 0 for empty string
        interner.data.push('\0');
        interner.entries.push((0, 0));
        interner
    }

    pub fn intern(&mut self, s: &str) -> StringId {
        if s.is_empty() {
            return StringId::EMPTY;
        }

        // Simple linear search for deduplication (can optimize with HashMap later)
        for (i, &(offset, len)) in self.entries.iter().enumerate().skip(1) {
            let existing = &self.data[offset as usize..(offset + len) as usize];
            if existing == s {
                return StringId::new(i as u32);
            }
        }

        let offset = self.data.len() as u32;
        let len = s.len() as u32;
        self.data.push_str(s);
        let id = StringId::new(self.entries.len() as u32);
        self.entries.push((offset, len));
        id
    }

    pub fn get(&self, id: StringId) -> Option<&str> {
        if id.is_empty() {
            return Some("");
        }
        let (offset, len) = *self.entries.get(id.0 as usize)?;
        self.data.get(offset as usize..(offset + len) as usize)
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.data.push('\0');
        self.entries.clear();
        self.entries.push((0, 0));
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// The main AST Arena using bumpalo for efficient allocation
/// Uses Structure of Arrays (SoA) for cache-efficient traversal
pub struct AstArena {
    /// Bump allocator for variable-sized data
    bump: Bump,

    /// Node kinds - parallel array
    pub kinds: Vec<u16>,

    /// Node flags - parallel array
    pub flags: Vec<u16>,

    /// Modifier flags - parallel array
    pub modifiers: Vec<u32>,

    /// Source spans - parallel array
    pub spans: Vec<Span>,

    /// Parent nodes - parallel array
    pub parents: Vec<NodeId>,

    /// Children lists - parallel array
    pub children: Vec<ChildList>,

    /// String IDs (identifiers, literals) - parallel array
    pub string_ids: Vec<StringId>,

    /// Extra data indices (type-specific) - parallel array
    pub extra: Vec<u32>,

    /// Flattened children storage
    pub child_nodes: Vec<NodeId>,

    /// String interner
    pub strings: StringInterner,

    /// Counter for node allocation
    node_count: Cell<u32>,
}

impl AstArena {
    pub fn new() -> Self {
        AstArena {
            bump: Bump::new(),
            kinds: Vec::new(),
            flags: Vec::new(),
            modifiers: Vec::new(),
            spans: Vec::new(),
            parents: Vec::new(),
            children: Vec::new(),
            string_ids: Vec::new(),
            extra: Vec::new(),
            child_nodes: Vec::new(),
            strings: StringInterner::new(),
            node_count: Cell::new(0),
        }
    }

    pub fn with_capacity(node_cap: usize, children_cap: usize) -> Self {
        AstArena {
            bump: Bump::with_capacity(node_cap * 64),
            kinds: Vec::with_capacity(node_cap),
            flags: Vec::with_capacity(node_cap),
            modifiers: Vec::with_capacity(node_cap),
            spans: Vec::with_capacity(node_cap),
            parents: Vec::with_capacity(node_cap),
            children: Vec::with_capacity(node_cap),
            string_ids: Vec::with_capacity(node_cap),
            extra: Vec::with_capacity(node_cap),
            child_nodes: Vec::with_capacity(children_cap),
            strings: StringInterner::new(),
            node_count: Cell::new(0),
        }
    }

    /// Allocate a new node, returns its ID
    pub fn alloc_node(&mut self, kind: u16, span: Span) -> NodeId {
        let id = NodeId::new(self.kinds.len() as u32);
        self.kinds.push(kind);
        self.flags.push(0);
        self.modifiers.push(0);
        self.spans.push(span);
        self.parents.push(NodeId::NONE);
        self.children.push(ChildList::EMPTY);
        self.string_ids.push(StringId::EMPTY);
        self.extra.push(0);
        self.node_count.set(self.node_count.get() + 1);
        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    #[inline]
    pub fn kind(&self, id: NodeId) -> Option<u16> {
        self.kinds.get(id.index()).copied()
    }

    #[inline]
    pub fn span(&self, id: NodeId) -> Option<Span> {
        self.spans.get(id.index()).copied()
    }

    #[inline]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.parents.get(id.index()).copied()
    }

    #[inline]
    pub fn set_parent(&mut self, id: NodeId, parent: NodeId) {
        if let Some(p) = self.parents.get_mut(id.index()) {
            *p = parent;
        }
    }

    #[inline]
    pub fn set_flags(&mut self, id: NodeId, flags: u16) {
        if let Some(f) = self.flags.get_mut(id.index()) {
            *f = flags;
        }
    }

    #[inline]
    pub fn add_flags(&mut self, id: NodeId, flags: u16) {
        if let Some(f) = self.flags.get_mut(id.index()) {
            *f |= flags;
        }
    }

    #[inline]
    pub fn set_modifiers(&mut self, id: NodeId, modifiers: u32) {
        if let Some(m) = self.modifiers.get_mut(id.index()) {
            *m = modifiers;
        }
    }

    #[inline]
    pub fn set_string(&mut self, id: NodeId, string_id: StringId) {
        if let Some(s) = self.string_ids.get_mut(id.index()) {
            *s = string_id;
        }
    }

    #[inline]
    pub fn set_extra(&mut self, id: NodeId, extra: u32) {
        if let Some(e) = self.extra.get_mut(id.index()) {
            *e = extra;
        }
    }

    /// Add children to a node
    pub fn add_children(&mut self, id: NodeId, node_children: &[NodeId]) {
        if id.is_none() || node_children.is_empty() {
            return;
        }

        let idx = id.index();
        if idx >= self.children.len() {
            return;
        }

        let start = self.child_nodes.len() as u32;
        let len = node_children.len().min(u16::MAX as usize) as u16;

        self.child_nodes.extend_from_slice(&node_children[..len as usize]);
        self.children[idx] = ChildList::new(start, len);

        // Set parent for all children
        for &child_id in &node_children[..len as usize] {
            self.set_parent(child_id, id);
        }
    }

    /// Get children of a node
    pub fn get_children(&self, id: NodeId) -> &[NodeId] {
        if id.is_none() {
            return &[];
        }

        let idx = id.index();
        if let Some(child_list) = self.children.get(idx) {
            if child_list.is_empty() {
                return &[];
            }
            let start = child_list.start as usize;
            let end = start + child_list.len as usize;
            if end <= self.child_nodes.len() {
                return &self.child_nodes[start..end];
            }
        }
        &[]
    }

    /// Intern a string
    pub fn intern(&mut self, s: &str) -> StringId {
        self.strings.intern(s)
    }

    /// Get interned string
    pub fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id)
    }

    /// Allocate variable-sized data in bump arena
    pub fn alloc_slice<T: Copy>(&self, data: &[T]) -> &[T] {
        self.bump.alloc_slice_copy(data)
    }

    /// Reset arena for reuse
    pub fn reset(&mut self) {
        self.bump.reset();
        self.kinds.clear();
        self.flags.clear();
        self.modifiers.clear();
        self.spans.clear();
        self.parents.clear();
        self.children.clear();
        self.string_ids.clear();
        self.extra.clear();
        self.child_nodes.clear();
        self.strings.clear();
        self.node_count.set(0);
    }
}

impl Default for AstArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_size() {
        assert_eq!(std::mem::size_of::<NodeId>(), 4);
    }

    #[test]
    fn test_span_size() {
        assert_eq!(std::mem::size_of::<Span>(), 8);
    }

    #[test]
    fn test_child_list_size() {
        assert_eq!(std::mem::size_of::<ChildList>(), 8);
    }

    #[test]
    fn test_arena_alloc() {
        let mut arena = AstArena::new();
        let id = arena.alloc_node(1, Span::new(0, 10));

        assert_eq!(arena.len(), 1);
        assert_eq!(arena.kind(id), Some(1));
        assert_eq!(arena.span(id), Some(Span::new(0, 10)));
    }

    #[test]
    fn test_arena_children() {
        let mut arena = AstArena::new();
        let parent = arena.alloc_node(1, Span::new(0, 100));
        let child1 = arena.alloc_node(2, Span::new(0, 50));
        let child2 = arena.alloc_node(3, Span::new(50, 100));

        arena.add_children(parent, &[child1, child2]);

        let children = arena.get_children(parent);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], child1);
        assert_eq!(children[1], child2);

        assert_eq!(arena.parent(child1), Some(parent));
        assert_eq!(arena.parent(child2), Some(parent));
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello"); // duplicate

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(interner.get(id1), Some("hello"));
        assert_eq!(interner.get(id2), Some("world"));
    }
}
