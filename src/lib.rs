#![feature(const_trait_impl)]
#![feature(stmt_expr_attributes)]
#![feature(bigint_helper_methods)]
#![feature(slice_as_array)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(generic_const_exprs)]
#![feature(const_ops)]
#![feature(const_cmp)]
#![feature(associated_type_defaults)]

use std::{fmt::Display, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign}};

use rand::{CryptoRng, Rng};


pub const MBITS: usize = 64;

pub type Limb = u64;

#[derive(Debug, Clone)]
pub enum MoMAError {
    ZeroMSL,

}

impl Display for MoMAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MoMAError::ZeroMSL => {
                Display::fmt("Most significant limb should not be zero.", f)
            }
        }
    }
}




#[derive(Debug, Clone, Copy)]
pub struct BigField<const N: usize> {
    pub(crate) limbs: [Limb; N]
}

impl<const N: usize> const PartialEq for BigField<N> {
    fn eq(&self, other: &Self) -> bool {
        self.limbs == other.limbs
    }
}

impl<const N: usize> const Eq for BigField<N> { }


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
        Self::zero()
    }
}

#[inline(always)]
const fn parse_be_bytes<const N: usize>(bytes: [u8; 8 * N]) -> [u64; N] {
    const S: usize = std::mem::size_of::<u64>();
    let mut result = [0u64; N];
    let mut idx = 0usize;
    while idx < N {
        let chunk: [u8; S] = *bytes[idx*S..(idx+1)*S].as_array().unwrap();
        result[N-1 - idx] = u64::from_be_bytes(chunk);
        idx += 1;
    }
    result
}

#[inline(always)]
const fn parse_le_bytes<const N: usize>(bytes: [u8; 8 * N]) -> [u64; N] {
    const S: usize = std::mem::size_of::<u64>();
    let mut result = [0u64; N];
    let mut idx = 0usize;
    while idx < N {
        let chunk: [u8; S] = *bytes[idx*S..(idx+1)*S].as_array::<S>().unwrap();
        result[idx] = u64::from_le_bytes(chunk);
        idx += 1;
    }
    result
}


impl<const N: usize> BigField<N> {
    #[inline(always)]
    pub const fn get_limbs(&self) -> [Limb; N] {
        self.limbs
    }

    #[inline(always)]
    pub const fn new(limbs: [Limb; N]) -> Self {
        Self {
            limbs
        }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Self { limbs: [0u64; N] }
    }

    #[cfg(feature = "random")]
    #[inline(always)]
    pub fn random<R: CryptoRng + Sized>(rng: &mut R) -> Self {
        let mut result = Self::zero();
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
    pub const fn to_be_bytes(&self, output: &mut [u8]) {
        let mut chunks: [[u8; 8]; N] = [[0u8; 8]; N];
        let mut i = 0usize;
        while i < N {
            chunks[i] = self.limbs[N - 1 - i].to_be_bytes();
            i += 1;
        }
        output.copy_from_slice(chunks.as_flattened());
    }

    #[inline(always)]
    pub const fn to_le_bytes(&self, output: &mut [u8]) {
        let mut chunks: [[u8; 8]; N] = [[0u8; 8]; N];
        let mut i = 0usize;
        while i < N {
            chunks[i] = self.limbs[i].to_le_bytes();
            i += 1;
        }
        output.copy_from_slice(chunks.as_flattened());
    }

    #[inline(always)]
    pub const fn from_limb(limb: Limb) -> Self {
        let mut limbs: [Limb; N] = [0; N];
        limbs[0] = limb;
        Self {
            limbs
        }
    }

    // Helper: Compare multi-limb
    #[inline(always)]
    const fn cmp_multi(a: &[Limb; N], b: &[Limb; N]) -> std::cmp::Ordering {
        let mut i1 = N;
        while i1 > 0 {
            let i = i1 -1;
            if a[i] > b[i] { return std::cmp::Ordering::Greater; }
            if a[i] < b[i] { return std::cmp::Ordering::Less; }
            i1 = i;
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

    pub(crate) fn add_multi(result: &mut [Limb; N], b: &[Limb; N], high: &mut Limb) {
        let mut carry = false;
        for i in 0..N {
            (result[i], carry) = result[i].carrying_add(b[i], carry)
        }
        if carry {
            *high += 1;
        }
    }

    pub(crate) fn add_multi_return(a: &Self, b: &Self, high: &mut Limb) -> Self {
        let mut carry = false;

        let mut result = Self::zero();
        for i in 0..N {
            (result.limbs[i], carry) = a.limbs[i].carrying_add(b.limbs[i], carry)
        }
        if carry {
            *high += 1;
        }
        result
    }

    #[inline(always)]
    pub const fn mod_multi(x: &mut [Limb; N], modulus: &[Limb; N], high: &mut Limb) {
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

    #[inline(always)]
    const fn sub_multi(result: &mut [Limb; N], b: &[Limb; N], high: &mut Limb) {
        let mut borrow: i128 = 0;
        let mut i = 0usize;
        while i < N {
            let temp = result[i] as i128 - b[i] as i128 - borrow;
            result[i] = temp as Limb;
            borrow = if temp < 0 { 1 } else { 0 };
            i += 1;
        }
        if borrow > 0 {
            *high -= 1;
        }
    }

    pub fn mul_mod(&self, other: &Self, modulus: &Self, mu: [Limb; N + 1]) -> Self {

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
                while k >= product.len() {
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

        // Remove the problematic while loop for borrow
        // Instead, if final borrow != 0 (r negative), add modulus
        if borrow != 0 {
            let mut carry = 0u128;
            for i in 0..N {
                if i < r.len() {
                    carry = carry + r[i] as u128 + modulus.limbs[i] as u128;
                    r[i] = carry as Limb;
                    carry >>= 64;
                } else {
                    break;  // No more r limbs
                }
            }
            let mut k = N;
            while carry > 0 && k < r.len() {
                carry += r[k] as u128;
                r[k] = carry as Limb;
                carry >>= 64;
                k += 1;
            }
            if carry > 0 {
                while k >= r.len() {
                    r.push(0);
                }
                r[k] = carry as Limb;
            }
        }

        // Extract low N limbs
        let mut result = [0; N];
        let res_len = std::cmp::min(r.len(), N);
        for i in 0..res_len {
            result[i] = r[i];
        }

        // Conditional sub loop (your code is fine, but add high handling if needed)
        let mut high = 0;
        loop {
            match Self::cmp_multi(&result, &modulus.limbs) {
                std::cmp::Ordering::Greater => {
                    Self::sub_multi(&mut result, &modulus.limbs, &mut high);
                },
                std::cmp::Ordering::Equal => {
                    return Self::zero();
                },
                std::cmp::Ordering::Less => {
                    return Self::new(result);
                }
            }
        }
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

    pub fn precompute_mu(modulus: &Self) -> [Limb; N+1] where [(); 2*N+1]: {
        let mut result: BigField<{N + 1}> = BigField::zero();
        let mut product = BigField::zero();
        let mut head: BigField<{N + 1}> = BigField::from_limb(1u64) << (64 * N);
        let dividend = BigField::from_limb(1u64) << (2 * 64 * N);
        let mut product_delta: BigField<{2 * N + 1}> = modulus.cast_to::<{2 * N + 1}>() <<  64 * N;
        let mut high = 0u64;

        loop {
            let new_result = result | head;
            let new_product = BigField::add_multi_return(&product, &product_delta, &mut high);
            match new_product.cmp(&dividend) {
                std::cmp::Ordering::Less => {
                    result = new_result;
                    product = new_product;
                },
                std::cmp::Ordering::Equal => {
                    result = new_result;
                    break;
                },
                std::cmp::Ordering::Greater => {

                }
            }
            head >>= 1usize;
            product_delta >>= 1usize;
            if head.limbs.iter().all(|limb|*limb == 0u64) {
                break;
            }
        }
        result.limbs
    }

    pub fn neg_mod(&self, modulus: &Self) -> Self {
        if self.is_zero() {
            *self
        } else {
            modulus.sub_mod(self, modulus)
        }
    }

    // sqrt_mod using Tonelli-Shanks
    pub fn sqrt_mod(&self, modulus: &Self, mu: [Limb; N + 1]) -> Option<Self> {

        if Self::legendre_symbol(self, modulus, &mu) != 1 {
            return None; // Not quadratic residue
        }

        
        let (one, two, four) = (Self::from_limb(1u64), Self::from_limb(2u64), Self::from_limb(4u64));

        
        if modulus.limbs[0] % 4 == 3 {
            // Simple case for p ≡ 3 mod 4: sqrt = a^{(p+1)/4} mod p
            let mut exp = modulus.add_mod(&Self::from_limb(1u64), modulus);
            exp = exp.pow_mod(&four, modulus, mu).inv_mod(modulus, mu).unwrap(); // (p+1)/4
            return Some(self.pow_mod(&exp, modulus, mu));
        }

        // General Tonelli-Shanks
        // Step 1: Write p-1 = 2^s * q, q odd
        let mut s = 0;
        let mut q = modulus.sub_mod(&Self::from_limb(1u64), modulus);
        while q.limbs[0] % 2 == 0 {
            q = q.pow_mod(&Self::from_limb(2u64), modulus, mu).inv_mod(modulus, mu).unwrap(); // divide by 2
            s += 1;
        }

        // Step 2: Find non-residue z
        let mut z = two.clone();
        while Self::legendre_symbol(&z, modulus, &mu) != -1 {
            z = z.add_mod(&one, modulus);
        }

        // Step 3: Set c = z^q mod p, r = a^{(q+1)/2} mod p, t = a^q mod p
        let mut c = z.pow_mod(&q, modulus, mu);
        let mut r = self.pow_mod(&q.add_mod(&one, modulus).pow_mod(&two, modulus, mu).inv_mod(modulus, mu).unwrap(), modulus, mu);
        let mut t = self.pow_mod(&q, modulus, mu);
        let mut m = s;

        loop {
            if t.is_one() {
                return Some(r); // r^2 ≡ a mod p
            }

            // Find smallest i such that t^{2^i} ≡ 1 mod p
            let mut i = 1;
            let mut tt = t.mul_mod(&t, modulus, mu);
            while !tt.is_one() {
                tt = tt.mul_mod(&tt, modulus, mu);
                i += 1;
            }

            // b = c^{2^{m-i-1}} mod p
            let mut b = c;
            for _ in 0..(m - i - 1) {
                b = b.mul_mod(&b, modulus, mu);
            }

            r = r.mul_mod(&b, modulus, mu);
            c = b.mul_mod(&b, modulus, mu);
            t = t.mul_mod(&c, modulus, mu);
            m = i;
        }
    }

    pub fn legendre_symbol(a: &Self, p: &Self, mu: &[Limb; N+1]) -> i32 where [(); N+1]: {
        // a^{(p-1)/2} mod p: 1 if QR, -1 if non-QR, 0 if a=0 mod p
        let exp = p.sub_mod(&Self::from_limb(1u64), p) >> 1usize;
        let res = a.pow_mod(&exp, p, *mu);
        if res.is_zero() {
            0
        } else if res.is_one() {
            1
        } else {
            -1
        }
    }

    pub fn square_mod(&self, modulus: &Self, mu: [u64; N + 1]) -> Self {
        self.mul_mod(self, modulus, mu)
    }

    pub fn pow_mod(&self, exp: &Self, modulus: &Self, mu: [u64; N + 1]) -> Self {
        let mut result = Self::from_limb(1u64);

        for &limb in exp.limbs.iter().rev() {
            for bit in (0usize..64).rev() {
                result = result.square_mod(modulus, mu);
                if (limb & (1u64 << bit)) != 0 {
                    result = result.mul_mod(self, modulus, mu);
                }
            }
        }
        result
    }

    pub fn get_minus_two_unchecked(&self) -> Self {
        let mut borrow = false;
        let mut result = Self::zero();
        (result.limbs[0], borrow) = self.limbs[0].borrowing_sub(2, borrow);
        for i in 1..N {
            (result.limbs[i], borrow) = self.limbs[i].borrowing_sub(0, borrow);
        }
        result
    }

    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        let (mut result, mut i) = (true, 0usize);
        while i < N {
            result = result && (self.limbs[i] == 0u64);
            i += 1;
        }
        result
    }

    #[inline(always)]
    pub const fn is_one(&self) -> bool {
        let (mut result, mut i) = (self.limbs[0] == 1u64, 1usize);
        while i < N {
            result = result && (self.limbs[i] == 0u64);
            i += 1;
        }
        result
    }

    pub fn inv_mod(&self, modulus: &Self, mu: [u64; N + 1]) -> Option<Self> {
        
        // Using Fermat's Little Theorem: a^{p-2} mod p
        let p_minus_two = {
            let mut p_minus_two = modulus.clone();
            let mut borrow: i128 = 2;
            for i in 0..N {
                let temp = p_minus_two.limbs[i] as i128 - borrow;
                p_minus_two.limbs[i] = (temp + (if temp < 0 { 1 << 64 } else { 0 })) as u64;
                borrow = if temp < 0 { 1 } else { 0 };
            }
            p_minus_two
        };
        let result = self.pow_mod(&p_minus_two, modulus, mu);

        if self.mul_mod(&result, modulus, mu).is_one() {
            Some(result)
        } else {
            None
        }
    }


    pub fn ntt(input: &mut [Self], precompute: &NttPrecompute<N> , modulus: &Self, mu: &[u64; N + 1]) {
        let n = input.len();
        let mut len = 1;
        let mut twiddle_idx = n / 2;
        while len < n {
            let half = len;
            len *= 2;
            twiddle_idx /= 2;
            for i in 0..half {
                let w = precompute.twiddles[i * twiddle_idx];
                let mut j = i;
                while j < n {
                    let temp = input[j + half].mul_mod(&w, modulus, *mu);
                    input[j + half] = input[j].sub_mod(&temp, modulus);
                    input[j] = input[j].add_mod(&temp, modulus);
                    j += len;
                }
            }
        }
    }

    #[inline(always)]
    pub const fn cast_to<const M: usize>(self) -> BigField<M> {
        let mut result = BigField::<M>::zero();
        let mut i = 0;
        let m = M;
        let n = N;
        let limit = std::cmp::min(n, m);
        while i < limit {
            result.limbs[i] = self.limbs[i];
            i += 1;
        }
        result
    }

    #[inline(always)]
    pub const fn flip(&self) -> Self {
        let mut i = 0usize;
        let mut result = Self::zero();
        while i < N {
            result.limbs[i] = self.limbs[i] ^ 0xFFFFFFFFFFFFFFFFu64;
            i += 1;
        }

        result
    }

    #[inline(always)]
    pub(crate) const fn get_2x_minus_1_over_3(&self) -> Self where [(); N+1]: {
        let r_limbs: [u64; N+1] = {
            let mut tmp = self.cast_to::<{N+1}>();
            tmp <<= 1usize;
            let mut borrow = false;
            (tmp.limbs[1], borrow) = tmp.limbs[0].borrowing_sub(1,borrow);
            let mut i = 1usize;
            while i < N {
                (tmp.limbs[i+1], borrow) = tmp.limbs[i].borrowing_sub(0,borrow);
                i += 1;
            }
            tmp.limbs
        };
        let s_limbs = {
            let mut tmp = [0u64; N];
            let mut i1 = N+1;
            let mut rem = 0u64;
            while i1 > 0 {
                let i = i1 - 1;
                let intemp = ((rem as u128) << 64) + r_limbs[i] as u128;
                if i < N {
                    tmp[i] = (intemp / 3) as u64;
                }
                rem = (intemp % 3) as u64;
                i1 = i;
            }
            tmp
        };
        Self::new(s_limbs)
    }

    pub fn cbrt_mod_unchecked(&self, modulus: &Self, mu: [u64; N + 1]) -> Self {
        let exp = modulus.get_2x_minus_1_over_3();
        self.pow_mod(&exp, modulus, mu)
    }
}

impl<const N: usize> const PartialOrd for BigField<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Self::cmp_multi(&self.limbs, &other.limbs))
    }
}

impl<const N: usize> const Ord for BigField<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FpComplex<const N: usize> {
    re: BigField<N>,
    im: BigField<N>
}

impl<const N: usize> const PartialEq for FpComplex<N> {
    fn eq(&self, other: &Self) -> bool {
        self.re == other.re && self.im == other.im
    }
}

impl<const N: usize> const Eq for FpComplex<N> { }


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
    pub const fn get_elements_mut_ref(&mut self) -> (&mut BigField<N>, &mut BigField<N>) {
        (&mut self.re, &mut self.im)
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

    pub fn mul_mod(&self, other: &Self, modulus: &BigField<N>, mu: [u64; N + 1]) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let (ore, oim) = other.get_elements_ref();
        let rr = sre.mul_mod(ore, modulus, mu);
        let ii = sim.mul_mod(oim, modulus, mu);
        let re = rr.sub_mod(&ii, modulus);
        let im = sre.mul_mod(oim, modulus, mu).add_mod(&sim.mul_mod(ore, modulus, mu), modulus);
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

    pub fn square_mod(&self, modulus: &BigField<N>, mu: [u64; N + 1]) -> Self {
        let (sre, sim) = self.get_elements_ref();
        let rr = sre.mul_mod(sre, modulus, mu);
        let ii = sim.mul_mod(sim, modulus, mu);
        let re = rr.sub_mod(&ii, modulus);
        let im = sre.mul_mod(sim, modulus, mu).mul_mod(&BigField::from_limb(2u64), modulus, mu);
        Self::new(re, im)
    }

    pub fn neg_mod(&self, modulus: &BigField<N>) -> Self {
        let (re, im) = self.get_elements();
        Self::new(re.neg_mod(modulus), im.neg_mod(modulus))
    }

    pub fn norm_mod(&self, modulus: &BigField<N>, mu: [u64; N + 1]) -> BigField<N> {
        let resq = self.re.square_mod(modulus, mu);
        let imsq = self.im.square_mod(modulus, mu);
        resq.add_mod(&imsq, modulus)
    }

    pub fn conjugate_mod(&self, modulus: &BigField<N>) -> Self {
        Self {
            re: self.re,
            im: self.im.neg_mod(modulus)
        }
    }

    pub fn inv_mod(&self, modulus: &BigField<N>, mu: [u64; N + 1]) -> Option<Self> {
        if let Some(inv_norm) = self.norm_mod(modulus, mu).inv_mod(modulus, mu) {
            Some(self.conjugate_mod(modulus).mul_mod(&Self { re: inv_norm, im: BigField::zero() }, modulus, mu))
        } else {
            None
        }        
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Self::new(BigField::zero(), BigField::zero())
    }

    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }

    #[cfg(feature = "random")]
    #[inline(always)]
    pub fn random<R: CryptoRng + Sized>(rng: &mut R) -> Self {
        let re = BigField::<N>::random(rng);
        let im = BigField::<N>::random(rng);
        Self::new(re, im)
    }

    pub fn pow_mod(&self, exp: &BigField<N>, modulus: &BigField<N>, mu: [u64; N + 1]) -> Self {
        let mut result = Self::new(BigField::from_limb(1u64), BigField::zero());

        for &limb in exp.limbs.iter().rev() {
            for bit in (0..64).rev() {
                result = result.square_mod(modulus, mu);
                if limb & (1u64 << bit) != 0 {
                    result = result.mul_mod(self, modulus, mu);
                }
            }
        }
        result
    }

    pub fn cbrt_mod_unchecked(&self, modulus: &BigField<N>, mu: [u64; N + 1]) -> Self {
        let exp = modulus.get_2x_minus_1_over_3();
        self.pow_mod(&exp, modulus, mu)
    }

    pub fn sqrt_mod(&self, modulus: &BigField<N>, mu: [u64; N+1]) -> Option<Self> {
        let (a, b) = (self.re, self.im);
        if b.is_zero() {
            return a.sqrt_mod(modulus, mu).map(|sqrt_a| Self::new(sqrt_a, BigField::zero()));
        }
        let gamma_sq = a.square_mod(modulus, mu).add_mod(&b.square_mod(modulus, mu), modulus);  // norm
        let gamma = gamma_sq.sqrt_mod(modulus, mu)?;

        let two = BigField::<N>::from_limb(2);
        let two_inv = two.inv_mod(modulus, mu)?;

        let alpha = gamma.add_mod(&a, modulus).mul_mod(&two_inv, modulus, mu);
        let alpha_sq = alpha.sqrt_mod(modulus, mu)?;
        let beta = gamma.sub_mod(&a, modulus).mul_mod(&two_inv, modulus, mu);
        let beta_sq = beta.sqrt_mod(modulus, mu)?;
        // sign 선택: check if 2 * alpha_sq * beta_sq == b.square()
        if two.mul_mod(&alpha_sq, modulus, mu).mul_mod(&beta_sq, modulus, mu).sub_mod(&b, modulus).is_zero() {
            Some(Self::new(alpha_sq, beta_sq))
        } else {
            Some(Self::new(alpha_sq, beta_sq.neg_mod(modulus)))
        }
    }
}

// Pre-computed twiddles for NTT
pub struct NttPrecompute<const N: usize> {
    twiddles: Vec<BigField<N>>,
}

impl<const N: usize> NttPrecompute<N> {
    pub fn new(omega: &BigField<N>, size: usize, modulus: &BigField<N>, mu: [u64; N+1]) -> Self {
        let mut twiddles = vec![BigField::from_limb(1u64); size / 2];
        let mut current = omega.clone();
        for i in 1..size / 2 {
            twiddles[i] = current;
            current = current.mul_mod(omega, modulus, mu);
        }
        Self { twiddles }
    }
}


#[cfg(all(target_family = "wasm", target_os = "unknown", feature = "wasm_benches"))]
mod wasm_bench;
#[cfg(all(target_family = "wasm", target_os = "unknown", feature = "wasm_benches"))]
pub use wasm_bench::*;



#[cfg(test)]
mod test {
    use crate::BigField;

    type Fp = BigField<7>;

    /* 
    fn get_modulus_and_mu() -> (Fp, [u64; 8]) {
        let modulus = Fp::new([
            0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
        ]);
        let mu: [u64; 8] = Fp::precompute_mu(&modulus);
        (modulus, mu)
    }
    */

    #[test]
    fn test_inv() {
        let modulus = Fp::new([
            0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
        ]);
        let x = Fp::new([
            7u64,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64
        ]);
        let y = Fp::new([
            5270498306774157604u64, 
            2635249153387078802u64, 
            10540996613548315209u64, 
            13176245763867560228u64, 
            15811494920322472813u64, 
            7905747460161236406u64, 
            13176245766935394011u64 
        ]);
        let mu = [
            2u64,
            0u64,
            0u64,
            4294967296u64,
            0u64,
            0u64,
            0u64,
            1u64
        ];

        let xy = x.mul_mod(&y, &modulus, mu);

        if xy.is_one() {
            println!(":) x * y mod p = 1")
        } else {
            panic!()
        }

    }

    #[test]
    fn test_sqrt() {
        let modulus = Fp::new([
            0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
        ]);
        let y = Fp::new([
            5270498306774157604u64, 
            2635249153387078802u64, 
            10540996613548315209u64, 
            13176245763867560228u64, 
            15811494920322472813u64, 
            7905747460161236406u64, 
            13176245766935394011u64 
        ]);
        let mu = [
            2u64,
            0u64,
            0u64,
            4294967296u64,
            0u64,
            0u64,
            0u64,
            1u64
        ];

        let y2 = y.square_mod(&modulus, mu);
        if let Some(yp) = y2.sqrt_mod(&modulus, mu) {
            if yp == y {
                println!(":) sqrtmod(y^2 mod p, p) = y")
            } else {
                println!(":( sqrtmod(y^2 mod p, p) = [");
                for i in 0..7 {
                    let limb = yp.limbs[i];
                    print!("\t{limb}");
                    if i < 6 {
                        println!(",");
                    }
                }
                println!("]\n")
            }
        } else {
            println!("X( sqrtmod(y^2 mod p, p)");
            panic!()
        }
    }

    #[test]
    fn test_precompute_mu() {
        let modulus = Fp::new([
            0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFEFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64, 0xFFFFFFFFFFFFFFFFu64
        ]);
        let x = Fp::new([
            7u64,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64,
            0u64
        ]);
        let y = Fp::new([
            5270498306774157604u64, 
            2635249153387078802u64, 
            10540996613548315209u64, 
            13176245763867560228u64, 
            15811494920322472813u64, 
            7905747460161236406u64, 
            13176245766935394011u64 
        ]);
        let mu: [u64; 8] = Fp::precompute_mu(&modulus);

        let xy = x.mul_mod(&y, &modulus, mu);

        if xy.is_one() {
            println!(":) x * y mod p = 1")
        } else {
            println!("{mu:#?}");
            panic!()
        }

    }

    #[test]
    fn test_shr() {
        if Fp::new([
            0u64,0u64,0u64,0u64,0u64,0u64,1
        ]) >> 1 != Fp::new([
            0u64,0u64,0u64,0u64,0u64,0x8000000000000000u64,0u64
        ]) {
            println!(":( x >> 1");
            panic!()
        }
    }

    #[test]
    fn test_shl() {
        if Fp::new([
            0u64,0u64,0u64,0u64,0u64,0u64,1
        ]) << 1 != Fp::new([
            0u64,0u64,0u64,0u64,0u64,0u64,2u64
        ]) {
            println!(":( x << 1");
            panic!()
        }
    }
}

#[inline(always)]
const fn shl_helper(lhs: u64, rhs: usize) -> (u64, u64) {
    if rhs == 0usize {
        return (0u64, lhs);
    } else if rhs < 64usize {
        let carry = lhs >> (64usize - rhs);
        return (carry, lhs << rhs);
    } else {
        return (lhs, 0u64);
    }
}

#[inline(always)]
const fn shr_helper(lhs: u64, rhs: usize) -> (u64, u64) {
    if rhs == 0usize {
        return (lhs, 0u64);
    } else if rhs < 64usize {
        let carry = lhs << (64usize - rhs);
        return (lhs >> rhs, carry);
    } else {
        return (0u64, lhs);
    }
}

impl<const N: usize> const Shl<usize> for BigField<N> {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        if rhs > 0 {
            if rhs < 64 {
                let mut pairs = [(0u64, 0u64); N];
                let mut i = 0usize;
                while i < N {
                    pairs[i] = shl_helper(self.limbs[i], rhs);
                    i += 1;
                }
                let mut result: Self = Self::zero();
                result.limbs[0] = pairs[0].1;
                i = 1;
                pairs[N-1].0 = 0;
                while i < N {
                    result.limbs[i] = pairs[i].1 | pairs[i-1].0;
                    i += 1;
                }
                result
            } else {
                let q = rhs / 64usize;
                if q < N {
                    let mut interm: Self = Self::zero();
                    let mut i = 0usize;
                    let r = rhs % 64usize;
                    while i < N-q {
                        interm.limbs[i+q] = self.limbs[i];
                        i += 1;
                    }
                    interm << r
                } else {
                    Self::zero()
                }
            }
        } else {
            self
        }
    }
}

impl<const N: usize> const ShlAssign<usize> for BigField<N> {
    fn shl_assign(&mut self, rhs: usize) {
        if rhs > 0 {
            if rhs < 64 {
                let mut pairs = [(0u64, 0u64); N];
                let mut i = 0usize;
                while i < N {
                    pairs[i] = shl_helper(self.limbs[i], rhs);
                    i += 1;
                }
                self.limbs[0] = pairs[0].1;
                i = 1;
                pairs[N-1].0 = 0;
                while i < N {
                    self.limbs[i] = pairs[i].1 | pairs[i-1].0;
                    i += 1;
                }
            } else {
                let q = rhs / 64usize;
                if q < N {
                    let mut i = N-1usize;
                    let r = rhs % 64usize;
                    while i >= q {
                        self.limbs[i] = self.limbs[i-q];
                        i += 1;
                    }
                    *self <<= r;
                } else {
                    *self = Self::zero();
                }
            }
        }
    }
}

impl<const N: usize> const Shr<usize> for BigField<N> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: usize) -> Self::Output {
        if rhs > 0 {
            if rhs < 64 {
                let mut pairs = [(0u64, 0u64); N];
                let mut i = 0usize;
                while i < N {
                    pairs[i] = shr_helper(self.limbs[i], rhs);
                    i += 1;
                }
                let mut result: Self = Self::zero();
                result.limbs[N-1] = pairs[N-1].0;
                i = 0;
                pairs[0].1 = 0;
                while i < N-1 {
                    result.limbs[i] = pairs[i].0 | pairs[i+1].1;
                    i += 1;
                }
                result
            } else {
                let q = rhs / 64usize;
                if q < N {
                    let mut interm: Self = Self::zero();
                    let mut i = 0usize;
                    let r = rhs % 64usize;
                    while i < N-q {
                        interm.limbs[i] = self.limbs[i+q];
                        i += 1;
                    }
                    interm >> r
                } else {
                    Self::zero()
                }
            }
        } else {
            self
        }
    }
}

impl<const N: usize> const ShrAssign<usize> for BigField<N> {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: usize) {
        if rhs > 0 {
            if rhs < 64 {
                let mut pairs = [(0u64, 0u64); N];
                let mut i = 0usize;
                while i < N {
                    pairs[i] = shr_helper(self.limbs[i], rhs);
                    i += 1;
                }
                self.limbs[N-1] = pairs[N-1].0;
                i = 0;
                pairs[0].1 = 0;
                while i < N-1 {
                    self.limbs[i] = pairs[i].0 | pairs[i+1].1;
                    i += 1;
                }
            } else {
                let q = rhs / 64usize;
                if q < N {
                    let mut i = N-1usize;
                    let r = rhs % 64usize;
                    while i >= q {
                        self.limbs[i-q] = self.limbs[i];
                        i += 1;
                    }
                    *self >>= r;
                } else {
                    *self = Self::zero();
                }
            }
        }
    }
}

impl<const N: usize> const BitOr for BigField<N> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        let mut i = 0;
        while i < N {
            result.limbs[i]  = self.limbs[i] | rhs.limbs[i];
            i += 1;
        }
        result
    }
}

impl<const N: usize> const BitXor for BigField<N> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        let mut i = 0;
        while i < N {
            result.limbs[i]  = self.limbs[i] ^ rhs.limbs[i];
            i += 1;
        }
        result
    }
}

impl<const N: usize> const BitAnd for BigField<N> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        let mut i = 0;
        while i < N {
            result.limbs[i]  = self.limbs[i] & rhs.limbs[i];
            i += 1;
        }
        result
    }
}


impl<const N: usize> const BitOrAssign for BigField<N> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        let mut i = 0;
        while i < N {
            self.limbs[i] |= rhs.limbs[i];
            i += 1;
        }
    }
}

impl<const N: usize> const BitXorAssign for BigField<N> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        let mut i = 0;
        while i < N {
            self.limbs[i] ^= rhs.limbs[i];
            i += 1;
        }
    }
}

impl<const N: usize> const BitAndAssign for BigField<N> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        let mut i = 0;
        while i < N {
            self.limbs[i] &= rhs.limbs[i];
            i += 1;
        }
    }
}