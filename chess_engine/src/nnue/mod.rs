use crate::board::Board;
use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i16x32, i32x16, prelude::*};

pub mod network;
pub use network::{HIDDEN_SIZE, INPUT_BUCKETS, NNUE, Network};

type SideAccumulator = [i16; HIDDEN_SIZE];

#[repr(align(64))]
struct FinnyTableEntry {
    accum: SideAccumulator,
    occupancies: [u64; 12],
}

impl Default for FinnyTableEntry {
    fn default() -> Self {
        Self {
            accum: NNUE.feature_biases,
            occupancies: [0; 12],
        }
    }
}

const PIECE_LOOKUP: [ColoredPiece; 12] = [
    ColoredPiece::new(Piece::Pawn, Color::White),
    ColoredPiece::new(Piece::Knight, Color::White),
    ColoredPiece::new(Piece::Bishop, Color::White),
    ColoredPiece::new(Piece::Rook, Color::White),
    ColoredPiece::new(Piece::Queen, Color::White),
    ColoredPiece::new(Piece::King, Color::White),
    ColoredPiece::new(Piece::Pawn, Color::Black),
    ColoredPiece::new(Piece::Knight, Color::Black),
    ColoredPiece::new(Piece::Bishop, Color::Black),
    ColoredPiece::new(Piece::Rook, Color::Black),
    ColoredPiece::new(Piece::Queen, Color::Black),
    ColoredPiece::new(Piece::King, Color::Black),
];

#[repr(align(64))]
#[derive(Default)]
pub struct FinnyTable([[[FinnyTableEntry; 2]; INPUT_BUCKETS]; Color::NB]);

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

        let mut occupancies = [0u64; 12];
        let white_bb = board.colors(Color::White);
        let black_bb = board.colors(Color::Black);
        for (i, &piece) in Piece::ALL.iter().enumerate() {
            let p_bb = board.pieces(piece);
            occupancies[i] = white_bb & p_bb;
            occupancies[i + 6] = black_bb & p_bb;
        }

        let level = Level::baseline();
        let sum = dispatch!(level, simd => {
            let mut sum = self.update_and_eval_side(simd, board, &occupancies, Color::White, white_out_w);
            sum += self.update_and_eval_side(simd, board, &occupancies, Color::Black, black_out_w);

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
        occupancies: &[u64; 12],
        perspective: Color,
        out_w: &[i16],
    ) -> i32x16<S> {
        let (bucket, flip) = Network::king_bucket_and_flip(perspective, board.king_sq(perspective));
        let flip_idx = if flip != 0 { 1 } else { 0 };
        let entry = &mut self.0[perspective as usize][bucket][flip_idx];

        let mut added_count = 0;
        let mut removed_count = 0;
        let mut added_features = [0usize; 32];
        let mut removed_features = [0usize; 32];

        let zero = i16x32::splat(simd, 0);
        let qa = i16x32::splat(simd, network::QA as i16);
        let mut total_sum = i32x16::splat(simd, 0);

        for (i, &current_bb) in occupancies.iter().enumerate() {
            let cached_bb = &mut entry.occupancies[i];
            if current_bb == *cached_bb {
                continue;
            }

            let added = current_bb & !*cached_bb;
            let removed = *cached_bb & !current_bb;
            *cached_bb = current_bb;

            let colored_piece = PIECE_LOOKUP[i];
            for_each_bit!(sq in added => {
                let index = Network::feature_index(perspective, colored_piece, sq, flip);
                unsafe { *added_features.get_unchecked_mut(added_count) = index };
                added_count += 1;
            });

            for_each_bit!(sq in removed => {
                let index = Network::feature_index(perspective, colored_piece, sq, flip);
                unsafe { *removed_features.get_unchecked_mut(removed_count) = index };
                removed_count += 1;
            });
        }

        let n = i16x32::<S>::LEN;
        const K: usize = 4;

        // Fast path 1: zero features changed. Pure read-only streaming, no store-slice!
        if added_count == 0 && removed_count == 0 {
            for b in (0..HIDDEN_SIZE).step_by(K * n) {
                let acc0 = i16x32::from_slice(simd, &entry.accum[b..b + n]);
                let acc1 = i16x32::from_slice(simd, &entry.accum[b + n..b + 2 * n]);
                let acc2 = i16x32::from_slice(simd, &entry.accum[b + 2 * n..b + 3 * n]);
                let acc3 = i16x32::from_slice(simd, &entry.accum[b + 3 * n..b + 4 * n]);

                Self::screlu_accumulate(simd, acc0, &out_w[b..b + n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc1, &out_w[b + n..b + 2 * n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc2, &out_w[b + 2 * n..b + 3 * n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc3, &out_w[b + 3 * n..b + 4 * n], zero, qa, &mut total_sum);
            }
            return total_sum;
        }

        // Fast path 2: exactly 1 quiet move (1 add, 1 remove)
        if added_count == 1 && removed_count == 1 {
            let add_w = NNUE.feature_weights(added_features[0], bucket);
            let sub_w = NNUE.feature_weights(removed_features[0], bucket);

            for b in (0..HIDDEN_SIZE).step_by(K * n) {
                let acc0 = i16x32::from_slice(simd, &entry.accum[b..b + n])
                    + i16x32::from_slice(simd, &add_w[b..b + n])
                    - i16x32::from_slice(simd, &sub_w[b..b + n]);
                let acc1 = i16x32::from_slice(simd, &entry.accum[b + n..b + 2 * n])
                    + i16x32::from_slice(simd, &add_w[b + n..b + 2 * n])
                    - i16x32::from_slice(simd, &sub_w[b + n..b + 2 * n]);
                let acc2 = i16x32::from_slice(simd, &entry.accum[b + 2 * n..b + 3 * n])
                    + i16x32::from_slice(simd, &add_w[b + 2 * n..b + 3 * n])
                    - i16x32::from_slice(simd, &sub_w[b + 2 * n..b + 3 * n]);
                let acc3 = i16x32::from_slice(simd, &entry.accum[b + 3 * n..b + 4 * n])
                    + i16x32::from_slice(simd, &add_w[b + 3 * n..b + 4 * n])
                    - i16x32::from_slice(simd, &sub_w[b + 3 * n..b + 4 * n]);

                acc0.store_slice(&mut entry.accum[b..b + n]);
                acc1.store_slice(&mut entry.accum[b + n..b + 2 * n]);
                acc2.store_slice(&mut entry.accum[b + 2 * n..b + 3 * n]);
                acc3.store_slice(&mut entry.accum[b + 3 * n..b + 4 * n]);

                Self::screlu_accumulate(simd, acc0, &out_w[b..b + n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc1, &out_w[b + n..b + 2 * n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc2, &out_w[b + 2 * n..b + 3 * n], zero, qa, &mut total_sum);
                Self::screlu_accumulate(simd, acc3, &out_w[b + 3 * n..b + 4 * n], zero, qa, &mut total_sum);
            }
            return total_sum;
        }

        // General multi-piece diff path
        let min_count = added_count.min(removed_count);

        for b in (0..HIDDEN_SIZE).step_by(K * n) {
            let mut acc0 = i16x32::from_slice(simd, &entry.accum[b..b + n]);
            let mut acc1 = i16x32::from_slice(simd, &entry.accum[b + n..b + 2 * n]);
            let mut acc2 = i16x32::from_slice(simd, &entry.accum[b + 2 * n..b + 3 * n]);
            let mut acc3 = i16x32::from_slice(simd, &entry.accum[b + 3 * n..b + 4 * n]);

            for i in 0..min_count {
                let add_w = NNUE.feature_weights(added_features[i], bucket);
                let sub_w = NNUE.feature_weights(removed_features[i], bucket);

                acc0 = acc0 + i16x32::from_slice(simd, &add_w[b..b + n])
                    - i16x32::from_slice(simd, &sub_w[b..b + n]);
                acc1 = acc1 + i16x32::from_slice(simd, &add_w[b + n..b + 2 * n])
                    - i16x32::from_slice(simd, &sub_w[b + n..b + 2 * n]);
                acc2 = acc2 + i16x32::from_slice(simd, &add_w[b + 2 * n..b + 3 * n])
                    - i16x32::from_slice(simd, &sub_w[b + 2 * n..b + 3 * n]);
                acc3 = acc3 + i16x32::from_slice(simd, &add_w[b + 3 * n..b + 4 * n])
                    - i16x32::from_slice(simd, &sub_w[b + 3 * n..b + 4 * n]);
            }

            for &feat in &added_features[min_count..added_count] {
                let add_w = NNUE.feature_weights(feat, bucket);
                acc0 += i16x32::from_slice(simd, &add_w[b..b + n]);
                acc1 += i16x32::from_slice(simd, &add_w[b + n..b + 2 * n]);
                acc2 += i16x32::from_slice(simd, &add_w[b + 2 * n..b + 3 * n]);
                acc3 += i16x32::from_slice(simd, &add_w[b + 3 * n..b + 4 * n]);
            }

            for &feat in &removed_features[min_count..removed_count] {
                let sub_w = NNUE.feature_weights(feat, bucket);
                acc0 -= i16x32::from_slice(simd, &sub_w[b..b + n]);
                acc1 -= i16x32::from_slice(simd, &sub_w[b + n..b + 2 * n]);
                acc2 -= i16x32::from_slice(simd, &sub_w[b + 2 * n..b + 3 * n]);
                acc3 -= i16x32::from_slice(simd, &sub_w[b + 3 * n..b + 4 * n]);
            }

            acc0.store_slice(&mut entry.accum[b..b + n]);
            acc1.store_slice(&mut entry.accum[b + n..b + 2 * n]);
            acc2.store_slice(&mut entry.accum[b + 2 * n..b + 3 * n]);
            acc3.store_slice(&mut entry.accum[b + 3 * n..b + 4 * n]);

            Self::screlu_accumulate(simd, acc0, &out_w[b..b + n], zero, qa, &mut total_sum);
            Self::screlu_accumulate(
                simd,
                acc1,
                &out_w[b + n..b + 2 * n],
                zero,
                qa,
                &mut total_sum,
            );
            Self::screlu_accumulate(
                simd,
                acc2,
                &out_w[b + 2 * n..b + 3 * n],
                zero,
                qa,
                &mut total_sum,
            );
            Self::screlu_accumulate(
                simd,
                acc3,
                &out_w[b + 3 * n..b + 4 * n],
                zero,
                qa,
                &mut total_sum,
            );
        }

        total_sum
    }
}
