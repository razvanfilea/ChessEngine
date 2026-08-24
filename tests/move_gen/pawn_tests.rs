use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{
    Black, Captures, MoveGenType, MoveList, NonEvasions, Player, Quiets, White, generate_pawn_moves,
};

fn get_moves<Us: Player, Type: MoveGenType>(board: &Board) -> Vec<(Sq, Sq, MoveFlags)> {
    let mut moves = MoveList::default();
    let new_pos = generate_pawn_moves::<Us, Type>(board, !0, moves.as_ptr());
    moves.update_size(new_pos);
    let slice = moves.as_slice();
    let mut res = Vec::new();
    for m in slice {
        res.push((m.from(), m.to(), m.flags()));
    }
    res
}

#[test]
fn test_white_pawn_pushes() {
    let board = Board::from_fen("8/8/8/8/8/8/P7/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::A2, Sq::A3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A2, Sq::A4, MoveFlags::DoublePawn)));
}

#[test]
fn test_black_pawn_pushes() {
    let board = Board::from_fen("8/p7/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::A7, Sq::A6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A7, Sq::A5, MoveFlags::DoublePawn)));
}

#[test]
fn test_white_pawn_captures() {
    let board = Board::from_fen("8/8/8/8/8/1p1p4/2P5/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::C2, Sq::B3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::C2, Sq::D3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::C2, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::C2, Sq::C4, MoveFlags::DoublePawn)));
}

#[test]
fn test_black_pawn_captures() {
    let board = Board::from_fen("8/2p5/1P1P4/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::C7, Sq::B6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::C7, Sq::D6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::C7, Sq::C6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::C7, Sq::C5, MoveFlags::DoublePawn)));
}

#[test]
fn test_white_en_passant() {
    let board = Board::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::E5, Sq::E6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E5, Sq::D6, MoveFlags::EnPassant)));
}

#[test]
fn test_black_en_passant() {
    let board = Board::from_fen("8/8/8/8/3Pp3/8/8/8 b - d3 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::E4, Sq::E3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::D3, MoveFlags::EnPassant)));
}

#[test]
fn test_white_promotions() {
    let board = Board::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::E7, Sq::E8, MoveFlags::PromoQueen)));
    assert!(moves.contains(&(Sq::E7, Sq::E8, MoveFlags::PromoRook)));
    assert!(moves.contains(&(Sq::E7, Sq::E8, MoveFlags::PromoBishop)));
    assert!(moves.contains(&(Sq::E7, Sq::E8, MoveFlags::PromoKnight)));
}

#[test]
fn test_white_promotion_captures() {
    let board = Board::from_fen("3q4/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 8);
    assert!(moves.contains(&(Sq::E7, Sq::D8, MoveFlags::PromoCaptureQueen)));
    assert!(moves.contains(&(Sq::E7, Sq::D8, MoveFlags::PromoCaptureRook)));
    assert!(moves.contains(&(Sq::E7, Sq::D8, MoveFlags::PromoCaptureBishop)));
    assert!(moves.contains(&(Sq::E7, Sq::D8, MoveFlags::PromoCaptureKnight)));
}

#[test]
fn test_black_promotions() {
    let board = Board::from_fen("8/8/8/8/8/8/4p3/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::E2, Sq::E1, MoveFlags::PromoQueen)));
    assert!(moves.contains(&(Sq::E2, Sq::E1, MoveFlags::PromoRook)));
    assert!(moves.contains(&(Sq::E2, Sq::E1, MoveFlags::PromoBishop)));
    assert!(moves.contains(&(Sq::E2, Sq::E1, MoveFlags::PromoKnight)));
}

#[test]
fn test_black_promotion_captures() {
    let board = Board::from_fen("8/8/8/8/8/8/4p3/3Q4 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 8); // 4 pushes + 4 promo captures
    assert!(moves.contains(&(Sq::E2, Sq::D1, MoveFlags::PromoCaptureQueen)));
    assert!(moves.contains(&(Sq::E2, Sq::D1, MoveFlags::PromoCaptureRook)));
    assert!(moves.contains(&(Sq::E2, Sq::D1, MoveFlags::PromoCaptureBishop)));
    assert!(moves.contains(&(Sq::E2, Sq::D1, MoveFlags::PromoCaptureKnight)));
}

#[test]
fn test_gen_captures_only() {
    let board = Board::from_fen("8/8/8/8/8/1p1p4/2P5/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Captures>(&board);
    assert_eq!(moves.len(), 2); // Should NOT include pushes, only captures
    assert!(moves.contains(&(Sq::C2, Sq::B3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::C2, Sq::D3, MoveFlags::Capture)));
}

#[test]
fn test_gen_quiets_only() {
    let board = Board::from_fen("8/8/8/8/8/1p1p4/2P5/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Quiets>(&board);
    assert_eq!(moves.len(), 2); // Should NOT include captures, only pushes
    assert!(moves.contains(&(Sq::C2, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::C2, Sq::C4, MoveFlags::DoublePawn)));
}

#[test]
fn test_pawn_capture_no_wrap_around() {
    let board = Board::from_fen("8/8/8/8/8/1p4p1/P6P/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // a2-a3, a2-a4, h2-h3, h2-h4 (4 pushes)
    // a2xb3, h2xg3 (2 captures)
    // total 6 moves
    assert_eq!(moves.len(), 6);
    assert!(moves.contains(&(Sq::A2, Sq::B3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::H2, Sq::G3, MoveFlags::Capture)));
    for m in moves {
        if m.0 == Sq::A2 && m.2 == MoveFlags::Capture {
            assert_eq!(m.1, Sq::B3);
        }
        if m.0 == Sq::H2 && m.2 == MoveFlags::Capture {
            assert_eq!(m.1, Sq::G3);
        }
    }
}

#[test]
fn test_pawn_push_blocked_by_enemy() {
    // Wait, pawn on a2, blocked by enemy pawn on a3
    let board = Board::from_fen("8/8/8/8/8/p7/P7/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 0); // Blocked
}

#[test]
fn test_pawn_double_push_blocked() {
    let board = Board::from_fen("8/8/8/8/p7/8/P7/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 1); // Only a2-a3 is possible, a2-a4 is blocked
    assert!(moves.contains(&(Sq::A2, Sq::A3, MoveFlags::Quiet)));
}

#[test]
fn test_pawn_double_push_blocked_on_first_square() {
    let board = Board::from_fen("8/8/8/8/8/p7/P7/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 0); // Both a3 and a4 are blocked because a3 is occupied
}
