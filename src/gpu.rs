use cubecl::{matmul::components::global::multi_stage::ordered::LL, prelude::*};

#[cube(launch)]
pub fn fp_add<I: Int>(n: u32, i_a: &Array<u64>, i_b: &Array<u64>, m: &Array<u64>, output: &mut Array<u64>) {
    if ABSOLUTE_POS * n < i_a.len() && ABSOLUTE_POS * n < i_b.len() {
        let a = i_a.slice(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n);
        let b = i_b.slice(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n);
        
        fp_add_kernel(&a, &b, n, &m.slice(0, n), &mut output.slice_mut(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n));
    }
}

#[cube]
fn fp_add_kernel(a: &Slice<u64>, b: &Slice<u64>, n: u32, m: &Slice<u64>, output: &mut SliceMut<u64>) {
    let mut carry: bool = false;
    let mut result = Array::<u64>::new(n);
    #[unroll]
    for i in 0..n {
        if carry {
            if (u64::MAX - 1 - a[i]) < b[i] {
                carry = true;
                let temp = b[i] - (u64::MAX - a[i] - 1);
                result[i] = temp;
            } else {
                carry = false;
                let temp = a[i] + b[i] + 1;
                result[i] = temp;
            }
        } else {
            if (u64::MAX - a[i]) < b[i] {
                carry = true;
                let temp = b[i] - (u64::MAX - a[i]);
                result[i] = temp;
            } else {
                carry = false;
                let temp = a[i] + b[i];
                result[i] = temp;
            }
        }
    }

    let mut ord = {
        let mut out = 0i32;
        fp_cmp_multi_kernel(&result.slice(0, n), m, n, &mut out);
        out
    };
    if carry {
        let mut high: u64 = 1;
        while ord >= 0 {
            fp_sub_multi_kernel(&mut result.to_slice_mut(),m, n, &mut high);
            fp_cmp_multi_kernel(&result.to_slice(), m, n, &mut ord);
        }
    } else {
        let mut high: u64 = 0;
        while ord >= 0 {
            fp_sub_multi_kernel(&mut result.to_slice_mut(), m, n, &mut high);
            fp_cmp_multi_kernel(&result.to_slice(), m, n, &mut ord);
        }
    }
    #[unroll]
    for j in 0..n {
        output[j] = result[j]
    }
}

#[cube]
fn fp_cmp_multi_kernel(a: &Slice<u64>, b: &Slice<u64>, n: u32, output: &mut i32) {
    for i in 0u32..n {
        if a[n-1-i] > b[n-1-i] { *output = 1; terminate!() }
        if a[n-1-i] < b[n-1-i] { *output = -1; terminate!() }
    }
    *output = 0;
    terminate!()
}

#[cube]
fn fp_add_multi_kernel(output: &mut SliceMut<u64>, b: &Slice<u64>, n: u32, high: &mut u64) {
    let mut carry: bool = false;
    #[unroll]
    for i in 0..n {
        if carry {
            if (u64::MAX - 1 - output[i]) < b[i] {
                carry = true;
                let temp = b[i] - (u64::MAX - output[i] - 1);
                output[i] = temp;
            } else {
                carry = false;
                let temp = output[i] + b[i] + 1;
                output[i] = temp;
            }
        } else {
            if (u64::MAX - output[i]) < b[i] {
                carry = true;
                let temp = b[i] - (u64::MAX - output[i]);
                output[i] = temp;
            } else {
                carry = false;
                let temp = output[i] + b[i];
                output[i] = temp;
            }
        }
    }

    if carry {
        *high += 1;
    }
}


#[cube]
fn fp_sub_multi_kernel(result: &mut SliceMut<u64>, b: &Slice<u64>, n: u32, high: &mut u64) {
    let mut borrow = false;
    #[unroll]
    for i in 0..n {
        if borrow {
            let temp = result[i] - b[i] - 1;
            borrow = result[i] < (b[i] + 1);
            result[i] = temp;
        } else {
            let temp = result[i] - b[i];
            borrow = result[i] < b[i];
            result[i] = temp;
        }
    }
    if borrow {
        *high -= 1;
    }
}


#[cube]
fn fp_mod_multi_kernel(result: &mut SliceMut<u64>, m: &Slice<u64>, n: u32, high: &mut u64) {
    let mut ord = 0i32;
    
    while *high > 0 {
        fp_sub_multi_kernel(result,m,n,high);
    }

    fp_cmp_multi_kernel(&result.to_slice(), m,n,&mut ord);

    while ord >= 0 {
        fp_sub_multi_kernel(result,m,n,high);
        fp_cmp_multi_kernel(&result.to_slice(), m,n,&mut ord);
    }
}

#[cube(launch)]
pub fn fp_sub<I: Int>(n: u32, i_a: &Array<u64>, i_b: &Array<u64>, m: &Array<u64>, output: &mut Array<u64>) {
    if ABSOLUTE_POS * n < i_a.len() && ABSOLUTE_POS * n < i_b.len() {
        let a = i_a.slice(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n);
        let b = i_b.slice(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n);
        
        fp_sub_kernel(&a, &b, n, &m.slice(0, n), &mut output.slice_mut(ABSOLUTE_POS * n, (ABSOLUTE_POS+1) * n));
    }
}

#[cube]
fn fp_sub_kernel(a: &Slice<u64>, b: &Slice<u64>, n: u32, m: &Slice<u64>, output: &mut SliceMut<u64>) {
    #[unroll]
    for i in 0..n {
        output.write(i, a[i]);
    }

    let mut high: u64 = 0;
    let mut cmp = 0i32;
    fp_cmp_multi_kernel(&output.to_slice(),b,n,&mut cmp);

    while high == 0 && cmp < 0 {
        fp_add_multi_kernel(output,m,n,&mut high);
    }
    fp_sub_multi_kernel(output,b,n,&mut high);
}



#[cube]
fn array_rotr(arr: &mut Array<u64>, k0: u32) {
    let l = arr.len();
    if l > 1 {
        let k = k0 % l;
        let mut output: Array<u64> = Array::new(l);
        for i in 0..(l-k) {
            output[k + i] = arr[i];
        }

        for j in 0..k {
            output[j] = arr[l - k + j];
        }

        for i in 0..l {
            arr[i] = output[i];
        }
    }
}



#[cube]
#[inline(always)]
fn u32_widening_add_kernel(a: u32, b: u32) -> (u32, u32) {
    let ta = a as u64;
    let tb = b as u64;
    let result = ta + tb;
    u64_to_u32pair(result)
}

#[cube]
#[inline(always)]
fn u32_widening_mul_kernel(a: u32, b: u32) -> (u32, u32) {
    let ta = a as u64;
    let tb = b as u64;
    let result = ta * tb;
    u64_to_u32pair(result)
}

#[cube]
#[inline(always)]
fn u64_to_u32pair(x: u64) -> (u32, u32) {
    let low = x as u32;
    let high = (x >> 32) as u32;
    (low, high)
}

#[cube]
#[inline(always)]
fn u32pair_to_u64(pair: (u32, u32)) -> u64 {
    ((pair.1 as u64) << 32) | (pair.0 as u64)
}

#[cube]
#[inline(always)]
fn u32pair_carrying_3add_inplace(a_low: &mut u32, a_high: &mut u32, b_low: u32, b_high: u32, c_low: u32, c_high: u32, carry: &mut u32) {
    let input_a: u64 = u32pair_to_u64((*a_low, *a_high));
    let input_b: u64 = u32pair_to_u64((b_low, b_high)) + *carry as u64;

    let mut low: u32 = 0;
    let mut high: u32 = 0;
    let mut carry_new: u32 = 0;

    if u64::MAX - input_a < input_b {
        let (l, h) = u64_to_u32pair(input_b - (u64::MAX - input_a));
        low = l;
        high = h;
        carry_new = 1u32;
    } else {
        let (l, h) = u64_to_u32pair(input_a + input_b);
        low = l;
        high = h;
        carry_new = 0u32;
    }

    let interm: u64 = u32pair_to_u64((low, high));
    let input_c: u64 = u32pair_to_u64((c_low, c_high));

    if u64::MAX - interm < input_c {
        let (l, h) = u64_to_u32pair(input_c - (u64::MAX - interm));
        low = l;
        high = h;
        carry_new += 1;
    } else {
        let (l, h) = u64_to_u32pair(interm + input_c);
        low = l;
        high = h;
    }

    *a_low = low;
    *a_high = high;
    *carry = carry_new;
}


#[cube]
#[inline(always)]
fn u64pair_carrying_3add_inplace(a_low: &mut u64, a_high: &mut u64, b_low: u64, b_high: u64, c_low: u64, c_high: u64, carry: &mut u64) {
    let mut carry_low: u64 = 0;
    let mut carry_high: u64 = 0;

    if u64::MAX - *carry - *a_low < b_low {
        *a_low = b_low - (u64::MAX - *carry - *a_low);
        carry_low += 1u64;
    } else {
        *a_low += b_low + *carry;
    }
    if u64::MAX - *a_low < c_low {
        *a_low = c_low - (u64::MAX - *a_low);
        carry_low += 1u64;
    } else {
        *a_low += c_low;
    };

    if u64::MAX - carry_low - *a_high < b_high {
        *a_high = b_high- (u64::MAX - carry_low - *a_high);
        carry_high += 1;
    } else {
        *a_low += b_high + carry_low;
    }
    if u64::MAX - *a_high < c_high {
        *a_high = c_high- (u64::MAX - *a_high);
        carry_high += 1;
    } else {
        *a_low += c_high;
    }

    *carry = carry_high;    
}


#[cube]
#[inline(always)]
fn u64_widening_mul_kernel(a: u64, b: u64) -> (u64, u64) {
    let (low_a, high_a) = u64_to_u32pair(a);
    let (low_b, high_b) = u64_to_u32pair(b);

    let (ll_low, ll_high) = u32_widening_mul_kernel(low_a, low_b);
    let (hh_low, hh_high) = u32_widening_mul_kernel(high_a, high_b);

    let (lh_low, lh_high) = u32_widening_mul_kernel(low_a, high_b);
    let (hl_low, hl_high) = u32_widening_mul_kernel(high_a, low_b);

    let (o1, mut o2, mut o3, mut o4) = (ll_low, ll_high, hh_low, hh_high);
    let mut carry = 0;
    u32pair_carrying_3add_inplace(&mut o2, &mut o3, lh_low, lh_high, hl_low, hl_high, &mut carry);

    o4 += carry;

    (u32pair_to_u64((o1, o2)), u32pair_to_u64((o3, o4)))
}

#[cube]
#[inline(always)]
fn cmp_wide_kernel(a: &Slice<u64>, b: &Slice<u64>, output: &mut i32) {
    let len_a = a.len();
    let len_b = b.len();

    if len_a > len_b {
        for i in len_b..len_a {
            if a[i] != 0 {
                *output = 1; terminate!()
            }
        }
        for i in 0u32..len_b {
            if a[len_a-1-i] > b[len_a-1-i] { *output = 1; terminate!() }
            if a[len_a-1-i] < b[len_a-1-i] { *output = -1; terminate!() }
        }
        *output = 0;
    } else if len_a < len_b {
        for i in len_a..len_b {
            if b[i] != 0 {
                *output = -1; terminate!()
            }
        }
        for i in 0u32..len_b {
            if a[len_a-1-i] > b[len_a-1-i] { *output = 1; terminate!() }
            if a[len_a-1-i] < b[len_a-1-i] { *output = -1; terminate!() }
        }
        *output = 0;
    } else {
        for i in 0u32..len_a {
            if a[len_a-1-i] > b[len_a-1-i] { *output = 1; terminate!() }
            if a[len_a-1-i] < b[len_a-1-i] { *output = -1; terminate!() }
        }
        *output = 0;
    }
}



    



#[cube]
fn mul_wide_kernel(a: &Slice<u64>, b: &Slice<u64>) -> Array<u64> {
    let len_a = a.len();
    let len_b = b.len();
    let mut output: Array<u64> = Array::new(len_a + len_b);
    for i in 0..(len_a + len_b) {
        output[i] = 0;
    }
    for i in 0..len_a {
        let mut carry_low = 0u64;
        let mut carry_high = 0u64;
        for j in 0..len_b {
            let (low, high) = u64_widening_mul_kernel(a[i], b[j]);
            let mut cc = 0u64;
            u64pair_carrying_3add_inplace(&mut carry_low, &mut carry_high, low, high, output[i + j], 0, &mut cc);
            output[i + j] = carry_low;
            carry_low = carry_high;
            carry_high = 0u64;
        }
        let mut k = i + len_b;
        while carry_low > 0 || carry_high > 0 {
            let mut cc = 0u64;
            u64pair_carrying_3add_inplace(&mut carry_low, &mut carry_high, output[k], 0, 0, 0, &mut cc);
            output[k] = carry_low;
            carry_low = carry_high;
            carry_high = 0u64;
            k += 1;
        }
    }
    output
}

#[cube]
#[inline(always)]
fn u64_i64_add_kernel(a_low: u64, a_high: i64, b_low: u64, b_high: i64) -> (u64, i64) {
    let mut low: u64 = 0;
    let mut high: u64 = 0;
    let mut carry: u64 = 0;

    u64pair_carrying_3add_inplace(&mut low, &mut high, a_low, a_high as u64, b_low, b_high as u64, &mut carry);

    (low, high as i64)
}

#[cube]
#[inline(always)]
fn u64_i64_neg_kernel(low: u64, high: i64) -> (u64, i64) {
    let mut l: u64 = 0;
    let mut h: i64 = 0;
    if low == 0u64 {
        l = low;
        h = -high;
    } else {
        l = (low ^ 0xFFFFFFFFFFFFFFFFu64) + 1u64;
        h = high ^ -1i64;
    }
    (l, h)
}

#[cube]
#[inline(always)]
fn u64_i64_sub_kernel(a_low: u64, a_high: i64, b_low: u64, b_high: i64) -> (u64, i64) {
    let (b_neg_low, b_neg_high) = u64_i64_neg_kernel(b_low, b_high);
    u64_i64_add_kernel(a_low, a_high, b_neg_low, b_neg_high)
}


#[cube]
pub fn fp_precompute_mu_kernel(modulus: &Slice<u64>, n: u32, output: &mut SliceMut<u64>) {
    let mut dividend = Array::<u64>::new(2*n + 1);
    #[unroll]
    for i in 0..(2*n) {
        dividend[i] = 0;
    }
    dividend[2*n] = 1;

    let mut rem = Array::<u64>::new(n);

    for i in 0..n {
        output[i] = 0;
        rem[i] = 0;
    }
    output[n] = 0;
    
    let mut qd: u64 = 0;

    for i0 in 0..(2*n + 1) {
        let i = 2*n - i0;
        
        array_rotr(&mut dividend, 1u32);
        rem[0] = dividend[1];

        if rem[n - 1] > modulus[n - 1] {
            qd = 0xFFFFFFFFFFFFFFFFu64;
        } else {
            let q1 = 0xF000000000000000u64 / modulus[n - 1];
            let r1 = 0xF000000000000000u64 % modulus[n - 1];

            let q_low = rem[n - 2] / modulus[n - 1];
            let q_high = 2 * q1 * r1 + ((2 * rem[n - 1] * r1) / modulus[n - 1]);

            qd = q_low + q_high;
        };

        while qd > 0 {
            let mut qds: Array<u64> = Array::new(1);
            qds[0] = qd;
            let temp: Array<u64> = mul_wide_kernel(&qds.to_slice(), modulus);
            let mut ord = 0i32;
            cmp_wide_kernel(&temp.to_slice(), &rem.to_slice(), &mut ord);
            if ord > 0 {
                qd -= 1;
            } else {
                break;
            }
        }

        let mut qds: Array<u64> = Array::new(1);
        qds[0] = qd;
        let temp = mul_wide_kernel(&qds.to_slice(), modulus);
        let mut borrow_low = 0u64;
        let mut borrow_high = 0i64;
        for j in 0..n {
            let borrow = u64_i64_sub_kernel(rem[j], 0i64, borrow_low, borrow_high);
            borrow_low = borrow.0;
            borrow_high = borrow.1;
            if j < temp.len() {
                let borrow2 = u64_i64_sub_kernel(borrow_low, borrow_high, temp[j], 0i64); 
                borrow_low = borrow2.0;
                borrow_high = borrow2.1;
            }
            rem[j] = borrow_low;

            if borrow_high < 0 {
                borrow_low = 1u64;
            } else {
                borrow_low = 0u64;
            }
            borrow_high = 0i64;
        }
        if (borrow_high == 0i64 && borrow_low > 0u64) || borrow_high > 0i64 {
            qd -= 1;

            let mut carry_low = 0u64;
            let mut carry_high = 0u64;
            let mut cc = 0u64;
            for j in 0..n {
                u64pair_carrying_3add_inplace(&mut carry_low, &mut carry_high, rem[j], 0u64, modulus[j], 0u64, &mut cc);
                rem[j] = carry_low;
                carry_low = carry_high;
                carry_high = 0u64;
            }
        }
        output[dividend.len() - 1 - i] = qd;
    }
}


