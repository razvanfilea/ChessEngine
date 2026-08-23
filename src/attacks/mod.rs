#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
mod magic;
#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
mod pext;

mod pattern;
use pattern::*;

mod sliders_attack {
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    pub use super::magic::*;
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    pub use super::pext::*;
}

use crate::bitboard::*;
use crate::square::Sq;
use crate::types::Color;

pub const fn pawn_moves(sq: Sq, color: Color) -> u64 {
    let bb = sq.bitboard();
    let rank = sq.rank();
    if color.as_bool() {
        if rank == 5 {
            sh_north_north(bb)
        } else {
            sh_north(bb)
        }
    } else {
        if rank == 2 {
            sh_south_south(bb)
        } else {
            sh_south(bb)
        }
    }
}

pub const fn pawn_attacks(sq: Sq, color: Color) -> u64 {
    let bb = sq.bitboard();
    if color.as_bool() {
        sh_north_west(bb) | sh_north_east(bb)
    } else {
        sh_south_west(bb) | sh_south_east(bb)
    }
}

#[inline(always)]
pub const fn bishop_xray_attacks(sq: Sq) -> u64 {
    BISHOP_XRAY_ATTACKS[sq.as_index()]
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
pub const fn rook_xray_attacks(sq: Sq) -> u64 {
    ROOK_XRAY_ATTACKS[sq.as_index()]
}

#[inline(always)]
pub const fn knight_attacks(sq: Sq) -> u64 {
    KNIGHT_ATTACKS[sq.as_index()]
}

#[inline(always)]
pub const fn king_attacks(sq: Sq) -> u64 {
    KING_ATTACKS[sq.as_index()]
}

