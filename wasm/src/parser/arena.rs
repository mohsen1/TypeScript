//! Node arena for AST storage.

use serde::Serialize;
use super::ast::{Node, NodeIndex};
use super::thin_node::{NodeAccess, NodeInfo};

/// Arena-based storage for AST nodes.
/// Nodes are stored contiguously and referenced by index.
#[derive(Debug, Default, Serialize)]
pub struct NodeArena {
    pub nodes: Vec<Node>,
}

impl NodeArena {
    pub fn new() -> NodeArena {
        NodeArena { nodes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> NodeArena {
        NodeArena { nodes: Vec::with_capacity(capacity) }
    }

    /// Add a node to the arena and return its index
    pub fn add(&mut self, node: Node) -> NodeIndex {
        let index = self.nodes.len() as u32;
        self.nodes.push(node);
        NodeIndex(index)
    }

    /// Get a node by index
    pub fn get(&self, index: NodeIndex) -> Option<&Node> {
        if index.is_none() {
            None
        } else {
            self.nodes.get(index.0 as usize)
        }
    }

    /// Get a mutable node by index
    pub fn get_mut(&mut self, index: NodeIndex) -> Option<&mut Node> {
        if index.is_none() {
            None
        } else {
            self.nodes.get_mut(index.0 as usize)
        }
    }

    /// Replace a node at the given index
    /// Returns the old node if successful
    pub fn replace(&mut self, index: NodeIndex, new_node: Node) -> Option<Node> {
        if index.is_none() {
            None
        } else {
            self.nodes.get_mut(index.0 as usize).map(|old| {
                std::mem::replace(old, new_node)
            })
        }
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Implementation of NodeAccess for NodeArena
impl NodeAccess for NodeArena {
    fn node_info(&self, index: NodeIndex) -> Option<NodeInfo> {
        let node = self.get(index)?;
        let base = node.base();
        Some(NodeInfo {
            kind: base.kind,
            flags: base.flags,
            modifier_flags: base.modifier_flags,
            pos: base.pos,
            end: base.end,
            parent: base.parent,
            id: base.id,
        })
    }

    fn kind(&self, index: NodeIndex) -> Option<u16> {
        self.get(index).map(|n| n.base().kind)
    }

    fn pos_end(&self, index: NodeIndex) -> Option<(u32, u32)> {
        self.get(index).map(|n| (n.base().pos, n.base().end))
    }

    fn get_identifier_text(&self, index: NodeIndex) -> Option<&str> {
        match self.get(index)? {
            Node::Identifier(ident) | Node::PrivateIdentifier(ident) => {
                Some(&ident.escaped_text)
            }
            _ => None,
        }
    }

    fn get_literal_text(&self, index: NodeIndex) -> Option<&str> {
        match self.get(index)? {
            Node::StringLiteral(lit) | Node::NoSubstitutionTemplateLiteral(lit) |
            Node::TemplateHead(lit) | Node::TemplateMiddle(lit) | Node::TemplateTail(lit) => {
                Some(&lit.text)
            }
            Node::NumericLiteral(lit) => Some(&lit.text),
            Node::BigIntLiteral(lit) => Some(&lit.text),
            Node::RegularExpressionLiteral(lit) => Some(&lit.text),
            _ => None,
        }
    }

    fn get_children(&self, index: NodeIndex) -> Vec<NodeIndex> {
        // TODO: Implement proper child enumeration based on node kind
        // For now, return empty - this would need kind-specific logic
        Vec::new()
    }
}
