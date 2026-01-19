//! Namespace Type Checking
//!
//! Handles type checking for TypeScript namespaces:
//! - Namespace declarations and merging
//! - Namespace exports
//! - Nested namespaces
//! - Module augmentation
//! - Ambient namespaces (declare namespace)

use std::collections::HashMap;

/// Export visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportVisibility {
    /// Exported (accessible from outside)
    Exported,
    /// Internal (only accessible within namespace)
    Internal,
}

/// Namespace member kind
#[derive(Debug, Clone, PartialEq)]
pub enum NamespaceMemberKind {
    /// Variable declaration
    Variable { is_const: bool },
    /// Function declaration
    Function { is_async: bool },
    /// Class declaration
    Class,
    /// Interface declaration
    Interface,
    /// Type alias
    TypeAlias,
    /// Enum declaration
    Enum { is_const: bool },
    /// Nested namespace
    Namespace,
    /// Module (external module reference)
    Module,
}

/// Namespace member
#[derive(Debug, Clone)]
pub struct NamespaceMember<'a> {
    /// Member name
    pub name: &'a str,
    /// Kind of member
    pub kind: NamespaceMemberKind,
    /// Export visibility
    pub visibility: ExportVisibility,
    /// Source span
    pub span: (usize, usize),
}

/// Namespace declaration
#[derive(Debug, Clone)]
pub struct NamespaceDeclaration<'a> {
    /// Namespace name (can be dotted like "A.B.C")
    pub name: &'a str,
    /// Members
    pub members: Vec<NamespaceMember<'a>>,
    /// Is ambient (declare namespace)
    pub is_ambient: bool,
    /// Is module (declare module "...")
    pub is_module: bool,
    /// Source span
    pub span: (usize, usize),
}

/// Resolved namespace with merged members
#[derive(Debug, Clone)]
pub struct ResolvedNamespace<'a> {
    /// Namespace name
    pub name: &'a str,
    /// All members (merged from multiple declarations)
    pub members: HashMap<&'a str, Vec<NamespaceMember<'a>>>,
    /// Nested namespaces
    pub nested: HashMap<&'a str, ResolvedNamespace<'a>>,
    /// Is ambient
    pub is_ambient: bool,
    /// Declaration spans (for merging)
    pub declaration_spans: Vec<(usize, usize)>,
}

/// Namespace check error
#[derive(Debug, Clone)]
pub enum NamespaceError {
    /// Duplicate non-mergeable export
    DuplicateExport { name: String, span: (usize, usize) },
    /// Invalid merge (e.g., variable with function)
    InvalidMerge { name: String, kinds: (String, String), span: (usize, usize) },
    /// Accessing non-exported member
    NonExportedAccess { namespace: String, member: String, span: (usize, usize) },
    /// Namespace not found
    NamespaceNotFound { name: String, span: (usize, usize) },
    /// Member not found
    MemberNotFound { namespace: String, member: String, span: (usize, usize) },
    /// Ambient namespace cannot have implementation
    AmbientWithImplementation { name: String, span: (usize, usize) },
}

/// Namespace checker
pub struct NamespaceChecker<'a> {
    /// Root-level namespaces
    namespaces: HashMap<&'a str, ResolvedNamespace<'a>>,
    /// Errors encountered
    errors: Vec<NamespaceError>,
}

impl<'a> NamespaceChecker<'a> {
    pub fn new() -> Self {
        NamespaceChecker {
            namespaces: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Register a namespace declaration (handles merging)
    pub fn register_namespace(
        &mut self,
        decl: NamespaceDeclaration<'a>,
    ) -> Result<(), NamespaceError> {
        // Handle dotted names (A.B.C)
        let parts: Vec<&str> = decl.name.split('.').collect();

        if parts.len() == 1 {
            // Simple namespace
            self.register_simple_namespace(decl)
        } else {
            // Nested namespace - register each level
            self.register_nested_namespace(&parts, decl)
        }
    }

    fn register_simple_namespace(
        &mut self,
        decl: NamespaceDeclaration<'a>,
    ) -> Result<(), NamespaceError> {
        let name = decl.name;

        if self.namespaces.contains_key(name) {
            // Merge with existing namespace
            let members_to_add = decl.members.clone();
            let span = decl.span;

            // Validate merge
            {
                let existing = self.namespaces.get(name).unwrap();
                for member in &members_to_add {
                    if let Some(existing_members) = existing.members.get(member.name) {
                        if !can_merge(&existing_members[0].kind, &member.kind) {
                            let err = NamespaceError::InvalidMerge {
                                name: member.name.to_string(),
                                kinds: (
                                    format!("{:?}", existing_members[0].kind),
                                    format!("{:?}", member.kind),
                                ),
                                span: member.span,
                            };
                            self.errors.push(err.clone());
                            return Err(err);
                        }
                    }
                }
            }

            // Perform merge
            let existing = self.namespaces.get_mut(name).unwrap();
            for member in members_to_add {
                existing
                    .members
                    .entry(member.name)
                    .or_insert_with(Vec::new)
                    .push(member);
            }
            existing.declaration_spans.push(span);
        } else {
            // Create new namespace
            let mut members = HashMap::new();
            for member in decl.members {
                members
                    .entry(member.name)
                    .or_insert_with(Vec::new)
                    .push(member);
            }

            let resolved = ResolvedNamespace {
                name,
                members,
                nested: HashMap::new(),
                is_ambient: decl.is_ambient,
                declaration_spans: vec![decl.span],
            };

            self.namespaces.insert(name, resolved);
        }

        Ok(())
    }

    fn register_nested_namespace(
        &mut self,
        parts: &[&'a str],
        decl: NamespaceDeclaration<'a>,
    ) -> Result<(), NamespaceError> {
        // Ensure parent namespaces exist
        let root_name = parts[0];

        if !self.namespaces.contains_key(root_name) {
            let root = ResolvedNamespace {
                name: root_name,
                members: HashMap::new(),
                nested: HashMap::new(),
                is_ambient: decl.is_ambient,
                declaration_spans: vec![],
            };
            self.namespaces.insert(root_name, root);
        }

        // Navigate to parent and create intermediate namespaces
        let mut current = self.namespaces.get_mut(root_name).unwrap();

        for &part in &parts[1..parts.len()-1] {
            if !current.nested.contains_key(part) {
                let nested = ResolvedNamespace {
                    name: part,
                    members: HashMap::new(),
                    nested: HashMap::new(),
                    is_ambient: decl.is_ambient,
                    declaration_spans: vec![],
                };
                current.nested.insert(part, nested);
            }
            current = current.nested.get_mut(part).unwrap();
        }

        // Add the final namespace
        let final_name = parts[parts.len() - 1];
        if current.nested.contains_key(final_name) {
            // Validate merge
            let members_to_add = decl.members.clone();
            let span = decl.span;

            {
                let existing = current.nested.get(final_name).unwrap();
                for member in &members_to_add {
                    if let Some(existing_members) = existing.members.get(member.name) {
                        if !can_merge(&existing_members[0].kind, &member.kind) {
                            let err = NamespaceError::InvalidMerge {
                                name: member.name.to_string(),
                                kinds: (
                                    format!("{:?}", existing_members[0].kind),
                                    format!("{:?}", member.kind),
                                ),
                                span: member.span,
                            };
                            self.errors.push(err.clone());
                            return Err(err);
                        }
                    }
                }
            }

            // Perform merge
            let existing = current.nested.get_mut(final_name).unwrap();
            for member in members_to_add {
                existing
                    .members
                    .entry(member.name)
                    .or_insert_with(Vec::new)
                    .push(member);
            }
            existing.declaration_spans.push(span);
        } else {
            let mut members = HashMap::new();
            for member in decl.members {
                members
                    .entry(member.name)
                    .or_insert_with(Vec::new)
                    .push(member);
            }

            let resolved = ResolvedNamespace {
                name: final_name,
                members,
                nested: HashMap::new(),
                is_ambient: decl.is_ambient,
                declaration_spans: vec![decl.span],
            };
            current.nested.insert(final_name, resolved);
        }

        Ok(())
    }

    /// Get a namespace by name
    pub fn get_namespace(&self, name: &str) -> Option<&ResolvedNamespace<'a>> {
        // Handle dotted names
        let parts: Vec<&str> = name.split('.').collect();

        if parts.len() == 1 {
            self.namespaces.get(name)
        } else {
            let mut current = self.namespaces.get(parts[0])?;
            for &part in &parts[1..] {
                current = current.nested.get(part)?;
            }
            Some(current)
        }
    }

    /// Check member access - returns the members if valid
    pub fn check_member_access(
        &mut self,
        namespace_name: &str,
        member_name: &str,
        span: (usize, usize),
    ) -> Result<Vec<NamespaceMember<'a>>, NamespaceError> {
        // First check if namespace exists
        let ns = match self.get_namespace(namespace_name) {
            Some(ns) => ns,
            None => {
                let err = NamespaceError::NamespaceNotFound {
                    name: namespace_name.to_string(),
                    span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }
        };

        // Check member exists
        let members = match ns.members.get(member_name) {
            Some(m) => m.clone(),
            None => {
                let err = NamespaceError::MemberNotFound {
                    namespace: namespace_name.to_string(),
                    member: member_name.to_string(),
                    span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }
        };

        // Check if at least one is exported
        if members.iter().any(|m| m.visibility == ExportVisibility::Exported) {
            Ok(members)
        } else {
            let err = NamespaceError::NonExportedAccess {
                namespace: namespace_name.to_string(),
                member: member_name.to_string(),
                span,
            };
            self.errors.push(err.clone());
            Err(err)
        }
    }

    /// Get all exported members of a namespace
    pub fn get_exports(&self, namespace_name: &str) -> Vec<NamespaceMember<'a>> {
        if let Some(ns) = self.get_namespace(namespace_name) {
            ns.members
                .values()
                .flatten()
                .filter(|m| m.visibility == ExportVisibility::Exported)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get errors
    pub fn errors(&self) -> &[NamespaceError] {
        &self.errors
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

impl<'a> Default for NamespaceChecker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two member kinds can be merged
fn can_merge(existing: &NamespaceMemberKind, new: &NamespaceMemberKind) -> bool {
    use NamespaceMemberKind::*;

    match (existing, new) {
        // Interfaces can merge
        (Interface, Interface) => true,
        // Namespaces can merge
        (Namespace, Namespace) => true,
        // Functions can be overloaded (merge)
        (Function { .. }, Function { .. }) => true,
        // Class can merge with namespace (for static members)
        (Class, Namespace) | (Namespace, Class) => true,
        // Function can merge with namespace
        (Function { .. }, Namespace) | (Namespace, Function { .. }) => true,
        // Enum can merge with namespace
        (Enum { .. }, Namespace) | (Namespace, Enum { .. }) => true,
        // Everything else cannot merge
        _ => false,
    }
}

/// Qualify a name with namespace prefix
pub fn qualify_name(namespace: &str, member: &str) -> String {
    format!("{}.{}", namespace, member)
}

/// Split a qualified name into namespace and member
pub fn split_qualified_name(name: &str) -> Option<(&str, &str)> {
    name.rsplit_once('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_simple_namespace() {
        let mut checker = NamespaceChecker::new();

        let decl = NamespaceDeclaration {
            name: "MyNamespace",
            members: vec![
                NamespaceMember {
                    name: "foo",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Exported,
                    span: (0, 10),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 50),
        };

        let result = checker.register_namespace(decl);
        assert!(result.is_ok());

        let ns = checker.get_namespace("MyNamespace");
        assert!(ns.is_some());
        assert!(ns.unwrap().members.contains_key("foo"));
    }

    #[test]
    fn test_namespace_merging() {
        let mut checker = NamespaceChecker::new();

        // First declaration
        let decl1 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "a",
                    kind: NamespaceMemberKind::Variable { is_const: true },
                    visibility: ExportVisibility::Exported,
                    span: (0, 5),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 20),
        };

        // Second declaration (merges)
        let decl2 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "b",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Exported,
                    span: (25, 30),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (21, 50),
        };

        checker.register_namespace(decl1).unwrap();
        checker.register_namespace(decl2).unwrap();

        let ns = checker.get_namespace("NS").unwrap();
        assert!(ns.members.contains_key("a"));
        assert!(ns.members.contains_key("b"));
        assert_eq!(ns.declaration_spans.len(), 2);
    }

    #[test]
    fn test_interface_merging() {
        let mut checker = NamespaceChecker::new();

        let decl1 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "IFace",
                    kind: NamespaceMemberKind::Interface,
                    visibility: ExportVisibility::Exported,
                    span: (0, 10),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 30),
        };

        let decl2 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "IFace",  // Same interface, should merge
                    kind: NamespaceMemberKind::Interface,
                    visibility: ExportVisibility::Exported,
                    span: (35, 45),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (31, 60),
        };

        checker.register_namespace(decl1).unwrap();
        let result = checker.register_namespace(decl2);
        assert!(result.is_ok());

        let ns = checker.get_namespace("NS").unwrap();
        // Should have two interface declarations with same name
        assert_eq!(ns.members.get("IFace").unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_merge() {
        let mut checker = NamespaceChecker::new();

        let decl1 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "x",
                    kind: NamespaceMemberKind::Variable { is_const: true },
                    visibility: ExportVisibility::Exported,
                    span: (0, 5),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 20),
        };

        let decl2 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "x",  // Same name, but Class cannot merge with Variable
                    kind: NamespaceMemberKind::Class,
                    visibility: ExportVisibility::Exported,
                    span: (25, 35),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (21, 50),
        };

        checker.register_namespace(decl1).unwrap();
        let result = checker.register_namespace(decl2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NamespaceError::InvalidMerge { .. }));
    }

    #[test]
    fn test_nested_namespace() {
        let mut checker = NamespaceChecker::new();

        let decl = NamespaceDeclaration {
            name: "A.B.C",
            members: vec![
                NamespaceMember {
                    name: "foo",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Exported,
                    span: (0, 10),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 50),
        };

        checker.register_namespace(decl).unwrap();

        // Should be able to access via dotted name
        let ns = checker.get_namespace("A.B.C");
        assert!(ns.is_some());
        assert!(ns.unwrap().members.contains_key("foo"));
    }

    #[test]
    fn test_member_access_exported() {
        let mut checker = NamespaceChecker::new();

        let decl = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "public_fn",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Exported,
                    span: (0, 10),
                },
                NamespaceMember {
                    name: "private_fn",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Internal,
                    span: (15, 25),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 50),
        };

        checker.register_namespace(decl).unwrap();

        // Exported member - OK
        let result = checker.check_member_access("NS", "public_fn", (0, 12));
        assert!(result.is_ok());

        // Internal member - Error
        let result = checker.check_member_access("NS", "private_fn", (0, 13));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NamespaceError::NonExportedAccess { .. }));
    }

    #[test]
    fn test_get_exports() {
        let mut checker = NamespaceChecker::new();

        let decl = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "exported1",
                    kind: NamespaceMemberKind::Variable { is_const: true },
                    visibility: ExportVisibility::Exported,
                    span: (0, 10),
                },
                NamespaceMember {
                    name: "internal",
                    kind: NamespaceMemberKind::Variable { is_const: false },
                    visibility: ExportVisibility::Internal,
                    span: (15, 25),
                },
                NamespaceMember {
                    name: "exported2",
                    kind: NamespaceMemberKind::Function { is_async: false },
                    visibility: ExportVisibility::Exported,
                    span: (30, 40),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 50),
        };

        checker.register_namespace(decl).unwrap();

        let exports = checker.get_exports("NS");
        assert_eq!(exports.len(), 2);
        assert!(exports.iter().all(|m| m.visibility == ExportVisibility::Exported));
    }

    #[test]
    fn test_class_namespace_merge() {
        let mut checker = NamespaceChecker::new();

        // Class declaration
        let decl1 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "MyClass",
                    kind: NamespaceMemberKind::Class,
                    visibility: ExportVisibility::Exported,
                    span: (0, 20),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (0, 30),
        };

        // Namespace with same name (for static members)
        let decl2 = NamespaceDeclaration {
            name: "NS",
            members: vec![
                NamespaceMember {
                    name: "MyClass",
                    kind: NamespaceMemberKind::Namespace,
                    visibility: ExportVisibility::Exported,
                    span: (35, 55),
                },
            ],
            is_ambient: false,
            is_module: false,
            span: (31, 60),
        };

        checker.register_namespace(decl1).unwrap();
        let result = checker.register_namespace(decl2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_qualify_name() {
        assert_eq!(qualify_name("NS", "foo"), "NS.foo");
        assert_eq!(qualify_name("A.B", "C"), "A.B.C");
    }

    #[test]
    fn test_split_qualified_name() {
        assert_eq!(split_qualified_name("NS.foo"), Some(("NS", "foo")));
        assert_eq!(split_qualified_name("A.B.C"), Some(("A.B", "C")));
        assert_eq!(split_qualified_name("simple"), None);
    }
}
