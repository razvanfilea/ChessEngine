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
const NOT_FILE_AB: u64 = !(FILE_A | FILE_B);
const NOT_FILE_GH: u64 = !(FILE_G | FILE_H);

// Rank masks
pub const RANK_1: u64 = 0x0000_0000_0000_00FF;
pub const RANK_2: u64 = 0x0000_0000_0000_FF00;
pub const RANK_3: u64 = 0x0000_0000_00FF_0000;
pub const RANK_4: u64 = 0x0000_0000_FF00_0000;
pub const RANK_5: u64 = 0x0000_00FF_0000_0000;
pub const RANK_6: u64 = 0x0000_FF00_0000_0000;
pub const RANK_7: u64 = 0x00FF_0000_0000_0000;
pub const RANK_8: u64 = 0xFF00_0000_0000_0000;

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

const fn sh_west_n(bb: u64, n: u8) -> u64 {
    if n == 0 {
        return bb;
    }

    sh_west_n(sh_west(bb), n - 1)
}

const fn sh_east_n(bb: u64, n: u8) -> u64 {
    if n == 0 {
        return bb;
    }

    sh_east_n(sh_east(bb), n - 1)
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

static BB_DIRECTIONS: [[u64; Sq::NB]; Dir::NB] = const {
    let mut result = [[0; Sq::NB]; Dir::NB];

    let mut dir_idx = 0;
    while dir_idx < Dir::ALL.len() {
        let dir = Dir::ALL[dir_idx];

        let mut sq_index = 0u8;
        while sq_index < 64 {
            let sq = Sq::from_raw(sq_index);
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
            sq_index += 1;
        }

        dir_idx += 1;
    }

    result
};

static BB_BETWEEN_SQUARES: [[u64; Sq::NB]; Sq::NB] = const {
    let mut result = [[0; Sq::NB]; Sq::NB];

    let mut sq1_index = 0u8;

    while sq1_index < 64 {
        let sq1 = Sq::from_raw(sq1_index);
        let mut sq2_index = 0u8;

        while sq2_index < 64 {
            let sq2 = Sq::from_raw(sq2_index);
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

            sq2_index += 1;
        }

        sq1_index += 1;
    }

    result
};

static BB_LINE_SQUARES: [[u64; Sq::NB]; Sq::NB] = const {
    let mut result = [[0; Sq::NB]; Sq::NB];

    let mut sq1_index = 0u8;

    while sq1_index < 64 {
        let sq1 = Sq::from_raw(sq1_index);
        let bb1 = sq1.bitboard();
        let mut sq2_index = 0u8;

        while sq2_index < 64 {
            let sq2 = Sq::from_raw(sq2_index);
            let bb2 = sq2.bitboard();

            result[sq1_index as usize][sq2_index as usize] =
                BB_BETWEEN_SQUARES[sq1_index as usize][sq2_index as usize] | bb1 | bb2;

            sq2_index += 1;
        }

        sq1_index += 1;
    }

    result
};

