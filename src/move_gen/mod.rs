use crate::board::Board;

pub mod attacks;


pub fn generate_moves(board: &Board) {
    let us = board.to_play;
    let them = !us;
}
