use std::str::FromStr;

use chess_base::prelude::*;

use super::Board;
use crate::zobrist::ZOBRIST_KEYS;

pub const START_POS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Parses a FEN (Forsyth–Edwards Notation) string into a `Board`.
pub fn parse_fen(fen: &str) -> Option<Board> {
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

    if board.to_play == Color::White {
        board.hash ^= ZOBRIST_KEYS.side();
    }
    board.hash ^= ZOBRIST_KEYS.castling(board.castling_rights);
    if let Some(ep_sq) = board.en_passant_target_sq {
        board.hash ^= ZOBRIST_KEYS.en_passant(ep_sq);
    }

    if board.color_piece(Piece::King, board.to_play) != 0 {
        board.set_checkers();
        board.set_pinned();
    }

    Some(board)
}

/// Formats a `Board` state into its standard FEN string representation.
pub fn format_fen(board: &Board) -> String {
    let mut fen = String::with_capacity(90);

    // 1. Piece placement
    for rank in (0..8u8).rev() {
        let mut empty_count = 0;
        for file in 0..8u8 {
            let sq = Sq::new(file, rank).unwrap();
            if let Some(cp) = board.piece_at(sq) {
                if empty_count > 0 {
                    fen.push((b'0' + empty_count) as char);
                    empty_count = 0;
                }
                let piece_char = match (cp.piece(), cp.color()) {
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
                };
                fen.push(piece_char);
            } else {
                empty_count += 1;
            }
        }
        if empty_count > 0 {
            fen.push((b'0' + empty_count) as char);
        }
        if rank > 0 {
            fen.push('/');
        }
    }

    // 2. Active color
    fen.push(' ');
    fen.push(if board.to_play == Color::White { 'w' } else { 'b' });

    // 3. Castling availability
    fen.push(' ');
    let mut any_castling = false;
    if board.castling_rights.contains(CastlingRights::WHITE_00) {
        fen.push('K');
        any_castling = true;
    }
    if board.castling_rights.contains(CastlingRights::WHITE_000) {
        fen.push('Q');
        any_castling = true;
    }
    if board.castling_rights.contains(CastlingRights::BLACK_00) {
        fen.push('k');
        any_castling = true;
    }
    if board.castling_rights.contains(CastlingRights::BLACK_000) {
        fen.push('q');
        any_castling = true;
    }
    if !any_castling {
        fen.push('-');
    }

    // 4. En passant target square
    fen.push(' ');
    if let Some(sq) = board.en_passant_target_sq {
        fen.push_str(&sq.to_string());
    } else {
        fen.push('-');
    }

    // 5. Halfmove clock
    fen.push(' ');
    fen.push_str(&board.half_move_clock.to_string());

    // 6. Fullmove counter
    fen.push(' ');
    let full_move = (board.ply / 2) + 1;
    fen.push_str(&full_move.to_string());

    fen
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_fen(s).ok_or(())
    }
}
