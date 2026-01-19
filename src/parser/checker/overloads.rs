//! Overload resolution for function calls
//!
//! This module implements the TypeScript overload resolution algorithm, which selects
//! the best matching signature when calling an overloaded function. The algorithm
//! considers argument types, type arguments, and signature specificity.

use crate::ast::Span;
use super::signatures::{
    Signature, SignatureId, SignatureStore,
    TypeId, TypeParameter,
};
use super::calls::{Argument, TypeArgument, CallError, CallErrorCode};

/// Result of overload resolution
#[derive(Debug, Clone)]
pub enum OverloadResolutionResult {
    /// A single signature matched
    Success {
        signature: SignatureId,
        inferred_type_args: Vec<TypeId>,
    },
    /// Multiple signatures could match (ambiguous)
    Ambiguous {
        candidates: Vec<SignatureId>,
    },
    /// No signature matched
    NoMatch {
        /// Errors from each attempted signature
        errors: Vec<OverloadMatchError>,
    },
}

/// Error from attempting to match a specific overload
#[derive(Debug, Clone)]
pub struct OverloadMatchError {
    /// The signature that failed to match
    pub signature: SignatureId,
    /// Why it failed
    pub reason: OverloadMismatchReason,
    /// Related error details
    pub error: CallError,
}

/// Reason why an overload didn't match
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverloadMismatchReason {
    /// Wrong number of arguments
    ArgumentCount,
    /// Argument type mismatch
    ArgumentType,
    /// Type argument constraint violation
    TypeConstraint,
    /// Wrong number of type arguments
    TypeArgumentCount,
    /// Cannot infer type arguments
    TypeInference,
    /// This context mismatch
    ThisType,
}

/// Candidate overload with match score
#[derive(Debug, Clone)]
struct OverloadCandidate {
    signature: SignatureId,
    score: OverloadScore,
    inferred_type_args: Vec<TypeId>,
    errors: Vec<CallError>,
}

/// Score for ranking overload candidates
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct OverloadScore {
    /// Number of exact type matches
    exact_matches: u16,
    /// Number of subtype matches (not exact but assignable)
    subtype_matches: u16,
    /// Number of parameters that required widening
    widening_matches: u16,
    /// Negative score for rest parameter usage
    rest_penalty: i16,
}

impl Default for OverloadScore {
    fn default() -> Self {
        OverloadScore {
            exact_matches: 0,
            subtype_matches: 0,
            widening_matches: 0,
            rest_penalty: 0,
        }
    }
}

impl OverloadScore {
    fn is_valid(&self) -> bool {
        self.exact_matches > 0 || self.subtype_matches > 0 || self.widening_matches > 0
    }
}

/// Overload resolution context
#[derive(Debug, Clone)]
pub struct OverloadContext {
    /// The call site span for error reporting
    pub call_span: Span,
    /// Whether to allow 'any' to match anything
    pub allow_any_widening: bool,
    /// Whether to prefer more specific overloads
    pub prefer_specific: bool,
    /// The 'this' type at the call site
    pub this_type: TypeId,
}

impl Default for OverloadContext {
    fn default() -> Self {
        OverloadContext {
            call_span: Span::new(0, 0),
            allow_any_widening: true,
            prefer_specific: true,
            this_type: TypeId::NONE,
        }
    }
}

/// Overload resolver
pub struct OverloadResolver<'a> {
    /// Signature store for accessing signatures
    signatures: &'a SignatureStore,
    /// Resolution context
    context: OverloadContext,
}

impl<'a> OverloadResolver<'a> {
    pub fn new(signatures: &'a SignatureStore) -> Self {
        OverloadResolver {
            signatures,
            context: OverloadContext::default(),
        }
    }

    pub fn with_context(mut self, context: OverloadContext) -> Self {
        self.context = context;
        self
    }

    /// Resolve overloads for a call expression
    pub fn resolve(
        &self,
        overloads: &[SignatureId],
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> OverloadResolutionResult {
        if overloads.is_empty() {
            return OverloadResolutionResult::NoMatch {
                errors: vec![],
            };
        }

        // Single overload - no resolution needed
        if overloads.len() == 1 {
            return self.try_single_overload(overloads[0], args, type_args);
        }

        // Try each overload and collect candidates
        let mut candidates: Vec<OverloadCandidate> = Vec::new();
        let mut errors: Vec<OverloadMatchError> = Vec::new();

        for &sig_id in overloads {
            match self.try_overload(sig_id, args, type_args) {
                Ok(candidate) => candidates.push(candidate),
                Err(error) => errors.push(error),
            }
        }

        // No candidates matched
        if candidates.is_empty() {
            return OverloadResolutionResult::NoMatch { errors };
        }

        // Single candidate - success
        if candidates.len() == 1 {
            let candidate = candidates.remove(0);
            return OverloadResolutionResult::Success {
                signature: candidate.signature,
                inferred_type_args: candidate.inferred_type_args,
            };
        }

        // Multiple candidates - find the best one
        self.select_best_candidate(candidates)
    }

    /// Try to match a single overload
    fn try_single_overload(
        &self,
        sig_id: SignatureId,
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> OverloadResolutionResult {
        match self.try_overload(sig_id, args, type_args) {
            Ok(candidate) => OverloadResolutionResult::Success {
                signature: candidate.signature,
                inferred_type_args: candidate.inferred_type_args,
            },
            Err(error) => OverloadResolutionResult::NoMatch {
                errors: vec![error],
            },
        }
    }

    /// Try to match an overload, returning a candidate or error
    fn try_overload(
        &self,
        sig_id: SignatureId,
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> Result<OverloadCandidate, OverloadMatchError> {
        let sig = match self.signatures.get(sig_id) {
            Some(s) => s,
            None => {
                return Err(OverloadMatchError {
                    signature: sig_id,
                    reason: OverloadMismatchReason::ArgumentCount,
                    error: CallError::new(CallErrorCode::NotCallable, self.context.call_span),
                });
            }
        };

        // Check argument count first (fast path)
        if !self.check_argument_count(sig, args) {
            return Err(OverloadMatchError {
                signature: sig_id,
                reason: OverloadMismatchReason::ArgumentCount,
                error: CallError::with_message(
                    CallErrorCode::ArgumentCountMismatch,
                    self.context.call_span,
                    format!(
                        "Expected {} arguments, but got {}.",
                        sig.min_argument_count,
                        args.len()
                    ),
                ),
            });
        }

        // Handle type arguments
        let inferred_type_args = if sig.is_generic() {
            match self.resolve_type_arguments(sig, type_args, args) {
                Ok(args) => args,
                Err(error) => return Err(error),
            }
        } else if type_args.is_some() && !type_args.unwrap().is_empty() {
            return Err(OverloadMatchError {
                signature: sig_id,
                reason: OverloadMismatchReason::TypeArgumentCount,
                error: CallError::new(CallErrorCode::TypeArgsOnNonGeneric, self.context.call_span),
            });
        } else {
            Vec::new()
        };

        // Check argument types and compute score
        let (score, errors) = self.check_argument_types(sig, args, &inferred_type_args);

        if !errors.is_empty() {
            return Err(OverloadMatchError {
                signature: sig_id,
                reason: OverloadMismatchReason::ArgumentType,
                error: errors.into_iter().next().unwrap(),
            });
        }

        Ok(OverloadCandidate {
            signature: sig_id,
            score,
            inferred_type_args,
            errors: Vec::new(),
        })
    }

    /// Check if argument count is valid for signature
    fn check_argument_count(&self, sig: &Signature, args: &[Argument]) -> bool {
        let arg_count = args.len();
        let min = sig.min_argument_count as usize;

        if arg_count < min {
            return false;
        }

        if let Some(max) = sig.max_argument_count() {
            if arg_count > max {
                return false;
            }
        }

        true
    }

    /// Resolve type arguments for a generic signature
    fn resolve_type_arguments(
        &self,
        sig: &Signature,
        type_args: Option<&[TypeArgument]>,
        args: &[Argument],
    ) -> Result<Vec<TypeId>, OverloadMatchError> {
        let type_params = &sig.type_parameters;

        if let Some(provided) = type_args {
            // Explicit type arguments
            let required = type_params.iter().filter(|tp| !tp.has_default()).count();
            if provided.len() < required {
                return Err(OverloadMatchError {
                    signature: sig.id,
                    reason: OverloadMismatchReason::TypeArgumentCount,
                    error: CallError::with_message(
                        CallErrorCode::TypeArgCountMismatch,
                        self.context.call_span,
                        format!(
                            "Expected {} type arguments, but got {}.",
                            required,
                            provided.len()
                        ),
                    ),
                });
            }

            // Check constraints
            let mut result = Vec::with_capacity(type_params.len());
            for (i, tp) in type_params.iter().enumerate() {
                let type_arg = provided.get(i).map(|ta| ta.type_id).unwrap_or(tp.default);

                if tp.has_constraint() && type_arg.is_some() {
                    if !self.satisfies_constraint(type_arg, tp.constraint) {
                        return Err(OverloadMatchError {
                            signature: sig.id,
                            reason: OverloadMismatchReason::TypeConstraint,
                            error: CallError::new(
                                CallErrorCode::TypeArgConstraintViolation,
                                provided.get(i).map(|ta| ta.span).unwrap_or(self.context.call_span),
                            ),
                        });
                    }
                }

                result.push(type_arg);
            }
            Ok(result)
        } else {
            // Infer type arguments
            Ok(self.infer_type_arguments(sig, args))
        }
    }

    /// Infer type arguments from argument types
    fn infer_type_arguments(&self, sig: &Signature, args: &[Argument]) -> Vec<TypeId> {
        let mut inferred: Vec<TypeId> = vec![TypeId::NONE; sig.type_parameters.len()];

        // First pass: infer from exact positions
        for (i, arg) in args.iter().enumerate() {
            if i >= sig.parameters.len() {
                break;
            }
            self.infer_from_argument(
                arg.type_id,
                sig.parameters[i].type_id,
                &sig.type_parameters,
                &mut inferred,
            );
        }

        // Fill in defaults
        for (i, tp) in sig.type_parameters.iter().enumerate() {
            if inferred[i].is_none() {
                inferred[i] = if tp.has_default() {
                    tp.default
                } else if tp.has_constraint() {
                    tp.constraint
                } else {
                    TypeId::UNKNOWN
                };
            }
        }

        inferred
    }

    /// Infer type parameter from argument and parameter type
    fn infer_from_argument(
        &self,
        arg_type: TypeId,
        param_type: TypeId,
        type_params: &[TypeParameter],
        inferred: &mut [TypeId],
    ) {
        // Direct type parameter reference
        for (i, tp) in type_params.iter().enumerate() {
            if tp.type_id == param_type && inferred[i].is_none() {
                inferred[i] = arg_type;
                return;
            }
        }

        // In a real implementation, we'd recursively analyze:
        // - Array<T> -> T[] infer T from element type
        // - Promise<T> -> infer T from awaited type
        // - Function types -> infer from parameter/return types
        // - Object types -> infer from property types
    }

    /// Check argument types and compute match score
    fn check_argument_types(
        &self,
        sig: &Signature,
        args: &[Argument],
        inferred_type_args: &[TypeId],
    ) -> (OverloadScore, Vec<CallError>) {
        let mut score = OverloadScore::default();
        let mut errors = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            let param = if i < sig.parameters.len() {
                Some(&sig.parameters[i])
            } else if sig.has_rest_parameter() {
                score.rest_penalty -= 1;
                sig.rest_parameter()
            } else {
                None
            };

            if let Some(param) = param {
                let param_type = self.substitute_type_args(param.type_id, sig, inferred_type_args);
                let match_result = self.check_type_match(arg.type_id, param_type);

                match match_result {
                    TypeMatchResult::Exact => score.exact_matches += 1,
                    TypeMatchResult::Subtype => score.subtype_matches += 1,
                    TypeMatchResult::Widening => score.widening_matches += 1,
                    TypeMatchResult::NoMatch => {
                        errors.push(CallError::with_message(
                            CallErrorCode::ArgumentTypeMismatch,
                            arg.span,
                            format!(
                                "Argument of type '{}' is not assignable to parameter of type '{}'.",
                                self.type_name(arg.type_id),
                                self.type_name(param_type)
                            ),
                        ));
                    }
                }
            }
        }

        (score, errors)
    }

    /// Check how well a source type matches a target type
    fn check_type_match(&self, source: TypeId, target: TypeId) -> TypeMatchResult {
        // Exact match
        if source == target {
            return TypeMatchResult::Exact;
        }

        // Any matches anything
        if target == TypeId::ANY || source == TypeId::ANY {
            return TypeMatchResult::Widening;
        }

        // Unknown accepts any
        if target == TypeId::UNKNOWN {
            return TypeMatchResult::Subtype;
        }

        // Never is assignable to anything
        if source == TypeId::NEVER {
            return TypeMatchResult::Subtype;
        }

        // Null/undefined assignability (simplified)
        if source == TypeId::NULL || source == TypeId::UNDEFINED {
            // In strict mode, only assignable to themselves or any/unknown
            return TypeMatchResult::NoMatch;
        }

        // In a real implementation, we'd have full structural type compatibility
        // For now, assume compatible for common cases
        TypeMatchResult::Widening
    }

    /// Substitute type arguments into a type
    fn substitute_type_args(
        &self,
        type_id: TypeId,
        sig: &Signature,
        type_args: &[TypeId],
    ) -> TypeId {
        for (i, tp) in sig.type_parameters.iter().enumerate() {
            if tp.type_id == type_id {
                if let Some(&arg) = type_args.get(i) {
                    if arg.is_some() {
                        return arg;
                    }
                }
            }
        }
        type_id
    }

    /// Check if a type satisfies a constraint
    fn satisfies_constraint(&self, _type_id: TypeId, constraint: TypeId) -> bool {
        if constraint == TypeId::ANY || constraint == TypeId::UNKNOWN {
            return true;
        }
        // Simplified - real impl does full compatibility check
        true
    }

    /// Select the best candidate from multiple matches
    fn select_best_candidate(
        &self,
        mut candidates: Vec<OverloadCandidate>,
    ) -> OverloadResolutionResult {
        // Sort by score (descending)
        candidates.sort_by(|a, b| b.score.cmp(&a.score));

        // Check if there's a clear winner
        if candidates.len() >= 2 && candidates[0].score == candidates[1].score {
            // Ambiguous - multiple signatures match equally well
            if self.context.prefer_specific {
                // Try to find the most specific signature
                if let Some(most_specific) = self.find_most_specific(&candidates) {
                    return OverloadResolutionResult::Success {
                        signature: most_specific.signature,
                        inferred_type_args: most_specific.inferred_type_args.clone(),
                    };
                }
            }

            return OverloadResolutionResult::Ambiguous {
                candidates: candidates.iter().map(|c| c.signature).collect(),
            };
        }

        // Clear winner
        let best = candidates.remove(0);
        OverloadResolutionResult::Success {
            signature: best.signature,
            inferred_type_args: best.inferred_type_args,
        }
    }

    /// Find the most specific signature among equally-scored candidates
    fn find_most_specific<'b>(
        &self,
        candidates: &'b [OverloadCandidate],
    ) -> Option<&'b OverloadCandidate> {
        // A signature is more specific if all its parameter types are subtypes
        // of the corresponding parameter types in another signature

        for (i, candidate) in candidates.iter().enumerate() {
            let sig = match self.signatures.get(candidate.signature) {
                Some(s) => s,
                None => continue,
            };

            let mut is_most_specific = true;
            for (j, other) in candidates.iter().enumerate() {
                if i == j {
                    continue;
                }

                let other_sig = match self.signatures.get(other.signature) {
                    Some(s) => s,
                    None => continue,
                };

                if !self.is_more_specific(sig, other_sig) {
                    is_most_specific = false;
                    break;
                }
            }

            if is_most_specific {
                return Some(candidate);
            }
        }

        None
    }

    /// Check if sig1 is more specific than sig2
    fn is_more_specific(&self, sig1: &Signature, sig2: &Signature) -> bool {
        // More specific means:
        // 1. All parameters of sig1 are subtypes of corresponding params in sig2
        // 2. sig1 has fewer or equal parameters

        if sig1.parameters.len() > sig2.parameters.len() {
            return false;
        }

        for (p1, p2) in sig1.parameters.iter().zip(sig2.parameters.iter()) {
            match self.check_type_match(p1.type_id, p2.type_id) {
                TypeMatchResult::Exact | TypeMatchResult::Subtype => {}
                _ => return false,
            }
        }

        true
    }

    /// Get type name for error messages
    fn type_name(&self, type_id: TypeId) -> &'static str {
        match type_id {
            TypeId::ANY => "any",
            TypeId::UNKNOWN => "unknown",
            TypeId::STRING => "string",
            TypeId::NUMBER => "number",
            TypeId::BOOLEAN => "boolean",
            TypeId::VOID => "void",
            TypeId::UNDEFINED => "undefined",
            TypeId::NULL => "null",
            TypeId::NEVER => "never",
            TypeId::OBJECT => "object",
            TypeId::SYMBOL => "symbol",
            TypeId::BIGINT => "bigint",
            _ => "Type",
        }
    }
}

/// Result of type matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeMatchResult {
    /// Types are identical
    Exact,
    /// Source is a subtype of target
    Subtype,
    /// Source can be widened to target (e.g., literal to primitive)
    Widening,
    /// Types are incompatible
    NoMatch,
}

/// Collect all overload signatures from a function symbol
pub fn collect_overload_signatures(
    signatures: &SignatureStore,
    first_sig: SignatureId,
) -> Vec<SignatureId> {
    let mut result = Vec::new();
    let mut current = first_sig;

    while current.is_some() {
        result.push(current);
        if let Some(sig) = signatures.get(current) {
            current = sig.next_overload;
        } else {
            break;
        }
    }

    result
}

/// Check if overload signatures are properly ordered (more specific first)
pub fn check_overload_order(signatures: &SignatureStore, overloads: &[SignatureId]) -> Vec<usize> {
    let mut out_of_order = Vec::new();

    for i in 1..overloads.len() {
        let prev = match signatures.get(overloads[i - 1]) {
            Some(s) => s,
            None => continue,
        };
        let current = match signatures.get(overloads[i]) {
            Some(s) => s,
            None => continue,
        };

        // Check if current should come before previous (more specific)
        let resolver = OverloadResolver::new(signatures);
        if resolver.is_more_specific(current, prev) {
            out_of_order.push(i);
        }
    }

    out_of_order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::StringId;
    use super::super::signatures::Parameter;

    fn create_test_store() -> SignatureStore {
        SignatureStore::new()
    }

    #[test]
    fn test_single_overload_match() {
        let mut store = create_test_store();

        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
            .with_return_type(TypeId::NUMBER);
        let sig_id = store.alloc_with(sig);

        let resolver = OverloadResolver::new(&store);
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, None);
        match result {
            OverloadResolutionResult::Success { signature, .. } => {
                assert_eq!(signature, sig_id);
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_argument_count_mismatch() {
        let mut store = create_test_store();

        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER),
            ])
            .with_return_type(TypeId::VOID);
        let sig_id = store.alloc_with(sig);

        let resolver = OverloadResolver::new(&store);
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, None);
        match result {
            OverloadResolutionResult::NoMatch { errors } => {
                assert!(!errors.is_empty());
                assert_eq!(errors[0].reason, OverloadMismatchReason::ArgumentCount);
            }
            _ => panic!("Expected no match"),
        }
    }

    #[test]
    fn test_multiple_overloads_select_best() {
        let mut store = create_test_store();

        // (x: string) => string
        let sig1 = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
            .with_return_type(TypeId::STRING);
        let sig1_id = store.alloc_with(sig1);

        // (x: number) => number
        let sig2 = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::NUMBER)])
            .with_return_type(TypeId::NUMBER);
        let sig2_id = store.alloc_with(sig2);

        let resolver = OverloadResolver::new(&store);

        // Call with string - should match first overload
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let result = resolver.resolve(&[sig1_id, sig2_id], &args, None);

        match result {
            OverloadResolutionResult::Success { signature, .. } => {
                assert_eq!(signature, sig1_id);
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_generic_overload() {
        let mut store = create_test_store();

        // <T>(x: T) => T
        let sig = Signature::new(SignatureId::NONE)
            .with_type_parameters(vec![
                TypeParameter::new(StringId::new(1), TypeId::new(100))
            ])
            .with_parameters(vec![Parameter::new(StringId::new(2), TypeId::new(100))])
            .with_return_type(TypeId::new(100));
        let sig_id = store.alloc_with(sig);

        let resolver = OverloadResolver::new(&store);
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, None);

        match result {
            OverloadResolutionResult::Success { inferred_type_args, .. } => {
                assert_eq!(inferred_type_args.len(), 1);
                assert_eq!(inferred_type_args[0], TypeId::STRING);
            }
            _ => panic!("Expected success with inferred type args"),
        }
    }

    #[test]
    fn test_explicit_type_arguments() {
        let mut store = create_test_store();

        // <T>(x: T) => T
        let sig = Signature::new(SignatureId::NONE)
            .with_type_parameters(vec![
                TypeParameter::new(StringId::new(1), TypeId::new(100))
            ])
            .with_parameters(vec![Parameter::new(StringId::new(2), TypeId::new(100))])
            .with_return_type(TypeId::new(100));
        let sig_id = store.alloc_with(sig);

        let resolver = OverloadResolver::new(&store);
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let type_args = vec![TypeArgument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, Some(&type_args));

        match result {
            OverloadResolutionResult::Success { inferred_type_args, .. } => {
                assert_eq!(inferred_type_args.len(), 1);
                assert_eq!(inferred_type_args[0], TypeId::STRING);
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_collect_overload_signatures() {
        let mut store = create_test_store();

        // Create a chain of overloads
        let sig3 = Signature::new(SignatureId::NONE).with_return_type(TypeId::BOOLEAN);
        let sig3_id = store.alloc_with(sig3);

        let sig2 = Signature::new(SignatureId::NONE)
            .with_return_type(TypeId::NUMBER);
        let sig2_id = store.alloc_with(sig2);

        let sig1 = Signature::new(SignatureId::NONE)
            .with_return_type(TypeId::STRING);
        let sig1_id = store.alloc_with(sig1);

        // Link them
        store.get_mut(sig1_id).unwrap().next_overload = sig2_id;
        store.get_mut(sig2_id).unwrap().next_overload = sig3_id;

        let overloads = collect_overload_signatures(&store, sig1_id);
        assert_eq!(overloads.len(), 3);
        assert_eq!(overloads[0], sig1_id);
        assert_eq!(overloads[1], sig2_id);
        assert_eq!(overloads[2], sig3_id);
    }

    #[test]
    fn test_type_args_on_non_generic() {
        let mut store = create_test_store();

        // Non-generic: (x: string) => void
        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
            .with_return_type(TypeId::VOID);
        let sig_id = store.alloc_with(sig);

        let resolver = OverloadResolver::new(&store);
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let type_args = vec![TypeArgument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, Some(&type_args));

        match result {
            OverloadResolutionResult::NoMatch { errors } => {
                assert_eq!(errors[0].reason, OverloadMismatchReason::TypeArgumentCount);
            }
            _ => panic!("Expected no match due to type args on non-generic"),
        }
    }
}
