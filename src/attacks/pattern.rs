use chess_base::{bitboard::*, for_each_square, prelude::*};

type AttacksBoard = [u64; Sq::NB];

pub static KNIGHT_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        let bb = sq.bitboard();

        result[sq.as_index()] = sh_west(sh_north_north(bb))
            | sh_east(sh_north_north(bb))
            | sh_west(sh_south_south(bb))
            | sh_east(sh_south_south(bb))
            | sh_north(sh_west(sh_west(bb)))
            | sh_south(sh_west(sh_west(bb)))
            | sh_north(sh_east(sh_east(bb)))
            | sh_south(sh_east(sh_east(bb)));
    });

    result
};

pub static KING_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        let bb = sq.bitboard();

        result[sq.as_index()] = sh_north_west(bb)
            | sh_north(bb)
            | sh_north_east(bb)
            | sh_east(bb)
            | sh_south_east(bb)
            | sh_south(bb)
            | sh_south_west(bb)
            | sh_west(bb);
    });

    result
};

pub static BISHOP_XRAY_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        result[sq.as_index()] = bb_from_dir(Dir::NorthEast, sq)
            | bb_from_dir(Dir::NorthWest, sq)
            | bb_from_dir(Dir::SouthEast, sq)
            | bb_from_dir(Dir::SouthWest, sq);
    });

    result
};

pub static ROOK_XRAY_ATTACKS: AttacksBoard = const {
    let mut result = [0; Sq::NB];

    for_each_square!(sq => {
        result[sq.as_index()] = bb_from_dir(Dir::West, sq)
            | bb_from_dir(Dir::North, sq)
            | bb_from_dir(Dir::East, sq)
            | bb_from_dir(Dir::South, sq);
    });

    result
};
