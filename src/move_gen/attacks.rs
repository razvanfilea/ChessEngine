use crate::bitboard::*;
use crate::square::Sq;
use crate::types::{Color, Dir};

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
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

type AttacksBoard = [u64; Sq::NB];

static KNIGHT_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    let mut sq_index = 0u8;

    while sq_index < 64 {
        let sq = Sq::from_raw(sq_index);
        let bb = sq.bitboard();

        result[sq_index as usize] = sh_west(sh_north_north(bb))
            | sh_east(sh_north_north(bb))
            | sh_west(sh_south_south(bb))
            | sh_east(sh_south_south(bb))
            | sh_north(sh_west(sh_west(bb)))
            | sh_south(sh_west(sh_west(bb)))
            | sh_north(sh_east(sh_east(bb)))
            | sh_south(sh_east(sh_east(bb)));
        sq_index += 1;
    }

    result
};

static KING_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    let mut sq_index = 0u8;

    while sq_index < 64 {
        let sq = Sq::from_raw(sq_index);
        let bb = sq.bitboard();

        result[sq_index as usize] = sh_north_west(bb)
            | sh_north(bb)
            | sh_north_east(bb)
            | sh_east(bb)
            | sh_south_east(bb)
            | sh_south(bb)
            | sh_south_west(bb)
            | sh_west(bb);
        sq_index += 1;
    }

    result
};

static BISHOP_XRAY_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    let mut sq_index = 0u8;

    while sq_index < 64 {
        let sq = Sq::from_raw(sq_index);

        result[sq_index as usize] = bb_from_dir(Dir::NorthEast, sq)
            | bb_from_dir(Dir::NorthWest, sq)
            | bb_from_dir(Dir::SouthEast, sq)
            | bb_from_dir(Dir::SouthWest, sq);
        sq_index += 1;
    }

    result
};

static ROOK_XRAY_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    let mut sq_index = 0u8;

    while sq_index < 64 {
        let sq = Sq::from_raw(sq_index);

        result[sq_index as usize] = bb_from_dir(Dir::West, sq)
            | bb_from_dir(Dir::North, sq)
            | bb_from_dir(Dir::East, sq)
            | bb_from_dir(Dir::South, sq);
        sq_index += 1;
    }

    result
};
