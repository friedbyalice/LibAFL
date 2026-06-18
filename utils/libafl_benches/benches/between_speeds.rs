//! Compare `Rand::between` variants.

use core::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use libafl_bolts::rands::{Rand, StdRand};

const SAFE_CASES: &[(&str, usize, usize)] = &[
    ("small", 0, 100),
    ("medium", 0, 100_000),
    ("offset", 10, 100_000),
];

const FULL_RANGE: (usize, usize) = (0, usize::MAX);

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("between");

    for &(label, lower, upper) in SAFE_CASES {
        let mut baseline = StdRand::with_seed(1337);
        group.bench_with_input(BenchmarkId::new("between", label), &(lower, upper), |b, bounds| {
            b.iter(|| baseline.between(black_box(bounds.0), black_box(bounds.1)))
        });

        let mut branch = StdRand::with_seed(1337);
        group.bench_with_input(
            BenchmarkId::new("between_branch", label),
            &(lower, upper),
            |b, bounds| {
                b.iter(|| branch.between_branch(black_box(bounds.0), black_box(bounds.1)))
            },
        );

        let mut wide = StdRand::with_seed(1337);
        group.bench_with_input(
            BenchmarkId::new("between_wide", label),
            &(lower, upper),
            |b, bounds| b.iter(|| wide.between_wide(black_box(bounds.0), black_box(bounds.1))),
        );
    }

    let mut branch = StdRand::with_seed(1337);
    group.bench_with_input(
        BenchmarkId::new("between_branch", "full"),
        &FULL_RANGE,
        |b, bounds| b.iter(|| branch.between_branch(black_box(bounds.0), black_box(bounds.1))),
    );

    let mut baseline = StdRand::with_seed(1337);
    group.bench_with_input(BenchmarkId::new("between", "full"), &FULL_RANGE, |b, bounds| {
        b.iter(|| baseline.between(black_box(bounds.0), black_box(bounds.1)))
    });

    let mut wide = StdRand::with_seed(1337);
    group.bench_with_input(BenchmarkId::new("between_wide", "full"), &FULL_RANGE, |b, bounds| {
        b.iter(|| wide.between_wide(black_box(bounds.0), black_box(bounds.1)))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
