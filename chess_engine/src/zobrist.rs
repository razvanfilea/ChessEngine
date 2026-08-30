use chess_core::{for_each_square, prelude::*, prng::Prng};

pub struct ZobristKeys {
    pieces: [[[u64; Color::NB]; Piece::NB]; Sq::NB],
    side: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 8],
}

impl ZobristKeys {
    const fn new() -> Self {
        let mut generator = Prng::new();
        let mut pieces = [[[0; Color::NB]; Piece::NB]; Sq::NB];

        for_each_square!(sq => {
            let mut i = 0;
            while i < Piece::NB {
            pieces[sq as usize][i] = generator.random_array();
            i+=1;
            }
        });

        Self {
            pieces,
            side: generator.random(),
            castling_rights: generator.random_array(),
            en_passant: generator.random_array(),
        }
    }

    #[inline(always)]
    pub const fn piece(&self, sq: Sq, piece: ColoredPiece) -> u64 {
        self.pieces[sq as usize][piece.piece() as usize][piece.color() as usize]
    }

    #[inline(always)]
    pub const fn side(&self) -> u64 {
        self.side
    }

    #[inline(always)]
    pub const fn castling(&self, rights: CastlingRights) -> u64 {
        self.castling_rights[rights.bits() as usize]
    }

    #[inline(always)]
    pub const fn en_passant(&self, sq: Sq) -> u64 {
        self.en_passant[sq.file() as usize]
    }
}

pub static ZOBRIST_KEYS: ZobristKeys = ZobristKeys::new();
