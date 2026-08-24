use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{
    Black, Captures, MoveGenType, MoveList, NonEvasions, Player, Quiets, White,
    generate_knight_moves,
};

fn get_moves<Us: Player, Type: MoveGenType>(board: &Board) -> Vec<(Sq, Sq, MoveFlags)> {
    let them = !Us::COLOR;
    let target_mask = if Type::EVASIONS {
        !0 // Simplification for these tests since we don't test evasions here
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
    let new_pos = generate_knight_moves::<Us, Type>(board, target_mask, moves.as_ptr());
    moves.update_size(new_pos);
    let slice = moves.as_slice();
    let mut res = Vec::new();
    for m in slice {
        res.push((m.from(), m.to(), m.flags()));
    }
    res
}

#[test]
fn test_knight_moves_center() {
    let board = Board::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 8);
    assert!(moves.contains(&(Sq::D4, Sq::C6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E2, MoveFlags::Quiet)));
}

#[test]
fn test_knight_moves_corner() {
    let board = Board::from_fen("n7/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::A8, Sq::B6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::C7, MoveFlags::Quiet)));
}

#[test]
fn test_knight_captures() {
    let board = Board::from_fen("8/8/2p1p3/1p3p2/3N4/1p3p2/2p1p3/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 8);
    assert!(moves.contains(&(Sq::D4, Sq::C6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::B5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::F5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::B3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::F3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::C2, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E2, MoveFlags::Capture)));
}

#[test]
fn test_knight_blocked_by_own_pieces() {
    let board = Board::from_fen("8/8/2P1P3/1P3P2/3N4/1P3P2/2P1P3/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_gen_captures_only() {
    let board = Board::from_fen("8/8/8/8/4n3/8/3P1P2/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, Captures>(&board);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::E4, Sq::D2, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::E4, Sq::F2, MoveFlags::Capture)));
}

#[test]
fn test_gen_quiets_only() {
    let board = Board::from_fen("8/8/8/8/4n3/8/3P1P2/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, Quiets>(&board);
    // e4 knight has 8 pseudo-legal moves. 2 are captures (d2, f2).
    // The other 6 are quiets.
    assert_eq!(moves.len(), 6);
    assert!(moves.contains(&(Sq::E4, Sq::C5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::D6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::G5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::G3, MoveFlags::Quiet)));
}
