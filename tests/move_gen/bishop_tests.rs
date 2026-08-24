use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{
    generate_bishop_moves, Black, Captures, MoveGenType, MoveList, NonEvasions, Player, Quiets,
    White,
};

fn get_moves<Us: Player, Type: MoveGenType>(board: &Board) -> Vec<(Sq, Sq, MoveFlags)> {
    let them = !Us::COLOR;
    let target_mask = if Type::EVASIONS {
        !0
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

    let mut moves = MoveList::default();
    let new_pos = generate_bishop_moves::<Us, Type>(board, target_mask, moves.as_ptr());
    moves.update_size(new_pos);
    let slice = moves.as_slice();
    let mut res = Vec::new();
    for m in slice {
        res.push((m.from(), m.to(), m.flags()));
    }
    res
}

#[test]
fn test_bishop_moves_center() {
    let board = Board::from_fen("8/8/8/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // d4 bishop on empty board has 13 diagonal moves:
    // NE: e5, f6, g7, h8 (4)
    // NW: c5, b6, a7 (3)
    // SE: e3, f2, g1 (3)
    // SW: c3, b2, a1 (3)
    assert_eq!(moves.len(), 13);
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A1, MoveFlags::Quiet)));
}

#[test]
fn test_bishop_moves_corner() {
    let board = Board::from_fen("8/8/8/8/8/8/8/b7 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    // a1 bishop on empty board has 7 diagonal moves along a1-h8 ray
    assert_eq!(moves.len(), 7);
    assert!(moves.contains(&(Sq::A1, Sq::B2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::D4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::E5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::H8, MoveFlags::Quiet)));
}

#[test]
fn test_bishop_blocked_by_friendly_pieces() {
    // Bishop on d4 blocked by friendly pawns on c5, e5, c3, e3
    let board = Board::from_fen("8/8/8/2P1P3/3B4/2P1P3/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_bishop_captures_enemy_pieces() {
    // Bishop on d4 with enemy pawns on c5, e5, c3, e3
    let board = Board::from_fen("8/8/8/2p1p3/3B4/2p1p3/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::D4, Sq::C5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::C3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E3, MoveFlags::Capture)));
}

#[test]
fn test_bishop_captures_only() {
    // Bishop on d4: enemy pawn on f6, quiet moves on e5, c5, b6, a7, c3, b2, a1, e3, f2, g1
    let board = Board::from_fen("8/8/5p2/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Captures>(&board);
    assert_eq!(moves.len(), 1);
    assert!(moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Capture)));
}

#[test]
fn test_bishop_quiets_only() {
    // Bishop on d4: enemy pawn on f6 (blocks g7, h8). Quiet moves: e5 (1) + NW (3) + SW (3) + SE (3) = 10
    let board = Board::from_fen("8/8/5p2/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Quiets>(&board);
    assert_eq!(moves.len(), 10);
    assert!(!moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Capture)));
    assert!(!moves.contains(&(Sq::D4, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Quiet)));
}
