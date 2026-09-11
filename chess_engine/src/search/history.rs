use std::{mem::MaybeUninit, ptr::NonNull};

use super::params::{MAX_HISTORY, MAX_KILLER_MOVES};
use chess_core::prelude::*;

#[inline(always)]
fn gravity(entry: &mut i16, bonus: i32) {
    let current = *entry as i32;
    let abs = bonus.abs().min(MAX_HISTORY);
    *entry =
        (current + bonus - current * abs / MAX_HISTORY).clamp(-MAX_HISTORY, MAX_HISTORY) as i16;
}

#[inline(always)]
fn depth_bonus(depth: u8) -> i32 {
    (depth as i32 * depth as i32).min(MAX_HISTORY)
}

pub struct HistoryTable([[[i16; Sq::NB]; Sq::NB]; Color::NB]);

impl Default for HistoryTable {
    fn default() -> Self {
        Self([[[0; Sq::NB]; Sq::NB]; Color::NB])
    }
}

impl HistoryTable {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    #[inline(always)]
    pub fn get(&self, side: Color, from: Sq, to: Sq) -> i16 {
        self.0[side as usize][from as usize][to as usize]
    }

    #[inline(always)]
    pub fn update_bonus(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        gravity(
            &mut self.0[side as usize][from as usize][to as usize],
            depth_bonus(depth),
        );
    }

    #[inline(always)]
    pub fn update_malus(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        gravity(
            &mut self.0[side as usize][from as usize][to as usize],
            -depth_bonus(depth),
        );
    }
}

pub type KillerMoves = [Move; MAX_KILLER_MOVES];

pub type ContHistEntry = [[i16; Sq::NB]; Piece::NB];
pub type ContHistPtr = Option<NonNull<ContHistEntry>>;

pub struct ContinuationHistoryTable(pub [[ContHistEntry; Sq::NB]; Piece::NB]);

impl Default for ContinuationHistoryTable {
    fn default() -> Self {
        Self([[[[0; Sq::NB]; Piece::NB]; Sq::NB]; Piece::NB])
    }
}

impl ContinuationHistoryTable {
    #[inline(always)]
    pub fn entry_ptr(&mut self, piece: Piece, to: Sq) -> NonNull<ContHistEntry> {
        NonNull::from(&mut self.0[piece as usize][to as usize])
    }

    #[inline(always)]
    pub fn update_bonus(
        &mut self,
        prev_piece: Piece,
        prev_to: Sq,
        curr_piece: Piece,
        curr_to: Sq,
        depth: u8,
    ) {
        gravity(
            &mut self.0[prev_piece as usize][prev_to as usize][curr_piece as usize]
                [curr_to as usize],
            depth_bonus(depth),
        );
    }

    #[inline(always)]
    pub fn update_malus(
        &mut self,
        prev_piece: Piece,
        prev_to: Sq,
        curr_piece: Piece,
        curr_to: Sq,
        depth: u8,
    ) {
        gravity(
            &mut self.0[prev_piece as usize][prev_to as usize][curr_piece as usize]
                [curr_to as usize],
            -depth_bonus(depth),
        );
    }
}

#[inline(always)]
pub fn conthist_score(ptr: ContHistPtr, piece: Piece, to: Sq) -> i16 {
    match ptr {
        Some(entry) => unsafe { (*entry.as_ptr())[piece as usize][to as usize] },
        None => 0,
    }
}

pub struct QuietsTried {
    arr: [MaybeUninit<Move>; 32],
    size: usize,
}

impl Default for QuietsTried {
    fn default() -> Self {
        Self {
            arr: [const { MaybeUninit::uninit() }; 32],
            size: 0,
        }
    }
}

impl QuietsTried {
    #[inline(always)]
    pub const fn as_slice(&self) -> &[Move] {
        unsafe { core::slice::from_raw_parts(self.arr.as_ptr().cast::<Move>(), self.size) }
    }

    #[inline(always)]
    pub const fn push_move(&mut self, mov: Move) {
        if self.size < self.arr.len() {
            self.arr[self.size].write(mov);
            self.size += 1;
        }
    }
}
