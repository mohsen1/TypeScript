//! Async Function Emitter
//!
//! Emits transformed async functions as __awaiter calls.
//!
//! # Output Pattern
//!
//! ```javascript
//! function asyncFn() {
//!     return __awaiter(this, void 0, void 0, function* () {
//!         const result = yield fetch(url);
//!         return result.json();
//!     });
//! }
//! ```

/// The __awaiter helper function (TypeScript runtime)
pub const AWAITER_HELPER: &str = r#"
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
"#;

/// Emits async functions transformed to __awaiter pattern
pub struct AsyncEmitter {
    output: String,
    indent_level: u32,
}

impl AsyncEmitter {
    pub fn new() -> Self {
        AsyncEmitter {
            output: String::with_capacity(4096),
            indent_level: 0,
        }
    }

    /// Emit a transformed async function
    pub fn emit_async_function(
        &mut self,
        func_name: Option<&str>,
        params: &[String],
        body: &str,
    ) -> String {
        self.output.clear();

        // function name(params) {
        self.write("function");
        if let Some(name) = func_name {
            self.write(" ");
            self.write(name);
        }
        self.write("(");
        self.write(&params.join(", "));
        self.write(") {");
        self.write_line();
        self.increase_indent();

        // return __awaiter(this, void 0, void 0, function* () {
        self.write_indent();
        self.write("return __awaiter(this, void 0, void 0, function* () {");
        self.write_line();
        self.increase_indent();

        // Body with await -> yield transformations
        for line in body.lines() {
            self.write_indent();
            // Transform await to yield
            let transformed = line.replace("await ", "yield ");
            self.write(&transformed);
            self.write_line();
        }

        // Close generator function
        self.decrease_indent();
        self.write_indent();
        self.write("});");
        self.write_line();

        // Close outer function
        self.decrease_indent();
        self.write("}");

        std::mem::take(&mut self.output)
    }

    /// Emit an async arrow function transformed to __awaiter
    pub fn emit_async_arrow(
        &mut self,
        params: &[String],
        body: &str,
    ) -> String {
        self.output.clear();

        // (params) => __awaiter(void 0, void 0, void 0, function* () { ... })
        self.write("(");
        self.write(&params.join(", "));
        self.write(") => __awaiter(void 0, void 0, void 0, function* () {");
        self.write_line();
        self.increase_indent();

        // Body
        for line in body.lines() {
            self.write_indent();
            let transformed = line.replace("await ", "yield ");
            self.write(&transformed);
            self.write_line();
        }

        self.decrease_indent();
        self.write("})");

        std::mem::take(&mut self.output)
    }

    // Helper methods
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self) {
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

impl Default for AsyncEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// The __asyncGenerator helper for async generators
pub const ASYNC_GENERATOR_HELPER: &str = r#"
var __asyncGenerator = (this && this.__asyncGenerator) || function (thisArg, _arguments, generator) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var g = generator.apply(thisArg, _arguments || []), i, q = [];
    return i = Object.create((typeof AsyncIterator === "function" ? AsyncIterator : Object).prototype), verb("next"), verb("throw"), verb("return", awaitReturn), i[Symbol.asyncIterator] = function () { return this; }, i;
    function awaitReturn(f) { return function (v) { return Promise.resolve(v).then(f, reject); }; }
    function verb(n, f) { if (g[n]) { i[n] = function (v) { return new Promise(function (a, b) { q.push([n, v, a, b]) > 1 || resume(n, v); }); }; if (f) i[n] = f(i[n]); } }
    function resume(n, v) { try { step(g[n](v)); } catch (e) { settle(q[0][3], e); } }
    function step(r) { r.value instanceof __await ? Promise.resolve(r.value.v).then(fulfill, reject) : settle(q[0][2], r); }
    function fulfill(value) { resume("next", value); }
    function reject(value) { resume("throw", value); }
    function settle(f, v) { if (f(v), q.shift(), q.length) resume(q[0][0], q[0][1]); }
};
"#;

/// The __await helper for async generators
pub const AWAIT_HELPER: &str = r#"
var __await = (this && this.__await) || function (v) { return this instanceof __await ? (this.v = v, this) : new __await(v); }
"#;

/// The __asyncValues helper for for-await-of
pub const ASYNC_VALUES_HELPER: &str = r#"
var __asyncValues = (this && this.__asyncValues) || function (o) {
    if (!Symbol.asyncIterator) throw new TypeError("Symbol.asyncIterator is not defined.");
    var m = o[Symbol.asyncIterator], i;
    return m ? m.call(o) : (o = typeof __values === "function" ? __values(o) : o[Symbol.iterator](), i = {}, verb("next"), verb("throw"), verb("return"), i[Symbol.asyncIterator] = function () { return this; }, i);
    function verb(n) { i[n] = o[n] && function (v) { return new Promise(function (resolve, reject) { v = o[n](v), settle(resolve, reject, v.done, v.value); }); }; }
    function settle(resolve, reject, d, v) { Promise.resolve(v).then(function(v) { resolve({ value: v, done: d }); }, reject); }
};
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_function_emit() {
        let mut emitter = AsyncEmitter::new();
        
        let output = emitter.emit_async_function(
            Some("fetchData"),
            &["url".to_string()],
            "const response = await fetch(url);\nreturn response.json();",
        );
        
        assert!(output.contains("__awaiter"), "Should contain __awaiter: {}", output);
        assert!(output.contains("function*"), "Should contain generator: {}", output);
        assert!(output.contains("yield fetch"), "Should transform await to yield: {}", output);
    }

    #[test]
    fn test_async_arrow_emit() {
        let mut emitter = AsyncEmitter::new();
        
        let output = emitter.emit_async_arrow(
            &["x".to_string()],
            "return await Promise.resolve(x * 2);",
        );
        
        assert!(output.contains("__awaiter"), "Should contain __awaiter: {}", output);
        assert!(output.contains("yield Promise.resolve"), "Should transform await to yield: {}", output);
    }

    #[test]
    fn test_helper_constants() {
        assert!(AWAITER_HELPER.contains("__awaiter"), "Should define __awaiter");
        assert!(ASYNC_GENERATOR_HELPER.contains("__asyncGenerator"), "Should define __asyncGenerator");
        assert!(ASYNC_VALUES_HELPER.contains("__asyncValues"), "Should define __asyncValues");
    }
}
