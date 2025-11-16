# cubemoma

[![Crates.io](https://img.shields.io/crates/v/cubemoma.svg)](https://crates.io/crates/cubemoma)
[![Docs.rs](https://docs.rs/cubemoma/badge.svg)](https://docs.rs/cubemoma)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**cubemoma** is a Rust crate providing an efficient CPU-based implementation of Multi-word Modular Arithmetic (MoMA) for cryptographic kernels, inspired by the paper "Code Generation for Cryptographic Kernels using Multi-word Modular Arithmetic on GPU" by Naifeng Zhang and Franz Franchetti (arXiv:2501.07535v1, published January 13, 2025). This crate focuses on the CPU version, implementing finite field arithmetic over large primes (e.g., 448-bit) without relying on a code generator. It supports operations essential for Fully Homomorphic Encryption (FHE) and Zero-Knowledge Proofs (ZKPs), such as modular addition, multiplication, inversion, square roots, and Number Theoretic Transforms (NTT).

## Description

This project adapts the MoMA formalization from the paper, which decomposes large integer arithmetic into machine-word operations (u64 limbs). Unlike the original GPU-focused work, cubemoma provides a static, const-generic Rust implementation for CPU environments. Key contributions from the paper include:

- **MoMA Rewrite System**: Breaks down large bit-width integers (e.g., 256-768 bits for ZKPs, up to 1000+ bits for FHE) into recursive word-level operations compatible with compilers.
- **Cryptographic Kernels**: Supports BLAS-like operations and NTT, with performance claims of orders-of-magnitude improvements over multi-precision libraries like GMP, and near-ASIC efficiency on GPUs (e.g., 14x faster than NVIDIA H100 for 256-bit NTT).

cubemoma translates these ideas to CPU, using Barrett reduction for modular ops and optimizations like pre-computed twiddles for NTT. It does not include GPU/CubeCL support yet.

For the full paper details, see the [arXiv link](https://arxiv.org/abs/2501.07535).

## Features

- **Finite Field Arithmetic (`BigField<N>`)**: Modular addition, subtraction, multiplication (schoolbook + Barrett), inversion (Fermat), and square root (Tonelli-Shanks with Legendre symbol check).
- **Quadratic Extension (`FpComplex<N>`)**: Operations like mul_mod, add_mod, sub_mod, square_mod, norm_mod, inv_mod.
- **NTT Optimization**: In-place Cooley-Tukey NTT with pre-computed twiddles for efficiency.
- **Endianness Support**: from_be/le, to_be/le for byte conversions.
- **Const Generics**: Parameterized by limb count `N` (e.g., N=7 for 448-bit fields).
- **Exceptions**: inv_mod and sqrt_mod return `Option` for non-invertible/non-residue cases.
- **WASM Support**: Basic bindings for browser-based benchmarks (CPU-only).

## Requirements

- Rust edition: 2024
- Rust toolchain: nightly-2025-10-01 or later
- Dependencies: rand (for random field elements), wasm-bindgen (for WASM builds)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cubemoma = "0.0.1"  # Replace with actual version
rand = "0.9"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

Build with `cargo build --release`. For WASM: `wasm-pack build --target web`.

## Usage

### Basic Field Operations

```rust
use cubemoma::{BigField, MBITS};

const N: usize = 7;  // 448-bit
let p_limbs: [u64; N] = [0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFEFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF];
let modulus = BigField::<N>::new(p_limbs);
let mu = BigField::<N>::precompute_mu(&modulus);

let a = BigField::<N>::random(&mut rand::thread_rng());
let b = BigField::<N>::random(&mut rand::thread_rng());

let sum = a.add_mod(&b, &modulus);
let product = a.mul_mod(&b, &modulus, mu);
if let Some(inv_a) = a.inv_mod(&modulus, mu) {
    let identity = a.mul_mod(&inv_a, &modulus, mu);  // Should be 1
}
if let Some(sqrt_a) = a.sqrt_mod(&modulus, mu) {
    let sq = sqrt_a.square_mod(&modulus, mu);  // Should equal a
}
```

### NTT Example

```rust
let omega = BigField::<N>::from_limb(7u64);  // Primitive root
let precompute = NttPrecompute::<N>::new(&omega, 256, &modulus, mu);
let mut input: Vec<BigField<N>> = (0..256).map(|_| BigField::<N>::random(&mut rand::thread_rng())).collect();
ntt(&mut input, &precompute, &modulus, mu);
```

### Benchmarks

Run `cargo bench` for CPU benchmarks. Example results (AMD Ryzen 7 5800H, Rust nightly-2025-10-01):

- fp_add_mod: ~8.13 ns
- fp_mul_mod: ~251.38 ns
- fp_square_mod: ~252.90 ns
- fp_repeated_mul_mod (1000): ~253.15 µs
- fp_ntt (n=256): ~277.69 µs
- fp_inv_mod (Fermat): ~223.33 µs

WASM results (browser, post-optimization): mul_mod ~4 µs (from repeated), NTT ~4 ms – 4-5x faster than initial.

## Contributing

Contributions welcome! Fork the repo, create a branch, and submit a PR. Focus on CPU optimizations (e.g., assembly intrinsics). Issues for bugs or features.

## License

2-Clause BSD License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Based on the paper by Naifeng Zhang and Franz Franchetti. Special thanks to xAI for assistance in development.