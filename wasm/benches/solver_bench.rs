//! Solver microbenchmarks (subtype, evaluate, infer).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasm::solver::{
    evaluate_type,
    infer_generic_function,
    is_subtype_of,
    CompatChecker,
    ConditionalType,
    FunctionShape,
    ParamInfo,
    PropertyInfo,
    TypeId,
    TypeInterner,
    TypeKey,
    TypeParamInfo,
};

fn build_subtype_fixtures(interner: &TypeInterner) -> (TypeId, TypeId, TypeId) {
    let name_x = interner.intern_string("x");
    let name_y = interner.intern_string("y");
    let name_z = interner.intern_string("z");

    let source = interner.object(vec![
        PropertyInfo {
            name: name_x,
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: name_y,
            type_id: TypeId::STRING,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let mismatch = interner.object(vec![PropertyInfo {
        name: name_x,
        type_id: TypeId::STRING,
        optional: false,
        readonly: false,
        is_method: false,
    }]);

    let extra_required = interner.object(vec![
        PropertyInfo {
            name: name_x,
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: name_z,
            type_id: TypeId::BOOLEAN,
            optional: false,
            readonly: false,
            is_method: false,
        },
    ]);

    let match_type = interner.object(vec![
        PropertyInfo {
            name: name_x,
            type_id: TypeId::NUMBER,
            optional: false,
            readonly: false,
            is_method: false,
        },
        PropertyInfo {
            name: name_y,
            type_id: TypeId::STRING,
            optional: true,
            readonly: false,
            is_method: false,
        },
    ]);

    let union_match = interner.union(vec![mismatch, extra_required, match_type]);
    let union_miss = interner.union(vec![mismatch, extra_required]);

    (source, union_match, union_miss)
}

fn build_conditional_type(interner: &TypeInterner) -> TypeId {
    let check = interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    let conditional = ConditionalType {
        check_type: check,
        extends_type: TypeId::NUMBER,
        true_type: TypeId::STRING,
        false_type: TypeId::BOOLEAN,
        is_distributive: true,
    };
    interner.intern(TypeKey::Conditional(conditional))
}

fn build_infer_fixture(interner: &TypeInterner) -> (FunctionShape, [TypeId; 1]) {
    let t_param = TypeParamInfo {
        name: interner.intern_string("T"),
        constraint: None,
        default: None,
    };
    let u_param = TypeParamInfo {
        name: interner.intern_string("U"),
        constraint: None,
        default: Some(TypeId::STRING),
    };
    let t_type = interner.intern(TypeKey::TypeParameter(t_param.clone()));
    let u_type = interner.intern(TypeKey::TypeParameter(u_param.clone()));
    let array_t = interner.array(t_type);

    let func = FunctionShape {
        type_params: vec![t_param, u_param],
        params: vec![ParamInfo {
            name: Some(interner.intern_string("items")),
            type_id: array_t,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: u_type,
        type_predicate: None,
        is_constructor: false,
    };

    let arg = interner.array(TypeId::NUMBER);
    (func, [arg])
}

fn bench_subtype(c: &mut Criterion) {
    let interner = TypeInterner::new();
    let (source, union_match, union_miss) = build_subtype_fixtures(&interner);

    c.bench_function("subtype_object_union_match", |b| {
        b.iter(|| black_box(is_subtype_of(&interner, source, union_match)))
    });

    c.bench_function("subtype_object_union_miss", |b| {
        b.iter(|| black_box(is_subtype_of(&interner, source, union_miss)))
    });
}

fn bench_evaluate(c: &mut Criterion) {
    let interner = TypeInterner::new();
    let conditional = build_conditional_type(&interner);

    c.bench_function("evaluate_conditional_distributive", |b| {
        b.iter(|| black_box(evaluate_type(&interner, conditional)))
    });
}

fn bench_infer(c: &mut Criterion) {
    let interner = TypeInterner::new();
    let (func, arg_types) = build_infer_fixture(&interner);

    c.bench_function("infer_generic_default_param", |b| {
        b.iter(|| {
            let mut checker = CompatChecker::new(&interner);
            let result = infer_generic_function(&interner, &mut checker, &func, &arg_types);
            black_box(result)
        })
    });
}

criterion_group!(solver_benches, bench_subtype, bench_evaluate, bench_infer);
criterion_main!(solver_benches);
