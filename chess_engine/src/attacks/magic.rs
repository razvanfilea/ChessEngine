use chess_core::bitboard::*;
use chess_core::prelude::*;

pub const fn bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    bb_bishop_attacks(sq, blockers)
}
pub const fn rook_attacks(sq: Sq, blockers: u64) -> u64 {
    bb_rook_attacks(sq, blockers)
}
