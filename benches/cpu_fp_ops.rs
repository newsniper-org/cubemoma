use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use cubemoma::{BigField};

const N: usize = 7;
type Fp = BigField<N>;

// p = 2^448 - 2^224 - 1 = "0x FFFFFFFFFFFFFFFF FFFFFFFFFFFFFFFF FFFFFFFFFFFFFFFF FFFFFFFEFFFFFFFF FFFFFFFFFFFFFFFF FFFFFFFFFFFFFFFF FFFFFFFFFFFFFFFF"

fn bench_cpu_fp_ops(c: &mut Criterion) {
    let modulus = Fp::new([
        0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
    ]);
    let mu = Fp::precompute_mu(&modulus);
    let mut rng = rand::rng();
    let a = Fp::random(&mut rng);
    let b = Fp::random(&mut rng);

    c.bench_function("fp_add_mod", |bencher| {
        bencher.iter(|| black_box(a.add_mod(&b, &modulus)));
    });

    c.bench_function("fp_mul_mod", |bencher| {
        bencher.iter(|| black_box(a.mul_mod(&b, &modulus, Some(mu))));
    });

    c.bench_function("fp_square_mod", |bencher| {
        bencher.iter(|| black_box(a.square_mod(&modulus, Some(mu))));
    });

    c.bench_function("fp_repeated_mul_mod", |bencher| {
        bencher.iter(|| {
            let mut res = a;
            for _ in 0..1000 {
                res = black_box(res.mul_mod(&b, &modulus, Some(mu)));
            }
            res
        });
    });

    // 긴 연산 1: NTT (n=256, 논문 Figure 1 참고)
    let n = 256; // 2^8
    let mut rng = rand::rng();
    let mut input: Vec<Fp> = (0..n).map(|_| Fp::random(&mut rng)).collect();
    let omega = Fp::from_limb(7u64);
    c.bench_function("fp_ntt", |bencher| {
        bencher.iter(|| {
            black_box(Fp::ntt(&mut input, &omega, &modulus, &mu));
        });
    });

    // 긴 연산 2: 모듈러 곱셈 역원
    c.bench_function("fp_inv_mod", |bencher| {
        bencher.iter(|| black_box(Fp::from_limb(7u64).inv_mod(&modulus, Some(mu))));
    });
}

criterion_group!(benches, bench_cpu_fp_ops);
criterion_main!(benches);