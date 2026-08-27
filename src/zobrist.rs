use chess_base::{for_each_square, prelude::*};

struct RandomGenerator {
    seed: u64,
}

impl RandomGenerator {
    pub const fn new() -> Self {
        Self { seed: 1070372 }
    }

    /*
     * Generate random numbers based on this paper: http://vigna.di.unimi.it/ftp/papers/xorshift.pdf
     */
    pub const fn random(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;

        self.seed.wrapping_mul(2685821657736338717)
    }

    pub const fn random_array<const N: usize>(&mut self) -> [u64; N] {
        let mut array = [0; N];
        let mut i = 0;
        while i < N {
            array[i] = self.random();
            i += 1;
        }
        array
    }
}

pub struct ZobristKeys {
    pieces: [[[u64; Color::NB]; Pieces::NB]; Sq::NB],
    side: u64,
    castling_rights: [u64; 16],
    en_passant: [u64; 8],
}

impl ZobristKeys {
    const fn new() -> Self {
        let mut generator = RandomGenerator::new();
        let mut pieces = [[[0; Color::NB]; Pieces::NB]; Sq::NB];

        for_each_square!(sq => {
            let mut i = 0;
            while i < Pieces::NB {
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
