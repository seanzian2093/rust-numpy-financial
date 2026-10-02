use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rfinancial::*;
use std::hint::black_box;

/// Deterministic cash flow series so benchmarks are reproducible across runs.
fn cash_flows(len: usize) -> Vec<f64> {
    let mut values = Vec::with_capacity(len);
    values.push(-10_000.0);
    for i in 1..len {
        values.push(500.0 + (i as f64 % 7.0) * 125.0);
    }
    values
}

/// Level annuity whose IRR is exactly `rate`, used to keep the `irr` benchmark on a
/// well-conditioned input with a known answer.
fn annuity(len: usize, rate: f64) -> Vec<f64> {
    let n = (len - 1) as f64;
    let principal = 10_000.0;
    let payment = principal * rate / (1.0 - (1.0 + rate).powf(-n));
    let mut values = vec![-principal];
    values.extend(std::iter::repeat(payment).take(len - 1));
    values
}

fn bench_closed_form(c: &mut Criterion) {
    let mut group = c.benchmark_group("closed_form");

    group.bench_function("fv", |b| {
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

    group.bench_function("pv", |b| {
        b.iter(|| {
            pv(
                black_box(0.05),
                black_box(20),
                black_box(-100.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.bench_function("pmt", |b| {
        b.iter(|| {
            pmt(
                black_box(0.08 / 12.0),
                black_box(60),
                black_box(15000.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.bench_function("nper", |b| {
        b.iter(|| {
            nper(
                black_box(0.07 / 12.0),
                black_box(-150.0),
                black_box(8000.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.bench_function("ipmt", |b| {
        b.iter(|| {
            ipmt(
                black_box(0.0824 / 12.0),
                black_box(1),
                black_box(24),
                black_box(2500.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.bench_function("ppmt", |b| {
        b.iter(|| {
            ppmt(
                black_box(0.0824 / 12.0),
                black_box(1),
                black_box(24),
                black_box(2500.0),
                black_box(0.0),
                black_box(WhenType::End),
            )
        })
    });

    group.finish();
}

fn bench_cash_flow_series(c: &mut Criterion) {
    let mut group = c.benchmark_group("cash_flow_series");

    for len in [8usize, 32, 128, 512, 2048] {
        let values = cash_flows(len);
        group.throughput(Throughput::Elements(len as u64));

        group.bench_with_input(BenchmarkId::new("npv", len), &values, |b, values| {
            b.iter(|| npv(black_box(values), black_box(0.08)))
        });

        group.bench_with_input(BenchmarkId::new("mirr", len), &values, |b, values| {
            b.iter(|| mirr(black_box(values), black_box(0.08), black_box(0.055)))
        });
    }

    group.finish();
}

/// Timed on a level annuity with a known IRR so the benchmark cannot silently measure a
/// wrong-answer path if the solver regresses.
fn bench_irr(c: &mut Criterion) {
    let mut group = c.benchmark_group("irr");

    for len in [8usize, 32, 128, 512, 2048] {
        let values = annuity(len, 0.08);

        let got = irr(&values)
            .expect("irr returned an error")
            .expect("irr found no root");
        assert!(
            (got - 0.08).abs() < 1e-6,
            "irr(len={len}) = {got}, expected 0.08"
        );

        group.throughput(Throughput::Elements(len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(len), &values, |b, values| {
            b.iter(|| irr(black_box(values)))
        });
    }

    group.finish();
}

/// `rate` is Newton-iterative, so cost is driven by convergence, not input size.
fn bench_rate_convergence(c: &mut Criterion) {
    let mut group = c.benchmark_group("rate_convergence");

    for (label, guess, tol) in [
        ("near_guess_1e-6", 0.1, 1e-6),
        ("far_guess_1e-6", 0.9, 1e-6),
        ("near_guess_1e-12", 0.1, 1e-12),
    ] {
        group.bench_function(label, |b| {
            b.iter(|| {
                rate(
                    black_box(10),
                    black_box(0.0),
                    black_box(-3500.0),
                    black_box(10000.0),
                    black_box(WhenType::End),
                    black_box(guess),
                    black_box(tol),
                    black_box(100),
                )
            })
        });
    }

    group.finish();
}

fn bench_nper_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("period_scaling");

    for nper in [12u32, 120, 600, 3600] {
        group.bench_with_input(BenchmarkId::new("fv", nper), &nper, |b, &nper| {
            b.iter(|| {
                fv(
                    black_box(0.05 / 12.0),
                    black_box(nper),
                    black_box(-250.0),
                    black_box(0.0),
                    black_box(WhenType::End),
                )
            })
        });

        group.bench_with_input(BenchmarkId::new("ipmt", nper), &nper, |b, &nper| {
            b.iter(|| {
                ipmt(
                    black_box(0.05 / 12.0),
                    black_box(nper / 2),
                    black_box(nper),
                    black_box(25000.0),
                    black_box(0.0),
                    black_box(WhenType::End),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_closed_form,
    bench_cash_flow_series,
    bench_irr,
    bench_rate_convergence,
    bench_nper_scaling
);
criterion_main!(benches);
