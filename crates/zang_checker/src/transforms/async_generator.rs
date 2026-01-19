//! Async/Await Transformation
//!
//! Transforms async functions and await expressions to ES5-compatible code
//! using Promise chains and generator-based state machines.

use zang_core::StringInterner;
use zang_parser::{
    ArrowFunction, BlockStatement, Expression, FunctionDeclaration, FunctionExpression,
    Statement, SourceFile, VariableDeclarationKind, VariableStatement,
};

/// Target ES version for async transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AsyncTransformTarget {
    /// Transform to ES5 using state machine
    ES5,
    /// Transform to ES2015 using generators
    ES2015,
    /// Transform to ES2017 (native async/await)
    ES2017,
}

impl Default for AsyncTransformTarget {
    fn default() -> Self {
        Self::ES2017
    }
}

/// Options for async transformation
#[derive(Debug, Clone)]
pub struct AsyncTransformOptions {
    /// Target ES version
    pub target: AsyncTransformTarget,
    /// Whether to use tslib helpers
    pub use_tslib: bool,
    /// Whether to emit __awaiter helper inline
    pub emit_helpers_inline: bool,
    /// Module name for helpers
    pub helpers_module: String,
}

impl Default for AsyncTransformOptions {
    fn default() -> Self {
        Self {
            target: AsyncTransformTarget::ES2017,
            use_tslib: false,
            emit_helpers_inline: true,
            helpers_module: "tslib".to_string(),
        }
    }
}

/// Result of an async transformation
#[derive(Debug, Clone)]
pub struct AsyncTransformResult {
    /// The transformed code
    pub code: String,
    /// Whether the __awaiter helper is needed
    pub needs_awaiter_helper: bool,
    /// Whether the __generator helper is needed
    pub needs_generator_helper: bool,
    /// Whether the __asyncGenerator helper is needed
    pub needs_async_generator_helper: bool,
    /// Whether the __asyncDelegator helper is needed
    pub needs_async_delegator_helper: bool,
    /// Whether the __asyncValues helper is needed
    pub needs_async_values_helper: bool,
}

/// Async/await transformer
pub struct AsyncTransformer<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Transform options
    options: AsyncTransformOptions,
    /// Counter for generating unique variable names
    temp_var_counter: u32,
    /// Current indentation level
    indent_level: u32,
    /// Whether we've encountered an async function
    has_async: bool,
    /// Whether we've encountered an async generator
    has_async_generator: bool,
    /// Whether we've encountered for-await-of
    has_for_await: bool,
}

impl<'a> AsyncTransformer<'a> {
    /// Creates a new async transformer
    pub fn new(interner: &'a StringInterner) -> Self {
        Self {
            interner,
            options: AsyncTransformOptions::default(),
            temp_var_counter: 0,
            indent_level: 0,
            has_async: false,
            has_async_generator: false,
            has_for_await: false,
        }
    }

    /// Sets the transform options
    pub fn with_options(mut self, options: AsyncTransformOptions) -> Self {
        self.options = options;
        self
    }

    /// Generates a unique temporary variable name
    fn gen_temp_var(&mut self) -> String {
        let name = format!("_a{}", self.temp_var_counter);
        self.temp_var_counter += 1;
        name
    }

    /// Returns the current indentation string
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level as usize)
    }

    /// Transforms a source file
    pub fn transform_source_file(&mut self, source: &SourceFile) -> AsyncTransformResult {
        let mut code = String::new();

        for statement in &source.statements {
            code.push_str(&self.transform_statement(statement));
            code.push('\n');
        }

        // Prepend helper functions if needed
        let mut helpers = String::new();
        if self.has_async && self.options.emit_helpers_inline && self.options.target < AsyncTransformTarget::ES2017 {
            helpers.push_str(&self.emit_awaiter_helper());
            helpers.push_str(&self.emit_generator_helper());
        }
        if self.has_async_generator && self.options.emit_helpers_inline {
            helpers.push_str(&self.emit_async_generator_helper());
        }
        if self.has_for_await && self.options.emit_helpers_inline {
            helpers.push_str(&self.emit_async_values_helper());
        }

        if !helpers.is_empty() {
            code = format!("{}\n{}", helpers, code);
        }

        AsyncTransformResult {
            code,
            needs_awaiter_helper: self.has_async && self.options.target < AsyncTransformTarget::ES2017,
            needs_generator_helper: self.has_async && self.options.target == AsyncTransformTarget::ES5,
            needs_async_generator_helper: self.has_async_generator,
            needs_async_delegator_helper: false, // Set if yield* is used in async generator
            needs_async_values_helper: self.has_for_await,
        }
    }

    /// Transforms a statement
    fn transform_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Function(func) => self.transform_function_declaration(func),
            Statement::Variable(var) => self.transform_variable_statement(var),
            Statement::Expression(expr) => {
                format!("{}{};", self.indent(), self.transform_expression(&expr.expression))
            }
            Statement::Return(ret) => {
                let expr_str = ret.expression
                    .as_ref()
                    .map(|e| self.transform_expression(e))
                    .unwrap_or_default();
                format!("{}return {};", self.indent(), expr_str)
            }
            Statement::Block(block) => self.transform_block(block),
            Statement::If(if_stmt) => {
                let cond = self.transform_expression(&if_stmt.condition);
                let then_stmt = self.transform_statement(&if_stmt.then_statement);
                let else_part = if_stmt.else_statement
                    .as_ref()
                    .map(|e| format!(" else {}", self.transform_statement(e)))
                    .unwrap_or_default();
                format!("{}if ({}) {}{}", self.indent(), cond, then_stmt, else_part)
            }
            _ => format!("{}/* statement */;", self.indent()),
        }
    }

    /// Transforms a block statement
    fn transform_block(&mut self, block: &BlockStatement) -> String {
        let mut code = format!("{}{{\n", self.indent());
        self.indent_level += 1;

        for stmt in &block.statements {
            code.push_str(&self.transform_statement(stmt));
            code.push('\n');
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));
        code
    }

    /// Transforms a function declaration
    fn transform_function_declaration(&mut self, func: &FunctionDeclaration) -> String {
        if !func.is_async {
            // Non-async function, just transform body
            return self.transform_regular_function(func);
        }

        self.has_async = true;

        if func.is_generator {
            self.has_async_generator = true;
            return self.transform_async_generator_function(func);
        }

        match self.options.target {
            AsyncTransformTarget::ES2017 => self.transform_regular_function(func),
            AsyncTransformTarget::ES2015 => self.transform_async_to_generator(func),
            AsyncTransformTarget::ES5 => self.transform_async_to_state_machine(func),
        }
    }

    /// Transforms a regular (non-async) function
    fn transform_regular_function(&mut self, func: &FunctionDeclaration) -> String {
        let name = func.name
            .as_ref()
            .map(|n| self.interner.lookup(n.name).unwrap_or_default())
            .unwrap_or_default();

        let async_keyword = if func.is_async { "async " } else { "" };
        let generator_star = if func.is_generator { "*" } else { "" };

        let params = self.transform_parameters(&func.parameters);

        let body = func.body
            .as_ref()
            .map(|b| self.transform_block(b))
            .unwrap_or_else(|| "{}".to_string());

        format!(
            "{}{}function {}{}{} {}",
            self.indent(),
            async_keyword,
            generator_star,
            name,
            params,
            body
        )
    }

    /// Transforms an async function to ES2015 generator
    fn transform_async_to_generator(&mut self, func: &FunctionDeclaration) -> String {
        let name = func.name
            .as_ref()
            .map(|n| self.interner.lookup(n.name).unwrap_or_default())
            .unwrap_or_default();

        let params = self.transform_parameters(&func.parameters);

        let body = func.body
            .as_ref()
            .map(|b| self.transform_async_body_to_generator(b))
            .unwrap_or_else(|| "{ }".to_string());

        // Wrap in __awaiter
        format!(
            "{}function {}{}  {{\n{}return __awaiter(this, void 0, void 0, function* () {});\n{}}}",
            self.indent(),
            name,
            params,
            self.indent(),
            body,
            self.indent()
        )
    }

    /// Transforms an async function to ES5 state machine
    fn transform_async_to_state_machine(&mut self, func: &FunctionDeclaration) -> String {
        let name = func.name
            .as_ref()
            .map(|n| self.interner.lookup(n.name).unwrap_or_default())
            .unwrap_or_default();

        let params = self.transform_parameters(&func.parameters);

        let body = func.body
            .as_ref()
            .map(|b| self.transform_async_body_to_state_machine(b))
            .unwrap_or_else(|| "".to_string());

        // Wrap in __awaiter with __generator
        format!(
            "{}function {}{} {{\n{}return __awaiter(this, void 0, void 0, function () {{\n{}return __generator(this, function (_a) {{\n{}\n{}}});\n{}}});\n{}}}",
            self.indent(),
            name,
            params,
            self.indent(),
            self.indent(),
            body,
            self.indent(),
            self.indent(),
            self.indent()
        )
    }

    /// Transforms an async generator function
    fn transform_async_generator_function(&mut self, func: &FunctionDeclaration) -> String {
        let name = func.name
            .as_ref()
            .map(|n| self.interner.lookup(n.name).unwrap_or_default())
            .unwrap_or_default();

        let params = self.transform_parameters(&func.parameters);

        let body = func.body
            .as_ref()
            .map(|b| self.transform_block(b))
            .unwrap_or_else(|| "{}".to_string());

        match self.options.target {
            AsyncTransformTarget::ES2017 => {
                format!(
                    "{}async function* {}{} {}",
                    self.indent(), name, params, body
                )
            }
            _ => {
                // Wrap in __asyncGenerator
                format!(
                    "{}function {}{} {{\n{}return __asyncGenerator(this, arguments, function* () {});\n{}}}",
                    self.indent(), name, params, self.indent(), body, self.indent()
                )
            }
        }
    }

    /// Transforms async function body for generator output
    fn transform_async_body_to_generator(&mut self, block: &BlockStatement) -> String {
        let mut code = String::from("{\n");
        self.indent_level += 2;

        for stmt in &block.statements {
            code.push_str(&self.transform_statement_for_generator(stmt));
            code.push('\n');
        }

        self.indent_level -= 2;
        code.push_str(&format!("{}}}", self.indent()));
        code
    }

    /// Transforms a statement for generator output (await -> yield)
    fn transform_statement_for_generator(&mut self, statement: &Statement) -> String {
        // This is a simplified version - in a real implementation,
        // we'd need to traverse all expressions and transform await to yield
        self.transform_statement(statement)
    }

    /// Transforms async function body to state machine
    fn transform_async_body_to_state_machine(&mut self, block: &BlockStatement) -> String {
        // This is a simplified state machine - in a real implementation,
        // we'd need to analyze control flow and create proper states

        let mut code = String::new();
        let mut state = 0;

        code.push_str(&format!("{}switch (_a.label) {{\n", self.indent()));
        self.indent_level += 1;

        code.push_str(&format!("{}case 0:\n", self.indent()));
        self.indent_level += 1;

        for stmt in &block.statements {
            code.push_str(&self.transform_statement_for_state_machine(stmt, &mut state));
            code.push('\n');
        }

        // Final return
        code.push_str(&format!("{}return [2 /*return*/];\n", self.indent()));

        self.indent_level -= 1;
        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    /// Transforms a statement for state machine output
    fn transform_statement_for_state_machine(&mut self, statement: &Statement, state: &mut u32) -> String {
        match statement {
            Statement::Return(ret) => {
                let expr = ret.expression
                    .as_ref()
                    .map(|e| self.transform_expression(e))
                    .unwrap_or_else(|| "void 0".to_string());
                format!("{}return [2 /*return*/, {}];", self.indent(), expr)
            }
            _ => self.transform_statement(statement),
        }
    }

    /// Transforms function parameters
    fn transform_parameters(&self, params: &[zang_parser::ParameterNode]) -> String {
        let param_strs: Vec<_> = params.iter().map(|p| {
            match &p.name {
                zang_parser::BindingName::Identifier(ident) => {
                    self.interner.lookup(ident.name).unwrap_or_default()
                }
                _ => "_".to_string(),
            }
        }).collect();

        format!("({})", param_strs.join(", "))
    }

    /// Transforms a variable statement
    fn transform_variable_statement(&mut self, var: &VariableStatement) -> String {
        let kind = match var.declaration_list.flags {
            VariableDeclarationKind::Var => "var",
            VariableDeclarationKind::Let => "let",
            VariableDeclarationKind::Const => "const",
        };

        let decls: Vec<_> = var.declaration_list.declarations.iter().map(|d| {
            let name = match &d.name {
                zang_parser::BindingName::Identifier(ident) => {
                    self.interner.lookup(ident.name).unwrap_or_default()
                }
                _ => "_".to_string(),
            };

            let init = d.initializer
                .as_ref()
                .map(|e| format!(" = {}", self.transform_expression(e)))
                .unwrap_or_default();

            format!("{}{}", name, init)
        }).collect();

        format!("{}{} {};", self.indent(), kind, decls.join(", "))
    }

    /// Transforms an expression
    fn transform_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(ident) => {
                self.interner.lookup(ident.name).unwrap_or_default()
            }
            Expression::StringLiteral(lit) => {
                let value = self.interner.lookup(lit.value).unwrap_or_default();
                format!("\"{}\"", value)
            }
            Expression::NumericLiteral(lit) => {
                format!("{}", lit.value)
            }
            Expression::BooleanLiteral(lit) => {
                if lit.value { "true".to_string() } else { "false".to_string() }
            }
            Expression::NullLiteral(_) => "null".to_string(),
            Expression::Call(call) => {
                let callee = self.transform_expression(&call.expression);
                let args: Vec<_> = call.arguments.iter()
                    .map(|a| self.transform_expression(a))
                    .collect();
                format!("{}({})", callee, args.join(", "))
            }
            Expression::PropertyAccess(access) => {
                let obj = self.transform_expression(&access.expression);
                let prop = self.interner.lookup(access.name.name).unwrap_or_default();
                format!("{}.{}", obj, prop)
            }
            Expression::Arrow(arrow) => {
                self.transform_arrow_function(arrow)
            }
            Expression::Function(func) => {
                self.transform_function_expression(func)
            }
            Expression::Binary(bin) => {
                let left = self.transform_expression(&bin.left);
                let right = self.transform_expression(&bin.right);
                let op = self.transform_binary_operator(&bin.operator);
                format!("{} {} {}", left, op, right)
            }
            Expression::Unary(unary) => {
                let operand = self.transform_expression(&unary.operand);
                let op = self.transform_unary_operator(&unary.operator);
                if unary.is_prefix {
                    format!("{}{}", op, operand)
                } else {
                    format!("{}{}", operand, op)
                }
            }
            Expression::Parenthesized(inner) => {
                format!("({})", self.transform_expression(inner))
            }
            _ => "/* expression */".to_string(),
        }
    }

    /// Transforms an arrow function
    fn transform_arrow_function(&mut self, arrow: &ArrowFunction) -> String {
        if arrow.is_async {
            self.has_async = true;
        }

        let async_keyword = if arrow.is_async && self.options.target == AsyncTransformTarget::ES2017 {
            "async "
        } else {
            ""
        };

        let params = self.transform_parameters(&arrow.parameters);

        let body = match &arrow.body {
            zang_parser::ArrowFunctionBody::Expression(expr) => {
                self.transform_expression(expr)
            }
            zang_parser::ArrowFunctionBody::Block(block) => {
                self.transform_block(block)
            }
        };

        if arrow.is_async && self.options.target < AsyncTransformTarget::ES2017 {
            format!("{} => __awaiter(void 0, void 0, void 0, function* () {{ return {}; }})", params, body)
        } else {
            format!("{}{} => {}", async_keyword, params, body)
        }
    }

    /// Transforms a function expression
    fn transform_function_expression(&mut self, func: &FunctionExpression) -> String {
        if func.is_async {
            self.has_async = true;
        }

        let async_keyword = if func.is_async && self.options.target == AsyncTransformTarget::ES2017 {
            "async "
        } else {
            ""
        };

        let name = func.name
            .as_ref()
            .map(|n| self.interner.lookup(n.name).unwrap_or_default())
            .unwrap_or_default();

        let params = self.transform_parameters(&func.parameters);
        let body = self.transform_block(&func.body);

        if func.is_async && self.options.target < AsyncTransformTarget::ES2017 {
            format!(
                "function {}{} {{ return __awaiter(this, void 0, void 0, function* () {}); }}",
                name, params, body
            )
        } else {
            format!("{}function {}{} {}", async_keyword, name, params, body)
        }
    }

    /// Transforms a binary operator to string
    fn transform_binary_operator(&self, op: &zang_parser::BinaryOperator) -> &'static str {
        use zang_parser::BinaryOperator::*;
        match op {
            Add => "+",
            Subtract => "-",
            Multiply => "*",
            Divide => "/",
            Modulo => "%",
            Power => "**",
            LessThan => "<",
            GreaterThan => ">",
            LessThanEquals => "<=",
            GreaterThanEquals => ">=",
            Equals => "==",
            NotEquals => "!=",
            StrictEquals => "===",
            StrictNotEquals => "!==",
            BitwiseAnd => "&",
            BitwiseOr => "|",
            BitwiseXor => "^",
            LogicalAnd => "&&",
            LogicalOr => "||",
            NullishCoalescing => "??",
            Assign => "=",
            _ => "/* op */",
        }
    }

    /// Transforms a unary operator to string
    fn transform_unary_operator(&self, op: &zang_parser::UnaryOperator) -> &'static str {
        use zang_parser::UnaryOperator::*;
        match op {
            Plus => "+",
            Minus => "-",
            BitwiseNot => "~",
            LogicalNot => "!",
            Increment => "++",
            Decrement => "--",
            TypeOf => "typeof ",
            Void => "void ",
            Delete => "delete ",
            Await => "await ",
        }
    }

    /// Emits the __awaiter helper function
    fn emit_awaiter_helper(&self) -> String {
        r#"var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
"#.to_string()
    }

    /// Emits the __generator helper function
    fn emit_generator_helper(&self) -> String {
        r#"var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
"#.to_string()
    }

    /// Emits the __asyncGenerator helper function
    fn emit_async_generator_helper(&self) -> String {
        r#"var __asyncGenerator = (this && this.__asyncGenerator) || function (thisArg, _arguments, generator) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var g = generator.apply(thisArg, _arguments || []), i, q = [];
    return i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i;
    function verb(n) { if (g[n]) i[n] = function (v) { return new Promise(function (a, b) { q.push([n, v, a, b]) > 1 || resume(n, v); }); }; }
    function resume(n, v) { try { step(g[n](v)); } catch (e) { settle(q[0][3], e); } }
    function step(r) { r.value instanceof __await ? Promise.resolve(r.value.v).then(fulfill, reject) : settle(q[0][2], r); }
    function fulfill(value) { resume("next", value); }
    function reject(value) { resume("throw", value); }
    function settle(f, v) { if (f(v), q.shift(), q.length) resume(q[0][0], q[0][1]); }
};
var __await = (this && this.__await) || function (v) { return this instanceof __await ? (this.v = v, this) : new __await(v); }
"#.to_string()
    }

    /// Emits the __asyncValues helper function
    fn emit_async_values_helper(&self) -> String {
        r#"var __asyncValues = (this && this.__asyncValues) || function (o) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator], i;
    return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i);
    function verb(n) { i[n] = o[n] && function (v) { return new Promise(function (resolve, reject) { v = o[n](v), settle(resolve, reject, v.done, v.value); }); }; }
    function settle(resolve, reject, d, v) { Promise.resolve(v).then(function(v) { resolve({ value: v, done: d }); }, reject); }
};
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_transform_target_default() {
        let target = AsyncTransformTarget::default();
        assert_eq!(target, AsyncTransformTarget::ES2017);
    }

    #[test]
    fn test_async_transform_options_default() {
        let options = AsyncTransformOptions::default();
        assert_eq!(options.target, AsyncTransformTarget::ES2017);
        assert!(!options.use_tslib);
        assert!(options.emit_helpers_inline);
    }

    #[test]
    fn test_async_transformer_creation() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        assert!(!transformer.has_async);
        assert!(!transformer.has_async_generator);
    }

    #[test]
    fn test_gen_temp_var() {
        let interner = StringInterner::new();
        let mut transformer = AsyncTransformer::new(&interner);

        let var1 = transformer.gen_temp_var();
        let var2 = transformer.gen_temp_var();
        let var3 = transformer.gen_temp_var();

        assert_eq!(var1, "_a0");
        assert_eq!(var2, "_a1");
        assert_eq!(var3, "_a2");
    }

    #[test]
    fn test_emit_awaiter_helper() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        let helper = transformer.emit_awaiter_helper();

        assert!(helper.contains("__awaiter"));
        assert!(helper.contains("Promise"));
        assert!(helper.contains("generator"));
    }

    #[test]
    fn test_emit_generator_helper() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        let helper = transformer.emit_generator_helper();

        assert!(helper.contains("__generator"));
        assert!(helper.contains("Symbol.iterator"));
    }

    #[test]
    fn test_emit_async_generator_helper() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        let helper = transformer.emit_async_generator_helper();

        assert!(helper.contains("__asyncGenerator"));
        assert!(helper.contains("Symbol.asyncIterator"));
    }

    #[test]
    fn test_transform_binary_operator() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        use zang_parser::BinaryOperator;

        assert_eq!(transformer.transform_binary_operator(&BinaryOperator::Add), "+");
        assert_eq!(transformer.transform_binary_operator(&BinaryOperator::Subtract), "-");
        assert_eq!(transformer.transform_binary_operator(&BinaryOperator::StrictEquals), "===");
        assert_eq!(transformer.transform_binary_operator(&BinaryOperator::LogicalAnd), "&&");
    }

    #[test]
    fn test_transform_unary_operator() {
        let interner = StringInterner::new();
        let transformer = AsyncTransformer::new(&interner);

        use zang_parser::UnaryOperator;

        assert_eq!(transformer.transform_unary_operator(&UnaryOperator::LogicalNot), "!");
        assert_eq!(transformer.transform_unary_operator(&UnaryOperator::TypeOf), "typeof ");
        assert_eq!(transformer.transform_unary_operator(&UnaryOperator::Await), "await ");
    }
}
