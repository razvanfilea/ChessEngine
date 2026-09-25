use crate::board::Board;
use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i16x32, i32x16, prelude::*};

pub mod network;
pub use network::{HIDDEN_SIZE, NNUE, Network};

type SideAccumulator = [i16; HIDDEN_SIZE];

#[repr(align(64))]
struct FinnyTableEntry {
    accum: SideAccumulator,
    occupancies: [[u64; Piece::NB]; Color::NB],
}

impl Default for FinnyTableEntry {
    fn default() -> Self {
        Self {
            accum: NNUE.feature_biases,
            occupancies: Default::default(),
        }
    }
}

#[repr(align(64))]
#[derive(Default)]
pub struct FinnyTable([FinnyTableEntry; Color::NB]);

impl FinnyTable {
    #[inline(always)]
    pub fn new(board: &Board) -> (Self, i16) {
        let mut table = Self::default();
        let eval = table.eval(board);
        (table, eval)
    }

    #[inline(always)]
    pub fn eval(&mut self, board: &Board) -> i16 {
        let bucket_index = Network::bucket_index(board.occupied().count_ones() as usize);
        let weights = &NNUE.output_weights[bucket_index];
        let (white_out_w, black_out_w) = if board.to_play == Color::White {
            (&weights[..HIDDEN_SIZE], &weights[HIDDEN_SIZE..])
        } else {
            (&weights[HIDDEN_SIZE..], &weights[..HIDDEN_SIZE])
        };

        let level = Level::baseline();
        let sum = dispatch!(level, simd => {
            let mut sum = self.update_and_eval_side(simd, board, Color::White, white_out_w);
            sum += self.update_and_eval_side(simd, board, Color::Black, black_out_w);

            sum.reduce_sum()
        });

        let mut out = sum / network::QA;
        out += NNUE.output_bias[bucket_index] as i32;
        out *= network::SCALE;
        out /= network::QA * network::QB;
        out as i16
    }

    #[inline(always)]
    fn screlu_accumulate<S: Simd>(
        simd: S,
        val_vec: i16x32<S>,
        out_w: &[i16],
        zero: i16x32<S>,
        qa: i16x32<S>,
        sum: &mut i32x16<S>,
    ) {
        let clamped = val_vec.max(zero).min(qa);

        #[cfg(all(target_arch = "x86_64", target_feature = "avx512bw"))]
        unsafe {
            use std::arch::x86_64::*;
            let c: __m512i = clamped.into();
            let w = _mm512_load_si512(out_w.as_ptr() as *const __m512i);
            let p = _mm512_mullo_epi16(c, w);
            let dot = _mm512_madd_epi16(c, p);
            let s: __m512i = (*sum).into();
            *sum = fearless_simd::SimdFrom::simd_from(simd, _mm512_add_epi32(s, dot));
        }

        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx512bw")))]
        {
            let (lower_val, upper_val) = clamped.widen();
            let (lower_weight, upper_weight) = i16x32::from_slice(simd, out_w).widen();
            *sum += lower_val * lower_val * lower_weight;
            *sum += upper_val * upper_val * upper_weight;
        }
    }

    #[inline(always)]
    fn update_and_eval_side<S: Simd>(
        &mut self,
        simd: S,
        board: &Board,
        perspective: Color,
        out_w: &[i16],
    ) -> i32x16<S> {
        let entry = &mut self.0[perspective as usize];
        let mut added_count = 0;
        let mut removed_count = 0;
        let mut added_features = [0usize; 32];
        let mut removed_features = [0usize; 32];

        let zero = i16x32::splat(simd, 0);
        let qa = i16x32::splat(simd, network::QA as i16);
        let mut total_sum = i32x16::splat(simd, 0);

        for color in [Color::White, Color::Black] {
            let color_bb = board.colors(color);
            let occupancies = &mut entry.occupancies[color as usize];

            for (piece_occupied, &piece) in occupancies.iter_mut().zip(&Piece::ALL) {
                let current_bb = color_bb & board.pieces(piece);

                if current_bb == *piece_occupied {
                    continue;
                }

                let added = current_bb & !*piece_occupied;
                let removed = *piece_occupied & !current_bb;
                *piece_occupied = current_bb;

                let colored_piece = ColoredPiece::new(piece, color);
                for_each_bit!(sq in added => {
                    if let Some(slot) = added_features.get_mut(added_count) {
                        *slot = Network::feature_index(perspective, colored_piece, sq);
                        added_count += 1;
                    }
                });

                for_each_bit!(sq in removed => {
                    if let Some(slot) = removed_features.get_mut(removed_count) {
                        *slot = Network::feature_index(perspective, colored_piece, sq);
                        removed_count += 1;
                    }
                });
            }
        }

        let n = i16x32::<S>::LEN;

        if added_count == 1 && removed_count == 1 {
            let add_w = NNUE.feature_weights(added_features[0]);
            let sub_w = NNUE.feature_weights(removed_features[0]);
            for (((t, a), s), w) in entry
                .accum
                .chunks_exact_mut(n)
                .zip(add_w.chunks_exact(n))
                .zip(sub_w.chunks_exact(n))
                .zip(out_w.chunks_exact(n))
            {
                let t_vec = i16x32::from_slice(simd, t);
                let a_vec = i16x32::from_slice(simd, a);
                let s_vec = i16x32::from_slice(simd, s);
                let val_vec = t_vec + a_vec - s_vec;
                val_vec.store_slice(t);
                Self::screlu_accumulate(simd, val_vec, w, zero, qa, &mut total_sum);
            }
        } else if added_count == 0 && removed_count == 0 {
            for (val, w) in entry.accum.chunks_exact(n).zip(out_w.chunks_exact(n)) {
                let val_vec = i16x32::from_slice(simd, val);
                Self::screlu_accumulate(simd, val_vec, w, zero, qa, &mut total_sum);
            }
        } else {
            let min_count = added_count.min(removed_count);

            for i in 0..min_count {
                let add_w = NNUE.feature_weights(added_features[i]);
                let sub_w = NNUE.feature_weights(removed_features[i]);
                Self::add_sub_weights(simd, &mut entry.accum, add_w, sub_w);
            }

            for &idx in &added_features[min_count..added_count] {
                Self::add_weights(simd, &mut entry.accum, NNUE.feature_weights(idx));
            }

            for &idx in &removed_features[min_count..removed_count] {
                Self::remove_weights(simd, &mut entry.accum, NNUE.feature_weights(idx));
            }

            for (val, w) in entry.accum.chunks_exact(n).zip(out_w.chunks_exact(n)) {
                let val_vec = i16x32::from_slice(simd, val);
                Self::screlu_accumulate(simd, val_vec, w, zero, qa, &mut total_sum);
            }
        }

        total_sum
    }

    #[inline(always)]
    fn add_sub_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        add_w: &[i16; HIDDEN_SIZE],
        sub_w: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::LEN;
        for ((t, a), s) in target
            .chunks_exact_mut(n)
            .zip(add_w.chunks_exact(n))
            .zip(sub_w.chunks_exact(n))
        {
            let t_vec = i16x32::from_slice(simd, t);
            let a_vec = i16x32::from_slice(simd, a);
            let s_vec = i16x32::from_slice(simd, s);
            (t_vec + a_vec - s_vec).store_slice(t);
        }
    }

    #[inline(always)]
    fn add_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        weights: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::LEN;
        for (t, w) in target.chunks_exact_mut(n).zip(weights.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let w_vec = i16x32::from_slice(simd, w);
            (t_vec + w_vec).store_slice(t);
        }
    }

    #[inline(always)]
    fn remove_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        weights: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::LEN;
        for (t, w) in target.chunks_exact_mut(n).zip(weights.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let w_vec = i16x32::from_slice(simd, w);
            (t_vec - w_vec).store_slice(t);
        }
    }
}
