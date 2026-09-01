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
    start_ptr: MoveListPtr,
    end_ptr: MoveListPtr,
    stage: GenStage,
    list_index: usize,
    quiescence: bool,
    tt_move: Move,
}

impl MoveGenerator {
    pub fn new(move_buffer: MoveListPtr, tt_move: Move) -> Self {
        Self {
            start_ptr: move_buffer,
            end_ptr: move_buffer,
            stage: GenStage::default(),
            list_index: 0,
            quiescence: false,
            tt_move,
        }
    }

    pub fn quiescence(move_buffer: MoveListPtr, tt_move: Move) -> Self {
        let mut move_gen = Self::new(move_buffer, tt_move);
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
            if self.list_index < self.len() {
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
    pub const fn next_ptr(&self) -> MoveListPtr {
        self.end_ptr
    }

    #[inline(always)]
    const fn len(&self) -> usize {
        unsafe { self.end_ptr.0.offset_from(self.start_ptr.0) as usize }
    }

    #[inline(always)]
    const fn as_slice_mut(&mut self) -> &mut [ScoredMove] {
        unsafe { core::slice::from_raw_parts_mut(self.start_ptr.0, self.len()) }
    }

    #[inline(always)]
    fn pick_next(&mut self) -> Move {
        let idx = self.list_index;
        let moves = &mut self.as_slice_mut()[idx..];

        let mut best_index = 0;
        let mut best_move = moves[0];

        for (i, mov) in moves.iter().enumerate().skip(1) {
            if mov.score > best_move.score {
                best_move = *mov;
                best_index = i;
            }
        }

        unsafe {
            std::hint::assert_unchecked(best_index < moves.len());
        }

        moves.swap(0, best_index);

        self.list_index += 1;
        best_move.mov
    }

    #[inline(never)]
    fn advance_stage(
        &mut self,
        board: &Board,
        killer_moves: KillerMoves,
        history: &HistoryTable,
    ) -> Option<Move> {
        self.end_ptr = self.start_ptr;
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
                    generate_moves::<White, Captures>(board, self.start_ptr)
                } else {
                    generate_moves::<Black, Captures>(board, self.start_ptr)
                };
                self.end_ptr = ptr;

                for scored_move in self.as_slice_mut() {
                    scored_move.score = scoring::score_capture(scored_move.mov, board);
                }

                if self.quiescence {
                    self.stage = GenStage::Done;
                } else {
                    self.stage = GenStage::Quiets;
                }
            }
            GenStage::Quiets => {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Quiets>(board, self.start_ptr)
                } else {
                    generate_moves::<Black, Quiets>(board, self.start_ptr)
                };
                self.end_ptr = ptr;

                for scored_move in self.as_slice_mut() {
                    scored_move.score =
                        scoring::score_quiet(scored_move.mov, killer_moves, history, board.to_play);
                }

                self.stage = GenStage::Done;
            }
            GenStage::Evasions => {
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Evasions>(board, self.start_ptr)
                } else {
                    generate_moves::<Black, Evasions>(board, self.start_ptr)
                };
                self.end_ptr = ptr;

                for scored_move in self.as_slice_mut() {
                    scored_move.score = if scored_move.mov.is_tactical() {
                        scoring::score_capture(scored_move.mov, board)
                    } else {
                        scoring::score_quiet(scored_move.mov, killer_moves, history, board.to_play)
                    };
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
