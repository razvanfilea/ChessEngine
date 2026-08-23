use std::arch::x86_64::_pext_u64;

use crate::{
    attacks::{bishop_xray_attacks, generate_bishop_attacks, generate_rook_attacks, rook_xray_attacks}, bitboard::*, for_each_square, square::*,
};

pub fn bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    let mask = BISHOP_MASK[sq.as_index()];
    let index = unsafe { _pext_u64(blockers, mask) };
    PEXT_DATA.table[PEXT_DATA.bishop_offsets[sq.as_index()] as usize + index as usize]
}

pub fn rook_attacks(sq: Sq, blockers: u64) -> u64 {
    let mask = ROOK_MASK[sq.as_index()];
    let index = unsafe { _pext_u64(blockers, mask) };
    PEXT_DATA.table[PEXT_DATA.rook_offsets[sq.as_index()] as usize + index as usize]
}

/// Pure const software fallback for _pdep_u64
const fn const_pdep(mut val: u64, mut mask: u64) -> u64 {
    let mut res = 0u64;
    let mut bb = 1u64;
    while mask != 0 {
        if (mask & 1) != 0 {
            if (val & 1) != 0 {
                res |= bb;
            }
            val >>= 1;
        }
        mask >>= 1;
        bb <<= 1;
    }
    res
}

static BISHOP_MASK: [u64; Sq::NB] = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        result[sq.as_index()] = bishop_xray_attacks(sq) & !bb_get_edge_filter(sq);
    });

    result
};

static ROOK_MASK: [u64; Sq::NB] = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        result[sq.as_index()] = rook_xray_attacks(sq) & !bb_get_edge_filter(sq);
    });

    result
};

const BISCHOP_ENTRIES: usize = 5248;
const ROOK_ENTRIES: usize = 102_400;

struct PextData {
    pub table: [u64; BISCHOP_ENTRIES + ROOK_ENTRIES],
    pub bishop_offsets: [u16; 64],
    pub rook_offsets: [u16; 64],
}

const fn genereate_pext_data() -> PextData {
    let mut table = [0; BISCHOP_ENTRIES + ROOK_ENTRIES];
    let mut bishop_offsets = [0; 64];
    let mut rook_offsets = [0; 64];

    let mut current_index = 0;

    for_each_square!(sq => {
        rook_offsets[sq.as_index()] = current_index as u16;
        let mask = ROOK_MASK[sq.as_index()];
        let combinations = 1usize << mask.count_ones();

        let mut j = 0;
        while j < combinations {
            let occupied = const_pdep(j as u64, mask);
            table[current_index] = generate_rook_attacks(sq, occupied);
            current_index += 1;
            j += 1;
        }
    });

    for_each_square!(sq => {
        bishop_offsets[sq.as_index()] = current_index as u16;
        let mask = BISHOP_MASK[sq.as_index()];
        let combinations = 1usize << mask.count_ones();

        let mut j = 0;
        while j < combinations {
            let occupied = const_pdep(j as u64, mask);
            table[current_index] = generate_bishop_attacks(sq, occupied);
            current_index += 1;
            j += 1;
        }
    });

    PextData {
        table,
        bishop_offsets,
        rook_offsets,
    }
}

static PEXT_DATA: PextData = genereate_pext_data();

