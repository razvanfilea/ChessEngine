use crate::search::{EVAL_NONE, MAX_PLY};

use super::history::KillerMoves;
use chess_core::prelude::*;

pub const STACK_ENTRIES_EXTRA_SIZE: usize = 32;
pub const STACK_OFFSET: usize = STACK_ENTRIES_EXTRA_SIZE / 2;
pub const STACK_SIZE: usize = MAX_PLY as usize + STACK_ENTRIES_EXTRA_SIZE;

#[derive(Clone)]
pub struct SearchStack {
    entries: [StackEntry; STACK_SIZE],
}

impl Default for SearchStack {
    fn default() -> Self {
        Self {
            entries: [StackEntry::default(); STACK_SIZE],
        }
    }
}

impl SearchStack {
    #[inline(always)]
    pub fn get(&self, ply: u16) -> &StackEntry {
        let idx = STACK_OFFSET + ply as usize;
        debug_assert!(idx < STACK_SIZE);
        unsafe { self.entries.get_unchecked(idx) }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, ply: u16) -> &mut StackEntry {
        let idx = STACK_OFFSET + ply as usize;
        debug_assert!(idx < STACK_SIZE);
        unsafe { self.entries.get_unchecked_mut(idx) }
    }

    #[inline(always)]
    pub fn relative(&self, ply: u16, offset: i32) -> &StackEntry {
        let idx = STACK_OFFSET as i32 + ply as i32 + offset;
        debug_assert!(idx >= 0 && (idx as usize) < STACK_SIZE);
        unsafe { self.entries.get_unchecked(idx as usize) }
    }

    #[inline(always)]
    pub fn relative_mut(&mut self, ply: u16, offset: i32) -> &mut StackEntry {
        let idx = STACK_OFFSET as i32 + ply as i32 + offset;
        debug_assert!(idx >= 0 && (idx as usize) < STACK_SIZE);
        unsafe { self.entries.get_unchecked_mut(idx as usize) }
    }

    #[inline(always)]
    pub fn clear_killers(&mut self, ply: u16) {
        self.get_mut(ply).killer_moves = [Move::NONE; 2];
    }

    #[inline(always)]
    pub fn get_killers(&self, ply: u16) -> KillerMoves {
        self.get(ply).killer_moves
    }

    #[inline(always)]
    pub fn set_killer(&mut self, ply: u16, mov: Move) {
        let [first_killer, second_killer] = &mut self.get_mut(ply).killer_moves;
        if *first_killer == mov {
            return;
        }
        *second_killer = *first_killer;
        *first_killer = mov;
    }
}

impl std::ops::Index<u16> for SearchStack {
    type Output = StackEntry;
    #[inline(always)]
    fn index(&self, ply: u16) -> &Self::Output {
        self.get(ply)
    }
}

impl std::ops::IndexMut<u16> for SearchStack {
    #[inline(always)]
    fn index_mut(&mut self, ply: u16) -> &mut Self::Output {
        self.get_mut(ply)
    }
}

#[derive(Clone, Copy)]
pub struct StackEntry {
    pub killer_moves: KillerMoves,
    pub eval: i16,
    pub pv_length: u16,
    pub stack_move: StackMove,
    pub hash: u64,
}

impl Default for StackEntry {
    #[inline(always)]
    fn default() -> Self {
        Self {
            killer_moves: [Move::NONE; 2],
            eval: EVAL_NONE,
            pv_length: 0,
            stack_move: StackMove::default(),
            hash: 0,
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
    }

    #[inline(always)]
    pub fn set_null_move(&mut self) {
        self.stack_move = StackMove::default();
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

    #[inline(always)]
    pub fn piece_to(&self) -> Option<(Piece, Sq)> {
        self.moved_piece.map(|cp| (cp.piece(), self.mov.to()))
    }
}
