//! SIMD kernels for the feature transformer: accumulator updates fused with the
//! SCReLU output dot product.

use super::{
    constants::*,
    network::{NNUE, SideAccumulator},
};
use fearless_simd::{Simd, i16x32, i32x16, prelude::*};

#[cfg(target_feature = "avx512bw")]
const PARALLEL_SUMS: usize = 4;
#[cfg(not(target_feature = "avx512bw"))]
const PARALLEL_SUMS: usize = 2;

pub(super) struct Delta<const N: usize> {
    adds: [usize; N],
    subs: [usize; N],
    n_adds: usize,
    n_subs: usize,
}

impl<const N: usize> Delta<N> {
    #[inline(always)]
    pub(super) fn new() -> Self {
        Self {
            adds: [0; N],
            subs: [0; N],
            n_adds: 0,
            n_subs: 0,
        }
    }

    #[inline(always)]
    pub(super) fn add(&mut self, index: usize) {
        self.adds[self.n_adds] = index;
        self.n_adds += 1;
    }

    #[inline(always)]
    pub(super) fn sub(&mut self, index: usize) {
        self.subs[self.n_subs] = index;
        self.n_subs += 1;
    }

    #[inline(always)]
    fn adds(&self) -> &[usize] {
        &self.adds[..self.n_adds]
    }

    #[inline(always)]
    fn subs(&self) -> &[usize] {
        &self.subs[..self.n_subs]
    }
}

#[inline(always)]
fn screlu_accumulate<S: Simd>(simd: S, v: i16x32<S>, out_w: &[i16], sum: &mut i32x16<S>) {
    let clamped = v
        .max(i16x32::splat(simd, 0))
        .min(i16x32::splat(simd, QA as i16));

    cfg_select! {
        all(target_arch = "x86_64", target_feature = "avx512bw") => {
            unsafe {
                use std::arch::x86_64::*;
                let c: __m512i = clamped.into();
                let w = _mm512_load_si512(out_w.as_ptr() as *const __m512i);
                let dot = _mm512_madd_epi16(c, _mm512_mullo_epi16(c, w));
                let s: __m512i = (*sum).into();
                *sum = SimdFrom::simd_from(simd, _mm512_add_epi32(s, dot));
            }
        }
        all(target_arch = "x86_64", target_feature = "avx2") => {
            unsafe {
                use std::arch::x86_64::*;
                let (c0, c1) = clamped.split();
                let (c0, c1): (__m256i, __m256i) = (c0.into(), c1.into());
                let out_w_ptr = out_w.as_ptr() as *const __m256i;
                let (w0, w1) = (_mm256_load_si256(out_w_ptr), _mm256_load_si256(out_w_ptr.offset(1)));
                let (p0, p1) = (_mm256_mullo_epi16(c0, w0), _mm256_mullo_epi16(c1, w1));
                let (dot0, dot1) = (_mm256_madd_epi16(c0, p0), _mm256_madd_epi16(c1, p1));
                let (s0, s1) = (*sum).split();
                let (s0, s1): (__m256i, __m256i) = (s0.into(), s1.into());
                let lo: fearless_simd::i32x8<S> = SimdFrom::simd_from(simd, _mm256_add_epi32(s0, dot0));
                let hi: fearless_simd::i32x8<S> = SimdFrom::simd_from(simd, _mm256_add_epi32(s1, dot1));
                *sum = lo.combine(hi);
            }
        }
        _ => {
            let (lower_val, upper_val) = clamped.widen();
            let (lower_weight, upper_weight) = i16x32::from_slice(simd, out_w).widen();
            *sum += lower_val * lower_val * lower_weight;
            *sum += upper_val * upper_val * upper_weight;
        }
    }
}

#[inline(always)]
pub(super) fn screlu_dot<S: Simd>(
    simd: S,
    acc: &SideAccumulator,
    out_w: &[i16; HIDDEN_SIZE],
) -> i32x16<S> {
    let n = i16x32::<S>::LEN;
    let lanes = |b: usize, k: usize| b + k * n..b + (k + 1) * n;
    let mut sums = [i32x16::splat(simd, 0); PARALLEL_SUMS];
    for b in (0..HIDDEN_SIZE).step_by(PARALLEL_SUMS * n) {
        for k in 0..PARALLEL_SUMS {
            let v = i16x32::from_slice(simd, &acc[lanes(b, k)]);
            screlu_accumulate(simd, v, &out_w[lanes(b, k)], &mut sums[k]);
        }
    }
    sums[1..].iter().fold(sums[0], |a, &b| a + b)
}

#[inline(always)]
pub(super) fn apply_delta_screlu_dot<S: Simd, const N: usize>(
    simd: S,
    src: &SideAccumulator,
    dst: &mut SideAccumulator,
    delta: &Delta<N>,
    bucket: usize,
    out_w: &[i16; HIDDEN_SIZE],
) -> i32x16<S> {
    let n = i16x32::<S>::LEN;
    let mut sum = i32x16::splat(simd, 0);
    let lanes = |b: usize, k: usize| b + k * n..b + (k + 1) * n;
    let load = |w: &[i16], b: usize| -> [i16x32<S>; PARALLEL_SUMS] {
        std::array::from_fn(|k| i16x32::from_slice(simd, &w[lanes(b, k)]))
    };

    for b in (0..HIDDEN_SIZE).step_by(PARALLEL_SUMS * n) {
        let mut acc = load(src, b);
        for &add in delta.adds() {
            let a = load(NNUE.feature_weights(add, bucket), b);
            for k in 0..PARALLEL_SUMS {
                acc[k] += a[k];
            }
        }
        for &sub in delta.subs() {
            let s = load(NNUE.feature_weights(sub, bucket), b);
            for k in 0..PARALLEL_SUMS {
                acc[k] -= s[k];
            }
        }
        for k in 0..PARALLEL_SUMS {
            acc[k].store_slice(&mut dst[lanes(b, k)]);
            screlu_accumulate(simd, acc[k], &out_w[lanes(b, k)], &mut sum);
        }
    }
    sum
}

#[inline(always)]
pub(super) fn clone_side_accumulator<S: Simd>(
    simd: S,
    src: &SideAccumulator,
    dst: &mut SideAccumulator,
) {
    let n = S::i16s::LEN;
    for (src_slice, dst_slice) in src.chunks_exact(n).zip(dst.chunks_exact_mut(n)) {
        S::i16s::from_slice(simd, src_slice).store_slice(dst_slice);
    }
}
