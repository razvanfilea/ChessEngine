use crate::{
    board::Board,
    nnue::network::{LAYER_2_SIZE, LAYER_3_SIZE},
};
use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i8x64, i16x32, i32x16, prelude::*};

pub mod network;
pub use network::{LAYER_1_SIZE, NNUE, Network};

#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub struct Accumulator([[i16; LAYER_1_SIZE]; Color::NB]);

impl Default for Accumulator {
    fn default() -> Self {
        Self([NNUE.feature_biases; Color::NB])
    }
}

impl Accumulator {
    pub fn eval(&self, to_play: Color) -> i16 {
        let level = Level::baseline();
        dispatch!(level, simd => self.eval_simd(simd, to_play))
    }

    #[inline(always)]
fn eval_simd<S: Simd>(&self, simd: S, to_play: Color) -> i16 {
    let n_i16 = i16x32::<S>::N;
    let n_i32 = i32x16::<S>::N;
    let us = to_play as usize;
    let them = (!to_play) as usize;

    let mut layer_1 = [0i8; LAYER_1_SIZE * 2];

    let zero = i16x32::splat(simd, 0);

    let process_half = |target: &mut [i8], source: &[i16]| {
        for (t, s) in target
            .chunks_exact_mut(n_i16 * 2)
            .zip(source.chunks_exact(n_i16 * 2))
        {
            let (s0, s1) = s.split_at(n_i16);
            let v0 = i16x32::from_slice(simd, s0).max(zero);
            let v1 = i16x32::from_slice(simd, s1).max(zero);
            let packed: i8x64<S> = v0.saturating_narrow(v1);
            packed.store_slice(t);
        }
    };

    process_half(&mut layer_1[..LAYER_1_SIZE], &self.0[us]);
    process_half(&mut layer_1[LAYER_1_SIZE..LAYER_1_SIZE * 2], &self.0[them]);

    let mut layer2_out = [0i8; LAYER_2_SIZE];
    // let layer2_sum_vec=  i32x16::from_slice(simd, &layer2_sums);
    let in_size = LAYER_1_SIZE * Color::NB;

    for i in 0..LAYER_2_SIZE {
        let mut sum = NNUE.layer2_biases[i];
        let offset = i * in_size;
        for (j, weight) in layer_1.iter().enumerate() {
            sum += *weight as i32 * NNUE.layer2_weights[offset + j] as i32;
        }
        layer2_out[i] = (sum >> 6).clamp(0, 127) as i8;
    }

    let mut layer3_out = [0i8; LAYER_3_SIZE];
    for i in 0..LAYER_3_SIZE {
        let mut sum = NNUE.layer3_biases[i];
        let offset = i * LAYER_2_SIZE;

        for j in 0..LAYER_2_SIZE {
            sum += layer2_out[j] as i32 * NNUE.layer3_weights[offset + j] as i32;
        }

        // Clamp and store directly into the output array!
        layer3_out[i] = (sum >> 6).clamp(0, 127) as i8;
    }
    // {
    //     let (s0, s1) = NNUE.layer3_biases.split_at(n_i32);
    //     let v0 = i32x16::from_slice(simd, s0);
    //     let v1 = i32x16::from_slice(simd, s1);
    // }

    let mut output = NNUE.output_biases;
    for j in 0..LAYER_3_SIZE {
        output += layer3_out[j] as i32 * NNUE.output_weights[j] as i32;
    }

    // Scale down for standard centipawns (FV_SCALE = 16)
    (output / 16) as i16
}

    #[inline]
    pub fn add_piece(&mut self, king_sq: Sq, king_color: Color, piece: ColoredPiece, piece_sq: Sq) {
        let color = &mut self.0[king_color as usize];
        let weights = NNUE.get_half_kp_weight(king_sq, king_color, piece, piece_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::add_weights(simd, color, weights));
    }

    #[inline]
    pub fn move_piece(
        &mut self,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        old_sq: Sq,
        new_sq: Sq,
    ) {
        let color = &mut self.0[king_color as usize];
        let weights_old = NNUE.get_half_kp_weight(king_sq, king_color, piece, old_sq);
        let weights_new = NNUE.get_half_kp_weight(king_sq, king_color, piece, new_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::update_weights(simd, color, weights_old, weights_new));
    }

    #[inline]
    pub fn remove_piece(
        &mut self,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        piece_sq: Sq,
    ) {
        let color = &mut self.0[king_color as usize];
        let weights = NNUE.get_half_kp_weight(king_sq, king_color, piece, piece_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::remove_weights(simd, color, weights));
    }

    #[inline]
    pub fn move_king(&mut self, board: &Board, king_color: Color, new_king_sq: Sq) {
        let color_acc = &mut self.0[king_color as usize];
        *color_acc = NNUE.feature_biases;

        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
        ] {
            for color in [Color::White, Color::Black] {
                let cp = ColoredPiece::new(piece, color);
                let bb = board.color_piece(piece, color);
                for_each_bit!(sq in bb => {
                    let weights = NNUE.get_half_kp_weight(new_king_sq, king_color, cp, sq);

                    let level = Level::baseline();
                    dispatch!(level, simd => Self::add_weights(simd, color_acc, weights));
                });
            }
        }
    }

    #[inline]
    pub fn from_board(board: &Board) -> Self {
        let mut acc = Self::default();
        for color in [Color::White, Color::Black] {
            let king_sq = board.king_sq(color);
            acc.move_king(board, color, king_sq);
        }
        acc
    }

    #[inline(always)]
    fn add_weights<S: Simd>(
        simd: S,
        target: &mut [i16; LAYER_1_SIZE],
        source: &[i16; LAYER_1_SIZE],
    ) {
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
        target: &mut [i16; LAYER_1_SIZE],
        source_old: &[i16; LAYER_1_SIZE],
        source_new: &[i16; LAYER_1_SIZE],
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
        target: &mut [i16; LAYER_1_SIZE],
        source: &[i16; LAYER_1_SIZE],
    ) {
        let n = i16x32::<S>::N;
        for (t, s) in target.chunks_exact_mut(n).zip(source.chunks_exact(n)) {
            let t_vec = i16x32::from_slice(simd, t);
            let s_vec = i16x32::from_slice(simd, s);
            (t_vec - s_vec).store_slice(t);
        }
    }
}
