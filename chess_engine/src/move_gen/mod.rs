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

// Quiets are sorted lazily: most cut nodes use 1–2 quiets
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
enum Order {
    #[default]
    Sorted,
    Unsorted,
    MaxFirst,
}

pub struct MoveGenerator {
    start_ptr: MoveListPtr,
    end_ptr: MoveListPtr,
    stage: GenStage,
    list_index: usize,
    quiescence: bool,
    tt_move: Move,
    order: Order,
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
            order: Order::default(),
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
                    self.advance_stage(board, killer_moves, history, cont_history, conthist_keys);
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
    const fn as_slice(&self) -> &[ScoredMove] {
        unsafe { core::slice::from_raw_parts(self.start_ptr.0, self.len()) }
    }

    #[inline(always)]
    const fn as_slice_mut(&mut self) -> &mut [ScoredMove] {
        unsafe { core::slice::from_raw_parts_mut(self.start_ptr.0, self.len()) }
    }

    #[inline(always)]
    fn pick_next(&mut self) -> ScoredMove {
        let next = self.list_index;
        self.order = match self.order {
            Order::Sorted => Order::Sorted,
            Order::Unsorted => {
                select_first_max(&mut self.as_slice_mut()[next..]);
                Order::MaxFirst
            }
            Order::MaxFirst => {
                insertion_sort(&mut self.as_slice_mut()[next..]);
                Order::Sorted
            }
        };

        let mov = unsafe { self.as_slice().get_unchecked(next).clone() };
        self.list_index += 1;
        mov
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

                insertion_sort(self.as_slice_mut());

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
                    scored_move.score = scoring::score_quiet(
                        scored_move.mov,
                        board,
                        killer_moves,
                        history,
                        cont_history,
                        conthist_keys,
                    );
                }

                self.order = Order::Unsorted;

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
                        scoring::score_quiet(
                            mov,
                            board,
                            killer_moves,
                            history,
                            cont_history,
                            conthist_keys,
                        )
                    };
                }

                insertion_sort(self.as_slice_mut());

                self.stage = GenStage::Done;
            }
            GenStage::Done => {}
        }

        None
    }
}

/// Moves the first highest-scored move to the front, keeping the others in order,
/// so the picks match a stable sort.
fn select_first_max(moves: &mut [ScoredMove]) {
    let mut best = 0;
    for i in 1..moves.len() {
        if moves[i].score > moves[best].score {
            best = i;
        }
    }
    let mov = moves[best];
    moves.copy_within(0..best, 1);
    moves[0] = mov;
}

fn insertion_sort(moves: &mut [ScoredMove]) {
    for p in 1..moves.len() {
        let tmp = moves[p];
        let mut q = p;
        while q > 0 && moves[q - 1].score < tmp.score {
            moves[q] = moves[q - 1];
            q -= 1;
        }
        moves[q] = tmp;
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
