use std::hint::assert_unchecked;

use chess_base::{
    bitboard::{bb_between, bb_scan_forward},
    get_castling_rights_mask,
    prelude::*,
};

use crate::attacks::*;

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

    #[inline]
    fn add_piece(&mut self, sq: Sq, piece: ColoredPiece) {
        self.mailbox[sq as usize] = Some(piece);
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) |= bitboard;
        *self.pieces_mut(piece.piece()) |= bitboard;
    }

    #[inline(always)]
    fn move_piece(&mut self, from: Sq, to: Sq) -> ColoredPiece {
        let piece = unsafe { self.mailbox[from as usize].take().unwrap_unchecked() };
        self.mailbox[to as usize] = Some(piece);

        let move_bb = from.bitboard() ^ to.bitboard();
        *self.colors_mut(piece.color()) ^= move_bb;
        *self.pieces_mut(piece.piece()) ^= move_bb;

        piece
    }

    #[inline]
    fn remove_piece(&mut self, sq: Sq) -> Option<ColoredPiece> {
        let piece = self.mailbox[sq as usize].take()?;
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) &= !bitboard;
        *self.pieces_mut(piece.piece()) &= !bitboard;

        Some(piece)
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

    #[inline(always)]
    pub fn king_sq(&self, color: Color) -> Sq {
        let king_bb = self.color_piece(Pieces::King, color);
        unsafe { bb_scan_forward(king_bb) }
    }

    pub fn legal(&self, mov: Move) -> bool {
        let occupied = self.occupied();
        let from = mov.from();
        let to = mov.to();
        let from_bb = from.bitboard();
        let to_bb = to.bitboard();
        let flags = mov.flags();
        let us = self.to_play;
        let them = !us;

        let moved_piece = self.piece_at(mov.from());
        unsafe {
            assert_unchecked(moved_piece.is_some());
        }
        if moved_piece.map(|p| p.piece()) == Some(Pieces::King) {
            if mov.is_castle() {
                let path = if flags == MoveFlags::CastleKing {
                    if self.to_play == Color::White {
                        [Sq::E1, Sq::F1, Sq::G1]
                    } else {
                        [Sq::E8, Sq::F8, Sq::G8]
                    }
                } else {
                    if self.to_play == Color::White {
                        [Sq::E1, Sq::D1, Sq::C1]
                    } else {
                        [Sq::E8, Sq::D8, Sq::C8]
                    }
                };

                for sq in path {
                    if self.generate_attackers(sq, them, occupied) != 0 {
                        return false;
                    }
                }
            }

            return self.generate_attackers(to, them, (occupied ^ from_bb) | to_bb) == 0;
        }

        let mut occ = occupied ^ from_bb;

        if flags == MoveFlags::EnPassant {
            let captured_pawn_sq = unsafe {
                to.shift_unchecked(if us == Color::White {
                    Dir::South
                } else {
                    Dir::North
                })
            };
            occ ^= captured_pawn_sq.bitboard();
        }

        let king_sq = self.king_sq(us);
        let attackers = self.generate_attackers(king_sq, them, occ);

        let enemy_sliders =
            self.pieces(Pieces::Queen) | self.pieces(Pieces::Rook) | self.pieces(Pieces::Bischop);
        let slider_attackers = attackers & enemy_sliders;

        if slider_attackers != 0 {
            // If removing this piece exposed the King to 2+ sliders, no single move can fix it.
            if (slider_attackers & (slider_attackers - 1)) != 0 {
                return false;
            }

            // The piece is pinned (or blocking a check).
            let pinner_sq = unsafe { bb_scan_forward(slider_attackers) };

            // The piece is only legally allowed to move to the pinner's square,
            // or anywhere strictly between the pinner and the King.
            let legal_ray = pinner_sq.bitboard() | bb_between(king_sq, pinner_sq);

            return (to_bb & legal_ray) != 0;
        }

        true
    }

    pub fn generate_attackers(
        &self,
        attacked_sq: Sq,
        attacking_color: Color,
        occupied: u64,
    ) -> u64 {
        let enemy = *self.colors(attacking_color);

        let enemy_bischop = (self.pieces(Pieces::Bischop) | self.pieces(Pieces::Queen)) & enemy;
        let enemy_rook = (self.pieces(Pieces::Rook) | self.pieces(Pieces::Queen)) & enemy;

        let enemy_pawns = self.pieces(Pieces::Pawn) & enemy;
        let enemy_knights = self.pieces(Pieces::Knight) & enemy;
        let enemy_kings = self.pieces(Pieces::King) & enemy;

        (pawn_attacks_color(attacked_sq, !attacking_color) & enemy_pawns)
            | (knight_attacks(attacked_sq) & enemy_knights)
            | (bishop_attacks(attacked_sq, occupied) & enemy_bischop)
            | (rook_attacks(attacked_sq, occupied) & enemy_rook)
            | (king_attacks(attacked_sq) & enemy_kings)
    }

    pub fn make_move(&mut self, mov: Move) {
        let from = mov.from();
        let to = mov.to();
        let flags = mov.flags();
        let us = self.to_play;

        debug_assert!(self.piece_at(from).is_some());

        let is_capture = mov.is_capture();
        if is_capture {
            if flags == MoveFlags::EnPassant {
                let captured_pawn_sq = unsafe {
                    to.shift_unchecked(if us == Color::White {
                        Dir::South
                    } else {
                        Dir::North
                    })
                };
                self.remove_piece(captured_pawn_sq);
            } else {
                self.remove_piece(to);
            }
        }

        let mut piece = self.move_piece(from, to);
        let is_pawn = piece.piece() == Pieces::Pawn;

        if mov.is_promotion() {
            let promo_piece = unsafe { mov.promotion_piece().unwrap_unchecked() };

            self.remove_piece(to);
            piece = ColoredPiece::new(promo_piece, us);
            self.add_piece(to, piece);
        }

        if mov.is_castle() {
            let (rook_from, rook_to) = if flags == MoveFlags::CastleKing {
                if us == Color::White {
                    (Sq::H1, Sq::F1)
                } else {
                    (Sq::H8, Sq::F8)
                }
            } else {
                if us == Color::White {
                    (Sq::A1, Sq::D1)
                } else {
                    (Sq::A8, Sq::D8)
                }
            };
            self.move_piece(rook_from, rook_to);
        }

        self.castling_rights &= get_castling_rights_mask(from, to);

        if flags == MoveFlags::DoublePawn {
            self.en_passant_target_sq = Some(unsafe {
                to.shift_unchecked(if us == Color::White {
                    Dir::South
                } else {
                    Dir::North
                })
            });
        } else {
            self.en_passant_target_sq = None;
        }

        if is_pawn || is_capture {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }

        self.ply += 1;
        self.to_play = !us;
    }
}
