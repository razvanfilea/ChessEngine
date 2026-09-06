use chess_core::prelude::*;

#[test]
fn test_castling_rights_mask_for_move() {
    // Moving from/to E1 clears WHITE_ANY
    let mask = CastlingRights::mask_for_move(Sq::E1, Sq::D2);
    assert!(!mask.contains(CastlingRights::WHITE_00));
    assert!(!mask.contains(CastlingRights::WHITE_000));
    assert!(mask.contains(CastlingRights::BLACK_ANY));

    // Moving from A1 clears WHITE_000
    let mask = CastlingRights::mask_for_move(Sq::A1, Sq::A2);
    assert!(mask.contains(CastlingRights::WHITE_00));
    assert!(!mask.contains(CastlingRights::WHITE_000));
    assert!(mask.contains(CastlingRights::BLACK_ANY));

    // Moving from H1 clears WHITE_00
    let mask = CastlingRights::mask_for_move(Sq::H1, Sq::H2);
    assert!(!mask.contains(CastlingRights::WHITE_00));
    assert!(mask.contains(CastlingRights::WHITE_000));
    assert!(mask.contains(CastlingRights::BLACK_ANY));

    // Moving from E8 clears BLACK_ANY
    let mask = CastlingRights::mask_for_move(Sq::E8, Sq::D8);
    assert!(mask.contains(CastlingRights::WHITE_ANY));
    assert!(!mask.contains(CastlingRights::BLACK_00));
    assert!(!mask.contains(CastlingRights::BLACK_000));

    // Moving from A8 clears BLACK_000
    let mask = CastlingRights::mask_for_move(Sq::A8, Sq::A7);
    assert!(mask.contains(CastlingRights::WHITE_ANY));
    assert!(mask.contains(CastlingRights::BLACK_00));
    assert!(!mask.contains(CastlingRights::BLACK_000));

    // Moving from H8 clears BLACK_00
    let mask = CastlingRights::mask_for_move(Sq::H8, Sq::H7);
    assert!(mask.contains(CastlingRights::WHITE_ANY));
    assert!(!mask.contains(CastlingRights::BLACK_00));
    assert!(mask.contains(CastlingRights::BLACK_000));

    // Irrelevant square clears nothing
    let mask = CastlingRights::mask_for_move(Sq::D4, Sq::D5);
    assert_eq!(mask, CastlingRights::ALL);

    // Target square also matters (capturing a rook on its home square)
    let mask = CastlingRights::mask_for_move(Sq::D4, Sq::A1);
    assert!(mask.contains(CastlingRights::WHITE_00));
    assert!(!mask.contains(CastlingRights::WHITE_000));
}

#[test]
fn test_dir_opposite() {
    assert_eq!(Dir::North.opposite(), Dir::South);
    assert_eq!(Dir::South.opposite(), Dir::North);
    assert_eq!(Dir::East.opposite(), Dir::West);
    assert_eq!(Dir::West.opposite(), Dir::East);
    assert_eq!(Dir::NorthEast.opposite(), Dir::SouthWest);
    assert_eq!(Dir::NorthWest.opposite(), Dir::SouthEast);
    assert_eq!(Dir::SouthEast.opposite(), Dir::NorthWest);
    assert_eq!(Dir::SouthWest.opposite(), Dir::NorthEast);
}

#[test]
fn test_dir_is_forwards() {
    assert_eq!(Dir::North.is_forwards(), true);
    assert_eq!(Dir::East.is_forwards(), true);
    assert_eq!(Dir::NorthEast.is_forwards(), true);
    assert_eq!(Dir::NorthWest.is_forwards(), true);

    assert_eq!(Dir::South.is_forwards(), false);
    assert_eq!(Dir::West.is_forwards(), false);
    assert_eq!(Dir::SouthEast.is_forwards(), false);
    assert_eq!(Dir::SouthWest.is_forwards(), false);
}

#[test]
fn test_capture_square() {
    // Standard captures target the `to` square
    let normal_cap = Move::new(Sq::E4, Sq::D5, MoveFlags::Capture);
    assert_eq!(normal_cap.capture_square(Color::White), Sq::D5);
    assert_eq!(normal_cap.capture_square(Color::Black), Sq::D5);

    // En passant capture: White moves to D6, captured pawn is on D5 (South of D6)
    let ep_white = Move::new(Sq::E5, Sq::D6, MoveFlags::EnPassant);
    assert_eq!(ep_white.capture_square(Color::White), Sq::D5);

    // En passant capture: Black moves to D3, captured pawn is on D4 (North of D3)
    let ep_black = Move::new(Sq::E4, Sq::D3, MoveFlags::EnPassant);
    assert_eq!(ep_black.capture_square(Color::Black), Sq::D4);
}
