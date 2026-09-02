use std::hint::assert_unchecked;

use chess_core::prelude::*;

pub const INPUT_FEATURES: usize = 41024;
pub const LAYER_1_SIZE: usize = 256;
pub const LAYER_2_SIZE: usize = 32;
pub const LAYER_3_SIZE: usize = 32;

#[repr(C, align(64))]
pub struct Network {
    pub ft_hash: u32,
    pub feature_biases: [i16; LAYER_1_SIZE],
    pub feature_weights: [i16; INPUT_FEATURES * LAYER_1_SIZE],
    pub net_hash: u32,
    pub layer2_biases: [i32; LAYER_2_SIZE],
    pub layer2_weights: [i8; LAYER_1_SIZE * Color::NB * LAYER_2_SIZE],
    pub layer3_biases: [i32; LAYER_3_SIZE],
    pub layer3_weights: [i8; LAYER_2_SIZE * LAYER_3_SIZE],
    pub output_biases: i32,
    pub output_weights: [i8; LAYER_3_SIZE],
}

include!(concat!(env!("OUT_DIR"), "/nnue_data.rs"));

impl Network {
    #[inline(always)]
    pub(super) fn get_half_kp_weight(
        &self,
        king_sq: Sq,
        king_color: Color,
        piece: ColoredPiece,
        piece_sq: Sq,
    ) -> &[i16; LAYER_1_SIZE] {
        let typ = piece.piece();
        debug_assert!(typ != Piece::King);
        let piece_index = typ as usize * 2 + if king_color == piece.color() { 0 } else { 1 };

        let (oriented_ksq, oriented_psq) = if king_color == Color::White {
            (king_sq, piece_sq)
        } else {
            (king_sq.rotate_180(), piece_sq.rotate_180())
        };

        let index = (oriented_ksq as usize * 641) + (piece_index * 64) + oriented_psq as usize + 1;
        let offset = index * LAYER_1_SIZE;

        unsafe {
            self.feature_weights[offset..offset + LAYER_1_SIZE]
                .try_into()
                .unwrap_unchecked()
        }
    }
}
