use std::{fmt, mem::transmute};

use crate::Dir;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sq {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Sq {
    pub const NB: usize = 64;

    #[inline]
    pub const fn new(file: u8, rank: u8) -> Option<Self> {
        const { assert!(size_of::<Option<Self>>() == 1) }

        let is_valid = ((file | rank) & !7) == 0;
        if is_valid {
            let val = (rank << 3) | file;
            Some(unsafe { Self::from_raw_unchecked(val) })
        } else {
            None
        }
    }

    #[inline(always)]
    pub const fn from_raw(val: u8) -> Option<Self> {
        if val < 64 {
            Some(unsafe { Self::from_raw_unchecked(val) })
        } else {
            None
        }
    }

    /// Construct directly without bounds checks.
    ///
    /// # Safety
    /// `val` must be < 64.
    #[inline(always)]
    pub const unsafe fn from_raw_unchecked(val: u8) -> Self {
        unsafe {
            core::hint::assert_unchecked(val < 64);
            transmute::<u8, Sq>(val)
        }
    }

    #[inline(always)]
    pub const fn file(self) -> u8 {
        self as u8 & 7
    }

    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self as u8 >> 3
    }

    /// Shifts the square by the given direction without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that shifting this square in the given direction
    /// will not wrap around the files or fall off the board.
    #[inline(always)]
    pub const unsafe fn shift(self, dir: Dir) -> Self {
        let offset = match dir {
            Dir::North => 8,
            Dir::South => -8,
            Dir::East => 1,
            Dir::West => -1,
            Dir::NorthEast => 9,
            Dir::NorthWest => 7,
            Dir::SouthEast => -7,
            Dir::SouthWest => -9,
        };
        let val = (self as i8 + offset) as u8;
        unsafe { Self::from_raw_unchecked(val) }
    }

    /// Generates the corresponding bitboard (bit 0 for A1, bit 63 for H8).
    #[inline(always)]
    pub const fn bitboard(self) -> u64 {
        1u64 << self as u8
    }

    pub fn distance_to_file_edge(self) -> u8 {
        let file = self.file();
        file.min(7 - file)
    }

    pub fn distance_to_rank_edge(self) -> u8 {
        let rank = self.rank();
        rank.min(7 - rank)
    }

    pub const fn distance_to(self, other: Self) -> u8 {
        let rank_dist = self.rank().abs_diff(other.rank());
        let file_dist = self.file().abs_diff(other.file());
        if rank_dist > file_dist {
            rank_dist
        } else {
            file_dist
        }
    }

    #[inline]
    pub const fn on_diagonal_with(self, other: Self) -> bool {
        let rank_dist = self.rank().abs_diff(other.rank());
        let file_dist = self.file().abs_diff(other.file());
        rank_dist == file_dist
    }

    pub const fn manhattan_distance_to(self, other: Self) -> u8 {
        self.rank().abs_diff(other.rank()) + self.file().abs_diff(other.file())
    }

    pub fn parse(val: &str) -> Option<Self> {
        let mut x = val.chars();
        let file = x.next()?;
        let rank = x.next()?;

        let file: u8 = match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return None,
        };

        let rank: u8 = match rank {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return None,
        };

        Sq::new(file, rank)
    }
}

impl fmt::Debug for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        write!(f, "{file_char}{rank_char}")
    }
}

impl fmt::Display for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
