use std::ops::Not;

use bitflags::bitflags;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Color {
    #[default]
    Black = 0,
    White = 1,
}

impl Color {
    pub const NB: usize = 2;

    #[inline(always)]
    pub const fn as_bool(self) -> bool {
        self as u8 != 0
    }

    #[inline(always)]
    pub const fn as_index(self) -> usize {
        self.as_bool() as usize
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl From<bool> for Color {
    fn from(value: bool) -> Self {
        if value { Color::White } else { Color::Black }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pieces {
    Pawn = 0,
    Knight = 1,
    Bischop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Pieces {
    pub const NB: usize = 7;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColoredPiece {
    pub piece: Pieces,
    pub color: Color,
}

impl ColoredPiece {
    pub fn new(piece: Pieces, color: Color) -> Self {
        Self { piece, color }
    }

    pub fn parse(val: char) -> Option<Self> {
        let piece = match val.to_ascii_uppercase() {
            'P' => Pieces::Pawn,
            'R' => Pieces::Rook,
            'N' => Pieces::Knight,
            'B' => Pieces::Bischop,
            'Q' => Pieces::Queen,
            'K' => Pieces::King,
            _ => return None,
        };

        Some(Self {
            piece,
            color: Color::from(val.is_uppercase()),
        })
    }
}

impl PartialEq<Pieces> for ColoredPiece {
    fn eq(&self, other: &Pieces) -> bool {
        self.piece == *other
    }
}

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
