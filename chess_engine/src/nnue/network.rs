use std::hint::assert_unchecked;

use super::constants::*;
use chess_core::prelude::*;

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
        let piece_color = piece.color() as usize;
        let piece_type = piece.piece() as usize;
        let pers = perspective as usize;
        let color_offset = (piece_color ^ pers) * 384;
        let v_flip = (pers ^ 1) * 56;
        let oriented_sq = (sq as usize) ^ v_flip ^ (flip as usize);
        color_offset + piece_type * 64 + oriented_sq
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
    pub fn feature_weights(&self, piece_idx: usize, bucket: usize) -> &[i16; HIDDEN_SIZE] {
        debug_assert!(piece_idx < 768);
        debug_assert!(bucket < INPUT_BUCKETS);
        unsafe {
            self.feature_weights
                .get_unchecked(bucket)
                .get_unchecked(piece_idx)
        }
    }

    #[inline(always)]
    pub fn bucket_index(piece_count: usize) -> usize {
        let bucket = (piece_count - 2) / 32usize.div_ceil(OUTPUT_BUCKETS);
        unsafe {
            assert_unchecked(bucket < OUTPUT_BUCKETS);
        }
        bucket
    }
}
