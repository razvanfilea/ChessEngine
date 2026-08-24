use std::ops::Not;

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
    #[inline(always)]
    fn from(value: bool) -> Self {
        if value { Color::White } else { Color::Black }
    }
}
