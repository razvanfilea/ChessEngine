use chess_base::prelude::*;

use crate::board::Board;

pub const INFINITY: i16 = 30_000;

pub fn eval_board(board: &Board) -> i16 {
    let pawns = (board.color_piece(Piece::Pawn, Color::White).count_ones() as i16
        - board.color_piece(Piece::Pawn, Color::Black).count_ones() as i16)
        * 150;
    let knight = (board.color_piece(Piece::Knight, Color::White).count_ones() as i16
        - board.color_piece(Piece::Knight, Color::Black).count_ones() as i16)
        * 300;
    let bishop = (board.color_piece(Piece::Bishop, Color::White).count_ones() as i16
        - board.color_piece(Piece::Bishop, Color::Black).count_ones() as i16)
        * 350;
    let rook = (board.color_piece(Piece::Rook, Color::White).count_ones() as i16
        - board.color_piece(Piece::Rook, Color::Black).count_ones() as i16)
        * 500;
    let queen = (board.color_piece(Piece::Queen, Color::White).count_ones() as i16
        - board.color_piece(Piece::Queen, Color::Black).count_ones() as i16)
        * 900;

    let total = pawns + knight + bishop + rook + queen;
    if board.to_play == Color::White {
        total
    } else {
        -total
    }
}
