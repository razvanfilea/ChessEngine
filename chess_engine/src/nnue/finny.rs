//! Finny table: one cached accumulator per (perspective, king bucket, mirror), refreshed by
//! diffing piece bitboards against the position it last saw.

use super::{
    constants::*,
    kernels::{Delta, apply_delta_screlu_dot},
    network::{NNUE, Network, SideAccumulator},
};
use crate::{board::Board, nnue::kernels::clone_side_accumulator};
use chess_core::{for_each_bit, prelude::*};
use fearless_simd::{Level, Simd, dispatch, i32x16, prelude::*};

#[repr(align(64))]
pub(super) struct FinnyTableEntry {
    pub(super) accum: SideAccumulator,
    occupancies: [u64; ColoredPiece::NB],
}

impl Default for FinnyTableEntry {
    fn default() -> Self {
        Self {
            accum: NNUE.feature_biases,
            occupancies: [0; ColoredPiece::NB],
        }
    }
}

#[repr(align(64))]
#[derive(Default)]
pub struct FinnyTable([[[FinnyTableEntry; 2]; INPUT_BUCKETS]; Color::NB]);

impl FinnyTable {
    #[inline(always)]
    pub(super) fn entry(
        &mut self,
        perspective: Color,
        bucket: usize,
        flip: u8,
    ) -> &mut FinnyTableEntry {
        &mut self.0[perspective as usize][bucket][(flip != 0) as usize]
    }

    pub fn eval(&mut self, board: &Board) -> i16 {
        let (out_bucket, white_w, black_w) = NNUE.output_layer(board);
        let mut tmp = [[0i16; HIDDEN_SIZE]; Color::NB];
        let sum = dispatch!(Level::baseline(), simd => {
            let [w, b] = &mut tmp;
            let sum1 = self.refresh(simd, board, Color::White, w, white_w);
            let sum2 = self.refresh(simd, board, Color::Black, b, black_w);
            (sum1 + sum2).reduce_sum()
        });
        NNUE.finalize(out_bucket, sum)
    }

    /// Brings the entry for `perspective`'s current king bucket up to date with `board`
    #[inline(never)]
    pub(super) fn refresh<S: Simd>(
        &mut self,
        simd: S,
        board: &Board,
        perspective: Color,
        dst: &mut SideAccumulator,
        out_w: &[i16; HIDDEN_SIZE],
    ) -> i32x16<S> {
        let (bucket, flip) = Network::king_bucket_and_flip(perspective, board.king_sq(perspective));
        let entry = self.entry(perspective, bucket, flip);

        let mut delta = Delta::<32>::new();
        for (i, &piece) in ColoredPiece::ALL.iter().enumerate() {
            let current = board.colors(piece.color()) & board.pieces(piece.piece());
            let cached = std::mem::replace(&mut entry.occupancies[i], current);
            let index = |sq| Network::feature_index(perspective, piece, sq, flip);
            for_each_bit!(sq in current & !cached => { delta.add(index(sq)) });
            for_each_bit!(sq in cached & !current => { delta.sub(index(sq)) });
        }

        let sum = apply_delta_screlu_dot(simd, &entry.accum, dst, &delta, bucket, out_w);
        clone_side_accumulator(simd, dst, &mut entry.accum);
        sum
    }
}
