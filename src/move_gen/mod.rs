use chess_base::{bitboard::*, for_each_bit, prelude::*};

use crate::{
    attacks::{self, bishop_attacks, knight_attacks, pawn_attacks, rook_attacks},
    board::Board,
};

mod move_list;
mod traits;

pub use move_list::*;
pub use traits::*;

pub fn compute_checkers<Us: Player>(board: &Board) -> u64 {
    let us = Us::COLOR;
    let them = !us;

    let king_bb = board.color_piece(Pieces::King, us);
    let king_sq = Sq::from_raw(bb_scan_forward(king_bb));
    let enemy = *board.colors(them);
    let occupied = board.occupied();

    let enemy_bischop = (board.pieces(Pieces::Bischop) | board.pieces(Pieces::Queen)) & enemy;
    let enemy_rook = (board.pieces(Pieces::Rook) | board.pieces(Pieces::Queen)) & enemy;

    (pawn_attacks::<Us>(king_sq) & board.color_piece(Pieces::Pawn, them))
        | (knight_attacks(king_sq) & board.color_piece(Pieces::Knight, them))
        | (bishop_attacks(king_sq, occupied) & enemy_bischop)
        | (rook_attacks(king_sq, occupied) & enemy_rook)
}

pub fn generate_moves<Us: Player, Type: MoveGenType>(board: &Board) -> MoveList {
    let us = Us::COLOR;
    let them = !us;

    let mut moves = MoveList::default();

    let king_bb = board.color_piece(Pieces::King, us);
    let king_sq = Sq::from_raw(bb_scan_forward(king_bb));
    let checkers = compute_checkers::<Us>(board);

    let mut moves_ptr = generate_king_moves::<Us, Type>(board, moves.as_ptr());

    // Double check: Only king can move
    if Type::EVASIONS && bb_several(checkers) {
        moves.update_size(moves_ptr);
        return moves;
    }

    let target_mask = if Type::EVASIONS {
        let checker_sq = unsafe { Sq::from_raw_unchecked(bb_scan_forward(checkers)) };
        checkers | bb_between(king_sq, checker_sq)
    } else {
        let mut mask = 0;
        if Type::CAPTURES {
            mask |= board.colors(them);
        }
        if Type::QUIETS {
            mask |= !board.occupied();
        }
        mask
    };

    moves_ptr = generate_knight_moves::<Us, Type>(board, target_mask, moves_ptr);
    moves_ptr = generate_pawn_moves::<Us, Type>(board, target_mask, moves_ptr);

    moves.update_size(moves_ptr);
    moves
}

pub fn generate_knight_moves<Us: Player, Type: MoveGenType>(
    board: &Board,
    target_mask: u64,
    mut moves: MoveListPtr,
) -> MoveListPtr {
    let us = Us::COLOR;
    let them = !us;

    let knights = board.color_piece(Pieces::Knight, us);
    let enemy = board.colors(them);

    for_each_bit!(knight_sq in knights => {
        let attacks = knight_attacks(knight_sq) & target_mask;

        for_each_bit!(to in attacks => {
            let bb = to.bitboard();
            moves.push(knight_sq, to, if bb & enemy != 0 {MoveFlags::Capture} else {MoveFlags::Quiet });
        });
    });

    moves
}

pub fn generate_pawn_moves<Us: Player, Type: MoveGenType>(
    board: &Board,
    target_mask: u64,
    mut moves: MoveListPtr,
) -> MoveListPtr {
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
    let enemies = *board.colors(them) & target_mask;
    let pawns = board.color_piece(Pieces::Pawn, us);

    if Type::CAPTURES {
        let (left, right) = if Us::COLOR == Color::White {
            (sh_north_west(pawns), sh_north_east(pawns))
        } else {
            (sh_south_east(pawns), sh_south_west(pawns))
        };

        let en_passant_sq = board.en_passant_target_sq;
        if en_passant_sq != Sq::NONE {
            let captured_pawn_sq = unsafe { en_passant_sq.shift_unchecked(backward) };
            let ep_mask = en_passant_sq.bitboard() | captured_pawn_sq.bitboard();
            let en_passant_bb = if ep_mask & target_mask != 0 {
                en_passant_sq.bitboard()
            } else {
                0
            };

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

        if Type::EVASIONS {
            let left_non_promo = left & !last_rank;
            if left_non_promo != 0 {
                let to = unsafe { Sq::from_raw_unchecked(left_non_promo.trailing_zeros() as u8) };
                let from = unsafe { to.shift_unchecked(backward_left) };
                moves.push(from, to, MoveFlags::Capture);
            }

            let right_non_promo = right & !last_rank;
            if right_non_promo != 0 {
                let to = unsafe { Sq::from_raw_unchecked(right_non_promo.trailing_zeros() as u8) };
                let from = unsafe { to.shift_unchecked(backward_right) };
                moves.push(from, to, MoveFlags::Capture);
            }

            let left_promo = left & last_rank;
            if left_promo != 0 {
                let to = unsafe { Sq::from_raw_unchecked(left_promo.trailing_zeros() as u8) };
                let from = unsafe { to.shift_unchecked(backward_left) };
                moves.push_promotions(from, to, true);
            }

            let right_promo = right & last_rank;
            if right_promo != 0 {
                let to = unsafe { Sq::from_raw_unchecked(right_promo.trailing_zeros() as u8) };
                let from = unsafe { to.shift_unchecked(backward_right) };
                moves.push_promotions(from, to, true);
            }
        } else {
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
    }

    if Type::QUIETS {
        let forward_pushed_pawn = sh_dir(forward, pawns) & empty_squares;
        let promotion_pushes = forward_pushed_pawn & last_rank & target_mask;
        let pushes = forward_pushed_pawn & !last_rank & target_mask;
        let double_pushes =
            sh_dir(forward, forward_pushed_pawn & third_rank) & empty_squares & target_mask;

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

pub fn generate_king_moves<Us: Player, Type: MoveGenType>(
    board: &Board,
    mut moves: MoveListPtr,
) -> MoveListPtr {
    moves
}

#[unsafe(no_mangle)]
pub fn generate_pawn_evasions(
    board: &Board,
    target_mask: u64,
    mut moves: MoveListPtr,
) -> MoveListPtr {
     generate_pawn_moves::<White, Evasions>(board, target_mask, moves)
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn generate_pawn_non(
    board: &Board,
    target_mask: u64,
    mut moves: MoveListPtr,
) -> MoveListPtr {
    generate_pawn_moves::<White, NonEvasions>(board, target_mask, moves)
}
