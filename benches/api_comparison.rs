//! Compares the overhead of the different entry points the crate exposes:
//! plain functions, struct constructors, `ParaMap`, and builders.

use criterion::{Criterion, criterion_group, criterion_main};
use rfinancial::*;
use std::hint::black_box;

fn fv_para_map() -> ParaMap {
    let mut map = ParaMap::new();
    map.insert("rate".into(), ParaType::F64(0.075));
    map.insert("nper".into(), ParaType::U32(20));
    map.insert("pmt".into(), ParaType::F64(-2000.0));
    map.insert("pv".into(), ParaType::F64(0.0));
    map.insert("when".into(), ParaType::When(WhenType::End));
    map
}

fn rate_para_map() -> ParaMap {
    let mut map = ParaMap::new();
    map.insert("nper".into(), ParaType::U32(10));
    map.insert("pmt".into(), ParaType::F64(0.0));
    map.insert("pv".into(), ParaType::F64(-3500.0));
    map.insert("fv".into(), ParaType::F64(10000.0));
    map.insert("when".into(), ParaType::When(WhenType::End));
    map.insert("guess".into(), ParaType::F64(0.1));
    map.insert("tol".into(), ParaType::F64(1e-6));
    map.insert("maxiter".into(), ParaType::U32(100));
    map
}

fn bench_fv_apis(c: &mut Criterion) {
    let mut group = c.benchmark_group("fv_api");

    group.bench_function("function", |b| {
        b.iter(|| {
            fv(
                black_box(0.075),
                black_box(20),
                black_box(-2000.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.bench_function("from_tuple", |b| {
        b.iter(|| {
            FutureValue::from_tuple(black_box((0.075, 20, -2000.0, 0.0, WhenType::End)))
                .and_then(|v| v.get())
        })
    });

    // Reuse the struct to isolate `get()` from construction cost.
    let prebuilt = FutureValue::from_tuple((0.075, 20, -2000.0, 0.0, WhenType::End)).unwrap();
    group.bench_function("prebuilt_get", |b| b.iter(|| black_box(&prebuilt).get()));

    group.bench_function("from_map_with_build", |b| {
        b.iter(|| fv_from_map(black_box(fv_para_map())))
    });

    group.finish();
}

fn bench_rate_apis(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_api");

    group.bench_function("function", |b| {
        b.iter(|| {
            rate(
                black_box(10),
                black_box(0.0),
                black_box(-3500.0),
                black_box(10000.0),
                black_box(WhenType::End),
                black_box(0.1),
                black_box(1e-6),
                black_box(100),
            )
        })
    });

    group.bench_function("builder", |b| {
        b.iter(|| {
            RateBuilder::new()
                .nper(black_box(10))
                .pmt(black_box(0.0))
                .pv(black_box(-3500.0))
                .fv(black_box(10000.0))
                .when(WhenType::End)
                .guess(black_box(0.1))
                .tol(black_box(1e-6))
                .maxiter(black_box(100))
                .build()
                .and_then(|r| r.get())
        })
    });

    group.bench_function("from_map_with_build", |b| {
        b.iter(|| rate_from_map(black_box(rate_para_map())))
    });

    group.finish();
}

criterion_group!(benches, bench_fv_apis, bench_rate_apis);
criterion_main!(benches);
