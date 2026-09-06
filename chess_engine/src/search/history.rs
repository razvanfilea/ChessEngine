use super::params::{MAX_HISTORY, MAX_KILLER_MOVES};
use chess_core::prelude::*;

pub struct HistoryTable([[[i16; Sq::NB]; Sq::NB]; Color::NB]);

impl Default for HistoryTable {
    fn default() -> Self {
        Self([[[0; Sq::NB]; Sq::NB]; Color::NB])
    }
}

impl HistoryTable {
    #[inline(always)]
    pub fn get(&self, side: Color, from: Sq, to: Sq) -> i16 {
        self.0[side as usize][from as usize][to as usize]
    }

    /// Gravity update formula: naturally bounds values in [-MAX_HISTORY, MAX_HISTORY]
    /// without ever overflowing i16 or requiring periodic resets.
    #[inline(always)]
    pub fn update(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        let bonus = depth as i32 * depth as i32;
        let entry = &mut self.0[side as usize][from as usize][to as usize];
        let current = *entry as i32;
        *entry = (current + bonus - (current * bonus / MAX_HISTORY)) as i16;
    }

    pub fn clear(&mut self) {
        self.0 = [[[0; Sq::NB]; Sq::NB]; Color::NB];
    }
}

pub type KillerMoves = [Move; MAX_KILLER_MOVES];
