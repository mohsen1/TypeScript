//! Declaration File Emitter (Phase 6.4)
//!
//! Generates .d.ts declaration files from TypeScript source.
//! This emitter strips implementation details and preserves only type information.

use crate::parser::{Node, NodeList, NodeIndex};
use crate::scanner::SyntaxKind;
use crate::emitter::{Printer, PrinterOptions};

/// Options for declaration emission.
#[derive(Clone, Debug, Default)]
pub struct DeclarationEmitterOptions {
    /// Include private members in output (for internal declarations)
    pub include_private: bool,
    /// Strip all implementation details
    pub strip_implementations: bool,
}

/// Declaration emitter that generates .d.ts content.
pub struct DeclarationEmitter {
    printer: Printer,
    options: DeclarationEmitterOptions,
}

impl DeclarationEmitter {
    /// Create a new declaration emitter.
    pub fn new() -> Self {
        DeclarationEmitter {
            printer: Printer::new(),
            options: DeclarationEmitterOptions::default(),
        }
    }

    /// Create a new declaration emitter with options.
    pub fn with_options(options: DeclarationEmitterOptions) -> Self {
        DeclarationEmitter {
            printer: Printer::new(),
            options,
        }
    }

    /// Get the emitted declaration output.
    pub fn get_output(&self) -> &str {
        self.printer.get_output()
    }

    /// Take the emitted output.
    pub fn take_output(self) -> String {
        self.printer.take_output()
    }

    /// Emit a source file as a declaration.
    pub fn emit_source_file(&mut self, node: &Node, arena: &crate::parser::NodeArena) {
        if let Node::SourceFile(sf) = node {
            for stmt_idx in &sf.statements.nodes {
                if let Some(stmt) = arena.get(*stmt_idx) {
                    if self.is_declaration_node(stmt, arena) {
                        self.emit_declaration(stmt, arena);
                        self.printer.write_line();
                    }
                }
            }
        }
    }

    /// Check if a node should be emitted in declarations.
    fn is_declaration_node(&self, node: &Node, arena: &crate::parser::NodeArena) -> bool {
        match node {
            // Always include these in declarations
            Node::InterfaceDeclaration(_) => true,
            Node::TypeAliasDeclaration(_) => true,
            Node::EnumDeclaration(_) => true,
            Node::ModuleDeclaration(_) => true,

            // Include classes, functions, variables if exported or ambient
            Node::ClassDeclaration(decl) => self.has_export_modifier(&decl.modifiers, arena),
            Node::FunctionDeclaration(decl) => self.has_export_modifier(&decl.modifiers, arena),
            Node::VariableStatement(stmt) => self.has_export_modifier(&stmt.modifiers, arena),

            // Include export declarations
            Node::ExportDeclaration(_) => true,
            Node::ExportAssignment(_) => true,
            Node::ImportDeclaration(_) => true,

            _ => false,
        }
    }

    /// Check if modifiers include export or declare.
    fn has_export_modifier(&self, modifiers: &Option<NodeList>, arena: &crate::parser::NodeArena) -> bool {
        if let Some(mods) = modifiers {
            for mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                    match SyntaxKind::try_from_u16(base.kind) {
                        Some(SyntaxKind::ExportKeyword) |
                        Some(SyntaxKind::DeclareKeyword) => return true,
                        _ => {}
                    }
                }
            }
        }
        false
    }

    /// Check if member is private.
    fn is_private_member(&self, modifiers: &Option<NodeList>, arena: &crate::parser::NodeArena) -> bool {
        if let Some(mods) = modifiers {
            for mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                    if let Some(SyntaxKind::PrivateKeyword) = SyntaxKind::try_from_u16(base.kind) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Emit a declaration node.
    fn emit_declaration(&mut self, node: &Node, arena: &crate::parser::NodeArena) {
        match node {
            Node::InterfaceDeclaration(decl) => self.emit_interface_declaration(decl, arena),
            Node::TypeAliasDeclaration(decl) => self.emit_type_alias_declaration(decl, arena),
            Node::EnumDeclaration(decl) => self.emit_enum_declaration(decl, arena),
            Node::ClassDeclaration(decl) => self.emit_class_declaration(decl, arena),
            Node::FunctionDeclaration(decl) => self.emit_function_declaration(decl, arena),
            Node::VariableStatement(stmt) => self.emit_variable_statement(stmt, arena),
            Node::ModuleDeclaration(decl) => self.emit_module_declaration(decl, arena),
            Node::ExportDeclaration(decl) => self.emit_export_declaration_or_inner(decl, arena),
            Node::ExportAssignment(decl) => self.emit_export_assignment(decl, arena),
            Node::ImportDeclaration(decl) => self.emit_import_declaration(decl, arena),
            _ => {}
        }
    }

    /// Emit an export declaration, or unwrap and emit the inner declaration with export keyword.
    fn emit_export_declaration_or_inner(&mut self, decl: &crate::parser::declarations::ExportDeclaration, arena: &crate::parser::NodeArena) {
        // Check if export_clause contains an actual declaration (like TypeAliasDeclaration)
        // vs named exports or namespace exports
        if let Some(inner) = arena.get(decl.export_clause) {
            match inner {
                // These are actual declarations that should be emitted with 'export' prefix
                Node::TypeAliasDeclaration(_) |
                Node::InterfaceDeclaration(_) |
                Node::EnumDeclaration(_) |
                Node::ClassDeclaration(_) |
                Node::FunctionDeclaration(_) |
                Node::VariableStatement(_) => {
                    self.printer.write("export ");
                    self.emit_declaration(inner, arena);
                    return;
                }
                // Named exports and namespace exports go through normal export declaration emit
                _ => {}
            }
        }
        // Fall through to normal export declaration emit
        self.emit_export_declaration(decl, arena);
    }

    fn emit_interface_declaration(&mut self, decl: &crate::parser::declarations::InterfaceDeclaration, arena: &crate::parser::NodeArena) {
        // Emit modifiers (export, declare)
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("interface ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        // Type parameters
        if let Some(ref type_params) = decl.type_parameters {
            self.printer.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.printer.write(">");
        }
        // Heritage clauses
        if let Some(ref heritage) = decl.heritage_clauses {
            for clause_idx in &heritage.nodes {
                if let Some(clause) = arena.get(*clause_idx) {
                    self.printer.write(" ");
                    self.printer.emit_node(clause, arena);
                }
            }
        }
        self.printer.write(" {");
        if !decl.members.nodes.is_empty() {
            self.printer.write_line();
            self.printer.increase_indent();
            for member_idx in &decl.members.nodes {
                if let Some(member) = arena.get(*member_idx) {
                    self.printer.emit_node(member, arena);
                    self.printer.write(";");
                    self.printer.write_line();
                }
            }
            self.printer.decrease_indent();
        }
        self.printer.write("}");
    }

    fn emit_type_alias_declaration(&mut self, decl: &crate::parser::declarations::TypeAliasDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("type ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        if let Some(ref type_params) = decl.type_parameters {
            self.printer.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.printer.write(">");
        }
        self.printer.write(" = ");
        if let Some(ty) = arena.get(decl.type_node) {
            self.printer.emit_node(ty, arena);
        }
        self.printer.write(";");
    }

    fn emit_enum_declaration(&mut self, decl: &crate::parser::declarations::EnumDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("enum ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        self.printer.write(" {");
        self.printer.write_line();
        self.printer.increase_indent();
        for (i, member_idx) in decl.members.nodes.iter().enumerate() {
            if let Some(member) = arena.get(*member_idx) {
                self.printer.emit_node(member, arena);
                if i < decl.members.nodes.len() - 1 {
                    self.printer.write(",");
                }
                self.printer.write_line();
            }
        }
        self.printer.decrease_indent();
        self.printer.write("}");
    }

    fn emit_class_declaration(&mut self, decl: &crate::parser::declarations::ClassDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("class ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        // Type parameters
        if let Some(ref type_params) = decl.type_parameters {
            self.printer.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.printer.write(">");
        }
        // Heritage clauses
        if let Some(ref heritage) = decl.heritage_clauses {
            for clause_idx in &heritage.nodes {
                if let Some(clause) = arena.get(*clause_idx) {
                    self.printer.write(" ");
                    self.printer.emit_node(clause, arena);
                }
            }
        }
        self.printer.write(" {");
        if !decl.members.nodes.is_empty() {
            self.printer.write_line();
            self.printer.increase_indent();
            for member_idx in &decl.members.nodes {
                if let Some(member) = arena.get(*member_idx) {
                    // Skip private members unless include_private is set
                    if !self.options.include_private && self.is_member_private(member, arena) {
                        continue;
                    }
                    self.emit_class_member(member, arena);
                    self.printer.write_line();
                }
            }
            self.printer.decrease_indent();
        }
        self.printer.write("}");
    }

    fn is_member_private(&self, member: &Node, arena: &crate::parser::NodeArena) -> bool {
        match member {
            Node::PropertyDeclaration(decl) => self.is_private_member(&decl.modifiers, arena),
            Node::MethodDeclaration(decl) => self.is_private_member(&decl.modifiers, arena),
            Node::ConstructorDeclaration(_) => false,
            Node::GetAccessorDeclaration(decl) => self.is_private_member(&decl.modifiers, arena),
            Node::SetAccessorDeclaration(decl) => self.is_private_member(&decl.modifiers, arena),
            _ => false,
        }
    }

    fn emit_class_member(&mut self, member: &Node, arena: &crate::parser::NodeArena) {
        match member {
            Node::PropertyDeclaration(decl) => {
                if let Some(ref mods) = decl.modifiers {
                    self.emit_modifiers(mods, arena);
                }
                if let Some(name) = arena.get(decl.name) {
                    self.printer.emit_node(name, arena);
                }
                if decl.question_token {
                    self.printer.write("?");
                }
                if !decl.type_annotation.is_none() {
                    self.printer.write(": ");
                    if let Some(ty) = arena.get(decl.type_annotation) {
                        self.printer.emit_node(ty, arena);
                    }
                }
                self.printer.write(";");
            }
            Node::MethodDeclaration(decl) => {
                if let Some(ref mods) = decl.modifiers {
                    self.emit_modifiers(mods, arena);
                }
                if let Some(name) = arena.get(decl.name) {
                    self.printer.emit_node(name, arena);
                }
                if decl.question_token {
                    self.printer.write("?");
                }
                // Type parameters
                if let Some(ref type_params) = decl.type_parameters {
                    self.printer.write("<");
                    self.emit_node_list(type_params, arena, ", ");
                    self.printer.write(">");
                }
                self.printer.write("(");
                self.emit_node_list(&decl.parameters, arena, ", ");
                self.printer.write(")");
                if !decl.type_annotation.is_none() {
                    self.printer.write(": ");
                    if let Some(ty) = arena.get(decl.type_annotation) {
                        self.printer.emit_node(ty, arena);
                    }
                }
                self.printer.write(";");
            }
            Node::ConstructorDeclaration(decl) => {
                self.printer.write("constructor(");
                self.emit_node_list(&decl.parameters, arena, ", ");
                self.printer.write(");");
            }
            Node::GetAccessorDeclaration(decl) => {
                if let Some(ref mods) = decl.modifiers {
                    self.emit_modifiers(mods, arena);
                }
                self.printer.write("get ");
                if let Some(name) = arena.get(decl.name) {
                    self.printer.emit_node(name, arena);
                }
                self.printer.write("()");
                if !decl.type_annotation.is_none() {
                    self.printer.write(": ");
                    if let Some(ty) = arena.get(decl.type_annotation) {
                        self.printer.emit_node(ty, arena);
                    }
                }
                self.printer.write(";");
            }
            Node::SetAccessorDeclaration(decl) => {
                if let Some(ref mods) = decl.modifiers {
                    self.emit_modifiers(mods, arena);
                }
                self.printer.write("set ");
                if let Some(name) = arena.get(decl.name) {
                    self.printer.emit_node(name, arena);
                }
                self.printer.write("(");
                self.emit_node_list(&decl.parameters, arena, ", ");
                self.printer.write(");");
            }
            // For any unrecognized members, just skip them
            _ => {
                // Don't emit anything for unknown member types
            }
        }
    }

    fn emit_function_declaration(&mut self, decl: &crate::parser::declarations::FunctionDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("function ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        // Type parameters
        if let Some(ref type_params) = decl.type_parameters {
            self.printer.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.printer.write(">");
        }
        self.printer.write("(");
        self.emit_node_list(&decl.parameters, arena, ", ");
        self.printer.write(")");
        if !decl.type_annotation.is_none() {
            self.printer.write(": ");
            if let Some(ty) = arena.get(decl.type_annotation) {
                self.printer.emit_node(ty, arena);
            }
        }
        self.printer.write(";");
    }

    fn emit_variable_statement(&mut self, stmt: &crate::parser::statements::VariableStatement, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = stmt.modifiers {
            self.emit_modifiers(mods, arena);
        }
        if let Some(decl_list) = arena.get(stmt.declaration_list) {
            self.printer.emit_node(decl_list, arena);
        }
        self.printer.write(";");
    }

    fn emit_module_declaration(&mut self, decl: &crate::parser::declarations::ModuleDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(ref mods) = decl.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.printer.write("namespace ");
        if let Some(name) = arena.get(decl.name) {
            self.printer.emit_node(name, arena);
        }
        self.printer.write(" {");
        if !decl.body.is_none() {
            if let Some(body) = arena.get(decl.body) {
                if let Node::ModuleBlock(block) = body {
                    self.printer.write_line();
                    self.printer.increase_indent();
                    for stmt_idx in &block.statements.nodes {
                        if let Some(stmt) = arena.get(*stmt_idx) {
                            if self.is_declaration_node(stmt, arena) {
                                self.emit_declaration(stmt, arena);
                                self.printer.write_line();
                            }
                        }
                    }
                    self.printer.decrease_indent();
                }
            }
        }
        self.printer.write("}");
    }

    fn emit_export_declaration(&mut self, decl: &crate::parser::declarations::ExportDeclaration, arena: &crate::parser::NodeArena) {
        self.printer.write("export ");
        if let Some(clause) = arena.get(decl.export_clause) {
            self.printer.emit_node(clause, arena);
        } else {
            self.printer.write("*");
        }
        if !decl.module_specifier.is_none() {
            self.printer.write(" from ");
            if let Some(spec) = arena.get(decl.module_specifier) {
                self.printer.emit_node(spec, arena);
            }
        }
        self.printer.write(";");
    }

    fn emit_export_assignment(&mut self, decl: &crate::parser::declarations::ExportAssignment, arena: &crate::parser::NodeArena) {
        self.printer.write("export ");
        if decl.is_export_equals {
            self.printer.write("= ");
        } else {
            self.printer.write("default ");
        }
        if let Some(expr) = arena.get(decl.expression) {
            self.printer.emit_node(expr, arena);
        }
        self.printer.write(";");
    }

    fn emit_import_declaration(&mut self, decl: &crate::parser::declarations::ImportDeclaration, arena: &crate::parser::NodeArena) {
        self.printer.write("import ");
        if let Some(clause) = arena.get(decl.import_clause) {
            self.printer.emit_node(clause, arena);
            self.printer.write(" from ");
        }
        if let Some(spec) = arena.get(decl.module_specifier) {
            self.printer.emit_node(spec, arena);
        }
        self.printer.write(";");
    }

    fn emit_modifiers(&mut self, modifiers: &NodeList, arena: &crate::parser::NodeArena) {
        for mod_idx in &modifiers.nodes {
            if let Some(mod_node) = arena.get(*mod_idx) {
                // Skip decorators in declarations
                if matches!(mod_node, Node::Decorator(_)) {
                    continue;
                }
                self.printer.emit_node(mod_node, arena);
                self.printer.write(" ");
            }
        }
    }

    fn emit_node_list(&mut self, list: &NodeList, arena: &crate::parser::NodeArena, separator: &str) {
        for (i, idx) in list.nodes.iter().enumerate() {
            if i > 0 {
                self.printer.write(separator);
            }
            if let Some(node) = arena.get(*idx) {
                self.printer.emit_node(node, arena);
            }
        }
    }
}

impl Default for DeclarationEmitter {
    fn default() -> Self {
        Self::new()
    }
}


