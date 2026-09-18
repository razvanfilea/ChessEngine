use chess_core::prelude::*;

use crate::{
    board::Board,
    search::{ContHistKeys, ContinuationHistoryTable, HistoryTable, KillerMoves},
};

mod generate;
mod move_list;
pub mod scoring;
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
        cont_history: &ContinuationHistoryTable,
        conthist_keys: &ContHistKeys,
    ) -> Option<ScoredMove> {
        loop {
            if self.list_index < self.len() {
                let mov = self.pick_next();
                if mov == self.tt_move {
                    continue;
                }

                // First generate quiets instead of bad captures
                if self.stage != GenStage::Done && mov.score < 0 {
                    self.list_index -= 1; // add back the move we were about to return
                    self.advance_stage(
                        board,
                        killer_moves,
                        history,
                        cont_history,
                        conthist_keys,
                    );
                    continue;
                }

                return Some(mov);
            }

            if self.stage == GenStage::Done {
                return None;
            }

            if let Some(mov) =
                self.advance_stage(board, killer_moves, history, cont_history, conthist_keys)
            {
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
    fn pick_next(&mut self) -> ScoredMove {
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
        best_move
    }

    #[inline(never)]
    fn advance_stage(
        &mut self,
        board: &Board,
        killer_moves: KillerMoves,
        history: &HistoryTable,
        cont_history: &ContinuationHistoryTable,
        conthist_keys: &ContHistKeys,
    ) -> Option<ScoredMove> {
        if self.list_index == self.len() {
            self.end_ptr = self.start_ptr;
            self.list_index = 0;
        }

        match self.stage {
            GenStage::Init => {
                self.stage = if board.checkers != 0 {
                    GenStage::Evasions
                } else {
                    GenStage::Captures
                };

                if board.pseudo_legal(self.tt_move) {
                    let mut mov = ScoredMove::new(self.tt_move);
                    mov.score = i16::MAX;
                    return Some(mov);
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
                let remaining_captures = self.len();
                let ptr = if board.to_play == Color::White {
                    generate_moves::<White, Quiets>(board, self.end_ptr)
                } else {
                    generate_moves::<Black, Quiets>(board, self.end_ptr)
                };
                self.end_ptr = ptr;

                for scored_move in &mut self.as_slice_mut()[remaining_captures..] {
                    let mov = scored_move.mov;
                    let mut score = scoring::score_quiet(mov, killer_moves, history, board.to_play);
                    if mov != killer_moves[0] && mov != killer_moves[1] {
                        let piece = unsafe { board.piece_type_at(mov.from()) };
                        score += cont_history.score(conthist_keys, piece, mov.to());
                    }
                    scored_move.score = score;
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
                    let mov = scored_move.mov;
                    scored_move.score = if mov.is_tactical() {
                        scoring::score_capture(mov, board)
                    } else {
                        let mut score =
                            scoring::score_quiet(mov, killer_moves, history, board.to_play);
                        if mov != killer_moves[0] && mov != killer_moves[1] {
                            let piece = unsafe { board.piece_type_at(mov.from()) };
                            score += cont_history.score(conthist_keys, piece, mov.to());
                        }
                        score
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
