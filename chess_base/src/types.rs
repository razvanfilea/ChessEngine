use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
    pub struct CastlingRights: u8 {
        const WHITE_00 = 1 << 0;
        const WHITE_000 = 1 << 1;
        const BLACK_00 = 1 << 2;
        const BLACK_000 = 1 << 3;

        const WHITE_ANY = Self::WHITE_00.bits() | Self::WHITE_000.bits();
        const BLACK_ANY = Self::BLACK_00.bits() | Self::BLACK_000.bits();
        const ALL = Self::WHITE_ANY.bits() | Self::BLACK_ANY.bits();
    }
}

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

    pub const fn oppsite(self) -> Self {
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

    pub const fn is_forwards(self) -> bool {
        match self {
            Dir::North | Dir::East | Dir::NorthWest | Dir::NorthEast => true,
            Dir::South | Dir::West | Dir::SouthEast | Dir::SouthWest => false,
        }
    }
}
