use crate::{dir::Dir, square::Sq};

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
pub const LIGHT_SQUARES: u64 = 0x55AA55AA55AA55AA;
pub const DARK_SQUARES: u64 = !LIGHT_SQUARES;

#[macro_export]
macro_rules! for_each_square {
    ($sq_name:ident => $body:block) => {
        let mut _i = 0u8;
        while _i < 64 {
            // SAFETY: `_i` is strictly in 0..64.
            let $sq_name = unsafe { $crate::Sq::from_raw_unchecked(_i) };

            $body

            _i += 1
        }
    };
}

/// Iterates over every set bit in a bitboard as an `Sq`.
#[macro_export]
macro_rules! for_each_bit {
    ($sq:pat in $bitboard:expr => $body:block) => {{
        let mut _bb: u64 = $bitboard;
        while _bb != 0 {
            // SAFETY: `_bb != 0` is guaranteed by the while loop condition.
            let $sq = unsafe { $crate::bitboard::bb_pop_lsb(&mut _bb) };
            $body
        }
    }};
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
    bb & (bb.wrapping_sub(1)) != 0
}

#[inline]
pub const fn bb_only_one(bb: u64) -> bool {
    bb != 0 && !bb_several(bb)
}

/// Returns the least significant square set in the bitboard without bounds checking.
///
/// # Safety
/// The caller must ensure that `bb` is non-zero (`bb != 0`).
#[inline(always)]
pub const unsafe fn bb_lsb(bb: u64) -> Sq {
    // SAFETY: The caller guarantees `bb != 0`, meaning `trailing_zeros()` is strictly < 64.
    unsafe {
        core::hint::assert_unchecked(bb != 0);
        Sq::from_raw_unchecked(bb.trailing_zeros() as u8)
    }
}

/// Returns the least significant square set in the bitboard, or `None` if the bitboard is empty.
#[inline(always)]
pub const fn bb_lsb_opt(bb: u64) -> Option<Sq> {
    if bb != 0 {
        // SAFETY: `bb != 0` check guarantees non-zero bitboard.
        Some(unsafe { bb_lsb(bb) })
    } else {
        None
    }
}

/// Returns the most significant square set in the bitboard without bounds checking.
///
/// # Safety
/// The caller must ensure that `bb` is non-zero (`bb != 0`).
#[inline(always)]
pub const unsafe fn bb_msb(bb: u64) -> Sq {
    // SAFETY: The caller guarantees `bb != 0`, meaning `leading_zeros()` is strictly < 64
    // and `63 ^ leading_zeros()` is in `0..=63`.
    unsafe {
        core::hint::assert_unchecked(bb != 0);
        Sq::from_raw_unchecked(63 ^ bb.leading_zeros() as u8)
    }
}

/// Returns the most significant square set in the bitboard, or `None` if the bitboard is empty.
#[inline(always)]
pub const fn bb_msb_opt(bb: u64) -> Option<Sq> {
    if bb != 0 {
        // SAFETY: `bb != 0` check guarantees non-zero bitboard.
        Some(unsafe { bb_msb(bb) })
    } else {
        None
    }
}

/// Pops the least significant square set in the bitboard and clears its bit without bounds checking.
///
/// # Safety
/// The caller must ensure that `*bb` is non-zero (`*bb != 0`).
#[inline]
pub const unsafe fn bb_pop_lsb(bb: &mut u64) -> Sq {
    // SAFETY: The caller guarantees `*bb != 0`, so `trailing_zeros()` is strictly < 64.
    unsafe {
        core::hint::assert_unchecked(*bb != 0);
        let sq = Sq::from_raw_unchecked(bb.trailing_zeros() as u8);
        *bb &= bb.wrapping_sub(1);
        sq
    }
}

/// Pops the least significant square set in the bitboard, or returns `None` if the bitboard is empty.
#[inline]
pub const fn bb_pop_lsb_opt(bb: &mut u64) -> Option<Sq> {
    let sq = Sq::from_raw(bb.trailing_zeros() as u8);
    *bb &= bb.wrapping_sub(1);
    sq
}

#[inline(always)]
pub const fn bb_flip_vertically(bb: u64) -> u64 {
    bb.swap_bytes()
}

#[inline(always)]
pub const fn bb_from_dir(dir: Dir, sq: Sq) -> u64 {
    BB_DIRECTIONS[dir as usize][sq as usize]
}

#[inline(always)]
pub const fn bb_between(sq1: Sq, sq2: Sq) -> u64 {
    BB_BETWEEN_SQUARES[sq1 as usize][sq2 as usize]
}

#[inline(always)]
pub const fn bb_line(sq1: Sq, sq2: Sq) -> u64 {
    BB_LINE[sq1 as usize][sq2 as usize]
}

pub const fn bb_get_edge_filter(sq: Sq) -> u64 {
    ((RANK_1 | RANK_8) & !bb_rank(sq.rank())) | ((FILE_A | FILE_H) & !bb_file(sq.file()))
}

pub const fn bb_generate_ray_attacks(sq: Sq, occupied: u64, dir: Dir) -> u64 {
    const FORWARD_SENTINEL: u64 = Sq::H8.bitboard();
    const BACKWARD_SENTINEL: u64 = Sq::A1.bitboard();

    let attacks = bb_from_dir(dir, sq);
    let blockers = attacks & occupied;
    let found_sq = if dir.is_forwards() {
        // SAFETY: Bitwise-OR with FORWARD_SENTINEL (bit 63 / H8) guarantees non-zero bitboard.
        unsafe { bb_lsb(blockers | FORWARD_SENTINEL) }
    } else {
        // SAFETY: Bitwise-OR with BACKWARD_SENTINEL (bit 0 / A1) guarantees non-zero bitboard.
        unsafe { bb_msb(blockers | BACKWARD_SENTINEL) }
    };

    attacks ^ bb_from_dir(dir, found_sq)
}

pub const fn bb_bishop_attacks(sq: Sq, blockers: u64) -> u64 {
    bb_generate_ray_attacks(sq, blockers, Dir::NorthWest)
        | bb_generate_ray_attacks(sq, blockers, Dir::NorthEast)
        | bb_generate_ray_attacks(sq, blockers, Dir::SouthWest)
        | bb_generate_ray_attacks(sq, blockers, Dir::SouthEast)
}

pub const fn bb_rook_attacks(sq: Sq, blockers: u64) -> u64 {
    bb_generate_ray_attacks(sq, blockers, Dir::North)
        | bb_generate_ray_attacks(sq, blockers, Dir::West)
        | bb_generate_ray_attacks(sq, blockers, Dir::East)
        | bb_generate_ray_attacks(sq, blockers, Dir::South)
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

            result[dir as usize][sq as usize] = ray;
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
                    let ray2 = bb_from_dir(dir.opposite(), sq2);
                    result[sq1 as usize][sq2 as usize] = ray1 & ray2;
                    break;
                }
                i += 1;
            }
        });
    });

    result
};

static BB_LINE: [[u64; Sq::NB]; Sq::NB] = const {
    let mut result = [[0; Sq::NB]; Sq::NB];

    for_each_square!(sq1 => {
        for_each_square!(sq2 => {
            let bb2 = sq2.bitboard();
            let mut i = 0;
            while i < Dir::ALL.len() {
                let dir = Dir::ALL[i];
                let ray1 = bb_from_dir(dir, sq1);
                if (ray1 & bb2) != 0 {
                    let ray_opp = bb_from_dir(dir.opposite(), sq1);
                    result[sq1 as usize][sq2 as usize] = ray1 | ray_opp | sq1.bitboard();
                    break;
                }
                i += 1;
            }
        });
    });

    result
};
