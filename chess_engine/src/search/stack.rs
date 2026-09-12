use crate::search::EVAL_NONE;

use super::history::{ContHistPtr, KillerMoves};
use chess_core::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct StackEntry {
    pub killer_moves: KillerMoves,
    pub eval: i16,
    pub pv_length: u16,
    pub stack_move: StackMove,
    pub conthist: ContHistPtr,
    pub acc_computed: bool,
}

impl Default for StackEntry {
    #[inline(always)]
    fn default() -> Self {
        Self {
            killer_moves: [Move::NONE; 2],
            eval: EVAL_NONE,
            pv_length: 0,
            stack_move: StackMove::default(),
            conthist: None,
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
        self.stack_move = StackMove::new(mov, moved_piece, captured);
        self.acc_computed = false;
    }

    #[inline(always)]
    pub fn set_null_move(&mut self) {
        self.stack_move = StackMove::default();
        self.conthist = None;
        self.acc_computed = false;
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct StackMove {
    pub mov: Move,
    pub moved_piece: Option<ColoredPiece>,
    pub captured: Option<ColoredPiece>,
}

impl StackMove {
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
