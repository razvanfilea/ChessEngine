use chess_core::prelude::*;

include!(concat!(env!("OUT_DIR"), "/magic_data.rs"));

#[inline(always)]
pub fn bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    let m = &BISHOP_MAGICS[sq as usize];
    let index =
        m.offset as usize + (((blockers & m.mask).wrapping_mul(m.magic)) >> m.shift) as usize;
    unsafe { *MAGIC_TABLE.get_unchecked(index) }
}

#[inline(always)]
pub fn rook_attacks(sq: Sq, blockers: u64) -> u64 {
    let m = &ROOK_MAGICS[sq as usize];
    let index =
        m.offset as usize + (((blockers & m.mask).wrapping_mul(m.magic)) >> m.shift) as usize;
    unsafe { *MAGIC_TABLE.get_unchecked(index) }
}
