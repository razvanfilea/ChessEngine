use chess_core::prelude::*;
use chess_engine::board::Board;

#[test]
fn test_insufficient_material_kvk() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    assert!(board.is_draw());
}

#[test]
fn test_insufficient_material_knvk() {
    let board = Board::from_fen("4k3/8/8/8/8/8/4N3/4K3 w - - 0 1").unwrap();
    assert!(board.is_draw());
}

#[test]
fn test_insufficient_material_kbvk() {
    let board = Board::from_fen("4k3/8/8/8/8/8/4B3/4K3 w - - 0 1").unwrap();
    assert!(board.is_draw());
}

#[test]
fn test_insufficient_material_kbvkb_same_color() {
    // b1 and g8 are both light squares
    let board = Board::from_fen("6bk/8/8/8/8/8/8/KB6 w - - 0 1").unwrap();
    assert!(board.is_draw());
}

#[test]
fn test_not_draw_kbvkb_opposite_color() {
    // b1 is light, f8 is dark
    let board = Board::from_fen("5b1k/8/8/8/8/8/8/KB6 w - - 0 1").unwrap();
    assert!(!board.is_draw());
}

#[test]
fn test_not_draw_knnvk() {
    let board = Board::from_fen("7k/8/8/8/8/8/8/KNN5 w - - 0 1").unwrap();
    assert!(!board.is_draw());
}

#[test]
fn test_not_draw_krvk() {
    let board = Board::from_fen("7k/8/8/8/8/8/8/KR6 w - - 0 1").unwrap();
    assert!(!board.is_draw());
}

#[test]
fn test_not_draw_kpvk() {
    let board = Board::from_fen("4k3/8/8/8/8/8/P7/K7 w - - 0 1").unwrap();
    assert!(!board.is_draw());
}

#[test]
fn test_fifty_move_rule_draw() {
    let board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 100 1").unwrap();
    assert!(board.is_draw());
}

#[test]
fn test_fifty_move_rule_not_draw() {
    let board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 99 1").unwrap();
    assert!(!board.is_draw());
}

#[test]
fn test_threefold_repetition() {
    let mut board = Board::start_pos();
    assert!(!board.is_draw());

    // g1f3, g8f6, f3g1, f6g8 — returns to start position
    board.make_move(Move::new(Sq::G1, Sq::F3, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::G8, Sq::F6, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::F3, Sq::G1, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::F6, Sq::G8, MoveFlags::Quiet));

    assert!(board.is_draw());
}

#[test]
fn test_start_pos_not_draw() {
    let board = Board::start_pos();
    assert!(!board.is_draw());
}

#[test]
fn test_repetition_with_game_history() {
    let mut board = Board::start_pos();
    // 1. Nf3 Nf6
    board.make_move(Move::new(Sq::G1, Sq::F3, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::G8, Sq::F6, MoveFlags::Quiet));
    // 2. Ng1 Ng8 (startpos repeated)
    board.make_move(Move::new(Sq::F3, Sq::G1, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::F6, Sq::G8, MoveFlags::Quiet));
    assert!(board.is_draw());

    // 3. Nc3 d6 (pawn move resets half_move_clock)
    board.make_move(Move::new(Sq::B1, Sq::C3, MoveFlags::Quiet));
    board.make_move(Move::new(Sq::D7, Sq::D6, MoveFlags::Quiet));
    assert!(!board.is_draw());
}
