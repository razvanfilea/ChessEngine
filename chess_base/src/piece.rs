use crate::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    pub const NB: usize = 6;

    pub const ALL: [Piece; Piece::NB] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColoredPiece(std::num::NonZeroU8);

impl ColoredPiece {
    const PIECE_MASK: u8 = 0b0111;
    const COLOR_MASK: u8 = 0b1000;

    #[inline(always)]
    pub const fn new(piece: Piece, color: Color) -> Self {
        const { assert!(size_of::<Option<ColoredPiece>>() == 1) };
        let val = (piece as u8 | ((color as u8) << 3)) + 1;
        Self(unsafe { std::num::NonZeroU8::new_unchecked(val) })
    }

    #[inline(always)]
    pub const fn piece(self) -> Piece {
        let val = self.0.get() - 1;
        unsafe { std::mem::transmute(val & Self::PIECE_MASK) }
    }

    #[inline(always)]
    pub const fn color(self) -> Color {
        let val = self.0.get() - 1;
        if val & Self::COLOR_MASK != 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn parse(val: char) -> Option<Self> {
        let piece = match val.to_ascii_uppercase() {
            'P' => Piece::Pawn,
            'R' => Piece::Rook,
            'N' => Piece::Knight,
            'B' => Piece::Bishop,
            'Q' => Piece::Queen,
            'K' => Piece::King,
            _ => return None,
        };

        Some(Self::new(piece, Color::from(val.is_uppercase())))
    }
}

impl PartialEq<Piece> for ColoredPiece {
    #[inline(always)]
    fn eq(&self, other: &Piece) -> bool {
        self.piece() == *other
    }
}
