use chess_core::prelude::*;

use crate::{board::Board, search::KillerMoves};

mod generate;
mod move_list;
mod traits;

pub use generate::*;
pub use move_list::*;
pub use traits::*;

// Indexed MVV_LVA[victim][attacker], both in `Piece` enum order (P, N, B, R, Q, K).
// Higher victim value and cheaper attacker => higher score.
const MVV_LVA: [[u8; Piece::NB]; Piece::NB] = [
    [15, 14, 13, 12, 11, 10], // victim P; attacker P, N, B, R, Q, K
    [25, 24, 23, 22, 21, 20], // victim N; attacker P, N, B, R, Q, K
    [35, 34, 33, 32, 31, 30], // victim B; attacker P, N, B, R, Q, K
    [45, 44, 43, 42, 41, 40], // victim R; attacker P, N, B, R, Q, K
    [55, 54, 53, 52, 51, 50], // victim Q; attacker P, N, B, R, Q, K
    [0, 0, 0, 0, 0, 0],       // victim K; never actually captured
];

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
enum GenStage {
    #[default]
    Init,
    Captures,
    Quiets,
    Evasions,
    Done,
}

pub struct MoveGenerator {
    list: MoveList,
    score_list: [u8; MAX_MOVES],
    stage: GenStage,
    list_index: usize,
    quiescence: bool,
    pub generated_count: usize,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self {
            list: MoveList::default(),
            score_list: [0; MAX_MOVES],
            stage: GenStage::default(),
            list_index: 0,
            quiescence: false,
            generated_count: 0,
        }
    }
}

impl MoveGenerator {
    pub fn quiescence() -> Self {
        Self {
            quiescence: true,
            ..Default::default()
        }
    }

    pub fn next(&mut self, board: &Board, killer_moves: KillerMoves) -> Option<Move> {
        loop {
            if self.list_index < self.list.len() {
                let mut best_index = self.list_index;
                let mut best_score = self.score_list[self.list_index];
                for i in (self.list_index + 1)..self.list.len() {
                    if self.score_list[i] > best_score {
                        best_score = self.score_list[i];
                        best_index = i;
                    }
                }

                self.list.as_slice_mut().swap(self.list_index, best_index);
                self.score_list.swap(self.list_index, best_index);

                let mov = self.list.as_slice()[self.list_index];
                self.list_index += 1;
                return Some(mov);
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
                    self.score_captures(board);

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
                    self.score_quiets(killer_moves);
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
                    self.score_list[..self.list.len()].fill(0); // TODO
                    self.stage = GenStage::Done;
                }
                GenStage::Done => return None,
            }
        }
    }

    fn score_captures(&mut self, board: &Board) {
        for (i, &mv) in self.list.as_slice().iter().enumerate() {
            // Missing piece during capture must mean en passant
            let to_piece = board
                .piece_at(mv.to())
                .map(|p| p.piece())
                .unwrap_or(Piece::Pawn);

            self.score_list[i] =
                MVV_LVA[to_piece as usize][board.piece_at(mv.from()).unwrap().piece() as usize];
        }
    }

    fn score_quiets(&mut self, killer_moves: KillerMoves) {
        for (i, &mv) in self.list.as_slice().iter().enumerate() {
            self.score_list[i] = if mv == killer_moves[0] {
                2
            } else if mv == killer_moves[1] {
                1
            } else {
                0
            };
        }
    }
}

pub fn gen_moves<Us: Player, Type: MoveGenType>(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let ptr = generate_moves::<Us, Type>(board, moves.as_ptr());
    moves.update_size(ptr);
    moves
}
