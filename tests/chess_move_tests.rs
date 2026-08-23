use chess_base::prelude::*;

#[test]
fn test_move_new_and_getters() {
    let m = Move::new(Sq::A1, Sq::H8, MoveFlags::Quiet);
    assert_eq!(m.from(), Sq::A1);
    assert_eq!(m.to(), Sq::H8);
    assert_eq!(m.flags(), MoveFlags::Quiet);
    assert!(!m.is_capture());
    assert!(!m.is_promotion());
    assert_eq!(m.promotion_piece(), None);
}

#[test]
fn test_move_new_quiet() {
    let m = Move::new_quiet(Sq::E2, Sq::E4);
    assert_eq!(m.from(), Sq::E2);
    assert_eq!(m.to(), Sq::E4);
    assert_eq!(m.flags(), MoveFlags::Quiet);
}

#[test]
fn test_move_flags() {
    // Capture
    let capture_move = Move::new(Sq::A1, Sq::A2, MoveFlags::Capture);
    assert!(capture_move.is_capture());
    assert!(!capture_move.is_promotion());

    // Promotion (non-capture)
    let promo_queen = Move::new(Sq::A7, Sq::A8, MoveFlags::PromoQueen);
    assert!(!promo_queen.is_capture()); // PromoQueen is 0b1011 (bit 2 is 0)
    assert!(promo_queen.is_promotion());
    assert_eq!(promo_queen.promotion_piece(), Some(Pieces::Queen));

    let promo_knight = Move::new(Sq::B7, Sq::B8, MoveFlags::PromoKnight);
    assert_eq!(promo_knight.promotion_piece(), Some(Pieces::Knight));

    // Promotion with capture
    let promo_capture_rook = Move::new(Sq::C7, Sq::D8, MoveFlags::PromoCaptureRook);
    assert!(promo_capture_rook.is_capture()); // PromoCaptureRook is 0b1110 (bit 2 is 1)
    assert!(promo_capture_rook.is_promotion());
    assert_eq!(promo_capture_rook.promotion_piece(), Some(Pieces::Rook));
}

#[test]
fn test_move_none() {
    assert_eq!(Move::NONE, Move::default());
    assert!(Move::NONE.is_none());
    assert!(!Move::new_quiet(Sq::A2, Sq::A3).is_none());
    assert_eq!(Move::NONE.from(), Sq::A1);
    assert_eq!(Move::NONE.to(), Sq::A1);
    assert_eq!(Move::NONE.flags(), MoveFlags::Quiet);
}
