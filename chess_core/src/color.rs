use std::ops::Not;

use crate::Dir;

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
    pub const fn forward(self) -> Dir {
        match self {
            Color::White => Dir::North,
            Color::Black => Dir::South,
        }
    }

    #[inline(always)]
    pub const fn backward(self) -> Dir {
        match self {
            Color::White => Dir::South,
            Color::Black => Dir::North,
        }
    }
}

impl Not for Color {
    type Output = Color;

    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl From<bool> for Color {
    #[inline(always)]
    fn from(value: bool) -> Self {
        if value { Color::White } else { Color::Black }
    }
}

pub trait Player: 'static + Copy + Eq {
    const COLOR: Color;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct White;

impl Player for White {
    const COLOR: Color = Color::White;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Black;

impl Player for Black {
    const COLOR: Color = Color::Black;
}
