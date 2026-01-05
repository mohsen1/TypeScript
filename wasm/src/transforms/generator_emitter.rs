//! Generator State Machine Emitter
//!
//! Emits the state machine code for transformed generator functions.
//!
//! # Output Pattern
//!
//! ```javascript
//! function gen() {
//!     return __generator(this, function (_a) {
//!         switch (_a.label) {
//!             case 0: return [4 /*yield*/, expr];
//!             case 1:
//!                 _a.sent();
//!                 return [2 /*return*/, value];
//!         }
//!     });
//! }
//! ```

use super::generators::Operation;

/// Emits a generator state machine as JavaScript code
pub struct GeneratorEmitter {
    output: String,
    indent_level: u32,
}

impl GeneratorEmitter {
    pub fn new() -> Self {
        GeneratorEmitter {
            output: String::with_capacity(4096),
            indent_level: 0,
        }
    }

    /// Emit a complete generator function
    pub fn emit_generator(
        &mut self,
        func_name: Option<&str>,
        params: &[String],
        operations: &[Operation],
        label_offsets: &[Option<u32>],
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

        // return __generator(this, function (_a) {
        self.write_indent();
        self.write("return __generator(this, function (_a) {");
        self.write_line();
        self.increase_indent();

        // switch (_a.label) {
        self.write_indent();
        self.write("switch (_a.label) {");
        self.write_line();
        self.increase_indent();

        // Emit cases
        self.emit_cases(operations, label_offsets);

        // Close switch
        self.decrease_indent();
        self.write_indent();
        self.write("}");
        self.write_line();

        // Close inner function
        self.decrease_indent();
        self.write_indent();
        self.write("});");
        self.write_line();

        // Close outer function
        self.decrease_indent();
        self.write("}");

        std::mem::take(&mut self.output)
    }

    /// Emit all cases in the state machine
    fn emit_cases(&mut self, operations: &[Operation], label_offsets: &[Option<u32>]) {
        // Build a map of operation index -> label
        let mut op_to_label: Vec<Option<u32>> = vec![None; operations.len() + 1];
        for (label, offset) in label_offsets.iter().enumerate() {
            if let Some(op_idx) = offset {
                op_to_label[*op_idx as usize] = Some(label as u32);
            }
        }

        let mut current_case_emitted = false;
        
        for (i, op) in operations.iter().enumerate() {
            // Check if we need a new case label
            if let Some(label) = op_to_label[i] {
                if current_case_emitted {
                    // Close previous case if needed (cases fall through by default)
                }
                self.write_indent();
                self.write(&format!("case {}: ", label));
                current_case_emitted = true;
            }

            // Emit the operation
            self.emit_operation(op);
        }
    }

    /// Emit a single operation
    fn emit_operation(&mut self, op: &Operation) {
        match op {
            Operation::Nop => {
                // No-op forces new case but emits nothing
            }
            Operation::Statement(_stmt_idx) => {
                // TODO: Would need access to arena to emit statement
                self.write_line();
                self.write_indent();
                self.write("/* statement */;");
            }
            Operation::Assign { .. } => {
                self.write_line();
                self.write_indent();
                self.write("/* assignment */;");
            }
            Operation::Break(label) => {
                self.write(&format!("return [3 /*break*/, {}];", label));
                self.write_line();
            }
            Operation::BreakWhenTrue { label, .. } => {
                self.write_line();
                self.write_indent();
                self.write(&format!("if (/* condition */) return [3 /*break*/, {}];", label));
                self.write_line();
            }
            Operation::BreakWhenFalse { label, .. } => {
                self.write_line();
                self.write_indent();
                self.write(&format!("if (!(/* condition */)) return [3 /*break*/, {}];", label));
                self.write_line();
            }
            Operation::Yield { is_delegating, .. } => {
                if *is_delegating {
                    // yield* - delegation
                    self.write("return [5 /*yield**/, /* expression */];");
                } else {
                    // Regular yield
                    self.write("return [4 /*yield*/, /* expression */];");
                }
                self.write_line();
            }
            Operation::Return(expr) => {
                if expr.is_some() {
                    self.write("return [2 /*return*/, /* expression */];");
                } else {
                    self.write("return [2 /*return*/];");
                }
                self.write_line();
            }
            Operation::Throw(_) => {
                self.write("throw /* expression */;");
                self.write_line();
            }
            Operation::EndFinally => {
                self.write("return [7 /*endfinally*/];");
                self.write_line();
            }
        }
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

impl Default for GeneratorEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// The __generator helper function (TypeScript runtime)
pub const GENERATOR_HELPER: &str = r#"
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
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
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_generator_emit() {
        let mut emitter = GeneratorEmitter::new();
        
        let operations = vec![
            Operation::Yield { expression: None, is_delegating: false },
            Operation::Return(None),
        ];
        
        let label_offsets = vec![None, Some(0), Some(1)];
        
        let output = emitter.emit_generator(
            Some("gen"),
            &[],
            &operations,
            &label_offsets,
        );
        
        assert!(output.contains("__generator"), "Should contain __generator: {}", output);
        assert!(output.contains("switch"), "Should contain switch: {}", output);
        assert!(output.contains("case 1"), "Should contain case: {}", output);
    }

    #[test]
    fn test_generator_with_params() {
        let mut emitter = GeneratorEmitter::new();
        
        let operations = vec![
            Operation::Return(Some(crate::parser::NodeIndex::NONE)),
        ];
        
        let label_offsets = vec![None, Some(0)];
        
        let output = emitter.emit_generator(
            Some("range"),
            &["start".to_string(), "end".to_string()],
            &operations,
            &label_offsets,
        );
        
        assert!(output.contains("start, end"), "Should contain params: {}", output);
    }

    #[test]
    fn test_yield_star_emit() {
        let mut emitter = GeneratorEmitter::new();
        
        let operations = vec![
            Operation::Yield { expression: None, is_delegating: true },
            Operation::Return(None),
        ];
        
        let label_offsets = vec![None, Some(0), Some(1)];
        
        let output = emitter.emit_generator(
            Some("delegating"),
            &[],
            &operations,
            &label_offsets,
        );
        
        assert!(output.contains("yield*"), "Should contain yield* comment: {}", output);
    }
}
