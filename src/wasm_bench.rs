use wasm_bindgen::prelude::*;

use crate::BigField as Fp;

#[wasm_bindgen]
pub struct FpBench {
    modulus: Fp<7>,
    mu: [u64; 8],  // N+1
    omega: Fp<7>,
}

#[wasm_bindgen]
impl FpBench {
    #[wasm_bindgen(constructor)]
    pub fn new() -> FpBench {
        let p_limbs = [
            0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
        ];
        let modulus = Fp::new(p_limbs);
        let mu = Fp::precompute_mu(&modulus);
        let omega = Fp::from_limb(7u64);  // 주어진 ω=7

        FpBench { modulus, mu, omega }
    }

    // 벤치용: single mul_mod
    pub fn bench_mul_mod(&self, a_limbs: &[u64], b_limbs: &[u64]) -> Vec<u64> {
        let a = Fp::new(a_limbs.try_into().unwrap());
        let b = Fp::new(b_limbs.try_into().unwrap());
        let res = a.mul_mod(&b, &self.modulus, Some(self.mu));
        res.limbs.to_vec()
    }

    // 벤치용: repeated mul_mod (1000회)
    pub fn bench_repeated_mul_mod(&self, a_limbs: &[u64], b_limbs: &[u64], iterations: usize) -> Vec<u64> {
        let mut res = Fp::new(a_limbs.try_into().unwrap());
        let b = Fp::new(b_limbs.try_into().unwrap());
        for _ in 0..iterations {
            res = res.mul_mod(&b, &self.modulus, Some(self.mu));
        }
        res.limbs.to_vec()
    }

    // 벤치용: NTT
    pub fn bench_ntt(&self, input_limbs: &[u64]) -> Vec<u64> {
        let mut input: Vec<Fp<7>> = input_limbs.chunks(7).map(|chunk| Fp::new(chunk.try_into().unwrap())).collect();
        Fp::ntt(&mut input, &self.omega, &self.modulus, &self.mu);  // 기존 ntt 함수 호출
        input.iter().flat_map(|fp| fp.limbs.to_vec()).collect()
    }
}

// JS에서 랜덤 데이터 생성 헬퍼 (WASM 측에서 rand 피함, JS에서 제공)
#[wasm_bindgen]
pub fn generate_random_limbs(seed: u32, count: usize) -> Vec<u64> {
    // 간단한 pseudo-random (WASM에서 rand crate 피함)
    let mut res = Vec::with_capacity(count);
    let mut x = seed as u64;
    for _ in 0..count {
        x = x.wrapping_mul(0xda942042e4dd58b5u64);
        res.push(x);
    }
    res
}