use chess_core::prelude::*;

pub const INPUT_FEATURES: usize = 768;
pub const HIDDEN_SIZE: usize = 1024;
pub const QA: i32 = 255;
pub const QB: i32 = 64;
pub const SCALE: i32 = 400;

#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN_SIZE],
}

#[repr(C, align(64))]
pub struct Network {
    pub feature_weights: [[i16; HIDDEN_SIZE]; INPUT_FEATURES],
    pub feature_biases: [i16; HIDDEN_SIZE],
    pub output_weights: [i16; 2 * HIDDEN_SIZE],
    pub output_bias: i16,
}

include!(concat!(env!("OUT_DIR"), "/nnue_data.rs"));

impl Network {
    #[inline(always)]
    pub fn feature_index(perspective: Color, piece: ColoredPiece, sq: Sq) -> usize {
        let color_bit = if piece.color() == perspective { 0 } else { 1 };
        let oriented_sq = if perspective == Color::White {
            sq as usize
        } else {
            sq as usize ^ 56
        };
        color_bit * 384 + piece.piece() as usize * 64 + oriented_sq
    }

    #[inline(always)]
    pub fn feature_weights(&self, index: usize) -> &[i16; HIDDEN_SIZE] {
        &self.feature_weights[index]
    }
}
