//! ThinNode - Cache-efficient AST node representation using Structure of Arrays (SoA).
//!
//! This module provides a memory-efficient representation of TypeScript AST nodes
//! designed for optimal cache locality. Instead of traditional Box-based tree structures,
//! we use indices into parallel arrays (SoA pattern).

use crate::node_kind::NodeKind;

/// A compact node index type that references a node in the AST arena.
/// Using u32 allows for ~4 billion nodes while keeping indices small.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct NodeId(pub u32);

impl NodeId {
    /// Sentinel value representing no node / null reference.
    pub const NONE: NodeId = NodeId(u32::MAX);

    /// Creates a new NodeId from a u32 index.
    #[inline]
    pub const fn new(index: u32) -> Self {
        NodeId(index)
    }

    /// Returns true if this is a valid node reference (not NONE).
    #[inline]
    pub const fn is_some(self) -> bool {
        self.0 != u32::MAX
    }

    /// Returns true if this is a null reference.
    #[inline]
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }

    /// Returns the index as usize for array access.
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns the raw u32 value.
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

/// Node flags for various boolean properties.
/// Packed into a single u16 for space efficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct NodeFlags(pub u16);

impl NodeFlags {
    pub const NONE: NodeFlags = NodeFlags(0);
    pub const LET: NodeFlags = NodeFlags(1 << 0);
    pub const CONST: NodeFlags = NodeFlags(1 << 1);
    pub const NESTED_NAMESPACE: NodeFlags = NodeFlags(1 << 2);
    pub const SYNTHESIZED: NodeFlags = NodeFlags(1 << 3);
    pub const NAMESPACE: NodeFlags = NodeFlags(1 << 4);
    pub const OPTIONAL_CHAIN: NodeFlags = NodeFlags(1 << 5);
    pub const EXPORT_CONTEXT: NodeFlags = NodeFlags(1 << 6);
    pub const AMBIENT: NodeFlags = NodeFlags(1 << 7);
    pub const IN_WITH_STATEMENT: NodeFlags = NodeFlags(1 << 8);
    pub const JSON_FILE: NodeFlags = NodeFlags(1 << 9);
    pub const TYPE_CACHED: NodeFlags = NodeFlags(1 << 10);
    pub const DEPRECATED: NodeFlags = NodeFlags(1 << 11);
    pub const HAS_IMPLICIT_RETURN: NodeFlags = NodeFlags(1 << 12);
    pub const HAS_EXPLICIT_RETURN: NodeFlags = NodeFlags(1 << 13);
    pub const GLOBAL_AUGMENTATION: NodeFlags = NodeFlags(1 << 14);
    pub const HAS_ASYNC_FUNCTIONS: NodeFlags = NodeFlags(1 << 15);

    #[inline]
    pub const fn contains(self, other: NodeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: NodeFlags) -> NodeFlags {
        NodeFlags(self.0 | other.0)
    }

    #[inline]
    pub const fn intersection(self, other: NodeFlags) -> NodeFlags {
        NodeFlags(self.0 & other.0)
    }

    #[inline]
    pub const fn difference(self, other: NodeFlags) -> NodeFlags {
        NodeFlags(self.0 & !other.0)
    }
}

impl std::ops::BitOr for NodeFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for NodeFlags {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl std::ops::BitOrAssign for NodeFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Modifier flags for declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ModifierFlags(pub u32);

impl ModifierFlags {
    pub const NONE: ModifierFlags = ModifierFlags(0);
    pub const EXPORT: ModifierFlags = ModifierFlags(1 << 0);
    pub const AMBIENT: ModifierFlags = ModifierFlags(1 << 1);
    pub const PUBLIC: ModifierFlags = ModifierFlags(1 << 2);
    pub const PRIVATE: ModifierFlags = ModifierFlags(1 << 3);
    pub const PROTECTED: ModifierFlags = ModifierFlags(1 << 4);
    pub const STATIC: ModifierFlags = ModifierFlags(1 << 5);
    pub const READONLY: ModifierFlags = ModifierFlags(1 << 6);
    pub const ACCESSOR: ModifierFlags = ModifierFlags(1 << 7);
    pub const ABSTRACT: ModifierFlags = ModifierFlags(1 << 8);
    pub const ASYNC: ModifierFlags = ModifierFlags(1 << 9);
    pub const DEFAULT: ModifierFlags = ModifierFlags(1 << 10);
    pub const CONST: ModifierFlags = ModifierFlags(1 << 11);
    pub const DEPRECATED: ModifierFlags = ModifierFlags(1 << 12);
    pub const OVERRIDE: ModifierFlags = ModifierFlags(1 << 13);
    pub const IN: ModifierFlags = ModifierFlags(1 << 14);
    pub const OUT: ModifierFlags = ModifierFlags(1 << 15);

    // Composed flags
    pub const ACCESSIBILITY_MODIFIER: ModifierFlags =
        ModifierFlags(Self::PUBLIC.0 | Self::PRIVATE.0 | Self::PROTECTED.0);
    pub const PARAMETER_PROPERTY_MODIFIER: ModifierFlags = ModifierFlags(
        Self::ACCESSIBILITY_MODIFIER.0 | Self::READONLY.0 | Self::OVERRIDE.0,
    );
    pub const NON_PUBLIC_ACCESSIBILITY_MODIFIER: ModifierFlags =
        ModifierFlags(Self::PRIVATE.0 | Self::PROTECTED.0);

    #[inline]
    pub const fn contains(self, other: ModifierFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: ModifierFlags) -> ModifierFlags {
        ModifierFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for ModifierFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// A text span representing a range in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct TextSpan {
    /// Start position (byte offset from beginning of source).
    pub start: u32,
    /// Length in bytes.
    pub length: u32,
}

impl TextSpan {
    #[inline]
    pub const fn new(start: u32, length: u32) -> Self {
        TextSpan { start, length }
    }

    #[inline]
    pub const fn end(self) -> u32 {
        self.start + self.length
    }

    #[inline]
    pub const fn contains(self, pos: u32) -> bool {
        pos >= self.start && pos < self.end()
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.length == 0
    }
}

/// Reference to a slice of child nodes.
/// Uses start index and count to reference a contiguous range in the children array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ChildrenRef {
    /// Start index in the children array.
    pub start: u32,
    /// Number of children.
    pub count: u16,
    /// Padding for alignment.
    _padding: u16,
}

impl ChildrenRef {
    pub const EMPTY: ChildrenRef = ChildrenRef {
        start: 0,
        count: 0,
        _padding: 0,
    };

    #[inline]
    pub const fn new(start: u32, count: u16) -> Self {
        ChildrenRef {
            start,
            count,
            _padding: 0,
        }
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }

    #[inline]
    pub const fn len(self) -> usize {
        self.count as usize
    }
}

/// Reference to a string in the string interner.
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

/// The core ThinNode structure - a lightweight, cache-friendly AST node.
///
/// This struct is designed to be exactly 32 bytes for optimal cache line usage.
/// All variable-length data (children, strings) are stored externally in arenas
/// and referenced by index.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ThinNode {
    /// The kind of AST node (2 bytes).
    pub kind: NodeKind,
    /// Node flags (2 bytes).
    pub flags: NodeFlags,
    /// Modifier flags for declarations (4 bytes).
    pub modifiers: ModifierFlags,
    /// Source position - start byte offset (4 bytes).
    pub pos: u32,
    /// Source position - end byte offset (4 bytes).
    pub end: u32,
    /// Parent node index (4 bytes).
    pub parent: NodeId,
    /// Reference to children array (8 bytes).
    pub children: ChildrenRef,
    /// Associated string data (identifier name, string literal value, etc.) (4 bytes).
    pub string_id: StringId,
}

impl ThinNode {
    /// Creates a new ThinNode with the given kind and source position.
    #[inline]
    pub const fn new(kind: NodeKind, pos: u32, end: u32) -> Self {
        ThinNode {
            kind,
            flags: NodeFlags::NONE,
            modifiers: ModifierFlags::NONE,
            pos,
            end,
            parent: NodeId::NONE,
            children: ChildrenRef::EMPTY,
            string_id: StringId::EMPTY,
        }
    }

    /// Returns the text span of this node.
    #[inline]
    pub const fn span(self) -> TextSpan {
        TextSpan::new(self.pos, self.end - self.pos)
    }

    /// Returns the length of this node in bytes.
    #[inline]
    pub const fn len(self) -> u32 {
        self.end - self.pos
    }

    /// Returns true if this node has children.
    #[inline]
    pub const fn has_children(self) -> bool {
        !self.children.is_empty()
    }

    /// Returns true if this node has a parent.
    #[inline]
    pub const fn has_parent(self) -> bool {
        self.parent.is_some()
    }

    /// Returns true if this node is synthesized (not from source).
    #[inline]
    pub const fn is_synthesized(self) -> bool {
        self.flags.contains(NodeFlags::SYNTHESIZED)
    }
}

/// The AST arena storing all nodes in a Structure of Arrays (SoA) layout.
///
/// This design provides excellent cache locality when iterating over specific
/// node properties (e.g., all node kinds, all positions, etc.).
#[derive(Debug, Default)]
pub struct NodeArena {
    /// Node kinds - parallel array.
    pub kinds: Vec<NodeKind>,
    /// Node flags - parallel array.
    pub flags: Vec<NodeFlags>,
    /// Modifier flags - parallel array.
    pub modifiers: Vec<ModifierFlags>,
    /// Start positions - parallel array.
    pub positions: Vec<u32>,
    /// End positions - parallel array.
    pub ends: Vec<u32>,
    /// Parent indices - parallel array.
    pub parents: Vec<NodeId>,
    /// Children references - parallel array.
    pub children_refs: Vec<ChildrenRef>,
    /// String IDs - parallel array.
    pub string_ids: Vec<StringId>,
    /// Flattened children array (all children stored contiguously).
    pub children: Vec<NodeId>,
    /// String interner data.
    pub strings: StringInterner,
}

impl NodeArena {
    /// Creates a new empty arena.
    pub fn new() -> Self {
        NodeArena {
            kinds: Vec::new(),
            flags: Vec::new(),
            modifiers: Vec::new(),
            positions: Vec::new(),
            ends: Vec::new(),
            parents: Vec::new(),
            children_refs: Vec::new(),
            string_ids: Vec::new(),
            children: Vec::new(),
            strings: StringInterner::new(),
        }
    }

    /// Creates a new arena with pre-allocated capacity.
    pub fn with_capacity(node_capacity: usize, children_capacity: usize) -> Self {
        NodeArena {
            kinds: Vec::with_capacity(node_capacity),
            flags: Vec::with_capacity(node_capacity),
            modifiers: Vec::with_capacity(node_capacity),
            positions: Vec::with_capacity(node_capacity),
            ends: Vec::with_capacity(node_capacity),
            parents: Vec::with_capacity(node_capacity),
            children_refs: Vec::with_capacity(node_capacity),
            string_ids: Vec::with_capacity(node_capacity),
            children: Vec::with_capacity(children_capacity),
            strings: StringInterner::new(),
        }
    }

    /// Returns the number of nodes in the arena.
    #[inline]
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    /// Returns true if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    /// Allocates a new node and returns its ID.
    pub fn alloc(&mut self, node: ThinNode) -> NodeId {
        let id = NodeId::new(self.kinds.len() as u32);
        self.kinds.push(node.kind);
        self.flags.push(node.flags);
        self.modifiers.push(node.modifiers);
        self.positions.push(node.pos);
        self.ends.push(node.end);
        self.parents.push(node.parent);
        self.children_refs.push(node.children);
        self.string_ids.push(node.string_id);
        id
    }

    /// Allocates a node with basic parameters.
    pub fn alloc_node(&mut self, kind: NodeKind, pos: u32, end: u32) -> NodeId {
        self.alloc(ThinNode::new(kind, pos, end))
    }

    /// Gets a node by ID, reconstructing the ThinNode from SoA data.
    #[inline]
    pub fn get(&self, id: NodeId) -> Option<ThinNode> {
        if id.is_none() {
            return None;
        }
        let i = id.index();
        if i >= self.kinds.len() {
            return None;
        }
        Some(ThinNode {
            kind: self.kinds[i],
            flags: self.flags[i],
            modifiers: self.modifiers[i],
            pos: self.positions[i],
            end: self.ends[i],
            parent: self.parents[i],
            children: self.children_refs[i],
            string_id: self.string_ids[i],
        })
    }

    /// Gets the kind of a node by ID.
    #[inline]
    pub fn kind(&self, id: NodeId) -> Option<NodeKind> {
        if id.is_none() {
            return None;
        }
        self.kinds.get(id.index()).copied()
    }

    /// Gets the source position span of a node.
    #[inline]
    pub fn span(&self, id: NodeId) -> Option<TextSpan> {
        if id.is_none() {
            return None;
        }
        let i = id.index();
        Some(TextSpan::new(
            *self.positions.get(i)?,
            self.ends.get(i)? - self.positions.get(i)?,
        ))
    }

    /// Gets the parent of a node.
    #[inline]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        if id.is_none() {
            return None;
        }
        self.parents.get(id.index()).copied()
    }

    /// Sets the parent of a node.
    #[inline]
    pub fn set_parent(&mut self, id: NodeId, parent: NodeId) {
        if id.is_some() {
            if let Some(p) = self.parents.get_mut(id.index()) {
                *p = parent;
            }
        }
    }

    /// Gets the children of a node.
    pub fn get_children(&self, id: NodeId) -> &[NodeId] {
        if id.is_none() {
            return &[];
        }
        let i = id.index();
        if let Some(children_ref) = self.children_refs.get(i) {
            if children_ref.is_empty() {
                return &[];
            }
            let start = children_ref.start as usize;
            let end = start + children_ref.count as usize;
            if end <= self.children.len() {
                return &self.children[start..end];
            }
        }
        &[]
    }

    /// Adds children to a node.
    pub fn add_children(&mut self, id: NodeId, node_children: &[NodeId]) {
        if id.is_none() || node_children.is_empty() {
            return;
        }
        let i = id.index();
        if i >= self.children_refs.len() {
            return;
        }

        let start = self.children.len() as u32;
        let count = node_children.len().min(u16::MAX as usize) as u16;

        self.children.extend_from_slice(&node_children[..count as usize]);
        self.children_refs[i] = ChildrenRef::new(start, count);

        // Set parent for all children
        for &child_id in node_children.iter().take(count as usize) {
            self.set_parent(child_id, id);
        }
    }

    /// Interns a string and returns its ID.
    pub fn intern_string(&mut self, s: &str) -> StringId {
        self.strings.intern(s)
    }

    /// Gets a string by ID.
    pub fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id)
    }

    /// Sets the string ID for a node.
    #[inline]
    pub fn set_string(&mut self, id: NodeId, string_id: StringId) {
        if id.is_some() {
            if let Some(s) = self.string_ids.get_mut(id.index()) {
                *s = string_id;
            }
        }
    }

    /// Sets flags for a node.
    #[inline]
    pub fn set_flags(&mut self, id: NodeId, new_flags: NodeFlags) {
        if id.is_some() {
            if let Some(f) = self.flags.get_mut(id.index()) {
                *f = new_flags;
            }
        }
    }

    /// Adds flags to a node.
    #[inline]
    pub fn add_flags(&mut self, id: NodeId, new_flags: NodeFlags) {
        if id.is_some() {
            if let Some(f) = self.flags.get_mut(id.index()) {
                *f |= new_flags;
            }
        }
    }

    /// Sets modifier flags for a node.
    #[inline]
    pub fn set_modifiers(&mut self, id: NodeId, new_modifiers: ModifierFlags) {
        if id.is_some() {
            if let Some(m) = self.modifiers.get_mut(id.index()) {
                *m = new_modifiers;
            }
        }
    }

    /// Clears all nodes from the arena.
    pub fn clear(&mut self) {
        self.kinds.clear();
        self.flags.clear();
        self.modifiers.clear();
        self.positions.clear();
        self.ends.clear();
        self.parents.clear();
        self.children_refs.clear();
        self.string_ids.clear();
        self.children.clear();
        self.strings.clear();
    }
}

/// Simple string interner for deduplicating strings.
#[derive(Debug, Default)]
pub struct StringInterner {
    /// All strings concatenated.
    data: String,
    /// (offset, length) pairs for each interned string.
    entries: Vec<(u32, u32)>,
    /// Map from string content to StringId for deduplication.
    /// Using a simple Vec for now; could be optimized with a HashMap.
    lookup: Vec<(u32, u32, StringId)>, // (hash, len, id)
}

impl StringInterner {
    pub fn new() -> Self {
        let mut interner = Self::default();
        // Reserve index 0 for empty string
        interner.data.push('\0');
        interner.entries.push((0, 0));
        interner
    }

    /// Interns a string, returning its ID.
    pub fn intern(&mut self, s: &str) -> StringId {
        if s.is_empty() {
            return StringId::EMPTY;
        }

        // Simple hash for lookup
        let hash = Self::hash_str(s);
        let len = s.len() as u32;

        // Check if already interned
        for &(h, l, id) in &self.lookup {
            if h == hash && l == len {
                if let Some(existing) = self.get(id) {
                    if existing == s {
                        return id;
                    }
                }
            }
        }

        // Intern new string
        let offset = self.data.len() as u32;
        self.data.push_str(s);
        let id = StringId::new(self.entries.len() as u32);
        self.entries.push((offset, len));
        self.lookup.push((hash, len, id));
        id
    }

    /// Gets a string by ID.
    pub fn get(&self, id: StringId) -> Option<&str> {
        if id.is_empty() {
            return Some("");
        }
        let (offset, len) = *self.entries.get(id.0 as usize)?;
        let start = offset as usize;
        let end = start + len as usize;
        self.data.get(start..end)
    }

    /// Clears all interned strings.
    pub fn clear(&mut self) {
        self.data.clear();
        self.data.push('\0');
        self.entries.clear();
        self.entries.push((0, 0));
        self.lookup.clear();
    }

    fn hash_str(s: &str) -> u32 {
        let mut hash: u32 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        hash
    }
}

/// Iterator over children of a node.
pub struct ChildrenIter<'a> {
    arena: &'a NodeArena,
    children: std::slice::Iter<'a, NodeId>,
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = (NodeId, ThinNode);

    fn next(&mut self) -> Option<Self::Item> {
        let id = *self.children.next()?;
        let node = self.arena.get(id)?;
        Some((id, node))
    }
}

impl NodeArena {
    /// Returns an iterator over the children of a node.
    pub fn children_iter(&self, id: NodeId) -> ChildrenIter<'_> {
        ChildrenIter {
            arena: self,
            children: self.get_children(id).iter(),
        }
    }
}

/// A view into the AST for read-only access.
#[derive(Debug)]
pub struct AstView<'a> {
    arena: &'a NodeArena,
    root: NodeId,
}

impl<'a> AstView<'a> {
    pub fn new(arena: &'a NodeArena, root: NodeId) -> Self {
        AstView { arena, root }
    }

    #[inline]
    pub fn root(&self) -> NodeId {
        self.root
    }

    #[inline]
    pub fn arena(&self) -> &NodeArena {
        self.arena
    }

    #[inline]
    pub fn get(&self, id: NodeId) -> Option<ThinNode> {
        self.arena.get(id)
    }

    #[inline]
    pub fn kind(&self, id: NodeId) -> Option<NodeKind> {
        self.arena.kind(id)
    }

    #[inline]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.arena.get_children(id)
    }

    #[inline]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.arena.parent(id)
    }

    /// Walks the AST in pre-order (parent before children).
    pub fn walk_preorder<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId, &ThinNode) -> bool,
    {
        let mut stack = vec![start];
        while let Some(id) = stack.pop() {
            if let Some(node) = self.arena.get(id) {
                if visitor(id, &node) {
                    // Add children in reverse order so they're processed left-to-right
                    let children = self.arena.get_children(id);
                    for &child in children.iter().rev() {
                        stack.push(child);
                    }
                }
            }
        }
    }

    /// Walks the AST in post-order (children before parent).
    pub fn walk_postorder<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId, &ThinNode),
    {
        #[derive(Clone, Copy)]
        enum State {
            Enter(NodeId),
            Exit(NodeId),
        }

        let mut stack = vec![State::Enter(start)];
        while let Some(state) = stack.pop() {
            match state {
                State::Enter(id) => {
                    if self.arena.get(id).is_some() {
                        stack.push(State::Exit(id));
                        let children = self.arena.get_children(id);
                        for &child in children.iter().rev() {
                            stack.push(State::Enter(child));
                        }
                    }
                }
                State::Exit(id) => {
                    if let Some(node) = self.arena.get(id) {
                        visitor(id, &node);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thin_node_size() {
        // Verify ThinNode is 32 bytes for cache efficiency
        assert_eq!(std::mem::size_of::<ThinNode>(), 32);
    }

    #[test]
    fn test_node_id_size() {
        assert_eq!(std::mem::size_of::<NodeId>(), 4);
    }

    #[test]
    fn test_text_span_size() {
        assert_eq!(std::mem::size_of::<TextSpan>(), 8);
    }

    #[test]
    fn test_children_ref_size() {
        assert_eq!(std::mem::size_of::<ChildrenRef>(), 8);
    }

    #[test]
    fn test_node_id_none() {
        assert!(NodeId::NONE.is_none());
        assert!(!NodeId::NONE.is_some());
        assert!(NodeId::new(0).is_some());
        assert!(!NodeId::new(0).is_none());
    }

    #[test]
    fn test_arena_alloc_and_get() {
        let mut arena = NodeArena::new();

        let node = ThinNode::new(NodeKind::Identifier, 0, 5);
        let id = arena.alloc(node);

        assert_eq!(id.0, 0);
        assert_eq!(arena.len(), 1);

        let retrieved = arena.get(id).unwrap();
        assert_eq!(retrieved.kind, NodeKind::Identifier);
        assert_eq!(retrieved.pos, 0);
        assert_eq!(retrieved.end, 5);
    }

    #[test]
    fn test_arena_children() {
        let mut arena = NodeArena::new();

        let parent_id = arena.alloc_node(NodeKind::Block, 0, 100);
        let child1_id = arena.alloc_node(NodeKind::VariableStatement, 1, 20);
        let child2_id = arena.alloc_node(NodeKind::ExpressionStatement, 21, 50);

        arena.add_children(parent_id, &[child1_id, child2_id]);

        let children = arena.get_children(parent_id);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], child1_id);
        assert_eq!(children[1], child2_id);

        // Verify parent was set
        assert_eq!(arena.parent(child1_id), Some(parent_id));
        assert_eq!(arena.parent(child2_id), Some(parent_id));
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello"); // Duplicate

        assert_eq!(id1, id3); // Should be deduplicated
        assert_ne!(id1, id2);

        assert_eq!(interner.get(id1), Some("hello"));
        assert_eq!(interner.get(id2), Some("world"));
        assert_eq!(interner.get(StringId::EMPTY), Some(""));
    }

    #[test]
    fn test_arena_string_operations() {
        let mut arena = NodeArena::new();

        let id = arena.alloc_node(NodeKind::Identifier, 0, 5);
        let string_id = arena.intern_string("myVar");
        arena.set_string(id, string_id);

        let node = arena.get(id).unwrap();
        assert_eq!(node.string_id, string_id);
        assert_eq!(arena.get_string(string_id), Some("myVar"));
    }

    #[test]
    fn test_node_flags() {
        let flags = NodeFlags::LET | NodeFlags::CONST;
        assert!(flags.contains(NodeFlags::LET));
        assert!(flags.contains(NodeFlags::CONST));
        assert!(!flags.contains(NodeFlags::SYNTHESIZED));
    }

    #[test]
    fn test_modifier_flags() {
        let modifiers = ModifierFlags::PUBLIC | ModifierFlags::STATIC;
        assert!(modifiers.contains(ModifierFlags::PUBLIC));
        assert!(modifiers.contains(ModifierFlags::STATIC));
        assert!(!modifiers.contains(ModifierFlags::PRIVATE));
        assert!(ModifierFlags::ACCESSIBILITY_MODIFIER.contains(ModifierFlags::PUBLIC));
    }

    #[test]
    fn test_ast_view_walk() {
        let mut arena = NodeArena::new();

        // Build a simple tree:
        //     SourceFile
        //     ├── FunctionDeclaration
        //     │   └── Block
        //     └── VariableStatement
        let source = arena.alloc_node(NodeKind::SourceFile, 0, 100);
        let func = arena.alloc_node(NodeKind::FunctionDeclaration, 0, 50);
        let block = arena.alloc_node(NodeKind::Block, 10, 50);
        let var_stmt = arena.alloc_node(NodeKind::VariableStatement, 51, 100);

        arena.add_children(source, &[func, var_stmt]);
        arena.add_children(func, &[block]);

        let view = AstView::new(&arena, source);

        // Test pre-order walk
        let mut preorder_kinds = Vec::new();
        view.walk_preorder(source, |_id, node| {
            preorder_kinds.push(node.kind);
            true
        });

        assert_eq!(
            preorder_kinds,
            vec![
                NodeKind::SourceFile,
                NodeKind::FunctionDeclaration,
                NodeKind::Block,
                NodeKind::VariableStatement,
            ]
        );

        // Test post-order walk
        let mut postorder_kinds = Vec::new();
        view.walk_postorder(source, |_id, node| {
            postorder_kinds.push(node.kind);
        });

        assert_eq!(
            postorder_kinds,
            vec![
                NodeKind::Block,
                NodeKind::FunctionDeclaration,
                NodeKind::VariableStatement,
                NodeKind::SourceFile,
            ]
        );
    }

    #[test]
    fn test_text_span() {
        let span = TextSpan::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 20);
        assert_eq!(span.end(), 30);
        assert!(span.contains(10));
        assert!(span.contains(29));
        assert!(!span.contains(30));
        assert!(!span.contains(9));
    }
}
