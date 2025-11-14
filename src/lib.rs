#![feature(const_trait_impl)]
#![feature(stmt_expr_attributes)]
#![feature(bigint_helper_methods)]
#![feature(slice_as_array)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(generic_const_exprs)]
#![feature(const_ops)]

use rand::{CryptoRng, Rng};


pub const MBITS: usize = 64;

pub type Limb = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct BigField<const N: usize> {
    pub(crate) limbs: [Limb; N]
}


pub const trait PairSized<T1: Sized, T2: Sized> : Sized {
    const IS_VALID: bool = size_of::<Self>() == size_of::<T1>() + size_of::<T2>();

    fn split_pair(self) -> (T1, T2);
    
    fn merge_pair(a: T1, b: T2) -> Self;
}

impl const PairSized<u64, u64> for u128 {
    #[inline(always)]
    fn split_pair(self) -> (u64, u64) {
        (self as u64, (self >> 64) as u64)
    }

    #[inline(always)]
    fn merge_pair(a: u64, b: u64) -> u128 {
        (b as u128) << 64 | (a as u128)
    }
}

impl const PairSized<u64, i64> for i128 {
    #[inline(always)]
    fn split_pair(self) -> (u64, i64) {
        (self as u64, (self >> 64) as i64)
    }

    #[inline(always)]
    fn merge_pair(a: u64, b: i64) -> i128 {
        (b as i128) << 64 | (a as i128)
    }
}

impl<T: Sized> const PairSized<T, T> for (T, T) {
    #[inline(always)]
    fn split_pair(self) -> (T, T) {
        self
    }

    #[inline(always)]
    fn merge_pair(a: T, b: T) -> Self {
        (a, b)
    }
}

impl<const N: usize> Default for BigField<N> {
    fn default() -> Self {
        Self { limbs: [0u64; N] }
    }
}

#[inline(always)]
const fn parse_be_bytes<const N: usize>(bytes: [u8; 8 * N]) -> [u64; N] {
    const S: usize = size_of::<u64>();
    let mut result = [0u64; N];
    let mut idx = 0usize;
    while idx < N {
        let chunk: [u8; S] = *bytes[idx*S..(idx+1)*S].as_array().unwrap();
        result[N-1 - idx] = u64::from_le_bytes(chunk);
        idx += 1;
    }
    result
}

#[inline(always)]
const fn parse_le_bytes<const N: usize>(bytes: [u8; 8 * N]) -> [u64; N] {
    const S: usize = size_of::<u64>();
    let mut result = [0u64; N];
    let mut idx = 0usize;
    while idx < N {
        let chunk: [u8; S] = *bytes[idx*S..(idx+1)*S].as_array().unwrap();
        result[idx] = u64::from_le_bytes(chunk);
        idx += 1;
    }
    result
}


impl<const N: usize> BigField<N> {
    #[inline(always)]
    pub const fn new(limbs: [Limb; N]) -> Self {
        Self {
            limbs
        }
    }

    #[inline(always)]
    pub fn random<R: CryptoRng + ?Sized>(rng: &mut R) -> Self {
        let mut result = Self::default();
        for limb in result.limbs.iter_mut() {
            *limb = rng.random::<u64>();
        }
        result
    }

    #[inline(always)]
    pub const fn from_be_bytes(bytes: [u8; 8 * N]) -> Self {
        let limbs = parse_be_bytes::<N>(bytes);
        Self::new(limbs)
    }

    #[inline(always)]
    pub const fn from_le_bytes(bytes: [u8; 8 * N]) -> Self {
        let limbs = parse_le_bytes(bytes);
        Self::new(limbs)
    }

    #[inline(always)]
    pub const fn from_limb(limb: Limb) -> Self {
        let mut limbs: [Limb; N] = [0; N];
        limbs[0] = limb;
        Self {
            limbs
        }
    }

    #[inline(always)]
    pub const fn from_bilimb(bilimb: u128) -> Self {
        let mut limbs: [Limb; N] = [0; N];
        (limbs[0], limbs[1]) = bilimb.split_pair();
        Self {
            limbs
        }
    }

    pub const fn from_bilimbs<const B: usize>(bilimbs: [u128; B]) -> Self
    where u128: const PairSized<u64, u64> {
        let mut limbs: [Limb; N] = [0; N];
        let mut i = 0usize;
        while i < B {
            (limbs[i], limbs[i+1]) = bilimbs[i].split_pair();
            i += 2;
        }
        Self {
            limbs
        }
    }

    // Helper: Compare multi-limb
    fn cmp_multi(a: &[Limb; N], b: &[Limb; N]) -> std::cmp::Ordering {
        for i in (0..N).rev() {
            if a[i] > b[i] { return std::cmp::Ordering::Greater; }
            if a[i] < b[i] { return std::cmp::Ordering::Less; }
        }
        std::cmp::Ordering::Equal
    }

    pub fn add_mod(&self, other: &Self, modulus: &Self) -> Self {
        let mut result = [0; N];
        let mut carry: Limb = 0;
        for i in 0..N {
            let temp = self.limbs[i] as u128 + other.limbs[i] as u128 + carry as u128;
            (result[i], carry) = temp.split_pair();
        }
        let mut high = carry;
        while high > 0 || Self::cmp_multi(&result, &modulus.limbs) != std::cmp::Ordering::Less {
            Self::sub_multi(&mut result, &modulus.limbs, &mut high);
        }
        Self::new(result)
    }

    fn add_multi(result: &mut [Limb; N], b: &[Limb; N], high: &mut Limb) {
        let mut carry = false;
        for i in 0..N {
            (result[i], carry) = result[i].carrying_add(b[i], carry)
        }
        if carry {
            *high += 1;
        }
    }

    fn mod_multi(x: &mut [Limb; N], modulus: &[Limb; N], high: &mut Limb) {
        loop {
            if *high > 0 {
                Self::sub_multi(x, modulus, high);
            } else {
                match Self::cmp_multi(x, modulus) {
                    std::cmp::Ordering::Equal => {
                        *x = [0; N];
                        break;
                    },
                    std::cmp::Ordering::Greater => {
                        Self::sub_multi(x, modulus, high);
                    },
                    std::cmp::Ordering::Less => {
                        break;
                    }
                }
            }
        }
    }

    pub fn sub_mod(&self, other: &Self, modulus: &Self) -> Self {
        let mut result = self.limbs.clone();
        let mut high: Limb = 0;
        while high == 0 && Self::cmp_multi(&result, &other.limbs) == std::cmp::Ordering::Less {
            Self::add_multi(&mut result, &modulus.limbs, &mut high);
        }
        Self::sub_multi(&mut result, &other.limbs, &mut high);
        Self::mod_multi(&mut result, &modulus.limbs, &mut high);
        Self::new(result)
    }

    fn sub_multi(result: &mut [Limb; N], b: &[Limb; N], high: &mut Limb) {
        let mut borrow: i128 = 0;
        for i in 0..N {
            let temp = result[i] as i128 - b[i] as i128 - borrow;
            result[i] = temp as Limb;
            borrow = if temp < 0 { 1 } else { 0 };
        }
        if borrow > 0 {
            *high -= 1;
        }
    }

    pub fn mul_mod(&self, other: &Self, modulus: &Self, pre_mu: Option<[u64; N + 1]>) -> Self {
        let mu = pre_mu.unwrap_or(Self::precompute_mu(modulus));

        // Schoolbook mul (논문 Equation 8 기반)
        let mut product: Vec<Limb> = vec![0; 2 * N];
        for i in 0..N {
            let mut carry = 0u128;
            for j in 0..N {
                carry = carry + self.limbs[i] as u128 * other.limbs[j] as u128 + product[i + j] as u128;
                product[i + j] = carry as Limb;
                carry >>= 64;
            }
            let mut k = i + N;
            while carry > 0 {
                if k >= product.len() {
                    product.push(0);
                }
                carry += product[k] as u128;
                product[k] = carry as Limb;
                carry >>= 64;
                k += 1;
            }
        }
        
        // Barrett reduction
        let full_mul = Self::mul_wide(&product, &mu);
        let shift = 2 * N;
        let mut q = vec![0; N + 1];
        for i in 0..(full_mul.len().saturating_sub(shift)).min(N + 1) {
            q[i] = full_mul[shift + i];
        }

        let qm = Self::mul_wide(&q, &modulus.limbs.to_vec());
        let mut r = product;
        let mut borrow = 0i128;
        for i in 0..r.len() {
            let sub = if i < qm.len() { qm[i] as i128 } else { 0 };
            borrow = r[i] as i128 - sub - borrow;
            r[i] = borrow as Limb;
            borrow = if borrow < 0 { 1 } else { 0 };
        }
        if borrow != 0 {
            // Handle remaining borrow if necessary
            while borrow != 0 && r.len() > 0 {
                let last_idx = r.len() - 1;
                borrow = r[last_idx] as i128 - borrow;
                r[last_idx] = borrow as Limb;
                borrow = if borrow < 0 { 1 } else { 0 };
                if borrow != 0 {
                    r.push(0);
                }
            }
        }

        let mut result = [0; N];
        let res_len = std::cmp::min(r.len(), N);
        for i in 0..res_len {
            result[i] = r[i];
        }

        // Conditional sub (at most twice for safety)
        if Self::cmp_multi(&result, &modulus.limbs) != std::cmp::Ordering::Less {
            let mut high = 0;
            Self::sub_multi(&mut result, &modulus.limbs, &mut high);
        }
        if Self::cmp_multi(&result, &modulus.limbs) != std::cmp::Ordering::Less {
            let mut high = 0;
            Self::sub_multi(&mut result, &modulus.limbs, &mut high);
        }

        Self::new(result)
    }

    fn mul_wide(a: &[Limb], b: &[Limb]) -> Vec<Limb> {
        let n = a.len();
        let m = b.len();
        let mut res = vec![0; n + m];
        for i in 0..n {
            let mut carry = 0u128;
            for j in 0..m {
                carry = carry + a[i] as u128 * b[j] as u128 + res[i + j] as u128;
                res[i + j] = carry as Limb;
                carry >>= 64;
            }
            let mut k = i + m;
            while carry > 0 {
                carry = carry + res[k] as u128;
                res[k] = carry as Limb;
                carry >>= 64;
                k += 1;
            }
        }
        res
    }

    fn cmp_wide(a: &[Limb], b: &[Limb]) -> std::cmp::Ordering {
        match a.len().cmp(&b.len()) {
            std::cmp::Ordering::Greater => {
                if let None = a[b.len()..].iter().find(|&limb| *limb != 0) {
                    for i in (0..b.len()).rev() {
                        if a[i] > b[i] { return std::cmp::Ordering::Greater; }
                        if a[i] < b[i] { return std::cmp::Ordering::Less; }
                    }
                    return std::cmp::Ordering::Equal
                } else {
                    return std::cmp::Ordering::Greater
                }
            },
            std::cmp::Ordering::Less => {
                if let None = b[a.len()..].iter().find(|&limb| *limb != 0) {
                    for i in (0..a.len()).rev() {
                        if a[i] > b[i] { return std::cmp::Ordering::Greater; }
                        if a[i] < b[i] { return std::cmp::Ordering::Less; }
                    }
                    return std::cmp::Ordering::Equal
                } else {
                    return std::cmp::Ordering::Less
                }
            },
            std::cmp::Ordering::Equal => {
                for i in (0..a.len()).rev() {
                    if a[i] > b[i] { return std::cmp::Ordering::Greater; }
                    if a[i] < b[i] { return std::cmp::Ordering::Less; }
                }
                std::cmp::Ordering::Equal
            }
        }
    }

    pub fn precompute_mu(modulus: &Self) -> [Limb; N + 1] {
        // 2^{128·N} = 1 << (64 * 2 * N)
        let mut dividend = vec![0u64; 2 * N];
        dividend.push(1);                     // MSB = 1

        let mut quot = [0u64; N + 1];         // μ (N+1 limbs)
        let mut rem  = [0u64; N];              // remainder (N limbs)

        // dividend.len() == 2·N + 1
        for i in (0..dividend.len()).rev() {
            // ---- 1) shift remainder left by one limb and bring in next dividend limb
            rem.rotate_right(1);
            rem[0] = dividend[i];

            // ---- 2) estimate quotient digit q̂
            let mut qd = if N == 0 {
                0
            } else if rem[N - 1] > modulus.limbs[N - 1] {
                u64::MAX
            } else {
                let hi = (rem[N - 1] as u128) << 64;
                let lo = if N >= 2 { rem[N - 2] as u128 } else { 0 };
                ((hi | lo) / modulus.limbs[N - 1] as u128) as u64
            };

            // ---- 3) refine q̂ (at most 2 subtractions)
            while qd > 0 {
                let prod = Self::mul_wide(&[qd], &modulus.limbs);
                if Self::cmp_wide(&prod, &rem.to_vec()) == std::cmp::Ordering::Greater {
                    qd -= 1;
                } else {
                    break;
                }
            }

            // ---- 4) subtract q̂·p from remainder
            let prod = Self::mul_wide(&[qd], &modulus.limbs);
            let mut borrow = 0i128;
            for j in 0..N {
                let sub = if j < prod.len() { prod[j] as i128 } else { 0 };
                borrow = rem[j] as i128 - sub - borrow;
                rem[j] = borrow as u64;
                borrow = if borrow < 0 { 1 } else { 0 };
            }

            // ---- 5) if under-subtracted, add p back and decrease q̂
            if borrow > 0 {
                qd -= 1;
                let mut carry = 0u128;
                for j in 0..N {
                    carry = carry + rem[j] as u128 + modulus.limbs[j] as u128;
                    rem[j] = carry as u64;
                    carry >>= 64;
                }
            }

            // ---- 6) store quotient digit
            // dividend 의 앞쪽 N limb 은 몫에 포함되지 않는다.
            // i ≥ N 일 때만 quot에 기록한다.
            if i >= N {
                let quot_idx = i - N;               // 0 … N
                quot[quot_idx] = qd;
            }
        }

        quot
    }

    pub fn neg_mod(&self, modulus: &Self) -> Self {
        modulus.sub_mod(self, modulus)
    }

    pub fn square_mod(&self, modulus: &Self, pre_mu: Option<[u64; N + 1]>) -> Self {
        let mu = pre_mu.unwrap_or(Self::precompute_mu(modulus));
        self.mul_mod(self, modulus, Some(mu))
    }

    pub fn pow_mod(&self, exp: &Self, modulus: &Self, pre_mu: Option<[u64; N + 1]>) -> Self {
        let mut result = Self::from_limb(1u64);
        let mu = pre_mu.unwrap_or(Self::precompute_mu(modulus));

        for i in (0..N).rev() {
            let limb = exp.limbs[i];
            for bit in (0usize..64).rev() {
                result = result.square_mod(modulus, Some(mu));
                if (limb & (1u64 << bit)) != 0 {
                    result = result.mul_mod(self, modulus, Some(mu));
                }
            }
        }
        result
    }

    pub fn get_minus_two_unchecked(&self) -> Self {
        let mut borrow = false;
        let mut result = Self::default();
        (result.limbs[0], borrow) = self.limbs[0].borrowing_sub(2, borrow);
        for i in 1..N {
            (result.limbs[i], borrow) = self.limbs[i].borrowing_sub(0, borrow);
        }
        result
    }

    pub fn inv_mod(&self, modulus: &Self, pre_mu: Option<[u64; N + 1]>) -> Self {
        let p_minus_2 = modulus.get_minus_two_unchecked();
        let mu = pre_mu.unwrap_or(Self::precompute_mu(modulus));
        self.pow_mod(&p_minus_2, modulus, Some(mu))
    }

    pub fn ntt(input: &mut [Self], omega: &Self /* primitive root ω for modulus p */ , modulus: &Self, mu: &[u64; N + 1]) {
        let n = input.len();
        let mut len = 1;
        while len < n {
            let half = len;
            len *= 2;
            let mut w: Self = Self::from_limb(1u64);
            for i in 0..half {
                let mut j = i;
                while j < n {
                    let temp = input[j + half].mul_mod(&w, modulus, Some(*mu));
                    input[j + half] = input[j].sub_mod(&temp, modulus);
                    input[j] = input[j].add_mod(&temp, modulus);
                    j += len;
                }
                w = w.mul_mod(omega, modulus, Some(*mu)); // twiddle factor
            }
        }
    }
}





impl<const N: usize> Ord for BigField<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Self::cmp_multi(&self.limbs, &other.limbs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FpComplex<const N: usize> {
    re: BigField<N>,
    im: BigField<N>
}

impl<const N: usize> FpComplex<N> {
    #[inline(always)]
    pub const fn get_elements(&self) -> (BigField<N>, BigField<N>) {
        (self.re, self.im)
    }

    #[inline(always)]
    pub const fn get_elements_ref(&self) -> (&BigField<N>, &BigField<N>) {
        (&self.re, &self.im)
    }

    #[inline(always)]
    pub const fn get_real(&self) -> BigField<N> {
        self.get_elements().0
    }

    #[inline(always)]
    pub const fn get_imag(&self) -> BigField<N> {
        self.get_elements().1
    }

    #[inline(always)]
    pub const fn new(re: BigField<N>, im: BigField<N>) -> Self {
        Self {
            re, im
        }
    }

    pub fn mul_mod(&self, other: &Self, modulus: &BigField<N>, pre_mu: Option<[u64; N + 1]>) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let (ore, oim) = other.get_elements_ref();
        let mu = pre_mu.unwrap_or(BigField::<N>::precompute_mu(modulus));
        let rr = sre.mul_mod(ore, modulus, Some(mu));
        let ii = sim.mul_mod(oim, modulus, Some(mu));
        let re = rr.sub_mod(&ii, modulus);
        let im = sre.mul_mod(oim, modulus, Some(mu)).add_mod(&sim.mul_mod(ore, modulus, Some(mu)), modulus);
        Self::new(re, im)
    }

    pub fn add_mod(&self, other: &Self, modulus: &BigField<N>) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let (ore, oim) = other.get_elements_ref();
        let re = BigField::<N>::add_mod(sre, ore, modulus);
        let im = BigField::<N>::add_mod(sim, oim, modulus);
        Self::new(re, im)
    }

    pub fn sub_mod(&self, other: &Self, modulus: &BigField<N>) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let (ore, oim) = other.get_elements_ref();
        let re = BigField::<N>::sub_mod(sre, ore, modulus);
        let im = BigField::<N>::sub_mod(sim, oim, modulus);
        Self::new(re, im)
    }

    pub fn square_mod(&self, modulus: &BigField<N>, pre_mu: Option<[u64; N + 1]>) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let mu = pre_mu.unwrap_or(BigField::precompute_mu(modulus));
        let rr = sre.mul_mod(sre, modulus, Some(mu));
        let ii = sim.mul_mod(sim, modulus, Some(mu));
        let re = rr.sub_mod(&ii, modulus);
        let im = sre.mul_mod(sim, modulus, Some(mu)).mul_mod(&BigField::from_limb(2u64), modulus, Some(mu));
        Self::new(re, im)
    }

    pub fn neg_mod(&self, modulus: &BigField<N>) -> Self {
        let (re, im) = self.get_elements();
        Self::new(re.neg_mod(modulus), im.neg_mod(modulus))
    }

    pub fn norm_mod(&self, modulus: &BigField<N>, pre_mu: Option<[u64; N + 1]>) -> BigField<N> {
        let mu = pre_mu.unwrap_or(BigField::precompute_mu(modulus));
        let resq = self.re.square_mod(modulus, Some(mu));
        let imsq = self.im.square_mod(modulus, Some(mu));
        resq.add_mod(&imsq, modulus)
    }

    pub fn inv_mod(&self, modulus: &BigField<N>, pre_mu: Option<[u64; N + 1]>) -> Self {
        let mu = pre_mu.unwrap_or(BigField::precompute_mu(modulus));
        let inv_norm = self.norm_mod(modulus, Some(mu)).inv_mod(modulus, Some(mu));
        Self {
            re: self.re.mul_mod(&inv_norm, modulus, Some(mu)),
            im: modulus.sub_mod(&self.im.mul_mod(&inv_norm, modulus, Some(mu)), modulus)
        }
    }
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
mod gpu;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod wasm_bench;
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use wasm_bench::*;
