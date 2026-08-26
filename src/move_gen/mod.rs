use chess_base::prelude::*;

use crate::board::Board;

mod generate;
mod move_list;
mod traits;

pub use generate::*;
pub use move_list::*;
pub use traits::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
enum GenStage {
    #[default]
    Init,
    Captures,
    Quiets,
    Evasions,
    Done,
}

#[derive(Default)]
pub struct MoveGenerator {
    list: MoveList,
    stage: GenStage,
    list_index: usize,
    quiescence: bool,
    pub generated_count: usize,
}

impl MoveGenerator {
    pub fn quiescence() -> Self {
        Self {
            quiescence: true,
            ..Default::default()
        }
    }

    pub fn next(&mut self, board: &Board) -> Option<Move> {
        loop {
            if self.list_index < self.list.len() {
                let mov = Some(self.list.as_slice()[self.list_index]);
                self.list_index += 1;
                return mov;
            }

            self.list.clear();
            self.list_index = 0;

            match self.stage {
                GenStage::Init => {
                    if board.checkers != 0 {
                        self.stage = GenStage::Evasions;
                    } else {
                        self.stage = GenStage::Captures;
                    }
                }
                GenStage::Captures => {
                    let ptr = if board.to_play == Color::White {
                        generate_moves::<White, Captures>(board, self.list.as_ptr())
                    } else {
                        generate_moves::<Black, Captures>(board, self.list.as_ptr())
                    };
                    self.list.update_size(ptr);

                    self.generated_count += self.list.len();
                    if self.quiescence {
                        self.stage = GenStage::Done;
                    } else {
                        self.stage = GenStage::Quiets;
                    }
                }
                GenStage::Quiets => {
                    let ptr = if board.to_play == Color::White {
                        generate_moves::<White, Quiets>(board, self.list.as_ptr())
                    } else {
                        generate_moves::<Black, Quiets>(board, self.list.as_ptr())
                    };
                    self.list.update_size(ptr);

                    self.generated_count += self.list.len();
                    self.stage = GenStage::Done;
                }
                GenStage::Evasions => {
                    let ptr = if board.to_play == Color::White {
                        generate_moves::<White, Evasions>(board, self.list.as_ptr())
                    } else {
                        generate_moves::<Black, Evasions>(board, self.list.as_ptr())
                    };
                    self.list.update_size(ptr);

                    self.generated_count += self.list.len();
                    self.stage = GenStage::Done;
                }
                GenStage::Done => return None,
            }
        }
    }
}

pub fn gen_moves<Us: Player, Type: MoveGenType>(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let ptr = generate_moves::<Us, Type>(board, moves.as_ptr());
    moves.update_size(ptr);
    moves
}
