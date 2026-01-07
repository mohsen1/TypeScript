use crate::solver::types::{IntrinsicKind, TypeId};
use crate::solver::TypeDatabase;

pub enum ApparentMemberKind {
    Value(TypeId),
    Method(TypeId),
}

pub struct ApparentMember {
    pub name: &'static str,
    pub kind: ApparentMemberKind,
}

const STRING_METHODS_RETURN_STRING: &[&str] = &[
    "at",
    "charAt",
    "concat",
    "padEnd",
    "padStart",
    "repeat",
    "slice",
    "substr",
    "substring",
    "toLocaleLowerCase",
    "toLocaleUpperCase",
    "toLowerCase",
    "toString",
    "toUpperCase",
    "trim",
    "trimEnd",
    "trimStart",
    "valueOf",
    "replace",
    "replaceAll",
];
const STRING_METHODS_RETURN_NUMBER: &[&str] = &[
    "charCodeAt",
    "codePointAt",
    "indexOf",
    "lastIndexOf",
    "localeCompare",
    "search",
];
const STRING_METHODS_RETURN_BOOLEAN: &[&str] = &["endsWith", "includes", "startsWith"];
const STRING_METHODS_RETURN_ANY: &[&str] = &["match", "matchAll"];
const STRING_METHODS_RETURN_STRING_ARRAY: &[&str] = &["split"];

const NUMBER_METHODS_RETURN_STRING: &[&str] = &[
    "toExponential",
    "toFixed",
    "toLocaleString",
    "toPrecision",
    "toString",
];

const BOOLEAN_METHODS_RETURN_STRING: &[&str] = &["toLocaleString", "toString"];

const BIGINT_METHODS_RETURN_STRING: &[&str] = &["toLocaleString", "toString"];

fn is_member(name: &str, list: &[&str]) -> bool {
    list.iter().any(|&item| item == name)
}

pub fn apparent_primitive_member_kind(
    interner: &dyn TypeDatabase,
    kind: IntrinsicKind,
    name: &str,
) -> Option<ApparentMemberKind> {
    match kind {
        IntrinsicKind::String => {
            if name == "length" {
                return Some(ApparentMemberKind::Value(TypeId::NUMBER));
            }
            if is_member(name, STRING_METHODS_RETURN_STRING) {
                return Some(ApparentMemberKind::Method(TypeId::STRING));
            }
            if is_member(name, STRING_METHODS_RETURN_NUMBER) {
                return Some(ApparentMemberKind::Method(TypeId::NUMBER));
            }
            if is_member(name, STRING_METHODS_RETURN_BOOLEAN) {
                return Some(ApparentMemberKind::Method(TypeId::BOOLEAN));
            }
            if is_member(name, STRING_METHODS_RETURN_ANY) {
                return Some(ApparentMemberKind::Method(TypeId::ANY));
            }
            if is_member(name, STRING_METHODS_RETURN_STRING_ARRAY) {
                let string_array = interner.array(TypeId::STRING);
                return Some(ApparentMemberKind::Method(string_array));
            }
            None
        }
        IntrinsicKind::Number => {
            if is_member(name, NUMBER_METHODS_RETURN_STRING) {
                return Some(ApparentMemberKind::Method(TypeId::STRING));
            }
            if name == "valueOf" {
                return Some(ApparentMemberKind::Method(TypeId::NUMBER));
            }
            None
        }
        IntrinsicKind::Boolean => {
            if is_member(name, BOOLEAN_METHODS_RETURN_STRING) {
                return Some(ApparentMemberKind::Method(TypeId::STRING));
            }
            if name == "valueOf" {
                return Some(ApparentMemberKind::Method(TypeId::BOOLEAN));
            }
            None
        }
        IntrinsicKind::Bigint => {
            if is_member(name, BIGINT_METHODS_RETURN_STRING) {
                return Some(ApparentMemberKind::Method(TypeId::STRING));
            }
            if name == "valueOf" {
                return Some(ApparentMemberKind::Method(TypeId::BIGINT));
            }
            None
        }
        IntrinsicKind::Symbol => {
            if name == "description" {
                let description = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
                return Some(ApparentMemberKind::Value(description));
            }
            if name == "toString" {
                return Some(ApparentMemberKind::Method(TypeId::STRING));
            }
            if name == "valueOf" {
                return Some(ApparentMemberKind::Method(TypeId::SYMBOL));
            }
            None
        }
        _ => None,
    }
}

pub fn apparent_primitive_members(
    interner: &dyn TypeDatabase,
    kind: IntrinsicKind,
) -> Vec<ApparentMember> {
    let mut members = Vec::new();

    match kind {
        IntrinsicKind::String => {
            members.push(ApparentMember {
                name: "length",
                kind: ApparentMemberKind::Value(TypeId::NUMBER),
            });
            for &name in STRING_METHODS_RETURN_STRING {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::STRING),
                });
            }
            for &name in STRING_METHODS_RETURN_NUMBER {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::NUMBER),
                });
            }
            for &name in STRING_METHODS_RETURN_BOOLEAN {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::BOOLEAN),
                });
            }
            for &name in STRING_METHODS_RETURN_ANY {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::ANY),
                });
            }
            let string_array = interner.array(TypeId::STRING);
            for &name in STRING_METHODS_RETURN_STRING_ARRAY {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(string_array),
                });
            }
        }
        IntrinsicKind::Number => {
            for &name in NUMBER_METHODS_RETURN_STRING {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::STRING),
                });
            }
            members.push(ApparentMember {
                name: "valueOf",
                kind: ApparentMemberKind::Method(TypeId::NUMBER),
            });
        }
        IntrinsicKind::Boolean => {
            for &name in BOOLEAN_METHODS_RETURN_STRING {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::STRING),
                });
            }
            members.push(ApparentMember {
                name: "valueOf",
                kind: ApparentMemberKind::Method(TypeId::BOOLEAN),
            });
        }
        IntrinsicKind::Bigint => {
            for &name in BIGINT_METHODS_RETURN_STRING {
                members.push(ApparentMember {
                    name,
                    kind: ApparentMemberKind::Method(TypeId::STRING),
                });
            }
            members.push(ApparentMember {
                name: "valueOf",
                kind: ApparentMemberKind::Method(TypeId::BIGINT),
            });
        }
        IntrinsicKind::Symbol => {
            let description = interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
            members.push(ApparentMember {
                name: "description",
                kind: ApparentMemberKind::Value(description),
            });
            members.push(ApparentMember {
                name: "toString",
                kind: ApparentMemberKind::Method(TypeId::STRING),
            });
            members.push(ApparentMember {
                name: "valueOf",
                kind: ApparentMemberKind::Method(TypeId::SYMBOL),
            });
        }
        _ => {}
    }

    members
}
