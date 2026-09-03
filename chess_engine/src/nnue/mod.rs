use crate::board::Board;
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
    pub fn eval(&self, board: &Board) -> i16 {
        let level = Level::baseline();
        dispatch!(level, simd => self.eval_simd(simd, board))
    }

    #[inline(always)]
    fn eval_simd<S: Simd>(&self, simd: S, board: &Board) -> i16 {
        let n_i16 = i16x32::<S>::N;
        let us = &self.0[board.to_play as usize];
        let them = &self.0[!board.to_play as usize];

        let mut total_sum = i32x16::splat(simd, 0);
        let mut screlu_half = |values: &[i16], weights: &[i16]| {
            let zero = i16x32::splat(simd, 0);
            let max_val_vec = i32x16::splat(simd, network::QA);

            for (value, weight) in values.chunks_exact(n_i16).zip(weights.chunks_exact(n_i16)) {
                let val_vec = i16x32::from_slice(simd, value).max(zero);
                let (lower_val, upper_val) = val_vec.widen();
                let lower_val = lower_val.min(max_val_vec);
                let upper_val = upper_val.min(max_val_vec);

                let (lower_weight, upper_weight) = i16x32::from_slice(simd, weight).widen();

                total_sum += lower_val * lower_val * lower_weight;
                total_sum += upper_val * upper_val * upper_weight;
            }
        };

        let bucket_index = Network::bucket_index(board.occupied().count_ones() as usize);
        let weights = &NNUE.output_weights[bucket_index];
        screlu_half(us, &weights[..HIDDEN_SIZE]);
        screlu_half(them, &weights[HIDDEN_SIZE..]);

        let mut buf = [0i32; 16];
        total_sum.store_slice(&mut buf);
        let mut out = buf.iter().sum::<i32>();

        out /= network::QA;
        out += NNUE.output_bias[bucket_index] as i32;
        out *= network::SCALE;
        out /= network::QA * network::QB;
        out as i16
    }

    #[inline]
    pub fn add_piece(&mut self, piece: ColoredPiece, sq: Sq) {
        let level = Level::baseline();
        for perspective in [Color::White, Color::Black] {
            let idx = Network::feature_index(perspective, piece, sq);
            let weights = NNUE.feature_weights(idx);
            let acc = &mut self.0[perspective as usize];
            dispatch!(level, simd => Self::add_weights(simd, acc, weights));
        }
    }

    #[inline]
    pub fn remove_piece(&mut self, piece: ColoredPiece, sq: Sq) {
        let level = Level::baseline();
        for perspective in [Color::White, Color::Black] {
            let idx = Network::feature_index(perspective, piece, sq);
            let weights = NNUE.feature_weights(idx);
            let acc = &mut self.0[perspective as usize];
            dispatch!(level, simd => Self::remove_weights(simd, acc, weights));
        }
    }

    #[inline]
    pub fn move_piece(&mut self, piece: ColoredPiece, from: Sq, to: Sq) {
        let level = Level::baseline();
        for perspective in [Color::White, Color::Black] {
            let idx_old = Network::feature_index(perspective, piece, from);
            let idx_new = Network::feature_index(perspective, piece, to);
            let weights_old = NNUE.feature_weights(idx_old);
            let weights_new = NNUE.feature_weights(idx_new);
            let acc = &mut self.0[perspective as usize];
            dispatch!(level, simd => Self::update_weights(simd, acc, weights_old, weights_new));
        }
    }

    pub fn from_board(board: &Board) -> Self {
        let mut acc = Self::default();
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
                    acc.add_piece(cp, sq);
                });
            }
        }
        acc
    }

    #[inline(always)]
    fn add_weights<S: Simd>(simd: S, target: &mut [i16; HIDDEN_SIZE], source: &[i16; HIDDEN_SIZE]) {
        let n = i16x32::<S>::N;
        for (t, s) in target.chunks_exact_mut(n).zip(source.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let s_vec = i16x32::from_slice(simd, s);
            (t_vec + s_vec).store_slice(t);
        }
    }

    #[inline(always)]
    fn update_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        source_old: &[i16; HIDDEN_SIZE],
        source_new: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::N;
        for ((t, s_old), s_new) in target
            .chunks_exact_mut(n)
            .zip(source_old.chunks_exact(n))
            .zip(source_new.chunks_exact(n))
        {
            let t_vec = i16x32::from_slice(simd, t);
            let s_old_vec = i16x32::from_slice(simd, s_old);
            let s_new_vec = i16x32::from_slice(simd, s_new);
            (t_vec - s_old_vec + s_new_vec).store_slice(t);
        }
    }

    #[inline(always)]
    fn remove_weights<S: Simd>(
        simd: S,
        target: &mut [i16; HIDDEN_SIZE],
        source: &[i16; HIDDEN_SIZE],
    ) {
        let n = i16x32::<S>::N;
        for (t, s) in target.chunks_exact_mut(n).zip(source.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let s_vec = i16x32::from_slice(simd, s);
            (t_vec - s_vec).store_slice(t);
        }
    }
}
