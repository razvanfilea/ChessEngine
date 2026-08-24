use std::mem::MaybeUninit;

use chess_base::{bitboard::*, for_each_bit, prelude::*};

use crate::board::Board;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GenType {
    All,
    Captures,
    Quiets,
}

pub trait Player {
    const COLOR: Color;
}
pub struct White;
impl Player for White {
    const COLOR: Color = Color::White;
}
pub struct Black;
impl Player for Black {
    const COLOR: Color = Color::Black;
}

pub trait MoveGenType {
    const TYPE: GenType;
}
pub struct AllMoves;
impl MoveGenType for AllMoves {
    const TYPE: GenType = GenType::All;
}
pub struct GenCaptures;
impl MoveGenType for GenCaptures {
    const TYPE: GenType = GenType::Captures;
}
pub struct GenQuiets;
impl MoveGenType for GenQuiets {
    const TYPE: GenType = GenType::Quiets;
}

pub struct MoveList {
    moves: [MaybeUninit<Move>; 256],
    size: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            size: 0,
        }
    }
}

impl MoveList {
    pub const fn as_ptr(&mut self) -> MoveListPtr {
        MoveListPtr(self.current_ptr())
    }

    pub const fn update_size(&mut self, new_position: MoveListPtr) {
        let size = unsafe { new_position.0.offset_from(self.current_ptr()) };
        self.size = size as usize;
    }

    pub const fn as_slice(&self) -> &[Move] {
        unsafe { core::slice::from_raw_parts(self.moves.as_ptr() as *const Move, self.size) }
    }

    const fn current_ptr(&mut self) -> *mut Move {
        (unsafe { self.moves.as_mut_ptr().add(self.size) }) as *mut Move
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MoveListPtr(pub *mut Move);

impl MoveListPtr {
    #[inline(always)]
    const fn push(&mut self, from: Sq, to: Sq, flags: MoveFlags) {
        unsafe {
            self.0.write(Move::new(from, to, flags));
            self.0 = self.0.add(1);
        }
    }

    #[inline(always)]
    const fn push_promotions(&mut self, from: Sq, to: Sq, is_capture: bool) {
        let moves = if is_capture {
            [
                Move::new(from, to, MoveFlags::PromoCaptureQueen),
                Move::new(from, to, MoveFlags::PromoCaptureRook),
                Move::new(from, to, MoveFlags::PromoCaptureBishop),
                Move::new(from, to, MoveFlags::PromoCaptureKnight),
            ]
        } else {
            [
                Move::new(from, to, MoveFlags::PromoQueen),
                Move::new(from, to, MoveFlags::PromoRook),
                Move::new(from, to, MoveFlags::PromoBishop),
                Move::new(from, to, MoveFlags::PromoKnight),
            ]
        };

        unsafe {
            let ptr = self.0 as *mut [Move; 4];
            ptr.write(moves);
            self.0 = self.0.add(4);
        }
    }
}

pub fn generate_moves<Us: Player, Type: MoveGenType>(board: &Board) {
    let mut moves = MoveList::default();
    generate_pawn_moves::<Us, Type>(board, moves.as_ptr());
}

pub fn generate_pawn_moves<Us: Player, Type: MoveGenType>(board: &Board, mut moves: MoveListPtr) -> MoveListPtr {
    let move_type = Type::TYPE;
    let us = Us::COLOR;
    let them = !us;
    let forward = if us == Color::White {
        Dir::North
    } else {
        Dir::South
    };
    let backward = forward.opposite();
    let last_rank = if us == Color::White { RANK_8 } else { RANK_1 };
    let third_rank = if us == Color::White { RANK_3 } else { RANK_6 };
    let backward_left = if us == Color::White {
        Dir::SouthEast
    } else {
        Dir::NorthWest
    };
    let backward_right = if us == Color::White {
        Dir::SouthWest
    } else {
        Dir::NorthEast
    };

    let empty_squares = !board.occupied();
    let enemies = board.colors(them);
    let pawns = board.color_piece(Pieces::Pawn, us);

    if move_type != GenType::Quiets {
        let (left, right) = if Us::COLOR == Color::White {
            (sh_north_west(pawns), sh_north_east(pawns))
        } else {
            (sh_south_east(pawns), sh_south_west(pawns))
        };

        let en_passant_sq = board.en_passant_target_sq;
        if en_passant_sq != Sq::NONE {
            let en_passant_bb = board.en_passant_target_sq.bitboard();

            if left & en_passant_bb != 0 {
                let from = unsafe { en_passant_sq.shift_unchecked(backward_left) };
                moves.push(from, en_passant_sq, MoveFlags::EnPassant);
            }

            if right & en_passant_bb != 0 {
                let from = unsafe { en_passant_sq.shift_unchecked(backward_right) };
                moves.push(from, en_passant_sq, MoveFlags::EnPassant);
            }
        }

        let left = left & enemies;
        let right = right & enemies;

        // Non-promotions
        for_each_bit!(to in left & !last_rank => {
            let from = unsafe { to.shift_unchecked(backward_left) };
            moves.push(from, to, MoveFlags::Capture);
        });

        for_each_bit!(to in right & !last_rank => {
            let from = unsafe { to.shift_unchecked(backward_right) };
            moves.push(from, to, MoveFlags::Capture);
        });

        // Promotions
        for_each_bit!(to in left & last_rank => {
            let from = unsafe { to.shift_unchecked(backward_left) };
            moves.push_promotions(from, to, true);
        });

        for_each_bit!(to in right & last_rank => {
            let from = unsafe { to.shift_unchecked(backward_right) };
            moves.push_promotions(from, to, true);
        });
    }

    if move_type != GenType::Captures {
        let forward_pushed_pawn = sh_dir(forward, pawns) & empty_squares;
        let promotion_pushes = forward_pushed_pawn & last_rank;
        let pushes = forward_pushed_pawn & !last_rank;
        let double_pushes = sh_dir(forward, forward_pushed_pawn & third_rank) & empty_squares;

        for_each_bit!(to in promotion_pushes => {
            let from = unsafe { to.shift_unchecked(backward) };
            moves.push_promotions(from, to, false);
        });

        for_each_bit!(to in pushes => {
            let from = unsafe { to.shift_unchecked(backward) };
            moves.push(from, to, MoveFlags::Quiet);
        });

        for_each_bit!(to in double_pushes => {
            let from = unsafe { to.shift_unchecked(backward).shift_unchecked(backward) };
            moves.push(from, to, MoveFlags::DoublePawn);
        });
    }

    moves
}

