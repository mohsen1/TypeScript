//! Enum Transform
//!
//! Transforms TypeScript enums to JavaScript objects.
//! Handles:
//! - Numeric enums with reverse mapping
//! - String enums (no reverse mapping)
//! - Const enum inlining (removes enum, replaces usages with values)
//! - Ambient enum handling (declaration only, no emit)

use crate::checker::enums::{EnumDeclaration, EnumMember, EnumMemberValue};

/// JavaScript code output
#[derive(Debug, Clone)]
pub struct JsOutput {
    /// Generated JavaScript code
    pub code: String,
    /// Whether this was a const enum (no runtime emit)
    pub is_const_enum: bool,
}

/// Enum transformer options
#[derive(Debug, Clone)]
pub struct EnumTransformOptions {
    /// Preserve const enums (emit runtime code instead of inlining)
    pub preserve_const_enums: bool,
    /// Use const enum values for inlining
    pub inline_const_enums: bool,
}

impl Default for EnumTransformOptions {
    fn default() -> Self {
        EnumTransformOptions {
            preserve_const_enums: false,
            inline_const_enums: true,
        }
    }
}

/// Enum transformer
pub struct EnumTransformer {
    options: EnumTransformOptions,
}

impl EnumTransformer {
    pub fn new(options: EnumTransformOptions) -> Self {
        EnumTransformer { options }
    }

    /// Transform an enum declaration to JavaScript
    pub fn transform<'a>(&self, decl: &EnumDeclaration<'a>) -> JsOutput {
        // Ambient enums don't emit code
        if decl.is_ambient {
            return JsOutput {
                code: String::new(),
                is_const_enum: false,
            };
        }

        // Const enums are typically inlined (unless preserved)
        if decl.is_const && !self.options.preserve_const_enums {
            return JsOutput {
                code: String::new(),
                is_const_enum: true,
            };
        }

        // Check if this is a string enum
        let is_string_enum = decl.members.iter().all(|m| {
            matches!(m.value, EnumMemberValue::String(_))
        });

        let code = if is_string_enum {
            self.transform_string_enum(decl)
        } else {
            self.transform_numeric_enum(decl)
        };

        JsOutput {
            code,
            is_const_enum: false,
        }
    }

    /// Transform a numeric enum with reverse mapping
    fn transform_numeric_enum<'a>(&self, decl: &EnumDeclaration<'a>) -> String {
        let mut code = String::new();

        // Emit IIFE pattern: var Enum; (function(Enum) { ... })(Enum || (Enum = {}));
        code.push_str(&format!("var {};\n", decl.name));
        code.push_str(&format!("(function ({}) {{\n", decl.name));

        for member in &decl.members {
            match &member.value {
                EnumMemberValue::Number(n) => {
                    // Forward mapping: Enum["Member"] = value
                    code.push_str(&format!(
                        "    {}[\"{}\"] = {};\n",
                        decl.name, member.name, n
                    ));
                    // Reverse mapping: Enum[value] = "Member"
                    code.push_str(&format!(
                        "    {}[{}] = \"{}\";\n",
                        decl.name, n, member.name
                    ));
                }
                EnumMemberValue::String(s) => {
                    // Mixed enum with string value (no reverse mapping for strings)
                    code.push_str(&format!(
                        "    {}[\"{}\"] = \"{}\";\n",
                        decl.name, member.name, s
                    ));
                }
                EnumMemberValue::Computed => {
                    // Computed value - emit expression as-is
                    code.push_str(&format!(
                        "    {}[\"{}\"] = /* computed */;\n",
                        decl.name, member.name
                    ));
                }
            }
        }

        code.push_str(&format!(
            "}})({} || ({} = {{}}));\n",
            decl.name, decl.name
        ));

        code
    }

    /// Transform a string enum (no reverse mapping)
    fn transform_string_enum<'a>(&self, decl: &EnumDeclaration<'a>) -> String {
        let mut code = String::new();

        code.push_str(&format!("var {};\n", decl.name));
        code.push_str(&format!("(function ({}) {{\n", decl.name));

        for member in &decl.members {
            if let EnumMemberValue::String(s) = &member.value {
                code.push_str(&format!(
                    "    {}[\"{}\"] = \"{}\";\n",
                    decl.name, member.name, s
                ));
            }
        }

        code.push_str(&format!(
            "}})({} || ({} = {{}}));\n",
            decl.name, decl.name
        ));

        code
    }

    /// Get the inline value for a const enum member access
    pub fn get_inline_value(&self, member: &EnumMember<'_>) -> String {
        match &member.value {
            EnumMemberValue::Number(n) => format!("{} /* {} */", n, member.name),
            EnumMemberValue::String(s) => format!("\"{}\" /* {} */", s, member.name),
            EnumMemberValue::Computed => format!("/* computed: {} */", member.name),
        }
    }

    /// Transform a const enum member access to its inline value
    pub fn inline_const_enum_access(
        &self,
        enum_name: &str,
        member_name: &str,
        value: &EnumMemberValue,
    ) -> String {
        match value {
            EnumMemberValue::Number(n) => {
                format!("{} /* {}.{} */", n, enum_name, member_name)
            }
            EnumMemberValue::String(s) => {
                format!("\"{}\" /* {}.{} */", s, enum_name, member_name)
            }
            EnumMemberValue::Computed => {
                format!("/* {}.{} (computed) */", enum_name, member_name)
            }
        }
    }
}

impl Default for EnumTransformer {
    fn default() -> Self {
        Self::new(EnumTransformOptions::default())
    }
}

/// Transform enum to object literal (alternative simpler emit)
pub fn transform_to_object_literal<'a>(decl: &EnumDeclaration<'a>) -> String {
    let mut code = String::new();

    code.push_str(&format!("const {} = {{\n", decl.name));

    for (i, member) in decl.members.iter().enumerate() {
        let value_str = match &member.value {
            EnumMemberValue::Number(n) => n.to_string(),
            EnumMemberValue::String(s) => format!("\"{}\"", s),
            EnumMemberValue::Computed => "undefined /* computed */".to_string(),
        };

        if i < decl.members.len() - 1 {
            code.push_str(&format!("    {}: {},\n", member.name, value_str));
        } else {
            code.push_str(&format!("    {}: {}\n", member.name, value_str));
        }
    }

    code.push_str("};\n");
    code
}

/// Create reverse mapping object for numeric enum
pub fn create_reverse_mapping<'a>(decl: &EnumDeclaration<'a>) -> String {
    let mut code = String::new();

    code.push_str(&format!("const {}Reverse = {{\n", decl.name));

    let mut first = true;
    for member in &decl.members {
        if let EnumMemberValue::Number(n) = &member.value {
            if !first {
                code.push_str(",\n");
            }
            code.push_str(&format!("    {}: \"{}\"", n, member.name));
            first = false;
        }
    }

    code.push_str("\n};\n");
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::enums::compute_enum_values;

    #[test]
    fn test_transform_numeric_enum() {
        let transformer = EnumTransformer::default();

        let members = compute_enum_values(&[
            ("A", Some(0), (0, 1)),
            ("B", Some(1), (2, 3)),
            ("C", Some(2), (4, 5)),
        ]);

        let decl = EnumDeclaration {
            name: "Color",
            members,
            is_const: false,
            is_ambient: false,
            span: (0, 20),
        };

        let output = transformer.transform(&decl);
        assert!(!output.is_const_enum);
        assert!(output.code.contains("var Color;"));
        assert!(output.code.contains("Color[\"A\"] = 0"));
        assert!(output.code.contains("Color[0] = \"A\""));  // Reverse mapping
    }

    #[test]
    fn test_transform_string_enum() {
        let transformer = EnumTransformer::default();

        let decl = EnumDeclaration {
            name: "Direction",
            members: vec![
                EnumMember {
                    name: "Up",
                    value: EnumMemberValue::String("UP".to_string()),
                    has_initializer: true,
                    span: (0, 2),
                },
                EnumMember {
                    name: "Down",
                    value: EnumMemberValue::String("DOWN".to_string()),
                    has_initializer: true,
                    span: (3, 7),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 20),
        };

        let output = transformer.transform(&decl);
        assert!(!output.is_const_enum);
        assert!(output.code.contains("Direction[\"Up\"] = \"UP\""));
        assert!(!output.code.contains("Direction[\"UP\"]"));  // No reverse mapping
    }

    #[test]
    fn test_const_enum_no_emit() {
        let transformer = EnumTransformer::default();

        let members = compute_enum_values(&[
            ("A", Some(0), (0, 1)),
        ]);

        let decl = EnumDeclaration {
            name: "ConstEnum",
            members,
            is_const: true,
            is_ambient: false,
            span: (0, 20),
        };

        let output = transformer.transform(&decl);
        assert!(output.is_const_enum);
        assert!(output.code.is_empty());
    }

    #[test]
    fn test_preserve_const_enum() {
        let options = EnumTransformOptions {
            preserve_const_enums: true,
            inline_const_enums: false,
        };
        let transformer = EnumTransformer::new(options);

        let members = compute_enum_values(&[
            ("A", Some(0), (0, 1)),
        ]);

        let decl = EnumDeclaration {
            name: "ConstEnum",
            members,
            is_const: true,
            is_ambient: false,
            span: (0, 20),
        };

        let output = transformer.transform(&decl);
        assert!(!output.is_const_enum);
        assert!(!output.code.is_empty());
    }

    #[test]
    fn test_ambient_enum_no_emit() {
        let transformer = EnumTransformer::default();

        let decl = EnumDeclaration {
            name: "AmbientEnum",
            members: vec![
                EnumMember {
                    name: "A",
                    value: EnumMemberValue::Number(0),
                    has_initializer: true,
                    span: (0, 1),
                },
            ],
            is_const: false,
            is_ambient: true,
            span: (0, 20),
        };

        let output = transformer.transform(&decl);
        assert!(output.code.is_empty());
    }

    #[test]
    fn test_inline_const_enum_access() {
        let transformer = EnumTransformer::default();

        let inline = transformer.inline_const_enum_access(
            "Color",
            "Red",
            &EnumMemberValue::Number(0),
        );
        assert_eq!(inline, "0 /* Color.Red */");

        let inline = transformer.inline_const_enum_access(
            "Direction",
            "Up",
            &EnumMemberValue::String("UP".to_string()),
        );
        assert_eq!(inline, "\"UP\" /* Direction.Up */");
    }

    #[test]
    fn test_transform_to_object_literal() {
        let decl = EnumDeclaration {
            name: "Simple",
            members: vec![
                EnumMember {
                    name: "A",
                    value: EnumMemberValue::Number(0),
                    has_initializer: false,
                    span: (0, 1),
                },
                EnumMember {
                    name: "B",
                    value: EnumMemberValue::Number(1),
                    has_initializer: false,
                    span: (2, 3),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 10),
        };

        let code = transform_to_object_literal(&decl);
        assert!(code.contains("const Simple = {"));
        assert!(code.contains("A: 0"));
        assert!(code.contains("B: 1"));
    }

    #[test]
    fn test_create_reverse_mapping() {
        let decl = EnumDeclaration {
            name: "Color",
            members: vec![
                EnumMember {
                    name: "Red",
                    value: EnumMemberValue::Number(0),
                    has_initializer: true,
                    span: (0, 3),
                },
                EnumMember {
                    name: "Green",
                    value: EnumMemberValue::Number(1),
                    has_initializer: true,
                    span: (4, 9),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 20),
        };

        let code = create_reverse_mapping(&decl);
        assert!(code.contains("const ColorReverse = {"));
        assert!(code.contains("0: \"Red\""));
        assert!(code.contains("1: \"Green\""));
    }
}
