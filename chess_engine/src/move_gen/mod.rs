use std::mem::MaybeUninit;

use chess_core::prelude::*;

use crate::{
    board::Board,
    search::{HistoryTable, KillerMoves},
};

mod generate;
mod move_list;
mod scoring;
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

pub struct MoveGenerator {
    list: MoveList,
    score_list: [MaybeUninit<i16>; MAX_MOVES],
    stage: GenStage,
    list_index: usize,
    quiescence: bool,
    tt_move: Move,
}

impl MoveGenerator {
    pub fn new(tt_move: Move) -> Self {
        Self {
            list: MoveList::default(),
            score_list: [const { MaybeUninit::uninit() }; MAX_MOVES],
            stage: GenStage::default(),
            list_index: 0,
            quiescence: false,
            tt_move,
        }
    }

    pub fn quiescence(tt_move: Move) -> Self {
        let mut move_gen = Self::new(tt_move);
        move_gen.quiescence = true;
        move_gen
    }

    #[inline(always)]
    pub fn next(
        &mut self,
        board: &Board,
        killer_moves: KillerMoves,
        history: &HistoryTable,
    ) -> Option<Move> {
        loop {
            if self.list_index < self.list.len() {
                let mov = self.pick_next();
                if mov == self.tt_move {
                    continue;
                }
                return Some(mov);
            }

            if self.stage == GenStage::Done {
                return None;
            }

            if let Some(mov) = self.advance_stage(board, killer_moves, history) {
                return Some(mov);
            }
        }
    }

    #[inline(always)]
    fn pick_next(&mut self) -> Move {
        let idx = self.list_index;
        let len = self.list.len();

        let moves = self.list.as_slice_mut();
        let scores = &mut self.score_list[..len];

        let mut best_index = idx;
        let mut best_score = unsafe { scores[idx].assume_init() };

        for i in (idx + 1)..len {
            let score = unsafe { scores[i].assume_init() };
            if score > best_score {
                best_score = score;
                best_index = i;
            }
        }

        moves.swap(idx, best_index);
        scores.swap(idx, best_index);

        self.list_index += 1;
        moves[idx]
    }

    #[inline(never)]
    fn advance_stage(
        &mut self,
        board: &Board,
        killer_moves: KillerMoves,
        history: &HistoryTable,
    ) -> Option<Move> {
        self.list.clear();
        self.list_index = 0;

        match self.stage {
            GenStage::Init => {
                self.stage = if board.checkers != 0 {
                    GenStage::Evasions
                } else {
                    GenStage::Captures
                };

                if board.pseudo_legal(self.tt_move) {
                    return Some(self.tt_move);
                }
            }
            GenStage::Captures => {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Captures>(board, self.list.as_ptr())
                } else {
                    generate_moves::<Black, Captures>(board, self.list.as_ptr())
                };
                self.list.update_size(ptr);

                let len = self.list.len();
                let moves = self.list.as_slice();
                for i in 0..len {
                    let score = scoring::score_capture(moves[i], board);
                    self.score_list[i].write(score);
                }

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

                let len = self.list.len();
                let moves = self.list.as_slice();
                for i in 0..len {
                    let score =
                        scoring::score_quiet(moves[i], killer_moves, history, board.to_play);
                    self.score_list[i].write(score);
                }

                self.stage = GenStage::Done;
            }
            GenStage::Evasions => {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Evasions>(board, self.list.as_ptr())
                } else {
                    generate_moves::<Black, Evasions>(board, self.list.as_ptr())
                };
                self.list.update_size(ptr);

                let len = self.list.len();
                let moves = self.list.as_slice();
                for i in 0..len {
                    let mov = moves[i];
                    let score = if mov.is_capture() || mov.is_promotion() {
                        scoring::score_capture(mov, board)
                    } else {
                        scoring::score_quiet(mov, killer_moves, history, board.to_play)
                    };
                    self.score_list[i].write(score);
                }

                self.stage = GenStage::Done;
            }
            GenStage::Done => {}
        }

        None
    }
}

pub fn gen_all_moves(board: &Board) -> MoveList {
    let in_check = board.checkers != 0;
    let mut moves = MoveList::default();
    let ptr = match (board.to_play, in_check) {
        (Color::White, true) => generate_moves::<White, Evasions>(board, moves.as_ptr()),
        (Color::White, false) => generate_moves::<White, NonEvasions>(board, moves.as_ptr()),
        (Color::Black, true) => generate_moves::<Black, Evasions>(board, moves.as_ptr()),
        (Color::Black, false) => generate_moves::<Black, NonEvasions>(board, moves.as_ptr()),
    };
    moves.update_size(ptr);
    moves
}

pub fn gen_moves<Us: Player, Type: MoveGenType>(board: &Board) -> MoveList {
    let mut moves = MoveList::default();
    let ptr = generate_moves::<Us, Type>(board, moves.as_ptr());
    moves.update_size(ptr);
    moves
}
