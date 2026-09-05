use chess_core::prelude::*;
use super::history::KillerMoves;

#[derive(Default, Clone, Copy)]
pub(super) struct StackEntry {
    pub killer_moves: KillerMoves,
    pub eval: i16,
    pub pv_length: u16,
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

