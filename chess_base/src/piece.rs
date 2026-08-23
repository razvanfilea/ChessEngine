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
