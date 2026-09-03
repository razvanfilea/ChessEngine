use chess_core::prelude::*;

pub const INPUT_FEATURES: usize = 768;
pub const HIDDEN_SIZE: usize = 1024;
pub const QA: i32 = 255;
pub const QB: i32 = 64;
pub const SCALE: i32 = 400;
pub const OUTPUT_BUCKETS: usize = 8;

#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN_SIZE],
}

#[repr(C, align(64))]
pub struct Network {
    pub feature_weights: [[i16; HIDDEN_SIZE]; INPUT_FEATURES],
    pub feature_biases: [i16; HIDDEN_SIZE],
    pub output_weights: [[i16; 2 * HIDDEN_SIZE]; OUTPUT_BUCKETS],
    pub output_bias: [i16; OUTPUT_BUCKETS],
}

include!(concat!(env!("OUT_DIR"), "/nnue_data.rs"));

impl Network {
    #[inline(always)]
    pub(super) fn feature_index(perspective: Color, piece: ColoredPiece, sq: Sq) -> usize {
        let color_bit = if piece.color() == perspective { 0 } else { 1 };
        let oriented_sq = if perspective == Color::White {
            sq
        } else {
            sq.flip_vertical()
        };
        color_bit * 384 + piece.piece() as usize * 64 + oriented_sq as usize
    }

    #[inline(always)]
    pub(super) fn feature_weights(&self, index: usize) -> &[i16; HIDDEN_SIZE] {
        &self.feature_weights[index]
    }

    #[inline(always)]
    pub(super) fn bucket_index(piece_count: usize) -> usize {
        (piece_count - 2) / 32usize.div_ceil(OUTPUT_BUCKETS)
    }
}
