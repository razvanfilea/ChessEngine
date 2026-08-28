use chess_core::{for_each_bit, prelude::*};

use crate::board::Board;

pub const INFINITY: i16 = 30_000;
pub const MATE_THRESHOLD: i16 = 29_000;

// Game phase weights for non-pawn pieces
const PHASE_WEIGHTS: [i32; Piece::NB] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

// Base piece values for Middlegame (MG) and Endgame (EG)
const PAWN_VALUE_MG: i32 = 82;
const PAWN_VALUE_EG: i32 = 94;
const KNIGHT_VALUE_MG: i32 = 337;
const KNIGHT_VALUE_EG: i32 = 281;
const BISHOP_VALUE_MG: i32 = 365;
const BISHOP_VALUE_EG: i32 = 297;
const ROOK_VALUE_MG: i32 = 477;
const ROOK_VALUE_EG: i32 = 512;
const QUEEN_VALUE_MG: i32 = 1025;
const QUEEN_VALUE_EG: i32 = 936;
const KING_VALUE_MG: i32 = 0;
const KING_VALUE_EG: i32 = 0;

const PIECE_VALUES_MG: [i32; Piece::NB] = [
    PAWN_VALUE_MG,
    KNIGHT_VALUE_MG,
    BISHOP_VALUE_MG,
    ROOK_VALUE_MG,
    QUEEN_VALUE_MG,
    KING_VALUE_MG,
];

const PIECE_VALUES_EG: [i32; Piece::NB] = [
    PAWN_VALUE_EG,
    KNIGHT_VALUE_EG,
    BISHOP_VALUE_EG,
    ROOK_VALUE_EG,
    QUEEN_VALUE_EG,
    KING_VALUE_EG,
];

// Piece-Square Tables (from White's perspective, index 0 = A1, index 63 = H8)
#[rustfmt::skip]
const PAWN_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
      0,   0,   0,   0,   0,   0,   0,   0,
    // Rank 2 (a2-h2)
    -35,  -1, -20, -23, -15,  24,  38, -22,
    // Rank 3 (a3-h3)
    -26,  -4,  -4, -10,   3,   3,  33, -12,
    // Rank 4 (a4-h4)
    -27,  -2,  -5,  12,  17,   6,  10, -25,
    // Rank 5 (a5-h5)
    -14,  13,   6,  21,  23,  12,  17, -23,
    // Rank 6 (a6-h6)
     -6,   7,  26,  31,  65,  56,  25, -20,
    // Rank 7 (a7-h7)
     98, 134,  61,  95,  68, 126,  34, -11,
    // Rank 8 (a8-h8)
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const PAWN_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
      0,   0,   0,   0,   0,   0,   0,   0,
    // Rank 2 (a2-h2)
     13,   8,   8,  10,  13,   0,   2,  -7,
    // Rank 3 (a3-h3)
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
    // Rank 4 (a4-h4)
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    // Rank 5 (a5-h5)
     32,  24,  13,   5,  -2,   4,  17,  17,
    // Rank 6 (a6-h6)
     94, 100,  85,  67,  56,  53,  82,  84,
    // Rank 7 (a7-h7)
    178, 173, 158, 134, 147, 132, 165, 187,
    // Rank 8 (a8-h8)
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -105, -21, -58, -33, -17, -28, -19,  -23,
    // Rank 2 (a2-h2)
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    // Rank 3 (a3-h3)
     -23,  -9,  12,  10,  19,  17,  25,  -16,
    // Rank 4 (a4-h4)
     -13,   4,  16,  13,  28,  19,  21,   -8,
    // Rank 5 (a5-h5)
      -9,  17,  19,  53,  37,  69,  18,   22,
    // Rank 6 (a6-h6)
     -47,  60,  37,  65,  84, 129,  73,   44,
    // Rank 7 (a7-h7)
     -73, -41,  72,  36,  23,  62,   7,  -17,
    // Rank 8 (a8-h8)
    -167, -89, -34, -49,  61, -97, -15, -107,
];

#[rustfmt::skip]
const KNIGHT_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -29, -51, -23, -15, -22, -18, -50, -64,
    // Rank 2 (a2-h2)
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    // Rank 3 (a3-h3)
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    // Rank 4 (a4-h4)
    -18,  -6,  16,  25,  16,  17,   4, -18,
    // Rank 5 (a5-h5)
    -17,   3,  22,  22,  22,  11,   8, -18,
    // Rank 6 (a6-h6)
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    // Rank 7 (a7-h7)
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    // Rank 8 (a8-h8)
    -58, -38, -13, -28, -31, -27, -63, -99,
];

#[rustfmt::skip]
const BISHOP_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -33,  -3, -14, -21, -13, -12, -39, -21,
    // Rank 2 (a2-h2)
      4,  15,  16,   0,   7,  21,  33,   1,
    // Rank 3 (a3-h3)
      0,  15,  15,  15,  14,  27,  18,  10,
    // Rank 4 (a4-h4)
     -6,  13,  13,  26,  34,  12,  10,   4,
    // Rank 5 (a5-h5)
     -4,   5,  19,  50,  37,  37,   7,  -2,
    // Rank 6 (a6-h6)
    -16,  37,  43,  40,  35,  50,  37,  -2,
    // Rank 7 (a7-h7)
    -26,  16, -18, -13,  30,  59,  18, -47,
    // Rank 8 (a8-h8)
    -29,   4, -82, -37, -25, -42,   7,  -8,
];

#[rustfmt::skip]
const BISHOP_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -23,  -9, -23,  -5,  -9, -16,  -5, -17,
    // Rank 2 (a2-h2)
    -14, -18,  -7,  -1,   4,  -9, -15, -27,
    // Rank 3 (a3-h3)
    -12,  -3,   8,  10,  13,   3,  -7, -15,
    // Rank 4 (a4-h4)
     -6,   3,  13,  19,   7,  10,  -3,  -9,
    // Rank 5 (a5-h5)
     -3,   9,  12,   9,  14,  10,   3,   2,
    // Rank 6 (a6-h6)
      2,  -8,   0,  -1,  -2,   6,   0,   4,
    // Rank 7 (a7-h7)
     -8,  -4,   7, -12,  -3, -13,  -4, -14,
    // Rank 8 (a8-h8)
    -14, -21, -11,  -8,  -7,  -9, -17, -24,
];

#[rustfmt::skip]
const ROOK_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -19, -13,   1,  17,  16,   7, -37, -26,
    // Rank 2 (a2-h2)
    -44, -16, -20,  -9,  -1,  11,  -6, -71,
    // Rank 3 (a3-h3)
    -45, -25, -16, -17,   3,   0,  -5, -33,
    // Rank 4 (a4-h4)
    -36, -26, -12,  -1,   9,  -7,   6, -23,
    // Rank 5 (a5-h5)
    -24, -11,   7,  26,  24,  35,  -8, -20,
    // Rank 6 (a6-h6)
     -5,  19,  26,  36,  17,  45,  61,  16,
    // Rank 7 (a7-h7)
     27,  32,  58,  62,  80,  67,  26,  44,
    // Rank 8 (a8-h8)
     32,  42,  32,  51,  63,   9,  31,  43,
];

#[rustfmt::skip]
const ROOK_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -9,   2,   3,  -1,  -5, -13,   4, -20,
    // Rank 2 (a2-h2)
    -6,  -6,   0,   2,  -9,  -9, -11,  -3,
    // Rank 3 (a3-h3)
    -4,   0,  -5,  -1,  -7, -12,  -8, -16,
    // Rank 4 (a4-h4)
     3,   5,   8,   4,  -5,  -6,  -8, -11,
    // Rank 5 (a5-h5)
     4,   3,  13,   1,   2,   1,  -1,   2,
    // Rank 6 (a6-h6)
     7,   7,   7,   5,   4,  -3,  -5,  -3,
    // Rank 7 (a7-h7)
    11,  13,  13,  11,  -3,   3,   8,   3,
    // Rank 8 (a8-h8)
    13,  10,  18,  15,  12,  12,   8,   5,
];

#[rustfmt::skip]
const QUEEN_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
     -1, -18,  -9,  10, -15, -25, -31, -50,
    // Rank 2 (a2-h2)
    -35,  -8,  11,   2,   8,  15,  -3,   1,
    // Rank 3 (a3-h3)
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    // Rank 4 (a4-h4)
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    // Rank 5 (a5-h5)
    -27, -27, -16, -16,  -1,  17,  -2,   1,
    // Rank 6 (a6-h6)
    -13, -17,   7,   8,  29,  56,  47,  57,
    // Rank 7 (a7-h7)
    -24, -39,  -5,   1, -16,  57,  28,  54,
    // Rank 8 (a8-h8)
    -28,   0,  29,  12,  59,  44,  43,  45,
];

#[rustfmt::skip]
const QUEEN_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -33, -28, -22, -43,  -5, -32, -20, -41,
    // Rank 2 (a2-h2)
    -22, -23, -30, -16, -16, -23, -36, -32,
    // Rank 3 (a3-h3)
    -16, -27,  15,   6,   9,  17,  10,   5,
    // Rank 4 (a4-h4)
    -18,  28,  19,  47,  31,  34,  39,  23,
    // Rank 5 (a5-h5)
       3,  22,  24,  45,  57,  40,  57,  36,
    // Rank 6 (a6-h6)
    -20,   6,   9,  49,  47,  35,  19,   9,
    // Rank 7 (a7-h7)
    -17,  20,  32,  41,  58,  25,  30,   0,
    // Rank 8 (a8-h8)
      -9,  22,  22,  27,  27,  19,  10,  20,
];

#[rustfmt::skip]
const KING_PST_MG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -15,  36,  12, -54,   8, -28,  24,  14,
    // Rank 2 (a2-h2)
       1,   7,  -8, -64, -43, -16,   9,   8,
    // Rank 3 (a3-h3)
    -14, -14, -22, -46, -44, -30, -15, -27,
    // Rank 4 (a4-h4)
    -49,  -1, -27, -39, -46, -44, -33, -51,
    // Rank 5 (a5-h5)
    -17, -20, -12, -27, -30, -25, -14, -36,
    // Rank 6 (a6-h6)
      -9,  24,   2, -16, -20,   6,  22, -22,
    // Rank 7 (a7-h7)
      29,  -1, -20,  -7,  -8,  -4, -38, -29,
    // Rank 8 (a8-h8)
    -65,  23,  16, -15, -56, -34,   2,  13,
];

#[rustfmt::skip]
const KING_PST_EG: [i32; 64] = [
    // Rank 1 (a1-h1)
    -53, -34, -21, -11, -28, -14, -24, -43,
    // Rank 2 (a2-h2)
    -27, -11,   4,  13,  14,   4,  -5, -17,
    // Rank 3 (a3-h3)
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    // Rank 4 (a4-h4)
    -18,  -4,  21,  24,  27,  23,   9, -11,
    // Rank 5 (a5-h5)
      -8,  22,  24,  27,  26,  33,  26,   3,
    // Rank 6 (a6-h6)
      10,  17,  23,  15,  20,  45,  44,  13,
    // Rank 7 (a7-h7)
     -12,  17,  14,  17,  17,  38,  23,  11,
    // Rank 8 (a8-h8)
    -74, -35, -18, -18, -11,  15,   4, -17,
];

/// Precomputed tables combining piece value + positional score for every (Color, Piece, Sq).
/// White uses the square directly, Black vertically flips the rank (`sq ^ 56`).
static PIECE_TABLES_MG: [[[i32; Sq::NB]; Piece::NB]; Color::NB] = const {
    let raw_tables = [
        PAWN_PST_MG,
        KNIGHT_PST_MG,
        BISHOP_PST_MG,
        ROOK_PST_MG,
        QUEEN_PST_MG,
        KING_PST_MG,
    ];

    let mut tables = [[[0; Sq::NB]; Piece::NB]; Color::NB];
    let mut p = 0;
    while p < Piece::NB {
        let piece_val = PIECE_VALUES_MG[p];
        let mut sq = 0;
        while sq < Sq::NB {
            tables[Color::White as usize][p][sq] = piece_val + raw_tables[p][sq];
            tables[Color::Black as usize][p][sq] = piece_val + raw_tables[p][sq ^ 56];
            sq += 1;
        }
        p += 1;
    }
    tables
};

static PIECE_TABLES_EG: [[[i32; Sq::NB]; Piece::NB]; Color::NB] = const {
    let raw_tables = [
        PAWN_PST_EG,
        KNIGHT_PST_EG,
        BISHOP_PST_EG,
        ROOK_PST_EG,
        QUEEN_PST_EG,
        KING_PST_EG,
    ];

    let mut tables = [[[0; Sq::NB]; Piece::NB]; Color::NB];
    let mut p = 0;
    while p < Piece::NB {
        let piece_val = PIECE_VALUES_EG[p];
        let mut sq = 0;
        while sq < Sq::NB {
            tables[Color::White as usize][p][sq] = piece_val + raw_tables[p][sq];
            tables[Color::Black as usize][p][sq] = piece_val + raw_tables[p][sq ^ 56];
            sq += 1;
        }
        p += 1;
    }
    tables
};

/// Evaluates the board position from the perspective of the side to move using tapered PeSTO evaluation.
pub fn eval_board(board: &Board) -> i16 {
    let mut white_mg = 0;
    let mut white_eg = 0;
    let mut black_mg = 0;
    let mut black_eg = 0;

    for piece in Piece::ALL {
        let p_idx = piece as usize;

        let white_bb = board.color_piece(piece, Color::White);
        for_each_bit!(sq in white_bb => {
            white_mg += PIECE_TABLES_MG[Color::White as usize][p_idx][sq as usize];
            white_eg += PIECE_TABLES_EG[Color::White as usize][p_idx][sq as usize];
        });

        let black_bb = board.color_piece(piece, Color::Black);
        for_each_bit!(sq in black_bb => {
            black_mg += PIECE_TABLES_MG[Color::Black as usize][p_idx][sq as usize];
            black_eg += PIECE_TABLES_EG[Color::Black as usize][p_idx][sq as usize];
        });
    }

    let mg_score = white_mg - black_mg;
    let eg_score = white_eg - black_eg;

    let phase = (board.pieces(Piece::Knight).count_ones()
        * (PHASE_WEIGHTS[Piece::Knight as usize] as u32)
        + board.pieces(Piece::Bishop).count_ones() * (PHASE_WEIGHTS[Piece::Bishop as usize] as u32)
        + board.pieces(Piece::Rook).count_ones() * (PHASE_WEIGHTS[Piece::Rook as usize] as u32)
        + board.pieces(Piece::Queen).count_ones() * (PHASE_WEIGHTS[Piece::Queen as usize] as u32))
        .min(MAX_PHASE as u32) as i32;

    let mg_phase = phase;
    let eg_phase = MAX_PHASE - phase;

    let score = (mg_score * mg_phase + eg_score * eg_phase) / MAX_PHASE;

    if board.to_play == Color::White {
        score as i16
    } else {
        -score as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_pos_eval_is_zero() {
        let board = Board::start_pos();
        assert_eq!(eval_board(&board), 0);
    }

    #[test]
    fn test_color_symmetry() {
        let white_lead =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let black_lead =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();

        assert!(eval_board(&white_lead) > 0);
        assert_eq!(eval_board(&white_lead), eval_board(&black_lead));
    }

    #[test]
    fn test_material_queen_advantage() {
        let white_extra_queen =
            Board::from_fen("rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let score = eval_board(&white_extra_queen);
        assert!(score >= QUEEN_VALUE_MG as i16 - 100);
    }

    #[test]
    fn test_knight_central_vs_corner() {
        let central_knight = Board::from_fen("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1").unwrap();
        let corner_knight = Board::from_fen("4k3/8/8/8/8/8/8/N3K3 w - - 0 1").unwrap();

        assert!(eval_board(&central_knight) > eval_board(&corner_knight));
    }

    #[test]
    fn test_passed_pawn_advancement() {
        let pawn_e2 = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
        let pawn_e7 = Board::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        assert!(eval_board(&pawn_e7) > eval_board(&pawn_e2));
    }

    #[test]
    fn test_endgame_king_centralization() {
        let central_king = Board::from_fen("7k/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
        let corner_king = Board::from_fen("7k/8/8/8/8/8/8/K7 w - - 0 1").unwrap();

        assert!(eval_board(&central_king) > eval_board(&corner_king));
    }
}
