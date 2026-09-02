use crate::{
    board::Board,
    nnue::network::{LAYER_1_SIZE, Network},
};

use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i16x32, prelude::*};

mod network;

#[derive(Clone)]
#[repr(C, align(64))]
pub struct Accumulator([[i16; LAYER_1_SIZE]; Color::NB]);

impl Accumulator {
    pub fn new(network: &Network) -> Self {
        Self([*network.feature_biases; Color::NB])
    }

    #[inline]
    pub fn add_piece(
        &mut self,
        network: &Network,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        piece_sq: Sq,
    ) {
        let color = &mut self.0[king_color as usize];
        let weights = network.get_half_kp_weight(king_sq, king_color, piece, piece_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::add_weights(simd, color, weights));
    }

    #[inline]
    pub fn move_piece(
        &mut self,
        network: &Network,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        old_sq: Sq,
        new_sq: Sq,
    ) {
        let color = &mut self.0[king_color as usize];
        let weights_old = network.get_half_kp_weight(king_sq, king_color, piece, old_sq);
        let weights_new = network.get_half_kp_weight(king_sq, king_color, piece, new_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::update_weights(simd, color, weights_old, weights_new));
    }

    #[inline]
    pub fn remove_piece(
        &mut self,
        network: &Network,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        piece_sq: Sq,
    ) {
        let color = &mut self.0[king_color as usize];
        let weights = network.get_half_kp_weight(king_sq, king_color, piece, piece_sq);

        let level = Level::baseline();
        dispatch!(level, simd => Self::remove_weights(simd, color, weights));
    }

    #[inline(never)]
    pub fn move_king(
        &mut self,
        network: &Network,
        board: &Board,
        king_color: Color,
        new_king_sq: Sq,
    ) {
        let color_acc = &mut self.0[king_color as usize];
        *color_acc = *network.feature_biases;

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
                    let weights = network.get_half_kp_weight(new_king_sq, king_color, cp, sq);

                    let level = Level::baseline();
                    dispatch!(level, simd => Self::add_weights(simd, color_acc, weights));
                });
            }
        }
    }

    #[inline(never)]
    pub fn from_board(network: &Network, board: &crate::board::Board) -> Self {
        let mut acc = Self::new(network);
        for color in [Color::White, Color::Black] {
            let king_sq = board.king_sq(color);
            acc.move_king(network, board, color, king_sq);
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
