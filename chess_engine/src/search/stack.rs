use crate::search::EVAL_NONE;

use super::history::KillerMoves;
use chess_core::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct StackEntry {
    pub killer_moves: KillerMoves,
    pub eval: i16,
    pub pv_length: u16,

    pub ply_move: PlyMove,

    pub acc_computed: bool,
}

impl Default for StackEntry {
    #[inline(always)]
    fn default() -> Self {
        Self {
            killer_moves: [Move::NONE; 2],
            eval: EVAL_NONE,
            pv_length: 0,
            ply_move: PlyMove::default(),
            acc_computed: false,
        }
    }
}

impl StackEntry {
    #[inline(always)]
    pub fn set_move(
        &mut self,
        mov: Move,
        moved_piece: Option<ColoredPiece>,
        captured: Option<ColoredPiece>,
    ) {
        self.ply_move = PlyMove::new(mov, moved_piece, captured);
        self.acc_computed = false;
    }

    #[inline(always)]
    pub fn set_null_move(&mut self) {
        self.ply_move = PlyMove::default();
        self.acc_computed = false;
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct PlyMove {
    pub mov: Move,
    pub moved_piece: Option<ColoredPiece>,
    pub captured: Option<ColoredPiece>,
}

impl PlyMove {
    #[inline(always)]
    pub const fn new(
        mov: Move,
        moved_piece: Option<ColoredPiece>,
        captured: Option<ColoredPiece>,
    ) -> Self {
        Self {
            mov,
            moved_piece,
            captured,
        }
    }
}
