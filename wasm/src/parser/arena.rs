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
            Node::PrefixUnaryExpression(unary) => {
                children.push(unary.operand);
            }
            Node::PostfixUnaryExpression(unary) => {
                children.push(unary.operand);
            }
            Node::CallExpression(call) => {
                children.push(call.expression);
                add_opt_list(&mut children, &call.type_arguments);
                add_list(&mut children, &call.arguments);
            }
            Node::NewExpression(new_expr) => {
                children.push(new_expr.expression);
                add_opt_list(&mut children, &new_expr.type_arguments);
                add_opt_list(&mut children, &new_expr.arguments);
            }
            Node::TaggedTemplateExpression(tagged) => {
                children.push(tagged.tag);
                add_opt_list(&mut children, &tagged.type_arguments);
                children.push(tagged.template);
            }
            Node::TemplateExpression(template) => {
                children.push(template.head);
                add_list(&mut children, &template.template_spans);
            }
            Node::PropertyAccessExpression(prop) => {
                children.push(prop.expression);
                children.push(prop.name);
            }
            Node::ElementAccessExpression(elem) => {
                children.push(elem.expression);
                children.push(elem.argument_expression);
            }
            Node::ConditionalExpression(cond) => {
                children.push(cond.condition);
                children.push(cond.when_true);
                children.push(cond.when_false);
            }
            Node::ArrowFunction(arrow) => {
                add_opt_list(&mut children, &arrow.modifiers);
                add_opt_list(&mut children, &arrow.type_parameters);
                add_list(&mut children, &arrow.parameters);
                add_opt(&mut children, arrow.type_annotation);
                children.push(arrow.body);
            }
            Node::FunctionExpression(func) => {
                add_opt_list(&mut children, &func.modifiers);
                add_opt(&mut children, func.name);
                add_opt_list(&mut children, &func.type_parameters);
                add_list(&mut children, &func.parameters);
                add_opt(&mut children, func.type_annotation);
                children.push(func.body);
            }
            Node::ObjectLiteralExpression(obj) => {
                add_list(&mut children, &obj.properties);
            }
            Node::ArrayLiteralExpression(arr) => {
                add_list(&mut children, &arr.elements);
            }
            Node::ParenthesizedExpression(paren) => {
                children.push(paren.expression);
            }
            Node::YieldExpression(yield_expr) => {
                add_opt(&mut children, yield_expr.expression);
            }
            Node::AwaitExpression(await_expr) => {
                children.push(await_expr.expression);
            }
            Node::SpreadElement(spread) => {
                children.push(spread.expression);
            }
            Node::AsExpression(as_expr) => {
                children.push(as_expr.expression);
                children.push(as_expr.type_node);
            }
            Node::SatisfiesExpression(sat) => {
                children.push(sat.expression);
                children.push(sat.type_node);
            }
            Node::NonNullExpression(non_null) => {
                children.push(non_null.expression);
            }
            Node::TypeAssertion(assertion) => {
                children.push(assertion.type_node);
                children.push(assertion.expression);
            }

            // Statements
            Node::VariableStatement(var_stmt) => {
                add_opt_list(&mut children, &var_stmt.modifiers);
                children.push(var_stmt.declaration_list);
            }
            Node::VariableDeclarationList(list) => {
                add_list(&mut children, &list.declarations);
            }
            Node::VariableDeclaration(decl) => {
                children.push(decl.name);
                add_opt(&mut children, decl.type_annotation);
                add_opt(&mut children, decl.initializer);
            }
            Node::ExpressionStatement(expr_stmt) => {
                children.push(expr_stmt.expression);
            }
            Node::IfStatement(if_stmt) => {
                children.push(if_stmt.expression);
                children.push(if_stmt.then_statement);
                add_opt(&mut children, if_stmt.else_statement);
            }
            Node::WhileStatement(while_stmt) => {
                children.push(while_stmt.expression);
                children.push(while_stmt.statement);
            }
            Node::DoStatement(do_stmt) => {
                children.push(do_stmt.statement);
                children.push(do_stmt.expression);
            }
            Node::ForStatement(for_stmt) => {
                add_opt(&mut children, for_stmt.initializer);
                add_opt(&mut children, for_stmt.condition);
                add_opt(&mut children, for_stmt.incrementor);
                children.push(for_stmt.statement);
            }
            Node::ForInStatement(for_in) => {
                children.push(for_in.initializer);
                children.push(for_in.expression);
                children.push(for_in.statement);
            }
            Node::ForOfStatement(for_of) => {
                children.push(for_of.initializer);
                children.push(for_of.expression);
                children.push(for_of.statement);
            }
            Node::SwitchStatement(switch) => {
                children.push(switch.expression);
                children.push(switch.case_block);
            }
            Node::CaseBlock(case_block) => {
                add_list(&mut children, &case_block.clauses);
            }
            Node::CaseClause(case_clause) => {
                add_opt(&mut children, case_clause.expression);
                add_list(&mut children, &case_clause.statements);
            }
            Node::DefaultClause(default) => {
                add_list(&mut children, &default.statements);
            }
            Node::ReturnStatement(ret) => {
                add_opt(&mut children, ret.expression);
            }
            Node::ThrowStatement(throw) => {
                children.push(throw.expression);
            }
            Node::TryStatement(try_stmt) => {
                children.push(try_stmt.try_block);
                add_opt(&mut children, try_stmt.catch_clause);
                add_opt(&mut children, try_stmt.finally_block);
            }
            Node::CatchClause(catch) => {
                add_opt(&mut children, catch.variable_declaration);
                children.push(catch.block);
            }
            Node::LabeledStatement(labeled) => {
                children.push(labeled.label);
                children.push(labeled.statement);
            }
            Node::BreakStatement(brk) => {
                add_opt(&mut children, brk.label);
            }
            Node::ContinueStatement(cont) => {
                add_opt(&mut children, cont.label);
            }
            Node::WithStatement(with_stmt) => {
                children.push(with_stmt.expression);
                children.push(with_stmt.statement);
            }
            Node::Block(block) => {
                add_list(&mut children, &block.statements);
            }

            // Declarations
            Node::FunctionDeclaration(func) => {
                add_opt_list(&mut children, &func.modifiers);
                add_opt(&mut children, func.name);
                add_opt_list(&mut children, &func.type_parameters);
                add_list(&mut children, &func.parameters);
                add_opt(&mut children, func.type_annotation);
                children.push(func.body);
            }
            Node::ClassDeclaration(class_decl) => {
                add_opt_list(&mut children, &class_decl.modifiers);
                add_opt(&mut children, class_decl.name);
                add_opt_list(&mut children, &class_decl.type_parameters);
                add_opt_list(&mut children, &class_decl.heritage_clauses);
                add_list(&mut children, &class_decl.members);
            }
            Node::InterfaceDeclaration(iface) => {
                add_opt_list(&mut children, &iface.modifiers);
                add_opt(&mut children, iface.name);
                add_opt_list(&mut children, &iface.type_parameters);
                add_opt_list(&mut children, &iface.heritage_clauses);
                add_list(&mut children, &iface.members);
            }
            Node::PropertySignature(prop) => {
                add_opt_list(&mut children, &prop.modifiers);
                add_opt(&mut children, prop.name);
                add_opt(&mut children, prop.type_annotation);
                add_opt(&mut children, prop.initializer);
            }
            Node::MethodSignature(method) => {
                add_opt_list(&mut children, &method.modifiers);
                add_opt(&mut children, method.name);
                add_opt_list(&mut children, &method.type_parameters);
                add_list(&mut children, &method.parameters);
                add_opt(&mut children, method.type_annotation);
            }
            Node::IndexSignatureDeclaration(idx) => {
                add_opt_list(&mut children, &idx.modifiers);
                add_list(&mut children, &idx.parameters);
                add_opt(&mut children, idx.type_annotation);
            }
            Node::CallSignature(call) => {
                add_opt_list(&mut children, &call.type_parameters);
                add_list(&mut children, &call.parameters);
                add_opt(&mut children, call.type_annotation);
            }
            Node::ConstructSignature(construct) => {
                add_opt_list(&mut children, &construct.type_parameters);
                add_list(&mut children, &construct.parameters);
                add_opt(&mut children, construct.type_annotation);
            }
            Node::TypeAliasDeclaration(alias) => {
                add_opt_list(&mut children, &alias.modifiers);
                add_opt(&mut children, alias.name);
                add_opt_list(&mut children, &alias.type_parameters);
                children.push(alias.type_node);
            }
            Node::EnumDeclaration(enum_decl) => {
                add_opt_list(&mut children, &enum_decl.modifiers);
                add_opt(&mut children, enum_decl.name);
                add_list(&mut children, &enum_decl.members);
            }
            Node::EnumMember(member) => {
                add_opt(&mut children, member.name);
                add_opt(&mut children, member.initializer);
            }
            Node::ModuleDeclaration(module) => {
                add_opt_list(&mut children, &module.modifiers);
                add_opt(&mut children, module.name);
                add_opt(&mut children, module.body);
            }
            Node::ModuleBlock(block) => {
                add_list(&mut children, &block.statements);
            }

            // Import/Export
            Node::ImportDeclaration(imp) => {
                add_opt_list(&mut children, &imp.modifiers);
                add_opt(&mut children, imp.import_clause);
                children.push(imp.module_specifier);
                add_opt(&mut children, imp.attributes);
            }
            Node::ImportClause(clause) => {
                add_opt(&mut children, clause.name);
                add_opt(&mut children, clause.named_bindings);
            }
            Node::NamespaceImport(ns) => {
                children.push(ns.name);
            }
            Node::NamedImports(named) => {
                add_list(&mut children, &named.elements);
            }
            Node::ImportSpecifier(spec) => {
                add_opt(&mut children, spec.property_name);
                children.push(spec.name);
            }
            Node::ExportDeclaration(exp) => {
                add_opt_list(&mut children, &exp.modifiers);
                add_opt(&mut children, exp.export_clause);
                add_opt(&mut children, exp.module_specifier);
                add_opt(&mut children, exp.attributes);
            }
            Node::NamedExports(named) => {
                add_list(&mut children, &named.elements);
            }
            Node::NamespaceExport(ns) => {
                children.push(ns.name);
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
            Node::TypeReference(type_ref) => {
                children.push(type_ref.type_name);
                add_opt_list(&mut children, &type_ref.type_arguments);
            }
            Node::FunctionType(func_type) => {
                add_opt_list(&mut children, &func_type.type_parameters);
                add_list(&mut children, &func_type.parameters);
                children.push(func_type.type_node);
            }
            Node::ConstructorType(ctor_type) => {
                add_opt_list(&mut children, &ctor_type.modifiers);
                add_opt_list(&mut children, &ctor_type.type_parameters);
                add_list(&mut children, &ctor_type.parameters);
                children.push(ctor_type.type_node);
            }
            Node::TypeQuery(type_query) => {
                children.push(type_query.expr_name);
                add_opt_list(&mut children, &type_query.type_arguments);
            }
            Node::TypeLiteral(type_lit) => {
                add_list(&mut children, &type_lit.members);
            }
            Node::ArrayType(arr_type) => {
                children.push(arr_type.element_type);
            }
            Node::TupleType(tuple) => {
                add_list(&mut children, &tuple.elements);
            }
            Node::OptionalType(opt) => {
                children.push(opt.type_node);
            }
            Node::RestType(rest) => {
                children.push(rest.type_node);
            }
            Node::UnionType(union) => {
                add_list(&mut children, &union.types);
            }
            Node::IntersectionType(intersection) => {
                add_list(&mut children, &intersection.types);
            }
            Node::ConditionalType(cond) => {
                children.push(cond.check_type);
                children.push(cond.extends_type);
                children.push(cond.true_type);
                children.push(cond.false_type);
            }
            Node::InferType(infer) => {
                children.push(infer.type_parameter);
            }
            Node::ParenthesizedType(paren) => {
                children.push(paren.type_node);
            }
            Node::TypeOperator(op) => {
                children.push(op.type_node);
            }
            Node::IndexedAccessType(idx) => {
                children.push(idx.object_type);
                children.push(idx.index_type);
            }
            Node::MappedType(mapped) => {
                add_opt(&mut children, mapped.type_parameter);
                add_opt(&mut children, mapped.name_type);
                add_opt(&mut children, mapped.type_node);
                add_opt_list(&mut children, &mapped.members);
            }
            Node::LiteralType(lit) => {
                add_opt(&mut children, lit.literal);
            }
            Node::TemplateLiteralType(template) => {
                children.push(template.head);
                add_list(&mut children, &template.template_spans);
            }
            Node::NamedTupleMember(named) => {
                children.push(named.name);
                children.push(named.type_node);
            }
            Node::TypePredicate(pred) => {
                children.push(pred.parameter_name);
                add_opt(&mut children, pred.type_node);
            }

            // Class members
            Node::PropertyDeclaration(prop) => {
                add_opt_list(&mut children, &prop.modifiers);
                add_opt(&mut children, prop.name);
                add_opt(&mut children, prop.type_annotation);
                add_opt(&mut children, prop.initializer);
            }
            Node::MethodDeclaration(method) => {
                add_opt_list(&mut children, &method.modifiers);
                add_opt(&mut children, method.name);
                add_opt_list(&mut children, &method.type_parameters);
                add_list(&mut children, &method.parameters);
                add_opt(&mut children, method.type_annotation);
                children.push(method.body);
            }
            Node::ConstructorDeclaration(ctor) => {
                add_opt_list(&mut children, &ctor.modifiers);
                add_opt_list(&mut children, &ctor.type_parameters);
                add_list(&mut children, &ctor.parameters);
                children.push(ctor.body);
            }
            Node::GetAccessorDeclaration(getter) => {
                add_opt_list(&mut children, &getter.modifiers);
                add_opt(&mut children, getter.name);
                add_opt_list(&mut children, &getter.type_parameters);
                add_list(&mut children, &getter.parameters);
                add_opt(&mut children, getter.type_annotation);
                children.push(getter.body);
            }
            Node::SetAccessorDeclaration(setter) => {
                add_opt_list(&mut children, &setter.modifiers);
                add_opt(&mut children, setter.name);
                add_opt_list(&mut children, &setter.type_parameters);
                add_list(&mut children, &setter.parameters);
                children.push(setter.body);
            }
            Node::ParameterDeclaration(param) => {
                add_opt_list(&mut children, &param.modifiers);
                add_opt(&mut children, param.name);
                add_opt(&mut children, param.type_annotation);
                add_opt(&mut children, param.initializer);
            }
            Node::TypeParameterDeclaration(type_param) => {
                add_opt_list(&mut children, &type_param.modifiers);
                children.push(type_param.name);
                add_opt(&mut children, type_param.constraint);
                add_opt(&mut children, type_param.default);
            }
            Node::Decorator(decorator) => {
                children.push(decorator.expression);
            }
            Node::HeritageClause(heritage) => {
                add_list(&mut children, &heritage.types);
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
            Node::PropertyAssignment(prop) => {
                add_opt_list(&mut children, &prop.modifiers);
                add_opt(&mut children, prop.name);
                children.push(prop.initializer);
            }
            Node::ShorthandPropertyAssignment(shorthand) => {
                add_opt_list(&mut children, &shorthand.modifiers);
                children.push(shorthand.name);
                add_opt(&mut children, shorthand.object_assignment_initializer);
            }
            Node::SpreadAssignment(spread) => {
                children.push(spread.expression);
            }

            // JSX nodes
            Node::JsxElement(elem) => {
                children.push(elem.opening_element);
                add_list(&mut children, &elem.children);
                add_opt(&mut children, elem.closing_element);
            }
            Node::JsxSelfClosingElement(elem) => {
                children.push(elem.tag_name);
                add_opt_list(&mut children, &elem.type_arguments);
                add_opt(&mut children, elem.attributes);
            }
            Node::JsxOpeningElement(elem) => {
                children.push(elem.tag_name);
                add_opt_list(&mut children, &elem.type_arguments);
                add_opt(&mut children, elem.attributes);
            }
            Node::JsxClosingElement(elem) => {
                children.push(elem.tag_name);
            }
            Node::JsxFragment(frag) => {
                children.push(frag.opening_fragment);
                add_list(&mut children, &frag.children);
                children.push(frag.closing_fragment);
            }
            Node::JsxOpeningFragment(_) => {}
            Node::JsxClosingFragment(_) => {}
            Node::JsxAttributes(attrs) => {
                add_list(&mut children, &attrs.properties);
            }
            Node::JsxAttribute(attr) => {
                children.push(attr.name);
                add_opt(&mut children, attr.initializer);
            }
            Node::JsxSpreadAttribute(spread) => {
                children.push(spread.expression);
            }
            Node::JsxExpression(expr) => {
                add_opt(&mut children, expr.expression);
            }
            Node::JsxText(_) => {}
            Node::JsxNamespacedName(ns) => {
                children.push(ns.namespace);
                children.push(ns.name);
            }

            // Misc
            Node::TemplateSpan(span) => {
                children.push(span.expression);
                children.push(span.literal);
            }

            // Source file
            Node::SourceFile(source_file) => {
                add_list(&mut children, &source_file.statements);
                children.push(source_file.end_of_file_token);
            }

            // Nodes with no children (tokens, identifiers, literals)
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
            | Node::EndOfFileToken(_)
            | Node::EmptyStatement(_)
            | Node::DebuggerStatement(_) => {}
        }

        children
    }
}
