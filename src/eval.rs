use chess_base::prelude::*;

use crate::board::Board;

pub const INFINITY: i16 = 30_000;

pub fn eval_board(board: &Board) -> i16 {
    let pawns = (board.color_piece(Pieces::Pawn, Color::White).count_ones() as i16
        - board.color_piece(Pieces::Pawn, Color::Black).count_ones() as i16)
        * 150;
    let knight = (board.color_piece(Pieces::Knight, Color::White).count_ones() as i16
        - board.color_piece(Pieces::Knight, Color::Black).count_ones() as i16)
        * 300;
    let bishop = (board
        .color_piece(Pieces::Bischop, Color::White)
        .count_ones() as i16
        - board
            .color_piece(Pieces::Bischop, Color::Black)
            .count_ones() as i16)
        * 350;
    let rook = (board.color_piece(Pieces::Rook, Color::White).count_ones() as i16
        - board.color_piece(Pieces::Rook, Color::Black).count_ones() as i16)
        * 500;
    let queen = (board.color_piece(Pieces::Queen, Color::White).count_ones() as i16
        - board.color_piece(Pieces::Queen, Color::Black).count_ones() as i16)
        * 900;

    let total = pawns + knight + bishop + rook + queen;
    if board.to_play == Color::White {
        total
    } else {
        -total
    }
}
