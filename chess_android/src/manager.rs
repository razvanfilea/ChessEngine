use chess_core::prelude::*;
use chess_engine::{
    board::{Board, UndoInfo},
    move_gen::gen_all_moves,
    nnue::Accumulator,
    search::piece_value,
};

const NO_PIECE: u8 = 255;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    None = 0,
    WinnerWhite = 1,
    WinnerBlack = 2,
    Draw = 3,
    WhiteInCheck = 4,
    BlackInCheck = 5,
    Invalid = 10,
}

pub fn calculate_game_state(board: &Board) -> GameState {
    if board.is_draw() {
        return GameState::Draw;
    }

    let us = board.to_play;
    let them = !us;
    let them_king_sq = board.king_sq(them);
    let other_in_check = board.generate_attackers(them_king_sq, us, board.occupied()) != 0;

    if board.in_check() && other_in_check {
        return GameState::Invalid;
    }

    let legal_moves = gen_all_moves(board);
    let has_legal_move = legal_moves.as_slice().iter().any(|sm| board.legal(sm.mov));

    if !has_legal_move {
        if board.in_check() {
            if us == Color::White {
                GameState::WinnerBlack
            } else {
                GameState::WinnerWhite
            }
        } else {
            GameState::Draw
        }
    } else if board.in_check() {
        if us == Color::White {
            GameState::WhiteInCheck
        } else {
            GameState::BlackInCheck
        }
    } else {
        GameState::None
    }
}

#[derive(Clone, Debug)]
pub struct BoardSnapshot {
    pub game_state: i32,
    pub eval_score: i32,
    pub search_time_ms: i64,
    pub uci_info: String,
    pub is_white_turn: bool,
    pub current_move_index: i32,
    pub pieces: Vec<i32>,
    pub moves_history: Vec<i32>,
}

#[derive(Clone)]
struct HistoryEntry {
    mov: Move,
    undo_info: UndoInfo,
    captured_id: u8,
}

pub struct ChessGame {
    board: Board,
    start_fen: String,
    piece_ids: [u8; 64],
    history: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    player_is_white: bool,

    // Debug stats
    search_time_ms: u64,
    advanced_stats: String,
}

fn init_piece_ids(board: &Board) -> [u8; 64] {
    let mut ids = [NO_PIECE; 64];
    for (sq, _) in board
        .mailbox
        .iter()
        .enumerate()
        .filter(|(_, p)| p.is_some())
    {
        ids[sq] = sq as u8;
    }
    ids
}

impl ChessGame {
    pub fn new(player_is_white: bool) -> Self {
        let board = Board::start_pos();
        let piece_ids = init_piece_ids(&board);
        Self {
            start_fen: board.to_fen(),
            board,
            piece_ids,
            history: Vec::with_capacity(64),
            redo_stack: Vec::new(),
            player_is_white,
            search_time_ms: 0,
            advanced_stats: String::new(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn record_search_stats(&mut self, time_ms: u64, advanced_stats: String) {
        self.search_time_ms = time_ms;
        if !advanced_stats.is_empty() {
            self.advanced_stats = advanced_stats;
        }
    }

    pub fn load_fen_moves(
        &mut self,
        fen: &str,
        moves: &[i32],
        player_is_white: bool,
    ) -> Option<BoardSnapshot> {
        let board = Board::from_fen(fen)?;
        self.board = board;
        self.start_fen = fen.to_string();
        self.piece_ids = init_piece_ids(&self.board);
        self.history.clear();
        self.redo_stack.clear();
        self.player_is_white = player_is_white;
        self.search_time_ms = 0;
        self.advanced_stats.clear();

        for &m_int in moves {
            if m_int == 0 {
                break;
            }
            let mov = Move::from_bits(m_int as u16)?;
            if !self.board.legal(mov) {
                return None;
            }
            self.apply_move_internal(mov);
        }

        Some(self.get_snapshot())
    }

    fn apply_move_internal(&mut self, mov: Move) {
        let from = mov.from() as usize;
        let to = mov.to() as usize;
        let moving_id = self.piece_ids[from];

        let captured_sq = if mov.flags() == MoveFlags::EnPassant {
            mov.capture_square(self.board.to_play) as usize
        } else if mov.is_capture() {
            to
        } else {
            64
        };

        let captured_id = if captured_sq < 64 {
            let id = self.piece_ids[captured_sq];
            self.piece_ids[captured_sq] = NO_PIECE;
            id
        } else {
            NO_PIECE
        };

        self.piece_ids[to] = moving_id;
        self.piece_ids[from] = NO_PIECE;

        if mov.is_castle() {
            let (rook_from, rook_to) = mov.flags().castling_rook_squares(self.board.to_play);
            let r_from = rook_from as usize;
            let r_to = rook_to as usize;
            self.piece_ids[r_to] = self.piece_ids[r_from];
            self.piece_ids[r_from] = NO_PIECE;
        }

        let undo_info = self.board.make_move(mov);
        self.history.push(HistoryEntry {
            mov,
            undo_info,
            captured_id,
        });
    }

    pub fn make_move(&mut self, move_content: i32) -> BoardSnapshot {
        let Some(mov) = Move::from_bits(move_content as u16) else {
            return self.get_snapshot();
        };
        if !self.board.legal(mov) {
            return self.get_snapshot();
        }

        self.apply_move_internal(mov);
        self.redo_stack.clear();
        self.get_snapshot()
    }

    fn undo_single_ply(&mut self) -> bool {
        let Some(entry) = self.history.pop() else {
            return false;
        };

        let mov = entry.mov;
        let from = mov.from() as usize;
        let to = mov.to() as usize;

        self.board.undo_move(mov, entry.undo_info.clone());

        let moving_id = self.piece_ids[to];
        self.piece_ids[from] = moving_id;
        self.piece_ids[to] = NO_PIECE;

        if entry.captured_id != NO_PIECE {
            let captured_sq = if mov.flags() == MoveFlags::EnPassant {
                mov.capture_square(self.board.to_play) as usize
            } else {
                to
            };
            self.piece_ids[captured_sq] = entry.captured_id;
        }

        if mov.is_castle() {
            let (rook_from, rook_to) = mov.flags().castling_rook_squares(self.board.to_play);
            let r_from = rook_from as usize;
            let r_to = rook_to as usize;
            self.piece_ids[r_from] = self.piece_ids[r_to];
            self.piece_ids[r_to] = NO_PIECE;
        }

        self.redo_stack.push(entry);
        true
    }

    fn redo_single_ply(&mut self) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        self.apply_move_internal(entry.mov);
        true
    }

    pub fn undo(&mut self) -> Option<BoardSnapshot> {
        if self.history.is_empty() {
            return None;
        }

        self.undo_single_ply();
        let is_player_turn = (self.board.to_play == Color::White) == self.player_is_white;
        if !is_player_turn && !self.history.is_empty() {
            self.undo_single_ply();
        }

        Some(self.get_snapshot())
    }

    pub fn redo(&mut self) -> Option<BoardSnapshot> {
        if self.redo_stack.is_empty() {
            return None;
        }

        self.redo_single_ply();
        let is_player_turn = (self.board.to_play == Color::White) == self.player_is_white;
        if !is_player_turn && !self.redo_stack.is_empty() {
            self.redo_single_ply();
        }

        Some(self.get_snapshot())
    }

    pub fn get_possible_moves(&self, square_raw: u8) -> Vec<i32> {
        let Some(sq) = Sq::from_raw(square_raw) else {
            return Vec::new();
        };
        let mut moves = Vec::new();
        for scored in gen_all_moves(&self.board).as_slice() {
            let m = scored.mov;
            if m.from() == sq && self.board.legal(m) {
                moves.push(m.bits() as i32);
            }
        }
        moves
    }

    pub fn get_snapshot(&self) -> BoardSnapshot {
        let mut pieces = Vec::with_capacity(32);
        for sq in 0..64 {
            if let Some(cp) = self.board.mailbox[sq] {
                let id = self.piece_ids[sq] as i32;
                let square = sq as i32;
                let piece_type = cp.piece() as i32;
                let is_white = if cp.color() == Color::White { 1 } else { 0 };
                pieces.push(id | (square << 8) | (piece_type << 16) | (is_white << 24));
            }
        }

        let mut moves_history: Vec<i32> =
            self.history.iter().map(|e| e.mov.bits() as i32).collect();
        for entry in self.redo_stack.iter().rev() {
            moves_history.push(entry.mov.bits() as i32);
        }

        let current_move_index = self.history.len() as i32 - 1;

        BoardSnapshot {
            game_state: calculate_game_state(&self.board) as i32,
            eval_score: self.get_board_evaluation(),
            search_time_ms: self.search_time_ms as i64,
            uci_info: self.advanced_stats.clone(),
            is_white_turn: self.board.to_play == Color::White,
            current_move_index,
            pieces,
            moves_history,
        }
    }

    pub fn get_current_fen(&self) -> String {
        self.board.to_fen()
    }

    pub fn get_start_fen(&self) -> String {
        self.start_fen.clone()
    }

    pub fn get_board_evaluation(&self) -> i32 {
        let acc = Accumulator::from_board(&self.board);
        let raw = acc.eval(&self.board) as i32;
        let white_score = if self.board.to_play == Color::White {
            raw
        } else {
            -raw
        };
        white_score * 100 / (piece_value(Piece::Pawn) as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_start_state() {
        let game = ChessGame::new(true);
        let snap = game.get_snapshot();
        assert_eq!(snap.pieces.len(), 32);
        assert_eq!(snap.current_move_index, -1);
        assert!(snap.is_white_turn);
        assert_eq!(snap.moves_history.len(), 0);
    }

    #[test]
    fn test_game_move_and_piece_ids() {
        let mut game = ChessGame::new(true);
        let e2e4 = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn).bits() as i32;
        let snap1 = game.make_move(e2e4);
        assert_eq!(snap1.current_move_index, 0);

        // Verify that the piece on E4 has the same ID as the starting piece on E2
        let e4_piece = snap1
            .pieces
            .iter()
            .find(|&&p| ((p >> 8) & 0xFF) == Sq::E4 as i32)
            .unwrap();
        let e4_id = e4_piece & 0xFF;
        assert_eq!(e4_id, Sq::E2 as i32);
    }

    #[test]
    fn test_game_undo_and_redo() {
        let mut game = ChessGame::new(true);
        let e2e4 = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn).bits() as i32;
        game.make_move(e2e4);
        let e7e5 = Move::new(Sq::E7, Sq::E5, MoveFlags::DoublePawn).bits() as i32;
        game.make_move(e7e5);

        // Player is White (true). Undoing rolls back both Black's e5 and White's e4 back to start.
        let snap = game.undo().unwrap();
        assert_eq!(snap.current_move_index, -1);
        assert!(snap.is_white_turn);
        // Redo stack has both moves
        assert_eq!(snap.moves_history.len(), 2);

        // Redo rolls forward to end
        let snap = game.redo().unwrap();
        assert_eq!(snap.current_move_index, 1);
        assert!(snap.is_white_turn);
    }

    #[test]
    fn test_game_capture_and_undo() {
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        let mut game = ChessGame::new(true);
        game.load_fen_moves(fen, &[], true);

        // exd5
        let exd5 = Move::new(Sq::E4, Sq::D5, MoveFlags::Capture).bits() as i32;
        let snap = game.make_move(exd5);
        assert_eq!(snap.pieces.len(), 31);

        // Undo
        game.undo_single_ply();
        let snap = game.get_snapshot();
        assert_eq!(snap.pieces.len(), 32);
    }

    #[test]
    fn test_game_castling_and_undo() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let mut game = ChessGame::new(true);
        game.load_fen_moves(fen, &[], true);

        let ks_castle = Move::new(Sq::E1, Sq::G1, MoveFlags::CastleKing).bits() as i32;
        game.make_move(ks_castle);
        let snap = game.get_snapshot();

        let g1_piece = snap
            .pieces
            .iter()
            .find(|&&p| ((p >> 8) & 0xFF) == Sq::G1 as i32)
            .unwrap();
        let f1_piece = snap
            .pieces
            .iter()
            .find(|&&p| ((p >> 8) & 0xFF) == Sq::F1 as i32)
            .unwrap();
        assert_eq!(g1_piece & 0xFF, Sq::E1 as i32);
        assert_eq!(f1_piece & 0xFF, Sq::H1 as i32);

        game.undo_single_ply();
        let snap = game.get_snapshot();
        let e1_piece = snap
            .pieces
            .iter()
            .find(|&&p| ((p >> 8) & 0xFF) == Sq::E1 as i32)
            .unwrap();
        let h1_piece = snap
            .pieces
            .iter()
            .find(|&&p| ((p >> 8) & 0xFF) == Sq::H1 as i32)
            .unwrap();
        assert_eq!(e1_piece & 0xFF, Sq::E1 as i32);
        assert_eq!(h1_piece & 0xFF, Sq::H1 as i32);
    }
}
