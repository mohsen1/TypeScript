//! Node arena for AST storage.

use super::ast::{Node, NodeIndex};
use super::thin_node::{NodeAccess, NodeInfo};
use serde::Serialize;

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
        NodeArena {
            nodes: Vec::with_capacity(capacity),
        }
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
            self.nodes
                .get_mut(index.0 as usize)
                .map(|old| std::mem::replace(old, new_node))
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
            Node::Identifier(ident) | Node::PrivateIdentifier(ident) => Some(&ident.escaped_text),
            _ => None,
        }
    }

    fn get_literal_text(&self, index: NodeIndex) -> Option<&str> {
        match self.get(index)? {
            Node::StringLiteral(lit)
            | Node::NoSubstitutionTemplateLiteral(lit)
            | Node::TemplateHead(lit)
            | Node::TemplateMiddle(lit)
            | Node::TemplateTail(lit) => Some(&lit.text),
            Node::NumericLiteral(lit) => Some(&lit.text),
            Node::BigIntLiteral(lit) => Some(&lit.text),
            Node::RegularExpressionLiteral(lit) => Some(&lit.text),
            _ => None,
        }
    }

    fn get_children(&self, index: NodeIndex) -> Vec<NodeIndex> {
        if index.is_none() {
            return Vec::new();
        }

        let node = match self.get(index) {
            Some(n) => n,
            None => return Vec::new(),
        };

        // Helper to add optional NodeIndex (ignoring NONE)
        let add_opt = |children: &mut Vec<NodeIndex>, idx: NodeIndex| {
            if idx.is_some() {
                children.push(idx);
            }
        };

        // Helper to add NodeList (expanding to individual nodes)
        let add_list = |children: &mut Vec<NodeIndex>, list: &super::ast::NodeList| {
            children.extend(list.nodes.iter().copied());
        };

        // Helper to add optional NodeList
        let add_opt_list = |children: &mut Vec<NodeIndex>, list: &Option<super::ast::NodeList>| {
            if let Some(l) = list {
                children.extend(l.nodes.iter().copied());
            }
        };

        let mut children = Vec::new();

        match node {
            // Tokens and simple nodes with no children
            Node::Token(_) | Node::EndOfFileToken(_) => {}
            Node::Identifier(_) | Node::PrivateIdentifier(_) => {}
            Node::StringLiteral(_)
            | Node::NoSubstitutionTemplateLiteral(_)
            | Node::TemplateHead(_)
            | Node::TemplateMiddle(_)
            | Node::TemplateTail(_)
            | Node::NumericLiteral(_)
            | Node::BigIntLiteral(_)
            | Node::RegularExpressionLiteral(_) => {}
            Node::DebuggerStatement(_) | Node::EmptyStatement(_) => {}

            // Names
            Node::QualifiedName { left, right, .. } => {
                children.push(*left);
                children.push(*right);
            }
            Node::ComputedPropertyName { expression, .. } => {
                children.push(*expression);
            }

            // Expressions
            Node::BinaryExpression(expr) => {
                children.push(expr.left);
                children.push(expr.right);
            }
            Node::PrefixUnaryExpression(expr) => {
                children.push(expr.operand);
            }
            Node::PostfixUnaryExpression(expr) => {
                children.push(expr.operand);
            }
            Node::CallExpression(expr) => {
                children.push(expr.expression);
                add_opt_list(&mut children, &expr.type_arguments);
                add_list(&mut children, &expr.arguments);
            }
            Node::NewExpression(expr) => {
                children.push(expr.expression);
                add_opt_list(&mut children, &expr.type_arguments);
                add_opt_list(&mut children, &expr.arguments);
            }
            Node::TaggedTemplateExpression(expr) => {
                children.push(expr.tag);
                add_opt_list(&mut children, &expr.type_arguments);
                children.push(expr.template);
            }
            Node::TemplateExpression(expr) => {
                children.push(expr.head);
                add_list(&mut children, &expr.template_spans);
            }
            Node::PropertyAccessExpression(expr) => {
                children.push(expr.expression);
                children.push(expr.name);
            }
            Node::ElementAccessExpression(expr) => {
                children.push(expr.expression);
                children.push(expr.argument_expression);
            }
            Node::ConditionalExpression(expr) => {
                children.push(expr.condition);
                children.push(expr.when_true);
                children.push(expr.when_false);
            }
            Node::ArrowFunction(expr) => {
                add_opt_list(&mut children, &expr.modifiers);
                add_opt_list(&mut children, &expr.type_parameters);
                add_list(&mut children, &expr.parameters);
                add_opt(&mut children, expr.type_annotation);
                children.push(expr.body);
            }
            Node::FunctionExpression(expr) => {
                add_opt_list(&mut children, &expr.modifiers);
                add_opt(&mut children, expr.name);
                add_opt_list(&mut children, &expr.type_parameters);
                add_list(&mut children, &expr.parameters);
                add_opt(&mut children, expr.type_annotation);
                children.push(expr.body);
            }
            Node::ObjectLiteralExpression(expr) => {
                add_list(&mut children, &expr.properties);
            }
            Node::ArrayLiteralExpression(expr) => {
                add_list(&mut children, &expr.elements);
            }
            Node::ParenthesizedExpression(expr) => {
                children.push(expr.expression);
            }
            Node::YieldExpression(expr) => {
                add_opt(&mut children, expr.expression);
            }
            Node::AwaitExpression(expr) => {
                children.push(expr.expression);
            }
            Node::SpreadElement(expr) => {
                children.push(expr.expression);
            }
            Node::AsExpression(expr) => {
                children.push(expr.expression);
                children.push(expr.type_node);
            }
            Node::SatisfiesExpression(expr) => {
                children.push(expr.expression);
                children.push(expr.type_node);
            }
            Node::NonNullExpression(expr) => {
                children.push(expr.expression);
            }
            Node::TypeAssertion(expr) => {
                children.push(expr.type_node);
                children.push(expr.expression);
            }

            // Statements
            Node::VariableStatement(stmt) => {
                add_opt_list(&mut children, &stmt.modifiers);
                children.push(stmt.declaration_list);
            }
            Node::VariableDeclarationList(stmt) => {
                add_list(&mut children, &stmt.declarations);
            }
            Node::VariableDeclaration(decl) => {
                children.push(decl.name);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.initializer);
            }
            Node::ExpressionStatement(stmt) => {
                children.push(stmt.expression);
            }
            Node::IfStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.then_statement);
                add_opt(&mut children, stmt.else_statement);
            }
            Node::WhileStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.statement);
            }
            Node::DoStatement(stmt) => {
                children.push(stmt.statement);
                children.push(stmt.expression);
            }
            Node::ForStatement(stmt) => {
                add_opt(&mut children, stmt.initializer);
                add_opt(&mut children, stmt.condition);
                add_opt(&mut children, stmt.incrementor);
                children.push(stmt.statement);
            }
            Node::ForInStatement(stmt) => {
                children.push(stmt.initializer);
                children.push(stmt.expression);
                children.push(stmt.statement);
            }
            Node::ForOfStatement(stmt) => {
                children.push(stmt.initializer);
                children.push(stmt.expression);
                children.push(stmt.statement);
            }
            Node::SwitchStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.case_block);
            }
            Node::CaseBlock(block) => {
                add_list(&mut children, &block.clauses);
            }
            Node::CaseClause(clause) => {
                children.push(clause.expression);
                add_list(&mut children, &clause.statements);
            }
            Node::DefaultClause(clause) => {
                add_list(&mut children, &clause.statements);
            }
            Node::ReturnStatement(stmt) => {
                add_opt(&mut children, stmt.expression);
            }
            Node::ThrowStatement(stmt) => {
                children.push(stmt.expression);
            }
            Node::TryStatement(stmt) => {
                children.push(stmt.try_block);
                add_opt(&mut children, stmt.catch_clause);
                add_opt(&mut children, stmt.finally_block);
            }
            Node::CatchClause(clause) => {
                add_opt(&mut children, clause.variable_declaration);
                children.push(clause.block);
            }
            Node::LabeledStatement(stmt) => {
                children.push(stmt.label);
                children.push(stmt.statement);
            }
            Node::BreakStatement(stmt) => {
                add_opt(&mut children, stmt.label);
            }
            Node::ContinueStatement(stmt) => {
                add_opt(&mut children, stmt.label);
            }
            Node::WithStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.statement);
            }
            Node::Block(block) => {
                add_list(&mut children, &block.statements);
            }

            // Declarations
            Node::FunctionDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                add_opt(&mut children, decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_list(&mut children, &decl.parameters);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.body);
            }
            Node::ClassDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                add_opt(&mut children, decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_opt_list(&mut children, &decl.heritage_clauses);
                add_list(&mut children, &decl.members);
            }
            Node::InterfaceDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_opt_list(&mut children, &decl.heritage_clauses);
                add_list(&mut children, &decl.members);
            }
            Node::PropertySignature(sig) => {
                add_opt_list(&mut children, &sig.modifiers);
                children.push(sig.name);
                add_opt(&mut children, sig.type_annotation);
                add_opt(&mut children, sig.initializer);
            }
            Node::MethodSignature(sig) => {
                add_opt_list(&mut children, &sig.modifiers);
                children.push(sig.name);
                add_opt_list(&mut children, &sig.type_parameters);
                add_list(&mut children, &sig.parameters);
                add_opt(&mut children, sig.type_annotation);
            }
            Node::IndexSignatureDeclaration(sig) => {
                add_opt_list(&mut children, &sig.modifiers);
                add_list(&mut children, &sig.parameters);
                add_opt(&mut children, sig.type_annotation);
            }
            Node::CallSignature(sig) => {
                add_opt_list(&mut children, &sig.type_parameters);
                add_list(&mut children, &sig.parameters);
                add_opt(&mut children, sig.type_annotation);
            }
            Node::ConstructSignature(sig) => {
                add_opt_list(&mut children, &sig.type_parameters);
                add_list(&mut children, &sig.parameters);
                add_opt(&mut children, sig.type_annotation);
            }
            Node::TypeAliasDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                children.push(decl.type_node);
            }
            Node::EnumDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_list(&mut children, &decl.members);
            }
            Node::EnumMember(member) => {
                children.push(member.name);
                add_opt(&mut children, member.initializer);
            }
            Node::ModuleDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt(&mut children, decl.body);
            }
            Node::ModuleBlock(block) => {
                add_list(&mut children, &block.statements);
            }

            // Import/Export
            Node::ImportDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                add_opt(&mut children, decl.import_clause);
                children.push(decl.module_specifier);
                add_opt(&mut children, decl.attributes);
            }
            Node::ImportClause(clause) => {
                add_opt(&mut children, clause.name);
                add_opt(&mut children, clause.named_bindings);
            }
            Node::NamespaceImport(import) => {
                children.push(import.name);
            }
            Node::NamedImports(imports) => {
                add_list(&mut children, &imports.elements);
            }
            Node::ImportSpecifier(spec) => {
                add_opt(&mut children, spec.property_name);
                children.push(spec.name);
            }
            Node::ExportDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                add_opt(&mut children, decl.export_clause);
                add_opt(&mut children, decl.module_specifier);
                add_opt(&mut children, decl.attributes);
            }
            Node::NamedExports(exports) => {
                add_list(&mut children, &exports.elements);
            }
            Node::NamespaceExport(export) => {
                children.push(export.name);
            }
            Node::ExportSpecifier(spec) => {
                add_opt(&mut children, spec.property_name);
                children.push(spec.name);
            }
            Node::ExportAssignment(assign) => {
                add_opt_list(&mut children, &assign.modifiers);
                children.push(assign.expression);
            }
            Node::ImportAttributes(attrs) => {
                add_list(&mut children, &attrs.elements);
            }
            Node::ImportAttribute(attr) => {
                children.push(attr.name);
                children.push(attr.value);
            }

            // Type nodes
            Node::TypeReference(ty) => {
                children.push(ty.type_name);
                add_opt_list(&mut children, &ty.type_arguments);
            }
            Node::FunctionType(ty) => {
                add_opt_list(&mut children, &ty.type_parameters);
                add_list(&mut children, &ty.parameters);
                children.push(ty.type_node);
            }
            Node::ConstructorType(ty) => {
                add_opt_list(&mut children, &ty.modifiers);
                add_opt_list(&mut children, &ty.type_parameters);
                add_list(&mut children, &ty.parameters);
                children.push(ty.type_node);
            }
            Node::TypeQuery(ty) => {
                children.push(ty.expr_name);
                add_opt_list(&mut children, &ty.type_arguments);
            }
            Node::TypeLiteral(ty) => {
                add_list(&mut children, &ty.members);
            }
            Node::ArrayType(ty) => {
                children.push(ty.element_type);
            }
            Node::TupleType(ty) => {
                add_list(&mut children, &ty.elements);
            }
            Node::OptionalType(ty) => {
                children.push(ty.type_node);
            }
            Node::RestType(ty) => {
                children.push(ty.type_node);
            }
            Node::UnionType(ty) => {
                add_list(&mut children, &ty.types);
            }
            Node::IntersectionType(ty) => {
                add_list(&mut children, &ty.types);
            }
            Node::ConditionalType(ty) => {
                children.push(ty.check_type);
                children.push(ty.extends_type);
                children.push(ty.true_type);
                children.push(ty.false_type);
            }
            Node::InferType(ty) => {
                children.push(ty.type_parameter);
            }
            Node::ParenthesizedType(ty) => {
                children.push(ty.type_node);
            }
            Node::TypeOperator(ty) => {
                children.push(ty.type_node);
            }
            Node::IndexedAccessType(ty) => {
                children.push(ty.object_type);
                children.push(ty.index_type);
            }
            Node::MappedType(ty) => {
                add_opt(&mut children, ty.type_parameter);
                add_opt(&mut children, ty.name_type);
                add_opt(&mut children, ty.type_node);
                add_opt_list(&mut children, &ty.members);
            }
            Node::LiteralType(ty) => {
                children.push(ty.literal);
            }
            Node::TemplateLiteralType(ty) => {
                children.push(ty.head);
                add_list(&mut children, &ty.template_spans);
            }
            Node::NamedTupleMember(member) => {
                children.push(member.name);
                children.push(member.type_node);
            }
            Node::TypePredicate(pred) => {
                children.push(pred.parameter_name);
                add_opt(&mut children, pred.type_node);
            }

            // Class members
            Node::PropertyDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.initializer);
            }
            Node::MethodDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_list(&mut children, &decl.parameters);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.body);
            }
            Node::ConstructorDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                add_opt_list(&mut children, &decl.type_parameters);
                add_list(&mut children, &decl.parameters);
                add_opt(&mut children, decl.body);
            }
            Node::GetAccessorDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_list(&mut children, &decl.parameters);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.body);
            }
            Node::SetAccessorDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt_list(&mut children, &decl.type_parameters);
                add_list(&mut children, &decl.parameters);
                add_opt(&mut children, decl.body);
            }
            Node::ParameterDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.initializer);
            }
            Node::TypeParameterDeclaration(decl) => {
                add_opt_list(&mut children, &decl.modifiers);
                children.push(decl.name);
                add_opt(&mut children, decl.constraint);
                add_opt(&mut children, decl.default);
            }
            Node::Decorator(dec) => {
                children.push(dec.expression);
            }
            Node::HeritageClause(clause) => {
                add_list(&mut children, &clause.types);
            }
            Node::ExpressionWithTypeArguments(expr) => {
                children.push(expr.expression);
                add_opt_list(&mut children, &expr.type_arguments);
            }

            // Binding patterns
            Node::ObjectBindingPattern(pattern) => {
                add_list(&mut children, &pattern.elements);
            }
            Node::ArrayBindingPattern(pattern) => {
                add_list(&mut children, &pattern.elements);
            }
            Node::BindingElement(elem) => {
                add_opt(&mut children, elem.property_name);
                children.push(elem.name);
                add_opt(&mut children, elem.initializer);
            }

            // Object literal members
            Node::PropertyAssignment(assign) => {
                add_opt_list(&mut children, &assign.modifiers);
                children.push(assign.name);
                children.push(assign.initializer);
            }
            Node::ShorthandPropertyAssignment(assign) => {
                add_opt_list(&mut children, &assign.modifiers);
                children.push(assign.name);
                add_opt(&mut children, assign.object_assignment_initializer);
            }
            Node::SpreadAssignment(assign) => {
                children.push(assign.expression);
            }

            // JSX nodes
            Node::JsxElement(elem) => {
                children.push(elem.opening_element);
                add_list(&mut children, &elem.children);
                children.push(elem.closing_element);
            }
            Node::JsxSelfClosingElement(elem) => {
                children.push(elem.tag_name);
                add_opt_list(&mut children, &elem.type_arguments);
                children.push(elem.attributes);
            }
            Node::JsxOpeningElement(elem) => {
                children.push(elem.tag_name);
                add_opt_list(&mut children, &elem.type_arguments);
                children.push(elem.attributes);
            }
            Node::JsxClosingElement(elem) => {
                children.push(elem.tag_name);
            }
            Node::JsxFragment(frag) => {
                children.push(frag.opening_fragment);
                add_list(&mut children, &frag.children);
                children.push(frag.closing_fragment);
            }
            Node::JsxOpeningFragment(_) | Node::JsxClosingFragment(_) => {
                // No children
            }
            Node::JsxAttributes(attrs) => {
                add_list(&mut children, &attrs.properties);
            }
            Node::JsxAttribute(attr) => {
                children.push(attr.name);
                add_opt(&mut children, attr.initializer);
            }
            Node::JsxSpreadAttribute(attr) => {
                children.push(attr.expression);
            }
            Node::JsxExpression(expr) => {
                add_opt(&mut children, expr.expression);
            }
            Node::JsxText(_) => {
                // No children
            }
            Node::JsxNamespacedName(name) => {
                children.push(name.namespace);
                children.push(name.name);
            }

            // Misc
            Node::TemplateSpan(span) => {
                children.push(span.expression);
                children.push(span.literal);
            }

            // Source file
            Node::SourceFile(sf) => {
                add_list(&mut children, &sf.statements);
                children.push(sf.end_of_file_token);
            }
        }

        children
    }
}
