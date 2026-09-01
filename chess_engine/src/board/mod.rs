use std::fmt;
use std::hint::assert_unchecked;

pub mod fen;

use crate::attacks::*;
use crate::move_gen::gen_all_moves;
use crate::zobrist::ZOBRIST_KEYS;
use chess_core::bitboard::{LIGHT_SQUARES, bb_several};
use chess_core::{
    bitboard::{bb_between, bb_line, bb_lsb, bb_only_one},
    for_each_bit,
    prelude::*,
};

const MAX_GAME_PLAY: usize = 1024;

#[derive(Clone, PartialEq)]
pub struct Board {
    pub mailbox: [Option<ColoredPiece>; Sq::NB],
    pub bit_colors: [u64; Color::NB],
    pub bit_pieces: [u64; Piece::NB],
    pub checkers: u64,
    pub pinned: u64,

    pub hash: u64,
    pub castling_rights: CastlingRights,
    pub to_play: Color,
    pub en_passant_target_sq: Option<Sq>,
    pub half_move_clock: u8, // 50 move draw rule
    pub ply: u16,
    pub hash_history: Box<[u64; MAX_GAME_PLAY]>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            mailbox: [None; Sq::NB],
            bit_colors: [0; Color::NB],
            bit_pieces: [0; Piece::NB],
            checkers: 0,
            pinned: 0,
            hash: 0,
            castling_rights: CastlingRights::empty(),
            to_play: Color::White,
            en_passant_target_sq: None,
            half_move_clock: 0,
            ply: 0,
            hash_history: Box::new([0; MAX_GAME_PLAY]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct UndoInfo {
    pub captured_piece: Option<ColoredPiece>,
    pub castling_rights: CastlingRights,
    pub en_passant_target_sq: Option<Sq>,
    pub half_move_clock: u8,
    pub checkers: u64,
    pub pinned: u64,
    pub hash: u64,
}

impl Board {
    pub fn start_pos() -> Self {
        fen::parse_fen(fen::START_POS_FEN).expect("Initial position is valid")
    }

    #[inline(always)]
    pub fn from_fen(fen: &str) -> Option<Self> {
        fen::parse_fen(fen)
    }

    #[inline(always)]
    pub fn to_fen(&self) -> String {
        fen::format_fen(self)
    }

    #[inline(always)]
    pub fn colors(&self, color: Color) -> u64 {
        self.bit_colors[color as usize]
    }

    #[inline(always)]
    pub fn colors_mut(&mut self, color: Color) -> &mut u64 {
        &mut self.bit_colors[color as usize]
    }

    #[inline(always)]
    pub fn pieces(&self, piece: Piece) -> u64 {
        self.bit_pieces[piece as usize]
    }

    #[inline(always)]
    pub fn pieces_mut(&mut self, piece: Piece) -> &mut u64 {
        &mut self.bit_pieces[piece as usize]
    }

    #[inline(always)]
    pub fn color_piece(&self, piece: Piece, color: Color) -> u64 {
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
        let king_bb = self.color_piece(Piece::King, color);
        unsafe { bb_lsb(king_bb) }
    }

    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.checkers != 0
    }

    pub fn is_draw(&self) -> bool {
        self.half_move_clock >= 100 || self.is_repetition() || self.has_insufficient_material()
    }

    pub fn generate_attackers(
        &self,
        attacked_sq: Sq,
        attacking_color: Color,
        occupied: u64,
    ) -> u64 {
        let enemy = self.colors(attacking_color);

        let enemy_bishop = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)) & enemy;
        let enemy_rook = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen)) & enemy;

        let enemy_pawns = self.pieces(Piece::Pawn) & enemy;
        let enemy_knights = self.pieces(Piece::Knight) & enemy;
        let enemy_kings = self.pieces(Piece::King) & enemy;

        (pawn_attacks(attacked_sq, !attacking_color) & enemy_pawns)
            | (knight_attacks(attacked_sq) & enemy_knights)
            | (bishop_attacks(attacked_sq, occupied) & enemy_bishop)
            | (rook_attacks(attacked_sq, occupied) & enemy_rook)
            | (king_attacks(attacked_sq) & enemy_kings)
    }

    pub fn pseudo_legal(&self, mov: Move) -> bool {
        if mov.is_none() {
            return false;
        }

        let from = mov.from();
        let to = mov.to();
        if from == to {
            return false;
        }

        let to_bb = to.bitboard();
        let occupied = self.occupied();
        let us = self.to_play;

        let Some(colored_piece) = self.piece_at(from).filter(|p| p.color() == us) else {
            return false;
        };
        let piece = colored_piece.piece();

        if to_bb & self.colors(us) != 0 {
            return false;
        }

        if mov.is_promotion() || mov.is_castle() || mov.flags() == MoveFlags::EnPassant {
            return gen_all_moves(self).as_slice().contains(&mov);
        }

        if self.checkers != 0 {
            if piece == Piece::King {
                if king_attacks(from) & to_bb == 0 {
                    return false;
                }
                return self.validate_capture_flag(mov, to_bb);
            }

            // Double check: only king moves can evade
            if bb_several(self.checkers) {
                return false;
            }

            // Single check: non-king piece must capture checker or interpose/block ray
            let checker_sq = unsafe { bb_lsb(self.checkers) };
            let king_sq = self.king_sq(us);
            let evasion_mask = self.checkers | bb_between(king_sq, checker_sq);

            let is_ep = mov.flags() == MoveFlags::EnPassant;
            if !is_ep && (to_bb & evasion_mask == 0) {
                return false;
            }
        }

        if !self.validate_capture_flag(mov, to_bb) {
            return false;
        }

        if mov.flags() == MoveFlags::DoublePawn && piece != Piece::Pawn {
            return false;
        }

        match piece {
            Piece::Pawn => {
                let promo_rank = if us == Color::White { 7 } else { 0 };
                if to.rank() == promo_rank {
                    return false;
                }

                let forward_dir = if us == Color::White {
                    Dir::North
                } else {
                    Dir::South
                };
                let start_rank = if us == Color::White { 1 } else { 6 };
                let single_push_sq = unsafe { from.shift(forward_dir) };

                if mov.is_capture() {
                    pawn_attacks(from, us) & to_bb != 0
                } else if to == single_push_sq {
                    mov.flags() == MoveFlags::Quiet
                } else if from.rank() == start_rank {
                    let double_push_sq = unsafe { single_push_sq.shift(forward_dir) };
                    to == double_push_sq
                        && (single_push_sq.bitboard() & occupied) == 0
                        && mov.flags() == MoveFlags::DoublePawn
                } else {
                    false
                }
            }
            Piece::Knight => knight_attacks(from) & to_bb != 0,
            Piece::Bishop => bishop_attacks(from, occupied) & to_bb != 0,
            Piece::Rook => rook_attacks(from, occupied) & to_bb != 0,
            Piece::Queen => queen_attacks(from, occupied) & to_bb != 0,
            Piece::King => king_attacks(from) & to_bb != 0,
        }
    }

    #[inline(always)]
    fn validate_capture_flag(&self, mov: Move, to_bb: u64) -> bool {
        if mov.is_capture() {
            (to_bb & self.colors(!self.to_play)) != 0
        } else {
            (to_bb & self.occupied()) == 0
        }
    }

    #[inline]
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
        if moved_piece.map(|p| p.piece()) == Some(Piece::King) {
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

        if flags == MoveFlags::EnPassant {
            let captured_pawn_sq = unsafe {
                to.shift(if us == Color::White {
                    Dir::South
                } else {
                    Dir::North
                })
            };
            let occ = ((occupied ^ from_bb) ^ captured_pawn_sq.bitboard()) | to_bb;
            let king_sq = self.king_sq(us);
            let attackers = self.generate_attackers(king_sq, them, occ);

            // Mask out the captured pawn
            return (attackers & !captured_pawn_sq.bitboard()) == 0;
        }

        if from_bb & self.pinned == 0 {
            return true;
        }

        // It can only move legally along the pin ray
        let king_sq = self.king_sq(us);
        let ray_mask = bb_line(king_sq, from);

        (to.bitboard() & ray_mask) != 0
    }

    pub fn make_move(&mut self, mov: Move) -> UndoInfo {
        let from = mov.from();
        let to = mov.to();
        let flags = mov.flags();
        let us = self.to_play;
        let original_hash = self.hash;
        self.hash_history[self.ply as usize] = original_hash;

        debug_assert!(self.piece_at(from).is_some());

        let is_capture = mov.is_capture();
        let captured_piece = if is_capture {
            if flags == MoveFlags::EnPassant {
                let captured_pawn_sq = unsafe {
                    to.shift(if us == Color::White {
                        Dir::South
                    } else {
                        Dir::North
                    })
                };
                self.remove_piece(captured_pawn_sq)
            } else {
                self.remove_piece(to)
            }
        } else {
            None
        };

        let undo_info = UndoInfo {
            captured_piece,
            castling_rights: self.castling_rights,
            en_passant_target_sq: self.en_passant_target_sq,
            half_move_clock: self.half_move_clock,
            checkers: self.checkers,
            pinned: self.pinned,
            hash: original_hash,
        };

        let mut piece = self.move_piece(from, to);
        let is_pawn = piece.piece() == Piece::Pawn;

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

        self.hash ^= ZOBRIST_KEYS.castling(self.castling_rights);
        self.castling_rights &= CastlingRights::mask_for_move(from, to);
        self.hash ^= ZOBRIST_KEYS.castling(self.castling_rights);

        if let Some(en_passsant) = self.en_passant_target_sq.take() {
            self.hash ^= ZOBRIST_KEYS.en_passant(en_passsant);
        }
        if flags == MoveFlags::DoublePawn {
            let target_sq = unsafe {
                to.shift(if us == Color::White {
                    Dir::South
                } else {
                    Dir::North
                })
            };
            self.hash ^= ZOBRIST_KEYS.en_passant(target_sq);
            self.en_passant_target_sq = Some(target_sq);
        }

        if is_pawn || is_capture {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }

        self.ply += 1;
        self.to_play = !us;
        self.hash ^= ZOBRIST_KEYS.side();
        self.set_checkers();
        self.set_pinned();

        undo_info
    }

    /// Safety: it's the callers responsibility to make sure the UndoInfo and the Move match
    pub fn undo_move(&mut self, mov: Move, undo: UndoInfo) {
        let from = mov.from();
        let to = mov.to();
        let flags = mov.flags();
        let us = !self.to_play;

        debug_assert!(self.piece_at(to).is_some());

        if mov.is_promotion() {
            self.remove_piece(to);
            self.add_piece(from, ColoredPiece::new(Piece::Pawn, us));
        } else {
            self.move_piece(to, from);
        }

        if mov.is_castle() {
            let (rook_from, rook_to) = if flags == MoveFlags::CastleKing {
                if us == Color::White {
                    (Sq::F1, Sq::H1)
                } else {
                    (Sq::F8, Sq::H8)
                }
            } else {
                if us == Color::White {
                    (Sq::D1, Sq::A1)
                } else {
                    (Sq::D8, Sq::A8)
                }
            };
            self.move_piece(rook_from, rook_to);
        }

        let is_capture = mov.is_capture();
        if is_capture {
            debug_assert!(undo.captured_piece.is_some());
            // Safety: this is a capture
            let captured_piece = unsafe { undo.captured_piece.unwrap_unchecked() };
            let captured_sq = if flags == MoveFlags::EnPassant {
                unsafe {
                    to.shift(if us == Color::White {
                        Dir::South
                    } else {
                        Dir::North
                    })
                }
            } else {
                to
            };
            self.add_piece(captured_sq, captured_piece);
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant_target_sq = undo.en_passant_target_sq;
        self.half_move_clock = undo.half_move_clock;
        self.checkers = undo.checkers;
        self.pinned = undo.pinned;
        self.hash = undo.hash;

        self.ply -= 1;
        self.hash_history[self.ply as usize] = 0;
        self.to_play = us;
    }

    pub fn make_null_move(&mut self) -> UndoInfo {
        debug_assert!(self.checkers == 0);

        let us = self.to_play;
        let original_hash = self.hash;
        self.hash_history[self.ply as usize] = original_hash;

        let info = UndoInfo {
            captured_piece: None,
            castling_rights: self.castling_rights,
            en_passant_target_sq: self.en_passant_target_sq,
            half_move_clock: self.half_move_clock,
            checkers: 0,
            pinned: self.pinned,
            hash: original_hash,
        };

        if let Some(en_passant) = self.en_passant_target_sq.take() {
            self.hash ^= ZOBRIST_KEYS.en_passant(en_passant);
        }

        self.half_move_clock += 1;
        self.ply += 1;
        self.to_play = !us;
        self.hash ^= ZOBRIST_KEYS.side();
        self.checkers = 0;
        self.set_pinned();

        info
    }

    pub fn undo_null_move(&mut self, info: UndoInfo) {
        self.castling_rights = info.castling_rights;
        self.en_passant_target_sq = info.en_passant_target_sq;
        self.half_move_clock = info.half_move_clock;
        self.pinned = info.pinned;
        self.hash = info.hash;

        self.ply -= 1;
        self.hash_history[self.ply as usize] = 0;
        self.to_play = !self.to_play;
    }

    #[inline]
    pub fn has_non_pawn_material(&self, color: Color) -> bool {
        (self.colors(color) & !(self.pieces(Piece::Pawn) | self.pieces(Piece::King))) != 0
    }
}

impl Board {
    #[inline]
    pub(super) fn set_checkers(&mut self) {
        self.checkers =
            self.generate_attackers(self.king_sq(self.to_play), !self.to_play, self.occupied());
    }

    #[inline]
    pub(super) fn set_pinned(&mut self) {
        let us = self.to_play;
        let king_sq = self.king_sq(us);

        let enemy = self.colors(!us);
        let our = self.colors(us);
        let occupied = self.occupied();

        let enemy_rooks = (self.pieces(Piece::Rook) | self.pieces(Piece::Queen)) & enemy;
        let enemy_bishops = (self.pieces(Piece::Bishop) | self.pieces(Piece::Queen)) & enemy;

        // generate attacks stopping ONLY at enemy pieces
        let potential_pinners = (rook_attacks(king_sq, enemy) & enemy_rooks)
            | (bishop_attacks(king_sq, enemy) & enemy_bishops);

        let mut pinned = 0;

        for_each_bit!(pinner_sq in potential_pinners => {
            let ray = bb_between(king_sq, pinner_sq);
            let blockers_on_ray = ray & occupied;

            // If there is exactly one piece on the ray and it's ours
            if bb_only_one(blockers_on_ray) {
                pinned |= blockers_on_ray & our;
            }
        });

        self.pinned = pinned;
    }

    #[inline]
    pub(super) fn add_piece(&mut self, sq: Sq, piece: ColoredPiece) {
        self.mailbox[sq as usize] = Some(piece);
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) |= bitboard;
        *self.pieces_mut(piece.piece()) |= bitboard;
        self.hash ^= ZOBRIST_KEYS.piece(sq, piece);
    }

    #[inline(always)]
    fn move_piece(&mut self, from: Sq, to: Sq) -> ColoredPiece {
        let piece = unsafe { self.mailbox[from as usize].take().unwrap_unchecked() };
        self.mailbox[to as usize] = Some(piece);

        let move_bb = from.bitboard() ^ to.bitboard();
        *self.colors_mut(piece.color()) ^= move_bb;
        *self.pieces_mut(piece.piece()) ^= move_bb;
        self.hash ^= ZOBRIST_KEYS.piece(from, piece);
        self.hash ^= ZOBRIST_KEYS.piece(to, piece);

        piece
    }

    #[inline]
    fn remove_piece(&mut self, sq: Sq) -> Option<ColoredPiece> {
        let piece = self.mailbox[sq as usize].take()?;
        let bitboard = sq.bitboard();
        *self.colors_mut(piece.color()) &= !bitboard;
        *self.pieces_mut(piece.piece()) &= !bitboard;
        self.hash ^= ZOBRIST_KEYS.piece(sq, piece);

        Some(piece)
    }

    #[inline(always)]
    fn has_insufficient_material(&self) -> bool {
        // If there are pawns, rooks, or queens, mate is possible
        let majors_and_pawns =
            self.pieces(Piece::Pawn) | self.pieces(Piece::Rook) | self.pieces(Piece::Queen);

        if majors_and_pawns != 0 {
            return false;
        }

        let knights = self.pieces(Piece::Knight);
        let bishops = self.pieces(Piece::Bishop);
        let minors = knights | bishops;

        // King vs King
        if minors == 0 {
            return true;
        }

        // King + Minor vs King (K+N vs K or K+B vs K)
        if bb_only_one(minors) {
            return true;
        }

        // King + Bishop vs King + Bishop on the same color squares
        if knights == 0 && bb_only_one(minors & minors.wrapping_sub(1)) {
            let white_bishops = bishops & self.colors(Color::White);
            let black_bishops = bishops & self.colors(Color::Black);
            if bb_only_one(white_bishops) && bb_only_one(black_bishops) {
                return (white_bishops & LIGHT_SQUARES != 0)
                    == (black_bishops & LIGHT_SQUARES != 0);
            }
        }

        false
    }

    #[inline(always)]
    fn is_repetition(&self) -> bool {
        let current_hash = self.hash;
        let count = self.half_move_clock as usize;
        let len = self.ply as usize;
        let limit = count.min(len);

        if limit < 4 {
            return false;
        }

        for i in (4..=limit).step_by(2) {
            if self.hash_history[len - i] == current_hash {
                return true;
            }
        }

        false
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  +---+---+---+---+---+---+---+---+")?;
        for rank in (0..8u8).rev() {
            write!(f, "{} |", rank + 1)?;
            for file in 0..8u8 {
                let sq = Sq::new(file, rank).unwrap();
                let ch = match self.mailbox[sq as usize] {
                    Some(cp) => match (cp.piece(), cp.color()) {
                        (Piece::Pawn, Color::White) => 'P',
                        (Piece::Knight, Color::White) => 'N',
                        (Piece::Bishop, Color::White) => 'B',
                        (Piece::Rook, Color::White) => 'R',
                        (Piece::Queen, Color::White) => 'Q',
                        (Piece::King, Color::White) => 'K',
                        (Piece::Pawn, Color::Black) => 'p',
                        (Piece::Knight, Color::Black) => 'n',
                        (Piece::Bishop, Color::Black) => 'b',
                        (Piece::Rook, Color::Black) => 'r',
                        (Piece::Queen, Color::Black) => 'q',
                        (Piece::King, Color::Black) => 'k',
                    },
                    None => ' ',
                };
                write!(f, " {} |", ch)?;
            }
            writeln!(f)?;
            writeln!(f, "  +---+---+---+---+---+---+---+---+")?;
        }
        writeln!(f, "    a   b   c   d   e   f   g   h")?;
        writeln!(f)?;
        writeln!(f, "Side to move: {:?}", self.to_play)?;
        writeln!(f, "Castling:     {:?}", self.castling_rights)?;
        writeln!(
            f,
            "En passant:   {}",
            match self.en_passant_target_sq {
                Some(sq) => format!("{}", sq),
                None => "-".to_string(),
            }
        )?;
        writeln!(f, "Half-move:    {}", self.half_move_clock)?;
        write!(f, "Ply:          {}", self.ply)
    }
}
