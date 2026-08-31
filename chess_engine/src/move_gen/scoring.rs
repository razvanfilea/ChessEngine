use chess_core::prelude::*;

use crate::{
    board::Board,
    search::{HistoryTable, KillerMoves},
};

const PIECE_VALUES: [i16; Piece::NB] = [1, 2, 3, 4, 5, 6]; // P, N, B, R, Q, K

static MVV_LVA: [[i16; Piece::NB]; Piece::NB] = const {
    let mut table = [[0; Piece::NB]; Piece::NB];
    let mut victim = 0;
    while victim < Piece::NB {
        let mut attacker = 0;
        while attacker < Piece::NB {
            // e.g. victim * 10 - attacker
            table[victim][attacker] = PIECE_VALUES[victim] * 10 - PIECE_VALUES[attacker];
            attacker += 1;
        }
        victim += 1;
    }
    table
};
pub const GOOD_CAPTURES: i16 = 20_000;
pub const KILLER_1: i16 = 15_000;
pub const KILLER_2: i16 = 14_000;
pub const BAD_CAPTURES: i16 = 5_000;

#[inline(always)]
fn mvv_lva(victim: Piece, attacker: Piece) -> i16 {
    MVV_LVA[victim as usize][attacker as usize]
}

#[inline(always)]
pub fn score_capture(mov: Move, board: &Board) -> i16 {
    let attacker = board
        .piece_at(mov.from())
        .map_or(Piece::Pawn, |p| p.piece());

    let victim = if mov.flags() == MoveFlags::EnPassant {
        Piece::Pawn
    } else {
        board.piece_at(mov.to()).map_or(Piece::Pawn, |p| p.piece())
    };

    let base_score = mvv_lva(victim, attacker);

    let is_good = (attacker as u8) <= (victim as u8) || mov.is_promotion();
    let tier = if is_good { GOOD_CAPTURES } else { BAD_CAPTURES };

    let mut score = tier + base_score;
    if let Some(promo) = mov.promotion_piece() {
        score += PIECE_VALUES[promo as usize] * 10;
    }
    score
}

#[inline(always)]
pub fn score_quiet(
    mov: Move,
    killer_moves: KillerMoves,
    history: &HistoryTable,
    side: Color,
) -> i16 {
    if mov == killer_moves[0] {
        KILLER_1
    } else if mov == killer_moves[1] {
        KILLER_2
    } else {
        history.get(side, mov.from(), mov.to())
    }
}
