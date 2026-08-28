#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
mod magic;
#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
mod pext;

mod sliders_attack {
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    pub use super::magic::*;
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    pub use super::pext::*;
}

use chess_core::{piece_tables::*, prelude::*};

#[inline(always)]
pub fn pawn_attacks(sq: Sq, color: Color) -> u64 {
    PAWN_ATTACKS[color as usize][sq as usize]
}

#[inline(always)]
pub const fn bishop_xray_attacks(sq: Sq) -> u64 {
    BISHOP_RAYS[sq as usize]
}

#[inline(always)]
pub const fn rook_xray_attacks(sq: Sq) -> u64 {
    ROOK_RAYS[sq as usize]
}

#[inline(always)]
pub fn bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    sliders_attack::bishop_attacks(sq, blockers)
}

#[inline(always)]
pub fn rook_attacks(sq: Sq, blockers: u64) -> u64 {
    sliders_attack::rook_attacks(sq, blockers)
}

#[inline(always)]
pub fn queen_attacks(sq: Sq, blockers: u64) -> u64 {
    sliders_attack::bishop_attacks(sq, blockers) | sliders_attack::rook_attacks(sq, blockers)
}

#[inline(always)]
pub const fn knight_attacks(sq: Sq) -> u64 {
    KNIGHT_ATTACKS[sq as usize]
}

#[inline(always)]
pub const fn king_attacks(sq: Sq) -> u64 {
    KING_ATTACKS[sq as usize]
}
