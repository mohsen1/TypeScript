//! Control Flow Analysis for type narrowing.
//!
//! This module provides flow-sensitive type analysis that walks the control flow
//! graph backwards from identifier usages to determine narrowed types.
//!
//! Example:
//! ```typescript
//! function foo(x: string | number) {
//!     if (typeof x === "string") {
//!         // FlowAnalyzer walks back and sees TRUE_CONDITION (typeof x === "string")
//!         // Returns: string (narrowed from string | number)
//!         console.log(x.length);
//!     } else {
//!         // FlowAnalyzer sees FALSE_CONDITION
//!         // Returns: number
//!         console.log(x.toFixed(2));
//!     }
//! }
//! ```

use crate::binder::{FlowNode, FlowNodeId, flow_flags};
use crate::interner::Atom;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::{NodeIndex, syntax_kind_ext};
use crate::scanner::SyntaxKind;
use crate::solver::{LiteralValue, TypeId, TypeInterner, TypeKey, NarrowingContext};
use crate::thin_binder::ThinBinderState;
use std::borrow::Cow;

/// Flow analyzer for control flow-based type narrowing.
///
/// Walks the control flow graph backwards from a reference point to determine
/// what type narrowing applies at that location.
pub struct FlowAnalyzer<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    interner: &'a TypeInterner,
}

impl<'a> FlowAnalyzer<'a> {
    /// Create a new FlowAnalyzer.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        interner: &'a TypeInterner,
    ) -> Self {
        Self { arena, binder, interner }
    }

    /// Get the narrowed type of a symbol at a specific flow node.
    ///
    /// This walks backwards through the flow graph, applying narrowing operations
    /// when it encounters condition nodes.
    pub fn get_flow_type(
        &self,
        reference: NodeIndex,
        initial_type: TypeId,
        flow_node: FlowNodeId,
    ) -> TypeId {
        if flow_node.is_none() {
            return initial_type;
        }

        self.check_flow(reference, initial_type, flow_node, &mut Vec::new())
    }

    /// Recursive flow graph traversal with cycle detection.
    fn check_flow(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow_id: FlowNodeId,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // Cycle detection
        if visited.contains(&flow_id) {
            return type_id;
        }
        visited.push(flow_id);

        let Some(flow) = self.binder.flow_nodes.get(flow_id) else {
            return type_id;
        };

        // Handle different flow node types
        if flow.has_any_flags(flow_flags::BRANCH_LABEL) {
            return self.handle_branch_label(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::LOOP_LABEL) {
            return self.handle_loop_label(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::CONDITION) {
            return self.handle_condition(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::START) {
            // Reached start of flow - return initial type
            return type_id;
        }

        // Default: continue to antecedent
        if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        }
    }

    /// Handle branch label (merge point) - union of types from all branches.
    fn handle_branch_label(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        if flow.antecedent.is_empty() {
            return type_id;
        }

        // Get types from all incoming branches
        let branch_types: Vec<TypeId> = flow.antecedent.iter()
            .map(|&ant| self.check_flow(reference, type_id, ant, &mut visited.clone()))
            .collect();

        // Union the types from different branches
        if branch_types.is_empty() {
            type_id
        } else if branch_types.len() == 1 {
            branch_types[0]
        } else {
            self.interner.union(branch_types)
        }
    }

    /// Handle loop label - for now, just take the type from entry.
    fn handle_loop_label(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // For loops, we ideally compute a fixed point.
        // For basic narrowing, we can just take the type from the entry antecedent.
        if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        }
    }

    /// Handle condition node (TRUE_CONDITION or FALSE_CONDITION).
    fn handle_condition(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // First get the type before this condition
        let pre_type = if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        };

        // Check if this condition narrows the reference we're interested in
        let is_true_branch = flow.has_any_flags(flow_flags::TRUE_CONDITION);

        // Apply narrowing based on the condition
        self.narrow_type_by_condition(pre_type, flow.node, reference, is_true_branch)
    }

    /// Apply type narrowing based on a condition expression.
    fn narrow_type_by_condition(
        &self,
        type_id: TypeId,
        condition_idx: NodeIndex,
        target: NodeIndex,
        is_true_branch: bool,
    ) -> TypeId {
        let condition_idx = self.skip_parenthesized(condition_idx);
        let Some(cond_node) = self.arena.get(condition_idx) else {
            return type_id;
        };

        let narrowing = NarrowingContext::new(self.interner);

        match cond_node.kind {
            // typeof x === "string"
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(cond_node) {
                    if let Some(narrowed) = self.narrow_by_logical_expr(type_id, bin, target, is_true_branch) {
                        return narrowed;
                    }
                    return self.narrow_by_binary_expr(type_id, bin, target, is_true_branch, &narrowing);
                }
            }

            // Prefix unary: !x
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(cond_node) {
                    // !x inverts the narrowing
                    if unary.operator == SyntaxKind::ExclamationToken as u16 {
                        return self.narrow_type_by_condition(type_id, unary.operand, target, !is_true_branch);
                    }
                }
            }

            // Truthiness check: if (x)
            _ => {
                if self.is_matching_reference(condition_idx, target) {
                    if is_true_branch {
                        // Remove null/undefined (truthy narrowing)
                        let narrowed = narrowing.narrow_excluding_type(type_id, TypeId::NULL);
                        return narrowing.narrow_excluding_type(narrowed, TypeId::UNDEFINED);
                    }
                    // False branch - keep only falsy types
                    return self.narrow_to_falsy(type_id);
                }
            }
        }

        type_id
    }

    /// Narrow type based on a binary expression (===, !==, typeof checks, etc.)
    fn narrow_by_binary_expr(
        &self,
        type_id: TypeId,
        bin: &crate::parser::thin_node::BinaryExprData,
        target: NodeIndex,
        is_true_branch: bool,
        narrowing: &NarrowingContext,
    ) -> TypeId {
        let operator = bin.operator_token;

        let (is_equals, is_strict) = match operator {
            k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => (true, true),
            k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => (false, true),
            k if k == SyntaxKind::EqualsEqualsToken as u16 => (true, false),
            k if k == SyntaxKind::ExclamationEqualsToken as u16 => (false, false),
            _ => return type_id,
        };

        let effective_truth = if is_equals { is_true_branch } else { !is_true_branch };

        if let Some(type_name) = self.typeof_comparison_literal(bin.left, bin.right, target) {
            if effective_truth {
                return narrowing.narrow_by_typeof(type_id, type_name);
            }
            return self.narrow_by_typeof_negation(type_id, type_name, narrowing);
        }

        if let Some(nullish) = self.nullish_comparison(bin.left, bin.right, target) {
            if is_strict {
                if effective_truth {
                    return nullish;
                }
                return narrowing.narrow_excluding_type(type_id, nullish);
            }

            let nullish_union = self.interner.union(vec![TypeId::NULL, TypeId::UNDEFINED]);
            if effective_truth {
                return nullish_union;
            }

            let narrowed = narrowing.narrow_excluding_type(type_id, TypeId::NULL);
            return narrowing.narrow_excluding_type(narrowed, TypeId::UNDEFINED);
        }

        if is_strict {
            if let Some((prop_name, literal_type)) = self.discriminant_comparison(bin.left, bin.right, target) {
                if effective_truth {
                    return narrowing.narrow_by_discriminant(type_id, prop_name, literal_type);
                }
                return narrowing.narrow_by_excluding_discriminant(type_id, prop_name, literal_type);
            }

            if let Some(literal_type) = self.literal_comparison(bin.left, bin.right, target) {
                if effective_truth {
                    let narrowed = narrowing.narrow_to_type(type_id, literal_type);
                    if narrowed != TypeId::NEVER {
                        return narrowed;
                    }
                    if self.literal_assignable_to(literal_type, type_id, narrowing) {
                        return literal_type;
                    }
                    return TypeId::NEVER;
                }
                return narrowing.narrow_excluding_type(type_id, literal_type);
            }
        }

        type_id
    }

    fn narrow_by_logical_expr(
        &self,
        type_id: TypeId,
        bin: &crate::parser::thin_node::BinaryExprData,
        target: NodeIndex,
        is_true_branch: bool,
    ) -> Option<TypeId> {
        let operator = bin.operator_token;

        if operator == SyntaxKind::AmpersandAmpersandToken as u16 {
            if is_true_branch {
                let left_true = self.narrow_type_by_condition(type_id, bin.left, target, true);
                let right_true = self.narrow_type_by_condition(left_true, bin.right, target, true);
                return Some(right_true);
            }

            let left_false = self.narrow_type_by_condition(type_id, bin.left, target, false);
            let left_true = self.narrow_type_by_condition(type_id, bin.left, target, true);
            let right_false = self.narrow_type_by_condition(left_true, bin.right, target, false);
            return Some(self.union_types(left_false, right_false));
        }

        if operator == SyntaxKind::BarBarToken as u16 {
            if is_true_branch {
                let left_true = self.narrow_type_by_condition(type_id, bin.left, target, true);
                let left_false = self.narrow_type_by_condition(type_id, bin.left, target, false);
                let right_true = self.narrow_type_by_condition(left_false, bin.right, target, true);
                return Some(self.union_types(left_true, right_true));
            }

            let left_false = self.narrow_type_by_condition(type_id, bin.left, target, false);
            let right_false = self.narrow_type_by_condition(left_false, bin.right, target, false);
            return Some(right_false);
        }

        None
    }

    fn union_types(&self, left: TypeId, right: TypeId) -> TypeId {
        if left == right {
            left
        } else {
            self.interner.union(vec![left, right])
        }
    }

    fn skip_parenthesized(&self, mut idx: NodeIndex) -> NodeIndex {
        loop {
            let Some(node) = self.arena.get(idx) else {
                return idx;
            };
            if node.kind == syntax_kind_ext::PARENTHESIZED_EXPRESSION {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    idx = paren.expression;
                    continue;
                }
            }
            return idx;
        }
    }

    fn typeof_comparison_literal(
        &self,
        left: NodeIndex,
        right: NodeIndex,
        target: NodeIndex,
    ) -> Option<&str> {
        if self.is_typeof_target(left, target) {
            return self.literal_string_from_node(right);
        }
        if self.is_typeof_target(right, target) {
            return self.literal_string_from_node(left);
        }
        None
    }

    fn is_typeof_target(&self, expr: NodeIndex, target: NodeIndex) -> bool {
        let expr = self.skip_parenthesized(expr);
        let node = match self.arena.get(expr) {
            Some(node) => node,
            None => return false,
        };

        if node.kind != syntax_kind_ext::PREFIX_UNARY_EXPRESSION {
            return false;
        }

        let Some(unary) = self.arena.get_unary_expr(node) else {
            return false;
        };

        if unary.operator != SyntaxKind::TypeOfKeyword as u16 {
            return false;
        }

        self.is_matching_reference(unary.operand, target)
    }

    fn literal_string_from_node(&self, idx: NodeIndex) -> Option<&str> {
        let idx = self.skip_parenthesized(idx);
        let node = self.arena.get(idx)?;

        if node.kind == SyntaxKind::StringLiteral as u16
            || node.kind == SyntaxKind::NoSubstitutionTemplateLiteral as u16
        {
            return self.arena.get_literal(node).map(|lit| lit.text.as_str());
        }

        None
    }

    fn literal_type_from_node(&self, idx: NodeIndex) -> Option<TypeId> {
        let idx = self.skip_parenthesized(idx);
        let node = self.arena.get(idx)?;

        match node.kind {
            k if k == SyntaxKind::StringLiteral as u16
                || k == SyntaxKind::NoSubstitutionTemplateLiteral as u16 =>
            {
                let lit = self.arena.get_literal(node)?;
                Some(self.interner.literal_string(&lit.text))
            }
            k if k == SyntaxKind::NumericLiteral as u16 => {
                let lit = self.arena.get_literal(node)?;
                let value = self.parse_numeric_literal_value(lit.value, &lit.text)?;
                Some(self.interner.literal_number(value))
            }
            k if k == SyntaxKind::BigIntLiteral as u16 => {
                let lit = self.arena.get_literal(node)?;
                let text = lit.text.strip_suffix('n').unwrap_or(&lit.text);
                let normalized = self.normalize_bigint_literal(text)?;
                Some(self.interner.literal_bigint(normalized.as_ref()))
            }
            k if k == SyntaxKind::TrueKeyword as u16 => Some(self.interner.literal_boolean(true)),
            k if k == SyntaxKind::FalseKeyword as u16 => Some(self.interner.literal_boolean(false)),
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                let unary = self.arena.get_unary_expr(node)?;
                let op = unary.operator;
                if op != SyntaxKind::MinusToken as u16 && op != SyntaxKind::PlusToken as u16 {
                    return None;
                }

                let operand = self.skip_parenthesized(unary.operand);
                let operand_node = self.arena.get(operand)?;
                match operand_node.kind {
                    k if k == SyntaxKind::NumericLiteral as u16 => {
                        let lit = self.arena.get_literal(operand_node)?;
                        let value = self.parse_numeric_literal_value(lit.value, &lit.text)?;
                        let value = if op == SyntaxKind::MinusToken as u16 { -value } else { value };
                        Some(self.interner.literal_number(value))
                    }
                    k if k == SyntaxKind::BigIntLiteral as u16 => {
                        let lit = self.arena.get_literal(operand_node)?;
                        let text = lit.text.strip_suffix('n').unwrap_or(&lit.text);
                        let normalized = self.normalize_bigint_literal(text)?;
                        let negative = op == SyntaxKind::MinusToken as u16;
                        Some(self.interner.literal_bigint_with_sign(negative, normalized.as_ref()))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn literal_assignable_to(
        &self,
        literal: TypeId,
        target: TypeId,
        narrowing: &NarrowingContext,
    ) -> bool {
        if literal == target || target == TypeId::ANY || target == TypeId::UNKNOWN {
            return true;
        }

        if let Some(TypeKey::Union(members)) = self.interner.lookup(target) {
            let members = self.interner.type_list(members);
            return members
                .iter()
                .any(|&member| self.literal_assignable_to(literal, member, narrowing));
        }

        narrowing.narrow_to_type(literal, target) != TypeId::NEVER
    }

    fn nullish_literal_type(&self, idx: NodeIndex) -> Option<TypeId> {
        let idx = self.skip_parenthesized(idx);
        let node = self.arena.get(idx)?;

        if node.kind == SyntaxKind::NullKeyword as u16 {
            return Some(TypeId::NULL);
        }
        if node.kind == SyntaxKind::UndefinedKeyword as u16 {
            return Some(TypeId::UNDEFINED);
        }

        None
    }

    fn nullish_comparison(
        &self,
        left: NodeIndex,
        right: NodeIndex,
        target: NodeIndex,
    ) -> Option<TypeId> {
        if self.is_matching_reference(left, target) {
            return self.nullish_literal_type(right);
        }
        if self.is_matching_reference(right, target) {
            return self.nullish_literal_type(left);
        }
        None
    }

    fn discriminant_property(&self, expr: NodeIndex, target: NodeIndex) -> Option<Atom> {
        let expr = self.skip_parenthesized(expr);
        let node = self.arena.get(expr)?;

        if node.kind == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
            let access = self.arena.get_access_expr(node)?;
            if access.question_dot_token || !self.is_matching_reference(access.expression, target) {
                return None;
            }
            let name_node = self.arena.get(access.name_or_argument)?;
            let ident = self.arena.get_identifier(name_node)?;
            return Some(self.interner.intern_string(&ident.escaped_text));
        }

        if node.kind == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION {
            let access = self.arena.get_access_expr(node)?;
            if access.question_dot_token || !self.is_matching_reference(access.expression, target) {
                return None;
            }
            let name = self.literal_string_from_node(access.name_or_argument)?;
            return Some(self.interner.intern_string(name));
        }

        None
    }

    fn discriminant_comparison(
        &self,
        left: NodeIndex,
        right: NodeIndex,
        target: NodeIndex,
    ) -> Option<(Atom, TypeId)> {
        if let Some(prop) = self.discriminant_property(left, target) {
            if let Some(literal) = self.literal_type_from_node(right) {
                return Some((prop, literal));
            }
        }

        if let Some(prop) = self.discriminant_property(right, target) {
            if let Some(literal) = self.literal_type_from_node(left) {
                return Some((prop, literal));
            }
        }

        None
    }

    fn literal_comparison(
        &self,
        left: NodeIndex,
        right: NodeIndex,
        target: NodeIndex,
    ) -> Option<TypeId> {
        if self.is_matching_reference(left, target) {
            return self.literal_type_from_node(right);
        }
        if self.is_matching_reference(right, target) {
            return self.literal_type_from_node(left);
        }
        None
    }

    fn narrow_by_typeof_negation(
        &self,
        type_id: TypeId,
        typeof_result: &str,
        narrowing: &NarrowingContext,
    ) -> TypeId {
        match typeof_result {
            "string" => narrowing.narrow_excluding_type(type_id, TypeId::STRING),
            "number" => narrowing.narrow_excluding_type(type_id, TypeId::NUMBER),
            "boolean" => narrowing.narrow_excluding_type(type_id, TypeId::BOOLEAN),
            "bigint" => narrowing.narrow_excluding_type(type_id, TypeId::BIGINT),
            "symbol" => narrowing.narrow_excluding_type(type_id, TypeId::SYMBOL),
            "undefined" => narrowing.narrow_excluding_type(type_id, TypeId::UNDEFINED),
            "object" => narrowing.narrow_excluding_type(type_id, TypeId::OBJECT),
            "function" => narrowing.narrow_excluding_function(type_id),
            _ => type_id,
        }
    }

    fn narrow_to_falsy(&self, type_id: TypeId) -> TypeId {
        if type_id == TypeId::ANY || type_id == TypeId::UNKNOWN {
            return type_id;
        }

        match self.falsy_component(type_id) {
            Some(falsy) => falsy,
            None => TypeId::NEVER,
        }
    }

    fn falsy_component(&self, type_id: TypeId) -> Option<TypeId> {
        if type_id == TypeId::NULL || type_id == TypeId::UNDEFINED {
            return Some(type_id);
        }
        if type_id == TypeId::BOOLEAN {
            return Some(self.interner.literal_boolean(false));
        }
        if type_id == TypeId::STRING {
            return Some(self.interner.literal_string(""));
        }
        if type_id == TypeId::NUMBER {
            return Some(self.interner.literal_number(0.0));
        }
        if type_id == TypeId::BIGINT {
            return Some(self.interner.literal_bigint("0"));
        }

        let key = self.interner.lookup(type_id)?;
        match key {
            TypeKey::Literal(literal) => {
                if self.literal_is_falsy(&literal) {
                    Some(type_id)
                } else {
                    None
                }
            }
            TypeKey::Union(members) => {
                let members = self.interner.type_list(members);
                let mut falsy_members = Vec::new();
                for &member in members.iter() {
                    if let Some(falsy) = self.falsy_component(member) {
                        falsy_members.push(falsy);
                    }
                }
                match falsy_members.len() {
                    0 => None,
                    1 => Some(falsy_members[0]),
                    _ => Some(self.interner.union(falsy_members)),
                }
            }
            TypeKey::TypeParameter(_) | TypeKey::Infer(_) => Some(type_id),
            _ => None,
        }
    }

    fn literal_is_falsy(&self, literal: &LiteralValue) -> bool {
        match literal {
            LiteralValue::Boolean(false) => true,
            LiteralValue::Number(value) => value.0 == 0.0,
            LiteralValue::String(atom) => self.interner.resolve_atom(*atom).is_empty(),
            LiteralValue::BigInt(atom) => self.interner.resolve_atom(*atom) == "0",
            _ => false,
        }
    }

    fn strip_numeric_separators<'b>(&self, text: &'b str) -> Cow<'b, str> {
        if !text.as_bytes().contains(&b'_') {
            return Cow::Borrowed(text);
        }

        let mut out = String::with_capacity(text.len());
        for &byte in text.as_bytes() {
            if byte != b'_' {
                out.push(byte as char);
            }
        }
        Cow::Owned(out)
    }

    fn parse_numeric_literal_value(&self, value: Option<f64>, text: &str) -> Option<f64> {
        if let Some(value) = value {
            return Some(value);
        }

        if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            return Self::parse_radix_digits(rest, 16);
        }
        if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
            return Self::parse_radix_digits(rest, 2);
        }
        if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
            return Self::parse_radix_digits(rest, 8);
        }

        if text.as_bytes().contains(&b'_') {
            let cleaned = self.strip_numeric_separators(text);
            return cleaned.as_ref().parse::<f64>().ok();
        }

        text.parse::<f64>().ok()
    }

    fn parse_radix_digits(text: &str, base: u32) -> Option<f64> {
        if text.is_empty() {
            return None;
        }

        let mut value = 0f64;
        let base_value = base as f64;
        let mut saw_digit = false;
        for &byte in text.as_bytes() {
            if byte == b'_' {
                continue;
            }

            let digit = match byte {
                b'0'..=b'9' => (byte - b'0') as u32,
                b'a'..=b'f' => (byte - b'a' + 10) as u32,
                b'A'..=b'F' => (byte - b'A' + 10) as u32,
                _ => return None,
            };
            if digit >= base {
                return None;
            }
            saw_digit = true;
            value = value * base_value + digit as f64;
        }

        if !saw_digit {
            return None;
        }

        Some(value)
    }

    fn normalize_bigint_literal<'b>(&self, text: &'b str) -> Option<Cow<'b, str>> {
        if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            return Self::bigint_base_to_decimal(rest, 16).map(Cow::Owned);
        }
        if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
            return Self::bigint_base_to_decimal(rest, 2).map(Cow::Owned);
        }
        if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
            return Self::bigint_base_to_decimal(rest, 8).map(Cow::Owned);
        }

        match self.strip_numeric_separators(text) {
            Cow::Borrowed(cleaned) => {
                let trimmed = cleaned.trim_start_matches('0');
                if trimmed.is_empty() {
                    return Some(Cow::Borrowed("0"));
                }
                if trimmed.len() == cleaned.len() {
                    return Some(Cow::Borrowed(cleaned));
                }
                Some(Cow::Borrowed(trimmed))
            }
            Cow::Owned(mut cleaned) => {
                let cleaned_ref = cleaned.as_str();
                let trimmed = cleaned_ref.trim_start_matches('0');
                if trimmed.is_empty() {
                    return Some(Cow::Borrowed("0"));
                }
                if trimmed.len() == cleaned_ref.len() {
                    return Some(Cow::Owned(cleaned));
                }

                let trim_len = cleaned_ref.len() - trimmed.len();
                cleaned.drain(..trim_len);
                Some(Cow::Owned(cleaned))
            }
        }
    }

    fn bigint_base_to_decimal(text: &str, base: u32) -> Option<String> {
        if text.is_empty() {
            return None;
        }

        let mut digits: Vec<u8> = vec![0];
        let mut saw_digit = false;
        for &byte in text.as_bytes() {
            if byte == b'_' {
                continue;
            }

            let digit = match byte {
                b'0'..=b'9' => (byte - b'0') as u32,
                b'a'..=b'f' => (byte - b'a' + 10) as u32,
                b'A'..=b'F' => (byte - b'A' + 10) as u32,
                _ => return None,
            };
            if digit >= base {
                return None;
            }
            saw_digit = true;

            let mut carry = digit;
            for slot in &mut digits {
                let value = (*slot as u32) * base + carry;
                *slot = (value % 10) as u8;
                carry = value / 10;
            }
            while carry > 0 {
                digits.push((carry % 10) as u8);
                carry /= 10;
            }
        }

        if !saw_digit {
            return None;
        }

        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }

        let mut out = String::with_capacity(digits.len());
        for digit in digits.iter().rev() {
            out.push(char::from(b'0' + *digit));
        }
        Some(out)
    }

    /// Check if two references point to the same symbol.
    fn is_matching_reference(&self, a: NodeIndex, b: NodeIndex) -> bool {
        let sym_a = self.reference_symbol(a);
        let sym_b = self.reference_symbol(b);
        sym_a.is_some() && sym_a == sym_b
    }

    fn reference_symbol(&self, idx: NodeIndex) -> Option<crate::binder::SymbolId> {
        let idx = self.skip_parenthesized(idx);
        self.binder.get_node_symbol(idx)
            .or_else(|| self.binder.resolve_identifier(self.arena, idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::PropertyInfo;
    use crate::thin_parser::ThinParserState;

    fn get_if_condition(arena: &ThinNodeArena, root: NodeIndex, stmt_index: usize) -> NodeIndex {
        let root_node = arena.get(root).expect("root node");
        let source_file = arena.get_source_file(root_node).expect("source file");
        let if_idx = *source_file.statements.nodes.get(stmt_index).expect("if statement");
        let if_node = arena.get(if_idx).expect("if node");
        let if_data = arena.get_if_statement(if_node).expect("if data");
        if_data.expression
    }

    #[test]
    fn test_truthiness_false_branch_narrows_to_falsy() {
        let source = r#"
let x: string | number | boolean | null | undefined;
if (x) {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let union = types.union(vec![
            TypeId::STRING,
            TypeId::NUMBER,
            TypeId::BOOLEAN,
            TypeId::NULL,
            TypeId::UNDEFINED,
        ]);
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, condition_idx, false);

        let falsy_string = types.literal_string("");
        let falsy_number = types.literal_number(0.0);
        let falsy_boolean = types.literal_boolean(false);

        let key = types.lookup(narrowed).expect("narrowed type");
        match key {
            TypeKey::Union(members) => {
                let members = types.type_list(members);
                assert!(members.contains(&falsy_string));
                assert!(members.contains(&falsy_number));
                assert!(members.contains(&falsy_boolean));
                assert!(members.contains(&TypeId::NULL));
                assert!(members.contains(&TypeId::UNDEFINED));
            }
            _ => panic!("Expected falsy union, got {:?}", key),
        }
    }

    #[test]
    fn test_typeof_false_branch_excludes_type() {
        let source = r#"
let x: string | number;
if (typeof x === "string") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let typeof_node = arena.get(binary.left).expect("typeof node");
        let unary = arena.get_unary_expr(typeof_node).expect("typeof data");
        let target_idx = unary.operand;

        let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, false);
        assert_eq!(narrowed, TypeId::NUMBER);
    }

    #[test]
    fn test_logical_and_applies_right_guard() {
        let source = r#"
let x: string | number;
if (x && typeof x === "string") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let target_idx = binary.left;

        let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, true);
        assert_eq!(narrowed, TypeId::STRING);
    }

    #[test]
    fn test_logical_or_narrows_to_union_of_literals() {
        let source = r#"
let x: "a" | "b" | "c";
if (x === "a" || x === "b") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let left_node = arena.get(binary.left).expect("left condition");
        let left_eq = arena.get_binary_expr(left_node).expect("left equality");
        let target_idx = left_eq.left;

        let lit_a = types.literal_string("a");
        let lit_b = types.literal_string("b");
        let lit_c = types.literal_string("c");
        let union = types.union(vec![lit_a, lit_b, lit_c]);

        let narrowed_true = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, true);
        let narrowed_false = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, false);

        assert_eq!(narrowed_true, types.union(vec![lit_a, lit_b]));
        assert_eq!(narrowed_false, lit_c);
    }

    #[test]
    fn test_discriminant_property_access_narrows_union() {
        let source = r#"
let action: any;
if (action.type === "add") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let access_node = arena.get(binary.left).expect("property access node");
        let access = arena.get_access_expr(access_node).expect("property access data");
        let target_idx = access.expression;

        let type_key = types.intern_string("type");
        let type_add = types.literal_string("add");
        let type_remove = types.literal_string("remove");

        let add_member = types.object(vec![
            PropertyInfo { name: type_key, type_id: type_add, optional: false, readonly: false, is_method: false },
        ]);
        let remove_member = types.object(vec![
            PropertyInfo { name: type_key, type_id: type_remove, optional: false, readonly: false, is_method: false },
        ]);

        let union = types.union(vec![add_member, remove_member]);
        let narrowed_true = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, true);
        let narrowed_false = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, false);

        assert_eq!(narrowed_true, add_member);
        assert_eq!(narrowed_false, remove_member);
    }

    #[test]
    fn test_literal_equality_narrows_to_literal() {
        let source = r#"
let x: string | number;
if (x === "a") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let target_idx = binary.left;

        let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);
        let literal_a = types.literal_string("a");
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, true);

        assert_eq!(narrowed, literal_a);
    }

    #[test]
    fn test_loose_nullish_equality_narrows_to_nullish_union() {
        let source = r#"
let x: string | null | undefined;
if (x == null) {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let target_idx = binary.left;

        let union = types.union(vec![TypeId::STRING, TypeId::NULL, TypeId::UNDEFINED]);
        let expected_true = types.union(vec![TypeId::NULL, TypeId::UNDEFINED]);

        let narrowed_true = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, true);
        let narrowed_false = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, false);

        assert_eq!(narrowed_true, expected_true);
        assert_eq!(narrowed_false, TypeId::STRING);
    }
}
