use crate::Color;

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
    pub const NB: usize = 6;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColoredPiece(std::num::NonZeroU8);

impl ColoredPiece {
    const PIECE_MASK: u8 = 0b0111;
    const COLOR_MASK: u8 = 0b1000;

    #[inline(always)]
    pub const fn new(piece: Pieces, color: Color) -> Self {
        const { assert!(size_of::<Option<ColoredPiece>>() == 1) };
        let val = (piece as u8 | ((color as u8) << 3)) + 1;
        Self(unsafe { std::num::NonZeroU8::new_unchecked(val) })
    }

    #[inline(always)]
    pub const fn piece(self) -> Pieces {
        let val = self.0.get() - 1;
        unsafe { std::mem::transmute(val & Self::PIECE_MASK) }
    }

    #[inline(always)]
    pub const fn color(self) -> Color {
        let val = self.0.get() - 1;
        if val & Self::COLOR_MASK != 0 { Color::White } else { Color::Black }
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

        Some(Self::new(piece, Color::from(val.is_uppercase())))
    }
}

impl PartialEq<Pieces> for ColoredPiece {
    #[inline(always)]
    fn eq(&self, other: &Pieces) -> bool {
        self.piece() == *other
    }
}
