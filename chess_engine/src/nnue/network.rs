use std::hint::assert_unchecked;

use chess_core::prelude::*;

pub const INPUT_BUCKETS: usize = 4;
pub const INPUT_FEATURES: usize = 768 * INPUT_BUCKETS;
pub const HIDDEN_SIZE: usize = 1536;
pub const QA: i32 = 255;
pub const QB: i32 = 64;
pub const SCALE: i32 = 400;
pub const OUTPUT_BUCKETS: usize = 8;

#[rustfmt::skip]
pub const BUCKET_LAYOUT: [usize; 32] = [
    0, 0, 0, 1, // rank 1: a1, b1, c1 | d1 (center)
    0, 0, 0, 1, // rank 2: a2, b2, c2 | d2 (center)
    2, 2, 2, 2, // rank 3: midfield
    2, 2, 2, 2, // rank 4: midfield
    3, 3, 3, 3, // rank 5: endgame
    3, 3, 3, 3, // rank 6
    3, 3, 3, 3, // rank 7
    3, 3, 3, 3, // rank 8
];

#[repr(C, align(64))]
pub struct Network {
    pub feature_weights: [[[i16; HIDDEN_SIZE]; 768]; INPUT_BUCKETS],
    pub feature_biases: [i16; HIDDEN_SIZE],
    pub output_weights: [[i16; 2 * HIDDEN_SIZE]; OUTPUT_BUCKETS],
    pub output_bias: [i16; OUTPUT_BUCKETS],
}

include!(concat!(env!("OUT_DIR"), "/nnue_data.rs"));

impl Network {
    #[inline(always)]
    pub(super) fn feature_index(
        perspective: Color,
        piece: ColoredPiece,
        sq: Sq,
        flip: u8,
    ) -> usize {
        let color_bit = if piece.color() == perspective { 0 } else { 1 };
        let oriented_sq = if perspective == Color::White {
            sq
        } else {
            sq.flip_vertical()
        };
        color_bit * 384 + piece.piece() as usize * 64 + (oriented_sq as u8 ^ flip) as usize
    }

    #[inline]
    pub fn king_bucket_and_flip(perspective: Color, king_sq: Sq) -> (usize, u8) {
        let king_sq = if perspective == Color::White {
            king_sq
        } else {
            king_sq.flip_vertical()
        };
        let file = king_sq.file();
        let rank = king_sq.rank();

        let flip = if file > 3 { 7 } else { 0 };
        let index = (rank * 4) + (file ^ flip);

        (BUCKET_LAYOUT[index as usize], flip)
    }

    #[inline(always)]
    pub(super) fn feature_weights(&self, piece_idx: usize, bucket: usize) -> &[i16; HIDDEN_SIZE] {
        debug_assert!(piece_idx < 768);
        debug_assert!(bucket < INPUT_BUCKETS);
        unsafe {
            self.feature_weights
                .get_unchecked(bucket)
                .get_unchecked(piece_idx)
        }
    }

    #[inline(always)]
    pub(super) fn bucket_index(piece_count: usize) -> usize {
        let bucket = (piece_count - 2) / 32usize.div_ceil(OUTPUT_BUCKETS);
        unsafe {
            assert_unchecked(bucket < OUTPUT_BUCKETS);
        }
        bucket
    }
}
