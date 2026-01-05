//! JavaScript Transforms
//!
//! This module contains transforms that convert TypeScript/ES2015+ code to
//! earlier JavaScript versions (ES5, ES3) for compatibility.
//!
//! The transforms follow TypeScript's transformer pipeline architecture.

pub mod async_gen;
pub mod class;
pub mod es2015;
pub mod generators;
pub mod helpers;
pub mod modules;

use crate::emitter::ScriptTarget;
use crate::parser::{NodeArena, NodeIndex};

/// Transform context passed through the transform chain.
pub struct TransformContext<'a> {
    /// Target ECMAScript version
    pub target: ScriptTarget,
    /// The AST arena
    pub arena: &'a mut NodeArena,
    /// Helper functions needed
    pub helpers_needed: HelpersNeeded,
    /// Generated variable counter for unique names
    var_counter: u32,
}

impl<'a> TransformContext<'a> {
    pub fn new(target: ScriptTarget, arena: &'a mut NodeArena) -> Self {
        TransformContext {
            target,
            arena,
            helpers_needed: HelpersNeeded::default(),
            var_counter: 0,
        }
    }

    /// Generate a unique variable name
    pub fn generate_unique_name(&mut self, prefix: &str) -> String {
        let name = format!("_{}{}", prefix, self.var_counter);
        self.var_counter += 1;
        name
    }

    /// Check if we need to downlevel for the target
    pub fn needs_downlevel(&self, min_target: ScriptTarget) -> bool {
        (self.target as u8) < (min_target as u8)
    }
}

/// Tracks which helper functions are needed in the output.
#[derive(Default, Clone)]
pub struct HelpersNeeded {
    pub extends: bool,
    pub assign: bool,
    pub rest: bool,
    pub decorate: bool,
    pub param: bool,
    pub metadata: bool,
    pub awaiter: bool,
    pub generator: bool,
    pub values: bool,
    pub read: bool,
    pub spread: bool,
    pub spread_arrays: bool,
    pub spread_array: bool,
    pub await_values: bool,
    pub async_generator: bool,
    pub async_delegator: bool,
    pub async_values: bool,
    pub export_star: bool,
    pub import_default: bool,
    pub import_star: bool,
    pub make_template_object: bool,
    pub class_private_field_get: bool,
    pub class_private_field_set: bool,
    pub class_private_field_in: bool,
    pub create_binding: bool,
    pub set_function_name: bool,
    pub prop_key: bool,
}

/// Trait for AST transformers
pub trait Transformer {
    /// Transform a node, potentially returning a replacement
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex>;

    /// Transform all children of a node
    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext);
}

/// Transform the AST based on target version
pub fn transform_source_file(
    source_file_idx: NodeIndex,
    target: ScriptTarget,
    arena: &mut NodeArena,
) -> HelpersNeeded {
    let mut ctx = TransformContext::new(target, arena);

    // Only transform if targeting ES5 or lower
    if ctx.needs_downlevel(ScriptTarget::ES2015) {
        // Run async/generator transformer first (handles async functions, generators)
        let mut async_transformer = async_gen::AsyncTransformer::new();
        async_transformer.visit_node(source_file_idx, &mut ctx);

        // Run class transformer (handles class declarations, super calls)
        let mut class_transformer = class::ClassTransformer::new();
        class_transformer.visit_node(source_file_idx, &mut ctx);

        // Then run ES2015 transformer (handles arrow functions, templates, etc.)
        let mut es2015_transformer = es2015::ES2015Transformer::new();
        es2015_transformer.visit_node(source_file_idx, &mut ctx);
    }

    ctx.helpers_needed
}
