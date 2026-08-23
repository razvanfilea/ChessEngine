use std::arch::x86_64::_pext_u64;

use crate::{square::*,
};

include!(concat!(env!("OUT_DIR"), "/pext_data.rs"));

pub fn bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    let mask = PEXT_BISHOP_MASKS[sq.as_index()];
    let index = unsafe { _pext_u64(blockers, mask) };
    PEXT_TABLE[PEXT_BISHOP_OFFSETS[sq.as_index()] as usize + index as usize]
}

pub fn rook_attacks(sq: Sq, blockers: u64) -> u64 {
    let mask = PEXT_ROOK_MASKS[sq.as_index()];
    let index = unsafe { _pext_u64(blockers, mask) };
    PEXT_TABLE[PEXT_ROOK_OFFSETS[sq.as_index()] as usize + index as usize]
}
