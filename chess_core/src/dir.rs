#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dir {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Dir {
    pub const NB: usize = 8;

    pub const ALL: [Self; Self::NB] = [
        Dir::North,
        Dir::South,
        Dir::East,
        Dir::West,
        Dir::NorthEast,
        Dir::NorthWest,
        Dir::SouthEast,
        Dir::SouthWest,
    ];

    #[inline(always)]
    pub const fn opposite(self) -> Self {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::East => Dir::West,
            Dir::West => Dir::East,
            Dir::NorthEast => Dir::SouthWest,
            Dir::NorthWest => Dir::SouthEast,
            Dir::SouthEast => Dir::NorthWest,
            Dir::SouthWest => Dir::NorthEast,
        }
    }

    #[inline(always)]
    pub const fn is_forwards(self) -> bool {
        match self {
            Dir::North | Dir::East | Dir::NorthWest | Dir::NorthEast => true,
            Dir::South | Dir::West | Dir::SouthEast | Dir::SouthWest => false,
        }
    }
}
