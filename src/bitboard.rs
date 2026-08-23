use crate::{square::Sq, types::Dir};

// File masks
pub const FILE_A: u64 = 0x0101_0101_0101_0101;
pub const FILE_B: u64 = 0x0202_0202_0202_0202;
pub const FILE_C: u64 = 0x0404_0404_0404_0404;
pub const FILE_D: u64 = 0x0808_0808_0808_0808;
pub const FILE_E: u64 = 0x1010_1010_1010_1010;
pub const FILE_F: u64 = 0x2020_2020_2020_2020;
pub const FILE_G: u64 = 0x4040_4040_4040_4040;
pub const FILE_H: u64 = 0x8080_8080_8080_8080;

// Inverted file masks for shift boundaries
const NOT_FILE_A: u64 = !FILE_A;
const NOT_FILE_H: u64 = !FILE_H;

// Rank masks
pub const RANK_1: u64 = 0x0000_0000_0000_00FF;
pub const RANK_2: u64 = 0x0000_0000_0000_FF00;
pub const RANK_3: u64 = 0x0000_0000_00FF_0000;
pub const RANK_4: u64 = 0x0000_0000_FF00_0000;
pub const RANK_5: u64 = 0x0000_00FF_0000_0000;
pub const RANK_6: u64 = 0x0000_FF00_0000_0000;
pub const RANK_7: u64 = 0x00FF_0000_0000_0000;
pub const RANK_8: u64 = 0xFF00_0000_0000_0000;

pub const EDGES: u64 = FILE_A | FILE_H | RANK_1 | RANK_8;

/// Iterates over all subsets of a bitboard mask.
/// This is implemented as a macro so it can be used inside `const fn` and `const {}` blocks!
#[macro_export]
macro_rules! for_subsets {
    ($subset_name:ident in $mask:expr => $body:block) => {
        let mut $subset_name = $mask;
        loop {
            $body

            $subset_name = $subset_name.wrapping_sub(1) & $mask;
            if $subset_name == $mask {
                break;
            }
        }
    };
}

#[macro_export]
macro_rules! for_each_square {
    ($sq_name:ident => $body:block) => {
        let mut _i = 0u8;
        while _i < 64 {
            let $sq_name = Sq::from_raw(_i);

            $body

            _i += 1
        }
    };
}

#[inline(always)]
pub const fn sh_dir(dir: Dir, bb: u64) -> u64 {
    match dir {
        Dir::North => sh_north(bb),
        Dir::South => sh_south(bb),
        Dir::East => sh_east(bb),
        Dir::West => sh_west(bb),
        Dir::NorthEast => sh_north_east(bb),
        Dir::NorthWest => sh_north_west(bb),
        Dir::SouthEast => sh_south_east(bb),
        Dir::SouthWest => sh_south_west(bb),
    }
}

#[inline(always)]
pub const fn sh_north(bb: u64) -> u64 {
    bb << 8
}

#[inline(always)]
pub const fn sh_south(bb: u64) -> u64 {
    bb >> 8
}

#[inline(always)]
pub const fn sh_east(bb: u64) -> u64 {
    (bb & NOT_FILE_H) << 1
}

#[inline(always)]
pub const fn sh_west(bb: u64) -> u64 {
    (bb & NOT_FILE_A) >> 1
}

#[inline(always)]
pub const fn sh_north_east(bb: u64) -> u64 {
    (bb & NOT_FILE_H) << 9
}

#[inline(always)]
pub const fn sh_north_west(bb: u64) -> u64 {
    (bb & NOT_FILE_A) << 7
}

#[inline(always)]
pub const fn sh_south_east(bb: u64) -> u64 {
    (bb & NOT_FILE_H) >> 7
}

#[inline(always)]
pub const fn sh_south_west(bb: u64) -> u64 {
    (bb & NOT_FILE_A) >> 9
}

#[inline(always)]
pub const fn sh_north_north(bb: u64) -> u64 {
    bb << 16
}

#[inline(always)]
pub const fn sh_south_south(bb: u64) -> u64 {
    bb >> 16
}

#[inline(always)]
pub const fn bb_rank(rank: u8) -> u64 {
    RANK_1 << (rank * 8)
}

#[inline(always)]
pub const fn bb_file(file: u8) -> u64 {
    FILE_A << file
}

#[inline]
pub const fn bb_several(bb: u64) -> bool {
    bb & (bb.wrapping_sub(1)) != 1
}

#[inline]
pub const fn bb_only_one(bb: u64) -> bool {
    bb != 0 && !bb_several(bb)
}

#[inline(always)]
pub const fn bb_scan_forward(bb: u64) -> u8 {
    bb.trailing_zeros() as u8
}

#[inline(always)]
pub const fn bb_scan_reverse(bb: u64) -> u8 {
    bb.leading_zeros() as u8
}

#[inline]
pub const fn bb_pop_lsb(mut bb: u64) -> (Sq, u64) {
    let sq = Sq::from_raw(bb.trailing_zeros() as u8);
    bb &= bb.wrapping_sub(1);
    (sq, bb)
}

#[inline(always)]
pub const fn bb_from_dir(dir: Dir, sq: Sq) -> u64 {
    BB_DIRECTIONS[dir as usize][sq.as_index()]
}

#[inline(always)]
pub const fn bb_between(sq1: Sq, sq2: Sq) -> u64 {
    BB_BETWEEN_SQUARES[sq1.as_index()][sq2.as_index()]
}

#[inline(always)]
pub const fn bb_line(sq1: Sq, sq2: Sq) -> u64 {
    BB_LINE_SQUARES[sq1.as_index()][sq2.as_index()]
}

pub const fn bb_get_edge_filter(sq: Sq) -> u64 {
    ((RANK_1 | RANK_8) & !bb_rank(sq.rank())) | ((FILE_A | FILE_H) & !bb_file(sq.file()))
}

pub const fn bb_generate_ray_attacks(sq: Sq, occupied: u64, dir: Dir) -> u64 {
    const FORWARD_SENTINEL: u64 = Sq::from_raw(63).bitboard();
    const BACKWARD_SENTINEL: u64 = Sq::from_raw(1).bitboard();

    let attacks = bb_from_dir(dir, sq);
    let blockers = attacks & occupied;
    let found_sq = if dir.is_forwards() {
        bb_scan_forward(blockers | FORWARD_SENTINEL)
    } else {
        bb_scan_reverse(blockers | BACKWARD_SENTINEL)
    };

    attacks ^ bb_from_dir(dir, unsafe { Sq::from_raw_unchecked(found_sq) })
}

static BB_DIRECTIONS: [[u64; Sq::NB]; Dir::NB] = const {
    let mut result = [[0; Sq::NB]; Dir::NB];

    let mut dir_idx = 0;
    while dir_idx < Dir::ALL.len() {
        let dir = Dir::ALL[dir_idx];

        for_each_square!(sq => {
            let mut ray = 0u64;
            let mut bb = sq.bitboard();

            // Step in `dir` until we fall off the board
            loop {
                bb = sh_dir(dir, bb);
                if bb == 0 {
                    break;
                }
                ray |= bb;
            }

            result[dir as usize][sq.as_index()] = ray;
        });

        dir_idx += 1;
    }

    result
};

static BB_BETWEEN_SQUARES: [[u64; Sq::NB]; Sq::NB] = const {
    let mut result = [[0; Sq::NB]; Sq::NB];

    for_each_square!(sq1 => {
        for_each_square!(sq2 => {
            let bb2 = sq2.bitboard();

            let mut i = 0;
            while i < Dir::ALL.len() {
                let dir = Dir::ALL[i];
                let ray1 = bb_from_dir(dir, sq1);
                if (ray1 & bb2) != 0 {
                    let ray2 = bb_from_dir(dir.oppsite(), sq2);
                    result[sq1.as_index()][sq2.as_index()] = ray1 & ray2;
                    break;
                }
                i += 1;
            }
        });
    });

    result
};

static BB_LINE_SQUARES: [[u64; Sq::NB]; Sq::NB] = const {
    let mut result = [[0; Sq::NB]; Sq::NB];

    for_each_square!(sq1 => {
        let bb1 = sq1.bitboard();
        for_each_square!(sq2 => {
            let bb2 = sq2.bitboard();

            result[sq1.as_index()][sq2.as_index()] = BB_BETWEEN_SQUARES[sq1.as_index()][sq2.as_index()] | bb1 | bb2;
        });
    });

    result
};
