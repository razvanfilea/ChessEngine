use crate::{board::Board, search::StackMove};
use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i16x32, i32x16, prelude::*};

pub mod network;
pub use network::{HIDDEN_SIZE, NNUE, Network};

#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub struct Accumulator([[i16; HIDDEN_SIZE]; Color::NB]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([NNUE.feature_biases; Color::NB])
    }
}

impl Accumulator {
    #[inline(always)]
    pub fn raw(&self) -> &[[i16; HIDDEN_SIZE]; Color::NB] {
        &self.0
    }

    pub fn eval(&self, board: &Board) -> i16 {
        let level = Level::baseline();
        dispatch!(level, simd => self.eval_simd(simd, board))
    }

    #[inline(always)]
    fn eval_simd<S: Simd>(&self, simd: S, board: &Board) -> i16 {
        let n = i16x32::<S>::N;
        let us = &self.0[board.to_play as usize];
        let them = &self.0[!board.to_play as usize];

        let zero = i16x32::splat(simd, 0);
        let qa = i16x32::splat(simd, network::QA as i16);
        let mut total_sum = i32x16::splat(simd, 0);

        let bucket_index = Network::bucket_index(board.occupied().count_ones() as usize);
        let weights = &NNUE.output_weights[bucket_index];

        for (val, w) in us
            .chunks_exact(n)
            .zip(weights[..HIDDEN_SIZE].chunks_exact(n))
        {
            let val_vec = i16x32::from_slice(simd, val);
            Self::screlu_accumulate(simd, val_vec, w, zero, qa, &mut total_sum);
        }
        for (val, w) in them
            .chunks_exact(n)
            .zip(weights[HIDDEN_SIZE..].chunks_exact(n))
        {
            let val_vec = i16x32::from_slice(simd, val);
            Self::screlu_accumulate(simd, val_vec, w, zero, qa, &mut total_sum);
        }

        let mut buf = [0i32; 16];
        total_sum.store_slice(&mut buf);
        Self::finalize_output(bucket_index, buf.iter().sum::<i32>())
    }

    #[inline(always)]
    fn finalize_output(bucket_index: usize, sum: i32) -> i16 {
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
    pub fn compute_from(&mut self, parent: &Accumulator, entry: StackMove) {
        self.update::<false>(parent, entry, None);
    }

    #[inline(always)]
    pub fn compute_and_eval(
        &mut self,
        parent: &Accumulator,
        entry: StackMove,
        board: &Board,
    ) -> i16 {
        self.update::<true>(parent, entry, Some(board))
    }

    fn update<const WITH_EVAL: bool>(
        &mut self,
        parent: &Accumulator,
        entry: StackMove,
        board: Option<&Board>,
    ) -> i16 {
        let moved_piece = match entry.moved_piece {
            Some(p) => p,
            None => {
                *self = parent.clone();
                return board.map_or(0, |b| self.eval(b));
            }
        };

        let (bucket_index, white_out_w, black_out_w) = if let Some(b) = board {
            let bucket = Network::bucket_index(b.occupied().count_ones() as usize);
            let weights = &NNUE.output_weights[bucket];
            let us = &weights[..HIDDEN_SIZE];
            let them = &weights[HIDDEN_SIZE..];
            if b.to_play == Color::White {
                (bucket, us, them)
            } else {
                (bucket, them, us)
            }
        } else {
            (0, &NNUE.feature_biases[..], &NNUE.feature_biases[..])
        };

        let level = Level::baseline();
        let total_sum = dispatch!(level, simd => {
            let mut sum = i32x16::splat(simd, 0);
            self.update_simd_half::<_, WITH_EVAL>(
                simd,
                parent,
                entry.mov,
                moved_piece,
                entry.captured,
                Color::White,
                white_out_w,
                &mut sum,
            );
            self.update_simd_half::<_, WITH_EVAL>(
                simd,
                parent,
                entry.mov,
                moved_piece,
                entry.captured,
                Color::Black,
                black_out_w,
                &mut sum,
            );
            if WITH_EVAL {
                let mut buf = [0i32; 16];
                sum.store_slice(&mut buf);
                buf.iter().sum::<i32>()
            } else {
                0
            }
        });

        if WITH_EVAL {
            Self::finalize_output(bucket_index, total_sum)
        } else {
            0
        }
    }

    #[inline(always)]
    fn update_simd_half<S: Simd, const WITH_EVAL: bool>(
        &mut self,
        simd: S,
        parent: &Accumulator,
        mov: Move,
        moved_piece: ColoredPiece,
        captured: Option<ColoredPiece>,
        perspective: Color,
        output_weights: &[i16],
        total_sum: &mut i32x16<S>,
    ) {
        let n = i16x32::<S>::N;
        let from = mov.from();
        let to = mov.to();
        let flags = mov.flags();

        let w_from = NNUE.feature_weights(Network::feature_index(perspective, moved_piece, from));
        let w_to = if mov.is_promotion() {
            let promo = unsafe { mov.promotion_piece().unwrap_unchecked() };
            let promo_colored = ColoredPiece::new(promo, moved_piece.color());
            NNUE.feature_weights(Network::feature_index(perspective, promo_colored, to))
        } else {
            NNUE.feature_weights(Network::feature_index(perspective, moved_piece, to))
        };

        let w_cap = captured.map(|cap| {
            let cap_sq = mov.capture_square(moved_piece.color());
            NNUE.feature_weights(Network::feature_index(perspective, cap, cap_sq))
        });

        let w_rook = if mov.is_castle() {
            let us = moved_piece.color();
            let (rf, rt) = if flags == MoveFlags::CastleKing {
                if us == Color::White {
                    (Sq::H1, Sq::F1)
                } else {
                    (Sq::H8, Sq::F8)
                }
            } else {
                if us == Color::White {
                    (Sq::A1, Sq::D1)
                } else {
                    (Sq::A8, Sq::D8)
                }
            };
            let rook = ColoredPiece::new(Piece::Rook, us);
            Some((
                NNUE.feature_weights(Network::feature_index(perspective, rook, rf)),
                NNUE.feature_weights(Network::feature_index(perspective, rook, rt)),
            ))
        } else {
            None
        };

        let target_acc = &mut self.0[perspective as usize];
        let source_acc = &parent.0[perspective as usize];

        let zero = i16x32::splat(simd, 0);
        let qa = i16x32::splat(simd, network::QA as i16);
        let mut sum = *total_sum;

        if let Some(cap) = w_cap {
            for (((((t, s), f), to), c), out_w) in target_acc
                .chunks_exact_mut(n)
                .zip(source_acc.chunks_exact(n))
                .zip(w_from.chunks_exact(n))
                .zip(w_to.chunks_exact(n))
                .zip(cap.chunks_exact(n))
                .zip(output_weights.chunks_exact(n))
            {
                let s_vec = i16x32::from_slice(simd, s);
                let from_vec = i16x32::from_slice(simd, f);
                let to_vec = i16x32::from_slice(simd, to);
                let cap_vec = i16x32::from_slice(simd, c);
                let val_vec = s_vec - from_vec + to_vec - cap_vec;
                val_vec.store_slice(t);

                if WITH_EVAL {
                    Self::screlu_accumulate(simd, val_vec, out_w, zero, qa, &mut sum);
                }
            }
        } else if let Some((rf, rt)) = w_rook {
            for ((((((t, s), f), to), r_from), r_to), out_w) in target_acc
                .chunks_exact_mut(n)
                .zip(source_acc.chunks_exact(n))
                .zip(w_from.chunks_exact(n))
                .zip(w_to.chunks_exact(n))
                .zip(rf.chunks_exact(n))
                .zip(rt.chunks_exact(n))
                .zip(output_weights.chunks_exact(n))
            {
                let s_vec = i16x32::from_slice(simd, s);
                let from_vec = i16x32::from_slice(simd, f);
                let to_vec = i16x32::from_slice(simd, to);
                let rf_vec = i16x32::from_slice(simd, r_from);
                let rt_vec = i16x32::from_slice(simd, r_to);
                let val_vec = s_vec - from_vec + to_vec - rf_vec + rt_vec;
                val_vec.store_slice(t);

                if WITH_EVAL {
                    Self::screlu_accumulate(simd, val_vec, out_w, zero, qa, &mut sum);
                }
            }
        } else {
            // Dominant fast path: quiet moves
            for ((((t, s), f), to), out_w) in target_acc
                .chunks_exact_mut(n)
                .zip(source_acc.chunks_exact(n))
                .zip(w_from.chunks_exact(n))
                .zip(w_to.chunks_exact(n))
                .zip(output_weights.chunks_exact(n))
            {
                let s_vec = i16x32::from_slice(simd, s);
                let from_vec = i16x32::from_slice(simd, f);
                let to_vec = i16x32::from_slice(simd, to);
                let val_vec = s_vec - from_vec + to_vec;
                val_vec.store_slice(t);

                if WITH_EVAL {
                    Self::screlu_accumulate(simd, val_vec, out_w, zero, qa, &mut sum);
                }
            }
        }

        if WITH_EVAL {
            *total_sum = sum;
        }
    }

    pub fn from_board(board: &Board) -> Self {
        let mut acc = Self::default();
        let level = Level::baseline();
        dispatch!(level, simd => {
            for piece_type in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
                Piece::King,
            ] {
                for color in [Color::White, Color::Black] {
                    let cp = ColoredPiece::new(piece_type, color);
                    let bb = board.color_piece(piece_type, color);
                    for_each_bit!(sq in bb => {
                        for perspective in [Color::White, Color::Black] {
                            let idx = Network::feature_index(perspective, cp, sq);
                            let weights = NNUE.feature_weights(idx);
                            Self::add_weights(simd, &mut acc.0[perspective as usize], weights);
                        }
                    });
                }
            }
        });
        acc
    }

    #[inline(always)]
    fn add_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        weights: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::N;
        for (t, w) in target.chunks_exact_mut(n).zip(weights.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let w_vec = i16x32::from_slice(simd, w);
            (t_vec + w_vec).store_slice(t);
        }
    }
}
