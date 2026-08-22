use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Sq(u8);

impl Sq {
    pub const NB: usize = 64;
    pub const NONE: Self = Self(64);
    pub const VALID_SQUARES_MASK: u8 = 0b00111111; // 63

    pub const A1: Self = Self(0);
    pub const B1: Self = Self(1);
    pub const C1: Self = Self(2);
    pub const D1: Self = Self(3);
    pub const E1: Self = Self(4);
    pub const F1: Self = Self(5);
    pub const G1: Self = Self(6);
    pub const H1: Self = Self(7);

    pub const A2: Self = Self(8);
    pub const B2: Self = Self(9);
    pub const C2: Self = Self(10);
    pub const D2: Self = Self(11);
    pub const E2: Self = Self(12);
    pub const F2: Self = Self(13);
    pub const G2: Self = Self(14);
    pub const H2: Self = Self(15);

    pub const A3: Self = Self(16);
    pub const B3: Self = Self(17);
    pub const C3: Self = Self(18);
    pub const D3: Self = Self(19);
    pub const E3: Self = Self(20);
    pub const F3: Self = Self(21);
    pub const G3: Self = Self(22);
    pub const H3: Self = Self(23);

    pub const A4: Self = Self(24);
    pub const B4: Self = Self(25);
    pub const C4: Self = Self(26);
    pub const D4: Self = Self(27);
    pub const E4: Self = Self(28);
    pub const F4: Self = Self(29);
    pub const G4: Self = Self(30);
    pub const H4: Self = Self(31);

    pub const A5: Self = Self(32);
    pub const B5: Self = Self(33);
    pub const C5: Self = Self(34);
    pub const D5: Self = Self(35);
    pub const E5: Self = Self(36);
    pub const F5: Self = Self(37);
    pub const G5: Self = Self(38);
    pub const H5: Self = Self(39);

    pub const A6: Self = Self(40);
    pub const B6: Self = Self(41);
    pub const C6: Self = Self(42);
    pub const D6: Self = Self(43);
    pub const E6: Self = Self(44);
    pub const F6: Self = Self(45);
    pub const G6: Self = Self(46);
    pub const H6: Self = Self(47);

    pub const A7: Self = Self(48);
    pub const B7: Self = Self(49);
    pub const C7: Self = Self(50);
    pub const D7: Self = Self(51);
    pub const E7: Self = Self(52);
    pub const F7: Self = Self(53);
    pub const G7: Self = Self(54);
    pub const H7: Self = Self(55);

    pub const A8: Self = Self(56);
    pub const B8: Self = Self(57);
    pub const C8: Self = Self(58);
    pub const D8: Self = Self(59);
    pub const E8: Self = Self(60);
    pub const F8: Self = Self(61);
    pub const G8: Self = Self(62);
    pub const H8: Self = Self(63);

    /// Branchless constructor from 0-indexed (file, rank).
    /// Clamps directly to `Sq::NONE` (64) if out of bounds.
    // #[inline(always)]
    #[inline(never)]
    #[unsafe(no_mangle)]
    pub const fn new(file: u8, rank: u8) -> Self {
        let is_valid = ((file | rank) & !7) == 0;
        let val = if is_valid { (rank << 3) | file } else { 64 };
        Self(val)
    }

    #[inline(always)]
    pub const fn from_raw(val: u8) -> Self {
        Self(if val < 64 { val } else { 64 })
    }

    /// Construct directly without bounds checks.
    ///
    /// # Safety
    /// `val` must be <= 64.
    #[inline(always)]
    pub const unsafe fn from_raw_unchecked(val: u8) -> Self {
        unsafe {
            core::hint::assert_unchecked(val <= 64);
        }
        Self(val)
    }

    /// Returns the inner raw u8, informing LLVM it is within 0..=64.
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        unsafe {
            core::hint::assert_unchecked(self.0 <= 64);
        }
        self.0
    }

    /// Returns the raw usize index, informing LLVM it is within 0..=64.
    #[inline(always)]
    pub const fn as_index(self) -> usize {
        self.as_u8() as usize
    }

    #[inline(always)]
    pub const fn file(self) -> u8 {
        self.as_u8() & 7
    }

    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self.as_u8() >> 3
    }

    #[inline(always)]
    pub const fn is_valid(self) -> bool {
        self.as_u8() < 64
    }

    #[inline(always)]
    pub const fn is_none(self) -> bool {
        self.as_u8() == 64
    }

    /// Generates the corresponding bitboard (bit 0 for A1, bit 63 for H8).
    /// Returns 0 for `Sq::NONE`.
    #[inline(always)]
    pub const fn bitboard(self) -> u64 {
        if self.is_valid() {
            1u64 << self.as_u8()
        } else {
            0
        }
    }
    
    pub fn distance_to_file_edge(self) -> u8 {
        let file = self.file();
        file.min(7 - file)
    }

    pub fn distance_to_rank_edge(self) -> u8 {
        let rank = self.rank();
        rank.min(7 - rank)
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    pub const fn distance(self, other: Self) -> u8 {
        let rank_dist = self.rank().abs_diff(other.rank());
        let file_dist = self.file().abs_diff(other.file());
        if rank_dist > file_dist {
            rank_dist
        } else {
            file_dist
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    pub const fn manhattan_distance(self, other: Self) -> u8 {
        self.rank().abs_diff(other.rank()) + self.file().abs_diff(other.file())
    }


    pub fn parse(val: &str) -> Self {
        let mut x = val.chars();
        let Some(file) = x.next() else {
            return Self::NONE;
        };
        let Some(rank) = x.next() else {
            return Self::NONE;
        };

        let file: u8 = match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Self::NONE,
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
            _ => return Self::NONE,
        };

        Sq::new(file, rank)
    }
}

impl Default for Sq {
    #[inline(always)]
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Debug for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            let file_char = (b'a' + self.file()) as char;
            let rank_char = (b'1' + self.rank()) as char;
            write!(f, "{file_char}{rank_char}")
        } else {
            write!(f, "None")
        }
    }
}

impl fmt::Display for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

