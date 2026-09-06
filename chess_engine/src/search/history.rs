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

    #[inline(always)]
    pub fn update_bonus(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        let bonus = (depth as i32 * depth as i32).min(MAX_HISTORY);
        self.apply_bonus(side, from, to, bonus);
    }

    #[inline(always)]
    pub fn update_malus(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        let bonus = (depth as i32 * depth as i32).min(MAX_HISTORY);
        self.apply_malus(side, from, to, bonus);
    }

    #[inline(always)]
    pub fn apply_bonus(&mut self, side: Color, from: Sq, to: Sq, bonus: i32) {
        let bonus = bonus.clamp(0, MAX_HISTORY);
        let entry = &mut self.0[side as usize][from as usize][to as usize];
        let current = *entry as i32;
        let next = current + bonus - (current * bonus / MAX_HISTORY);
        *entry = next.clamp(-MAX_HISTORY, MAX_HISTORY) as i16;
    }

    #[inline(always)]
    pub fn apply_malus(&mut self, side: Color, from: Sq, to: Sq, bonus: i32) {
        let bonus = bonus.clamp(0, MAX_HISTORY);
        let entry = &mut self.0[side as usize][from as usize][to as usize];
        let current = *entry as i32;
        let next = current - bonus + (current * bonus / MAX_HISTORY);
        *entry = next.clamp(-MAX_HISTORY, MAX_HISTORY) as i16;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub type KillerMoves = [Move; MAX_KILLER_MOVES];
