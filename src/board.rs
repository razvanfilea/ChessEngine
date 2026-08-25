use chess_base::prelude::*;

#[derive(Clone)]
pub struct Board {
    pub mailbox: [Option<ColoredPiece>; Sq::NB],
    pub bit_colors: [u64; Color::NB],
    pub bit_pieces: [u64; Pieces::NB],

    pub castling_rights: CastlingRights,
    pub to_play: Color,
    pub en_passant_target_sq: Option<Sq>,
    pub half_move_clock: u8, // 50 move draw rule
    pub ply: u16,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            mailbox: [None; Sq::NB],
            bit_colors: [0; Color::NB],
            bit_pieces: [0; Pieces::NB],
            castling_rights: CastlingRights::empty(),
            to_play: Color::White,
            en_passant_target_sq: None,
            half_move_clock: 0,
            ply: 0,
        }
    }
}

impl Board {
    pub fn start_pos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Initial position is valid")
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut board = Board::default();

        let mut tokens = fen.split_ascii_whitespace();
        let piece_placement = tokens.next()?;
        let side_to_move = tokens.next();
        let castling = tokens.next();
        let en_passant = tokens.next();
        let half_move = tokens.next();
        let full_move = tokens.next();

        let mut rank: i8 = 7;
        let mut file: u8 = 0;

        for ch in piece_placement.chars() {
            match ch {
                '1'..='8' => {
                    file += (ch as u8) - b'0';
                }
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                'p' | 'r' | 'n' | 'b' | 'q' | 'k' | 'P' | 'R' | 'N' | 'B' | 'Q' | 'K'
                    if rank >= 0 && file < 8 =>
                {
                    // TODO: Add proper error handling
                    let sq = Sq::new(file, rank as u8)?;
                    board.add_piece(sq, ColoredPiece::parse(ch)?);
                    file += 1;
                }
                _ => {}
            }
        }

        board.to_play = match side_to_move {
            Some("b") | Some("B") => Color::Black,
            _ => Color::White,
        };

        if let Some(castling_rights) = castling {
            for ch in castling_rights.chars() {
                let rights = match ch {
                    'K' => CastlingRights::WHITE_00,
                    'Q' => CastlingRights::WHITE_000,
                    'k' => CastlingRights::BLACK_00,
                    'q' => CastlingRights::BLACK_000,
                    _ => CastlingRights::empty(),
                };

                board.castling_rights |= rights;
            }
        }

        if let Some(en_passant_sq) = en_passant {
            let sq = Sq::parse(en_passant_sq);
            let valid_rank = if board.to_play == Color::White { 5 } else { 2 };
            if sq.filter(|sq| sq.rank() == valid_rank).is_some() {
                board.en_passant_target_sq = sq;
            }
        }

        board.half_move_clock = half_move
            .and_then(|val| val.parse::<u8>().ok())
            .unwrap_or(0);

        let counter = full_move
            .and_then(|val| val.parse::<u16>().ok())
            .unwrap_or(1);
        board.ply =
            (counter.saturating_sub(1)) * 2 + if board.to_play == Color::Black { 1 } else { 0 };

        Some(board)
    }

    pub fn add_piece(&mut self, sq: Sq, piece: ColoredPiece) {
        self.mailbox[sq as usize] = Some(piece);
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) |= bitboard;
        *self.pieces_mut(piece.piece()) |= bitboard;
    }

    pub fn remove_piece(&mut self, sq: Sq) {
        let Some(piece) = self.mailbox[sq as usize].take() else {
            return;
        };
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) &= !bitboard;
        *self.pieces_mut(piece.piece()) &= !bitboard;
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> &u64 {
        &self.bit_colors[color as usize]
    }

    #[inline(always)]
    pub fn colors_mut(&mut self, color: Color) -> &mut u64 {
        &mut self.bit_colors[color as usize]
    }

    #[inline(always)]
    pub fn pieces(&self, piece: Pieces) -> &u64 {
        &self.bit_pieces[piece as usize]
    }

    #[inline(always)]
    pub fn pieces_mut(&mut self, piece: Pieces) -> &mut u64 {
        &mut self.bit_pieces[piece as usize]
    }

    #[inline(always)]
    pub fn color_piece(&self, piece: Pieces, color: Color) -> u64 {
        self.colors(color) & self.pieces(piece)
    }

    #[inline(always)]
    pub fn colored_piece(&self, piece: ColoredPiece) -> u64 {
        self.colors(piece.color()) & self.pieces(piece.piece())
    }

    #[inline(always)]
    pub fn piece_at(&self, sq: Sq) -> Option<ColoredPiece> {
        self.mailbox[sq as usize]
    }

    #[inline(always)]
    pub fn empty(&self) -> u64 {
        !self.occupied()
    }

    #[inline(always)]
    pub fn occupied(&self) -> u64 {
        self.bit_colors[0] | self.bit_colors[1]
    }

    pub fn legal(&self, mov: Move) {
        // TODO:
        // • If the move is a King move (from == king_sq):
        //     1. Remove from from occupied: occ = (board.occupied() ^ from.bitboard()) | to.bitboard();
        //     2. Ask: Is to attacked by any enemy piece using blockers occ?
        //     3. If attacked → Illegal. If not attacked → Legal.
        // • If the move is a Non-King piece:
        //     1. Verify the piece is not pinned to the King (or if it is pinned, it only moves along the pin ray).
        //     2. If en-passant, verify the special horizontal double-pawn pin.

        // TODO: The transit squares (the square the King crosses and lands on) are not attacked by the enemy:
        //   - White Kingside: !is_attacked(E1) && !is_attacked(F1) && !is_attacked(G1)
        //   - White Queenside: !is_attacked(E1) && !is_attacked(D1) && !is_attacked(C1) (Note: b1 only needs to be empty, not unattacked)
    }
}
