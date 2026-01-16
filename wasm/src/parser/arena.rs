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
        let mut children = Vec::new();

        let Some(node) = self.get(index) else {
            return children;
        };

        use super::ast::Node;

        match node {
            // Names
            Node::QualifiedName { left, right, .. } => {
                children.push(*left);
                children.push(*right);
            }
            Node::ComputedPropertyName { expression, .. } => {
                children.push(*expression);
            }

            // Expressions
            Node::BinaryExpression(bin) => {
                children.push(bin.left);
                children.push(bin.right);
            }
            Node::PrefixUnaryExpression(expr) => {
                children.push(expr.operand);
            }
            Node::PostfixUnaryExpression(expr) => {
                children.push(expr.operand);
            }
            Node::CallExpression(expr) => {
                children.push(expr.expression);
                if let Some(type_args) = expr.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
                children.extend(expr.arguments.nodes.iter().copied());
            }
            Node::NewExpression(expr) => {
                children.push(expr.expression);
                if let Some(type_args) = expr.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
                if let Some(args) = expr.arguments.as_ref() {
                    children.extend(args.nodes.iter().copied());
                }
            }
            Node::TaggedTemplateExpression(expr) => {
                children.push(expr.tag);
                if let Some(type_args) = expr.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
                children.push(expr.template);
            }
            Node::TemplateExpression(expr) => {
                children.push(expr.head);
                children.extend(expr.template_spans.nodes.iter().copied());
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
            Node::ArrowFunction(func) => {
                if let Some(modifiers) = func.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if let Some(type_params) = func.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(func.parameters.nodes.iter().copied());
                if !func.type_annotation.is_none() {
                    children.push(func.type_annotation);
                }
                children.push(func.body);
            }
            Node::FunctionExpression(func) => {
                if let Some(modifiers) = func.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if !func.name.is_none() {
                    children.push(func.name);
                }
                if let Some(type_params) = func.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(func.parameters.nodes.iter().copied());
                if !func.type_annotation.is_none() {
                    children.push(func.type_annotation);
                }
                children.push(func.body);
            }
            Node::ObjectLiteralExpression(expr) => {
                children.extend(expr.properties.nodes.iter().copied());
            }
            Node::ArrayLiteralExpression(expr) => {
                children.extend(expr.elements.nodes.iter().copied());
            }
            Node::ParenthesizedExpression(expr) => {
                children.push(expr.expression);
            }
            Node::YieldExpression(expr) => {
                if !expr.expression.is_none() {
                    children.push(expr.expression);
                }
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
                if let Some(modifiers) = stmt.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(stmt.declaration_list);
            }
            Node::VariableDeclarationList(list) => {
                children.extend(list.declarations.nodes.iter().copied());
            }
            Node::VariableDeclaration(decl) => {
                children.push(decl.name);
                if !decl.type_annotation.is_none() {
                    children.push(decl.type_annotation);
                }
                if !decl.initializer.is_none() {
                    children.push(decl.initializer);
                }
            }
            Node::ExpressionStatement(stmt) => {
                children.push(stmt.expression);
            }
            Node::IfStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.then_statement);
                if !stmt.else_statement.is_none() {
                    children.push(stmt.else_statement);
                }
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
                if !stmt.initializer.is_none() {
                    children.push(stmt.initializer);
                }
                if !stmt.condition.is_none() {
                    children.push(stmt.condition);
                }
                if !stmt.incrementor.is_none() {
                    children.push(stmt.incrementor);
                }
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
                children.extend(block.clauses.nodes.iter().copied());
            }
            Node::CaseClause(clause) => {
                children.push(clause.expression);
                children.extend(clause.statements.nodes.iter().copied());
            }
            Node::DefaultClause(clause) => {
                children.extend(clause.statements.nodes.iter().copied());
            }
            Node::ReturnStatement(stmt) => {
                if !stmt.expression.is_none() {
                    children.push(stmt.expression);
                }
            }
            Node::ThrowStatement(stmt) => {
                children.push(stmt.expression);
            }
            Node::TryStatement(stmt) => {
                children.push(stmt.try_block);
                if !stmt.catch_clause.is_none() {
                    children.push(stmt.catch_clause);
                }
                if !stmt.finally_block.is_none() {
                    children.push(stmt.finally_block);
                }
            }
            Node::CatchClause(clause) => {
                if !clause.variable_declaration.is_none() {
                    children.push(clause.variable_declaration);
                }
                children.push(clause.block);
            }
            Node::LabeledStatement(stmt) => {
                children.push(stmt.label);
                children.push(stmt.statement);
            }
            Node::BreakStatement(stmt) => {
                if !stmt.label.is_none() {
                    children.push(stmt.label);
                }
            }
            Node::ContinueStatement(stmt) => {
                if !stmt.label.is_none() {
                    children.push(stmt.label);
                }
            }
            Node::WithStatement(stmt) => {
                children.push(stmt.expression);
                children.push(stmt.statement);
            }
            Node::Block(block) => {
                children.extend(block.statements.nodes.iter().copied());
            }

            // Declarations
            Node::FunctionDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if !decl.name.is_none() {
                    children.push(decl.name);
                }
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                if !decl.type_annotation.is_none() {
                    children.push(decl.type_annotation);
                }
                if !decl.body.is_none() {
                    children.push(decl.body);
                }
            }
            Node::ClassDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if !decl.name.is_none() {
                    children.push(decl.name);
                }
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                if let Some(heritage) = decl.heritage_clauses.as_ref() {
                    children.extend(heritage.nodes.iter().copied());
                }
                children.extend(decl.members.nodes.iter().copied());
            }
            Node::InterfaceDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                if let Some(heritage) = decl.heritage_clauses.as_ref() {
                    children.extend(heritage.nodes.iter().copied());
                }
                children.extend(decl.members.nodes.iter().copied());
            }
            Node::PropertySignature(sig) => {
                if let Some(modifiers) = sig.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(sig.name);
                if !sig.type_annotation.is_none() {
                    children.push(sig.type_annotation);
                }
                if !sig.initializer.is_none() {
                    children.push(sig.initializer);
                }
            }
            Node::MethodSignature(sig) => {
                if let Some(modifiers) = sig.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(sig.name);
                if let Some(type_params) = sig.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(sig.parameters.nodes.iter().copied());
                if !sig.type_annotation.is_none() {
                    children.push(sig.type_annotation);
                }
            }
            Node::IndexSignatureDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                children.push(decl.type_annotation);
            }
            Node::CallSignature(sig) => {
                if let Some(type_params) = sig.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(sig.parameters.nodes.iter().copied());
                if !sig.type_annotation.is_none() {
                    children.push(sig.type_annotation);
                }
            }
            Node::ConstructSignature(sig) => {
                if let Some(type_params) = sig.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(sig.parameters.nodes.iter().copied());
                if !sig.type_annotation.is_none() {
                    children.push(sig.type_annotation);
                }
            }
            Node::TypeAliasDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.push(decl.type_node);
            }
            Node::EnumDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                children.extend(decl.members.nodes.iter().copied());
            }
            Node::EnumMember(member) => {
                children.push(member.name);
                if !member.initializer.is_none() {
                    children.push(member.initializer);
                }
            }
            Node::ModuleDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                children.push(decl.body);
            }
            Node::ModuleBlock(block) => {
                children.extend(block.statements.nodes.iter().copied());
            }
            Node::PropertyDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if !decl.type_annotation.is_none() {
                    children.push(decl.type_annotation);
                }
                if !decl.initializer.is_none() {
                    children.push(decl.initializer);
                }
            }
            Node::MethodDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                if !decl.type_annotation.is_none() {
                    children.push(decl.type_annotation);
                }
                if !decl.body.is_none() {
                    children.push(decl.body);
                }
            }
            Node::ConstructorDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                children.push(decl.body);
            }
            Node::GetAccessorDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                if !decl.type_annotation.is_none() {
                    children.push(decl.type_annotation);
                }
                children.push(decl.body);
            }
            Node::SetAccessorDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(decl.name);
                if let Some(type_params) = decl.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(decl.parameters.nodes.iter().copied());
                children.push(decl.body);
            }
            Node::ParameterDeclaration(param) => {
                if let Some(modifiers) = param.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(param.name);
                if !param.type_annotation.is_none() {
                    children.push(param.type_annotation);
                }
                if !param.initializer.is_none() {
                    children.push(param.initializer);
                }
            }
            Node::TypeParameterDeclaration(param) => {
                if let Some(modifiers) = param.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(param.name);
                if !param.constraint.is_none() {
                    children.push(param.constraint);
                }
                if !param.default.is_none() {
                    children.push(param.default);
                }
            }
            Node::Decorator(decorator) => {
                children.push(decorator.expression);
            }
            Node::HeritageClause(clause) => {
                children.extend(clause.types.nodes.iter().copied());
            }
            Node::ExpressionWithTypeArguments(expr) => {
                children.push(expr.expression);
                if let Some(type_args) = expr.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
            }
            Node::ImportDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if !decl.import_clause.is_none() {
                    children.push(decl.import_clause);
                }
                children.push(decl.module_specifier);
                if !decl.attributes.is_none() {
                    children.push(decl.attributes);
                }
            }
            Node::ImportClause(clause) => {
                if !clause.name.is_none() {
                    children.push(clause.name);
                }
                if !clause.named_bindings.is_none() {
                    children.push(clause.named_bindings);
                }
            }
            Node::NamespaceImport(import) => {
                children.push(import.name);
            }
            Node::NamedImports(import) => {
                children.extend(import.elements.nodes.iter().copied());
            }
            Node::ImportSpecifier(spec) => {
                if !spec.property_name.is_none() {
                    children.push(spec.property_name);
                }
                children.push(spec.name);
            }
            Node::ExportDeclaration(decl) => {
                if let Some(modifiers) = decl.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if !decl.export_clause.is_none() {
                    children.push(decl.export_clause);
                }
                if !decl.module_specifier.is_none() {
                    children.push(decl.module_specifier);
                }
                if !decl.attributes.is_none() {
                    children.push(decl.attributes);
                }
            }
            Node::NamedExports(exports) => {
                children.extend(exports.elements.nodes.iter().copied());
            }
            Node::NamespaceExport(exp) => {
                children.push(exp.name);
            }
            Node::ExportSpecifier(spec) => {
                if !spec.property_name.is_none() {
                    children.push(spec.property_name);
                }
                children.push(spec.name);
            }
            Node::ExportAssignment(assignment) => {
                if let Some(modifiers) = assignment.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(assignment.expression);
            }
            Node::ImportAttributes(attrs) => {
                children.extend(attrs.elements.nodes.iter().copied());
            }
            Node::ImportAttribute(attr) => {
                children.push(attr.name);
                children.push(attr.value);
            }

            // Binding patterns
            Node::ObjectBindingPattern(pattern) => {
                children.extend(pattern.elements.nodes.iter().copied());
            }
            Node::ArrayBindingPattern(pattern) => {
                children.extend(pattern.elements.nodes.iter().copied());
            }
            Node::BindingElement(elem) => {
                if !elem.property_name.is_none() {
                    children.push(elem.property_name);
                }
                children.push(elem.name);
                if !elem.initializer.is_none() {
                    children.push(elem.initializer);
                }
            }

            // Object literal members
            Node::PropertyAssignment(assign) => {
                if let Some(modifiers) = assign.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(assign.name);
                children.push(assign.initializer);
            }
            Node::ShorthandPropertyAssignment(assign) => {
                if let Some(modifiers) = assign.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                children.push(assign.name);
                if !assign.object_assignment_initializer.is_none() {
                    children.push(assign.object_assignment_initializer);
                }
            }
            Node::SpreadAssignment(assign) => {
                children.push(assign.expression);
            }

            // JSX nodes
            Node::JsxElement(el) => {
                children.push(el.opening_element);
                children.extend(el.children.nodes.iter().copied());
                children.push(el.closing_element);
            }
            Node::JsxSelfClosingElement(el) => {
                children.push(el.tag_name);
                if let Some(type_args) = el.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
                children.push(el.attributes);
            }
            Node::JsxOpeningElement(el) => {
                children.push(el.tag_name);
                if let Some(type_args) = el.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
                children.push(el.attributes);
            }
            Node::JsxClosingElement(el) => {
                children.push(el.tag_name);
            }
            Node::JsxFragment(frag) => {
                children.push(frag.opening_fragment);
                children.extend(frag.children.nodes.iter().copied());
                children.push(frag.closing_fragment);
            }
            Node::JsxAttributes(attrs) => {
                children.extend(attrs.properties.nodes.iter().copied());
            }
            Node::JsxAttribute(attr) => {
                children.push(attr.name);
                children.push(attr.initializer);
            }
            Node::JsxSpreadAttribute(attr) => {
                children.push(attr.expression);
            }
            Node::JsxExpression(expr) => {
                if !expr.expression.is_none() {
                    children.push(expr.expression);
                }
            }
            Node::JsxNamespacedName(name) => {
                children.push(name.namespace);
                children.push(name.name);
            }

            // Template spans
            Node::TemplateSpan(span) => {
                children.push(span.literal);
                children.push(span.expression);
            }

            // Type nodes
            Node::TypeReference(ty) => {
                children.push(ty.type_name);
                if let Some(type_args) = ty.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
            }
            Node::FunctionType(ty) => {
                if let Some(type_params) = ty.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(ty.parameters.nodes.iter().copied());
                children.push(ty.type_node);
            }
            Node::ConstructorType(ty) => {
                if let Some(modifiers) = ty.modifiers.as_ref() {
                    children.extend(modifiers.nodes.iter().copied());
                }
                if let Some(type_params) = ty.type_parameters.as_ref() {
                    children.extend(type_params.nodes.iter().copied());
                }
                children.extend(ty.parameters.nodes.iter().copied());
                children.push(ty.type_node);
            }
            Node::TypeQuery(ty) => {
                children.push(ty.expr_name);
                if let Some(type_args) = ty.type_arguments.as_ref() {
                    children.extend(type_args.nodes.iter().copied());
                }
            }
            Node::TypeLiteral(ty) => {
                children.extend(ty.members.nodes.iter().copied());
            }
            Node::ArrayType(ty) => {
                children.push(ty.element_type);
            }
            Node::TupleType(ty) => {
                children.extend(ty.elements.nodes.iter().copied());
            }
            Node::OptionalType(ty) => {
                children.push(ty.type_node);
            }
            Node::RestType(ty) => {
                children.push(ty.type_node);
            }
            Node::UnionType(ty) => {
                children.extend(ty.types.nodes.iter().copied());
            }
            Node::IntersectionType(ty) => {
                children.extend(ty.types.nodes.iter().copied());
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
                children.push(ty.type_parameter);
                children.push(ty.name_type);
                if let Some(members) = ty.members.as_ref() {
                    children.extend(members.nodes.iter().copied());
                }
            }
            Node::LiteralType(ty) => {
                children.push(ty.literal);
            }
            Node::TemplateLiteralType(ty) => {
                children.push(ty.head);
                children.extend(ty.template_spans.nodes.iter().copied());
            }
            Node::NamedTupleMember(member) => {
                children.push(member.name);
                children.push(member.type_node);
            }
            Node::TypePredicate(pred) => {
                children.push(pred.parameter_name);
                children.push(pred.type_node);
            }

            // Source file
            Node::SourceFile(file) => {
                children.extend(file.statements.nodes.iter().copied());
                children.push(file.end_of_file_token);
            }

            // Leaf nodes (no children)
            Node::Token(_)
            | Node::Identifier(_)
            | Node::PrivateIdentifier(_)
            | Node::StringLiteral(_)
            | Node::NumericLiteral(_)
            | Node::BigIntLiteral(_)
            | Node::RegularExpressionLiteral(_)
            | Node::NoSubstitutionTemplateLiteral(_)
            | Node::TemplateHead(_)
            | Node::TemplateMiddle(_)
            | Node::TemplateTail(_)
            | Node::JsxText(_)
            | Node::JsxOpeningFragment(_)
            | Node::JsxClosingFragment(_)
            | Node::EmptyStatement(_)
            | Node::DebuggerStatement(_)
            | Node::EndOfFileToken(_) => {
                // No children
            }
        }

        children
    }
}
