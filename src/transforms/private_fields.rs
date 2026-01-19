//! ES5 transform for private class fields using WeakMap.
//!
//! This module transforms private class fields to use WeakMap-based storage
//! for ES5 compatibility. Each private field gets its own WeakMap, and field
//! access is transformed to WeakMap get/set calls.
//!
//! Example transformation:
//! ```typescript
//! class Counter {
//!     #count = 0;
//!     increment() { this.#count++; }
//!     get value() { return this.#count; }
//! }
//! ```
//! Transforms to:
//! ```javascript
//! var _count = new WeakMap();
//! var Counter = (function() {
//!     function Counter() {
//!         _count.set(this, 0);
//!     }
//!     Counter.prototype.increment = function() {
//!         var _a;
//!         _count.set(this, ((_a = _count.get(this)) !== null && _a !== void 0 ? _a : 0) + 1);
//!     };
//!     Object.defineProperty(Counter.prototype, "value", {
//!         get: function() { return _count.get(this); },
//!         enumerable: false,
//!         configurable: true
//!     });
//!     return Counter;
//! }());
//! ```

use std::collections::HashMap;

/// Identifier for a generated WeakMap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeakMapId {
    /// Original private field name (without #).
    pub field_name: String,
    /// Generated unique variable name.
    pub var_name: String,
    /// Class this belongs to.
    pub class_name: String,
    /// Whether this is for a static field.
    pub is_static: bool,
}

impl WeakMapId {
    pub fn new(field_name: &str, class_name: &str, is_static: bool, unique_id: u32) -> Self {
        let prefix = if is_static { "_static" } else { "_" };
        Self {
            field_name: field_name.to_string(),
            var_name: format!("{}{}{}", prefix, field_name, unique_id),
            class_name: class_name.to_string(),
            is_static,
        }
    }
}

/// Private field information for transformation.
#[derive(Debug, Clone)]
pub struct PrivateFieldInfo {
    /// The WeakMap identifier.
    pub weak_map: WeakMapId,
    /// Initial value expression (if any).
    pub initializer: Option<String>,
    /// Type annotation (for documentation/debugging).
    pub type_annotation: Option<String>,
    /// Whether this field is readonly.
    pub is_readonly: bool,
}

/// Private method information for transformation.
#[derive(Debug, Clone)]
pub struct PrivateMethodInfo {
    /// The WeakSet identifier (for brand checking).
    pub weak_set: WeakMapId,
    /// The method implementation.
    pub implementation: String,
    /// Method parameters.
    pub parameters: Vec<String>,
    /// Whether this is static.
    pub is_static: bool,
}

/// Private accessor information for transformation.
#[derive(Debug, Clone)]
pub struct PrivateAccessorInfo {
    /// The WeakMap identifier.
    pub weak_map: WeakMapId,
    /// Getter implementation (if any).
    pub getter: Option<String>,
    /// Setter implementation (if any).
    pub setter: Option<String>,
    /// Whether this is static.
    pub is_static: bool,
}

/// Transform context for a class.
#[derive(Debug, Clone, Default)]
pub struct ClassTransformContext {
    /// Private fields to transform.
    pub fields: Vec<PrivateFieldInfo>,
    /// Private methods to transform.
    pub methods: Vec<PrivateMethodInfo>,
    /// Private accessors to transform.
    pub accessors: Vec<PrivateAccessorInfo>,
    /// Generated unique ID counter.
    unique_counter: u32,
}

impl ClassTransformContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a private field to transform.
    pub fn add_field(
        &mut self,
        field_name: &str,
        class_name: &str,
        is_static: bool,
        initializer: Option<String>,
        type_annotation: Option<String>,
        is_readonly: bool,
    ) -> &PrivateFieldInfo {
        let weak_map = WeakMapId::new(field_name, class_name, is_static, self.unique_counter);
        self.unique_counter += 1;

        self.fields.push(PrivateFieldInfo {
            weak_map,
            initializer,
            type_annotation,
            is_readonly,
        });

        self.fields.last().unwrap()
    }

    /// Add a private method to transform.
    pub fn add_method(
        &mut self,
        method_name: &str,
        class_name: &str,
        is_static: bool,
        implementation: String,
        parameters: Vec<String>,
    ) -> &PrivateMethodInfo {
        let weak_set = WeakMapId::new(method_name, class_name, is_static, self.unique_counter);
        self.unique_counter += 1;

        self.methods.push(PrivateMethodInfo {
            weak_set,
            implementation,
            parameters,
            is_static,
        });

        self.methods.last().unwrap()
    }

    /// Add a private accessor to transform.
    pub fn add_accessor(
        &mut self,
        accessor_name: &str,
        class_name: &str,
        is_static: bool,
        getter: Option<String>,
        setter: Option<String>,
    ) -> &PrivateAccessorInfo {
        let weak_map = WeakMapId::new(accessor_name, class_name, is_static, self.unique_counter);
        self.unique_counter += 1;

        self.accessors.push(PrivateAccessorInfo {
            weak_map,
            getter,
            setter,
            is_static,
        });

        self.accessors.last().unwrap()
    }

    /// Get the WeakMap variable name for a field.
    pub fn get_field_weak_map(&self, field_name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.weak_map.field_name == field_name)
            .map(|f| f.weak_map.var_name.as_str())
    }
}

/// Private fields transformer.
pub struct PrivateFieldsTransformer {
    /// Transform contexts by class name.
    contexts: HashMap<String, ClassTransformContext>,
    /// Current class being transformed.
    current_class: Option<String>,
    /// Helper function names.
    helpers: TransformHelpers,
}

/// Names of helper functions used in transformation.
#[derive(Debug, Clone)]
pub struct TransformHelpers {
    /// Helper for class private field get.
    pub class_private_field_get: String,
    /// Helper for class private field set.
    pub class_private_field_set: String,
    /// Helper for class private field in.
    pub class_private_field_in: String,
    /// Helper for class private method get.
    pub class_private_method_get: String,
}

impl Default for TransformHelpers {
    fn default() -> Self {
        Self {
            class_private_field_get: "__classPrivateFieldGet".to_string(),
            class_private_field_set: "__classPrivateFieldSet".to_string(),
            class_private_field_in: "__classPrivateFieldIn".to_string(),
            class_private_method_get: "__classPrivateMethodGet".to_string(),
        }
    }
}

impl PrivateFieldsTransformer {
    /// Create a new transformer.
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            current_class: None,
            helpers: TransformHelpers::default(),
        }
    }

    /// Enter a class for transformation.
    pub fn enter_class(&mut self, class_name: &str) {
        self.current_class = Some(class_name.to_string());
        self.contexts
            .entry(class_name.to_string())
            .or_insert_with(ClassTransformContext::new);
    }

    /// Exit the current class.
    pub fn exit_class(&mut self) {
        self.current_class = None;
    }

    /// Get the current class context.
    pub fn current_context(&mut self) -> Option<&mut ClassTransformContext> {
        let class_name = self.current_class.as_ref()?;
        self.contexts.get_mut(class_name)
    }

    /// Generate WeakMap declarations for a class.
    pub fn generate_weak_map_declarations(&self, class_name: &str) -> Vec<String> {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return vec![],
        };

        let mut declarations = Vec::new();

        // Field WeakMaps
        for field in &ctx.fields {
            if field.weak_map.is_static {
                // Static fields use a simple object
                declarations.push(format!(
                    "var {} = {{ value: {} }};",
                    field.weak_map.var_name,
                    field.initializer.as_deref().unwrap_or("void 0")
                ));
            } else {
                declarations.push(format!("var {} = new WeakMap();", field.weak_map.var_name));
            }
        }

        // Method WeakSets (for brand checking)
        for method in &ctx.methods {
            if !method.is_static {
                declarations.push(format!("var {} = new WeakSet();", method.weak_set.var_name));
            }
        }

        // Accessor WeakMaps
        for accessor in &ctx.accessors {
            if !accessor.is_static {
                declarations.push(format!("var {} = new WeakMap();", accessor.weak_map.var_name));
            }
        }

        declarations
    }

    /// Generate constructor initialization code.
    pub fn generate_constructor_init(&self, class_name: &str) -> Vec<String> {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return vec![],
        };

        let mut inits = Vec::new();

        // Initialize instance fields
        for field in &ctx.fields {
            if !field.weak_map.is_static {
                let init_value = field.initializer.as_deref().unwrap_or("void 0");
                inits.push(format!(
                    "{}.set(this, {});",
                    field.weak_map.var_name, init_value
                ));
            }
        }

        // Add instance to method WeakSets
        for method in &ctx.methods {
            if !method.is_static {
                inits.push(format!("{}.add(this);", method.weak_set.var_name));
            }
        }

        // Initialize accessor storage
        for accessor in &ctx.accessors {
            if !accessor.is_static {
                inits.push(format!(
                    "{}.set(this, {{ get: {}, set: {} }});",
                    accessor.weak_map.var_name,
                    accessor.getter.as_deref().unwrap_or("void 0"),
                    accessor.setter.as_deref().unwrap_or("void 0")
                ));
            }
        }

        inits
    }

    /// Transform a private field read expression.
    pub fn transform_field_get(&self, class_name: &str, field_name: &str, receiver: &str) -> String {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return format!("/* unknown private field #{} */", field_name),
        };

        // Find the field
        if let Some(field) = ctx.fields.iter().find(|f| f.weak_map.field_name == field_name) {
            if field.weak_map.is_static {
                format!("{}.value", field.weak_map.var_name)
            } else {
                format!(
                    "{}({}, {}, \"{}\")",
                    self.helpers.class_private_field_get,
                    receiver,
                    field.weak_map.var_name,
                    field_name
                )
            }
        } else {
            format!("/* unknown private field #{} */", field_name)
        }
    }

    /// Transform a private field write expression.
    pub fn transform_field_set(
        &self,
        class_name: &str,
        field_name: &str,
        receiver: &str,
        value: &str,
    ) -> String {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return format!("/* unknown private field #{} */", field_name),
        };

        if let Some(field) = ctx.fields.iter().find(|f| f.weak_map.field_name == field_name) {
            if field.weak_map.is_static {
                format!("{}.value = {}", field.weak_map.var_name, value)
            } else {
                format!(
                    "{}({}, {}, {}, \"{}\")",
                    self.helpers.class_private_field_set,
                    receiver,
                    field.weak_map.var_name,
                    value,
                    field_name
                )
            }
        } else {
            format!("/* unknown private field #{} */", field_name)
        }
    }

    /// Transform a private field `in` expression.
    pub fn transform_field_in(&self, class_name: &str, field_name: &str, target: &str) -> String {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return format!("/* unknown private field #{} */", field_name),
        };

        if let Some(field) = ctx.fields.iter().find(|f| f.weak_map.field_name == field_name) {
            if field.weak_map.is_static {
                // Static field - check if target is the class
                format!("{} === {}", target, class_name)
            } else {
                format!(
                    "{}({}, {})",
                    self.helpers.class_private_field_in, target, field.weak_map.var_name
                )
            }
        } else {
            format!("/* unknown private field #{} */", field_name)
        }
    }

    /// Transform a private method call.
    pub fn transform_method_call(
        &self,
        class_name: &str,
        method_name: &str,
        receiver: &str,
        args: &[&str],
    ) -> String {
        let ctx = match self.contexts.get(class_name) {
            Some(c) => c,
            None => return format!("/* unknown private method #{} */", method_name),
        };

        if let Some(method) = ctx
            .methods
            .iter()
            .find(|m| m.weak_set.field_name == method_name)
        {
            let args_str = args.join(", ");
            if method.is_static {
                format!("{}_{}.call({}, {})", class_name, method_name, receiver, args_str)
            } else {
                format!(
                    "{}({}, {}, {}_{}).call({}, {})",
                    self.helpers.class_private_method_get,
                    receiver,
                    method.weak_set.var_name,
                    class_name,
                    method_name,
                    receiver,
                    args_str
                )
            }
        } else {
            format!("/* unknown private method #{} */", method_name)
        }
    }

    /// Generate helper function definitions.
    pub fn generate_helpers(&self) -> String {
        format!(
            r#"var {get} = function (receiver, state, kind, f) {{
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
}};
var {set} = function (receiver, state, value, kind, f) {{
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
}};
var {has} = function (state, receiver) {{
    if (typeof state === "function") return receiver === state;
    return state.has(receiver);
}};"#,
            get = self.helpers.class_private_field_get,
            set = self.helpers.class_private_field_set,
            has = self.helpers.class_private_field_in
        )
    }
}

impl Default for PrivateFieldsTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Output of transforming a class with private fields.
#[derive(Debug, Clone)]
pub struct TransformedClass {
    /// WeakMap/WeakSet declarations (go before class).
    pub pre_declarations: Vec<String>,
    /// Constructor initialization statements.
    pub constructor_init: Vec<String>,
    /// Transformed class body.
    pub class_body: String,
    /// Whether helpers are needed.
    pub needs_helpers: bool,
}

/// Transform a class with private fields to ES5.
pub fn transform_class_to_es5(
    class_name: &str,
    transformer: &PrivateFieldsTransformer,
) -> TransformedClass {
    let pre_declarations = transformer.generate_weak_map_declarations(class_name);
    let constructor_init = transformer.generate_constructor_init(class_name);

    let needs_helpers = !pre_declarations.is_empty();

    TransformedClass {
        pre_declarations,
        constructor_init,
        class_body: String::new(), // Would be filled in by actual transformation
        needs_helpers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_map_id() {
        let id = WeakMapId::new("count", "Counter", false, 0);
        assert_eq!(id.field_name, "count");
        assert_eq!(id.var_name, "_count0");
        assert_eq!(id.class_name, "Counter");
        assert!(!id.is_static);

        let static_id = WeakMapId::new("total", "Counter", true, 1);
        assert_eq!(static_id.var_name, "_statictotal1");
        assert!(static_id.is_static);
    }

    #[test]
    fn test_class_transform_context() {
        let mut ctx = ClassTransformContext::new();

        ctx.add_field("count", "Counter", false, Some("0".to_string()), Some("number".to_string()), false);
        ctx.add_field("name", "Counter", false, None, Some("string".to_string()), true);

        assert_eq!(ctx.fields.len(), 2);
        assert_eq!(ctx.get_field_weak_map("count"), Some("_count0"));
        assert_eq!(ctx.get_field_weak_map("name"), Some("_name1"));
    }

    #[test]
    fn test_transformer_field_declarations() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, Some("0".to_string()), None, false);
            ctx.add_field("total", "Counter", true, Some("100".to_string()), None, false);
        }

        let declarations = transformer.generate_weak_map_declarations("Counter");
        assert_eq!(declarations.len(), 2);
        assert!(declarations[0].contains("new WeakMap()"));
        assert!(declarations[1].contains("value: 100"));
    }

    #[test]
    fn test_transformer_constructor_init() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, Some("0".to_string()), None, false);
        }

        let init = transformer.generate_constructor_init("Counter");
        assert_eq!(init.len(), 1);
        assert!(init[0].contains(".set(this, 0)"));
    }

    #[test]
    fn test_transformer_field_get() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, Some("0".to_string()), None, false);
        }

        transformer.exit_class();

        let get_expr = transformer.transform_field_get("Counter", "count", "this");
        assert!(get_expr.contains("__classPrivateFieldGet"));
        assert!(get_expr.contains("_count0"));
    }

    #[test]
    fn test_transformer_field_set() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, Some("0".to_string()), None, false);
        }

        transformer.exit_class();

        let set_expr = transformer.transform_field_set("Counter", "count", "this", "42");
        assert!(set_expr.contains("__classPrivateFieldSet"));
        assert!(set_expr.contains("42"));
    }

    #[test]
    fn test_transformer_static_field_get() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("total", "Counter", true, Some("0".to_string()), None, false);
        }

        transformer.exit_class();

        let get_expr = transformer.transform_field_get("Counter", "total", "Counter");
        assert!(get_expr.contains(".value"));
        assert!(!get_expr.contains("__classPrivateFieldGet"));
    }

    #[test]
    fn test_transformer_field_in() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, None, None, false);
        }

        transformer.exit_class();

        let in_expr = transformer.transform_field_in("Counter", "count", "obj");
        assert!(in_expr.contains("__classPrivateFieldIn"));
    }

    #[test]
    fn test_transformer_method() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_method(
                "increment",
                "Counter",
                false,
                "this.count++".to_string(),
                vec![],
            );
        }

        let declarations = transformer.generate_weak_map_declarations("Counter");
        assert!(declarations.iter().any(|d| d.contains("new WeakSet()")));

        let init = transformer.generate_constructor_init("Counter");
        assert!(init.iter().any(|i| i.contains(".add(this)")));
    }

    #[test]
    fn test_generate_helpers() {
        let transformer = PrivateFieldsTransformer::new();
        let helpers = transformer.generate_helpers();

        assert!(helpers.contains("__classPrivateFieldGet"));
        assert!(helpers.contains("__classPrivateFieldSet"));
        assert!(helpers.contains("__classPrivateFieldIn"));
        assert!(helpers.contains("Cannot read private member"));
        assert!(helpers.contains("Cannot write private member"));
    }

    #[test]
    fn test_transform_class_to_es5() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_field("count", "Counter", false, Some("0".to_string()), None, false);
        }

        transformer.exit_class();

        let result = transform_class_to_es5("Counter", &transformer);
        assert!(!result.pre_declarations.is_empty());
        assert!(!result.constructor_init.is_empty());
        assert!(result.needs_helpers);
    }

    #[test]
    fn test_accessor_transform() {
        let mut transformer = PrivateFieldsTransformer::new();
        transformer.enter_class("Counter");

        if let Some(ctx) = transformer.current_context() {
            ctx.add_accessor(
                "value",
                "Counter",
                false,
                Some("function() { return this._value; }".to_string()),
                Some("function(v) { this._value = v; }".to_string()),
            );
        }

        let declarations = transformer.generate_weak_map_declarations("Counter");
        assert!(declarations.iter().any(|d| d.contains("new WeakMap()")));

        let init = transformer.generate_constructor_init("Counter");
        assert!(init.iter().any(|i| i.contains("get:")));
        assert!(init.iter().any(|i| i.contains("set:")));
    }
}
