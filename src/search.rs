use crate::{board::Board, eval::eval_board};

pub fn search(board: Board, depth: i32) {

}

pub fn negamax(board: &Board, depth: i32) -> i32 {
    if depth == 0 {
        return eval_board(board);
    }

    let mut value = i32::MIN;

    return value;
}
