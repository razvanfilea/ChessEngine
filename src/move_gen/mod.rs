use std::mem::MaybeUninit;

use chess_base::{bitboard::*, prelude::*};

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
    #[inline(always)]
    pub fn push(&mut self, from: Sq, to: Sq, flags: MoveFlags) {
        unsafe {
            self.moves
                .get_unchecked_mut(self.size)
                .write(Move::new(from, to, flags));
        }
        self.size += 1;
    }

    pub fn as_slice(&self) -> &[Move] {
        unsafe { core::slice::from_raw_parts(self.moves.as_ptr() as *const Move, self.size) }
    }
}

pub fn generate_moves<Us: Player, Type: MoveGenType>(board: &Board) {
    let us = Us::COLOR;
    let them = !us;

    let mut moves = MoveList::default();
    generate_pawn_moves::<Us, Type>(board, &mut moves);
}

pub fn generate_pawn_moves<Us: Player, Type: MoveGenType>(board: &Board, moves: &mut MoveList) {
    let move_type = Type::TYPE;
    let us = Us::COLOR;
    let them = !us;
    let forward = if us == Color::White {
        Dir::North
    } else {
        Dir::South
    };
    let backward = forward.opposite();
    let last_rank = if us == Color::White { RANK_7 } else { RANK_2 };
    let third_rank = if us == Color::White { RANK_3 } else { RANK_6 };
    let backward_left = if us == Color::White {
        Dir::SouthEast
    } else {
        Dir::NorthEast
    };
    let backward_right = if us == Color::White {
        Dir::SouthWest
    } else {
        Dir::NorthWest
    };

    let empty_squares = !board.occupied();
    let enemies = board.colors(them);
    let pawns = board.color_piece(Pieces::Pawn, us);
    let promo_pawns = pawns & last_rank;
    let non_promp_pawns = pawns & !promo_pawns;

    if move_type != GenType::Quiets && board.en_passant_target_sq != Sq::NONE {
        let en_passant_bb = board.en_passant_target_sq.bitboard();
    }

    let forward_pushed_pawn = sh_dir(forward, pawns);

    if move_type != GenType::Quiets {
        let (mut left, mut right) = if Us::COLOR == Color::White {
            (
                ((pawns & !FILE_A) << 7) & enemies,
                ((pawns & !FILE_H) << 9) & enemies,
            )
        } else {
            (
                ((pawns & !FILE_H) >> 7) & enemies,
                ((pawns & !FILE_A) >> 9) & enemies,
            )
        };

        while left != 0 {
            let to = bb_pop_lsb(&mut left);
            let from = unsafe { to.shift_unchecked(backward_left) };
            moves.push(from, to, MoveFlags::Capture);
        }

        while right != 0 {
            let to = bb_pop_lsb(&mut right);
            let from = unsafe { to.shift_unchecked(backward_right) };
            moves.push(from, to, MoveFlags::Capture);
        }
    }

    if move_type != GenType::Captures {
        let mut pushes = forward_pushed_pawn & empty_squares;
        let mut double_pushes = sh_dir(forward, pushes & third_rank) & empty_squares;
        while pushes != 0 {
            let to = bb_pop_lsb(&mut pushes);
            let from = unsafe { to.shift_unchecked(backward) };
            moves.push(from, to, MoveFlags::Quiet);
        }

        while double_pushes != 0 {
            let to = bb_pop_lsb(&mut double_pushes);
            let from = unsafe { to.shift_unchecked(backward).shift_unchecked(backward) };
            moves.push(from, to, MoveFlags::DoublePawn);
        }
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn pawn_moves_white_all(board: &Board, moves: &mut MoveList) {
    generate_pawn_moves::<White, AllMoves>(board, moves);
}
