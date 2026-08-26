use chess_base::prelude::*;

use crate::board::Board;

mod generate;
mod move_list;
mod traits;

pub use generate::*;
pub use move_list::*;
pub use traits::*;

#[derive(Default)]
pub struct MoveGenerator {
    list: MoveList,
    generated_captures: bool,
    generated_quiets: bool,
    list_index: usize,
    pub generated_count: usize,
}

impl MoveGenerator {
    pub fn next(&mut self, board: &Board) -> Option<Move> {
        while self.list_index == self.list.len() {
            self.list.clear();
            self.list_index = 0;

            if board.checkers != 0 {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Evasions>(board, self.list.as_ptr())
                } else {
                    generate_moves::<Black, Evasions>(board, self.list.as_ptr())
                };
                self.list.update_size(ptr);

                self.generated_count += self.list.len();
                self.generated_captures = true;
                self.generated_quiets = true;
                continue;
            }

            if !self.generated_captures {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Captures>(board, self.list.as_ptr())
                } else {
                    generate_moves::<Black, Captures>(board, self.list.as_ptr())
                };
                self.list.update_size(ptr);

                self.generated_count += self.list.len();
                self.generated_captures = true;
                continue;
            }

            if !self.generated_quiets {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Quiets>(board, self.list.as_ptr())
                } else {
                    generate_moves::<Black, Quiets>(board, self.list.as_ptr())
                };
                self.list.update_size(ptr);

                self.generated_count += self.list.len();
                self.generated_quiets = true;
                break; // Nothing to generate further
            }
        }

        if self.list_index < self.list.len() {
            let mov= Some(self.list.as_slice()[self.list_index]);
            self.list_index += 1;
            return mov;
        }

        None
    }
}

pub fn gen_moves<Us: Player, Type: MoveGenType>(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let ptr = generate_moves::<Us, Type>(board, moves.as_ptr());
    moves.update_size(ptr);
    moves
}

