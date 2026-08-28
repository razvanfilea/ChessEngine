use chess_core::prelude::*;

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
    let m = Move::new(Sq::E2, Sq::E4, MoveFlags::Quiet);
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
    assert_eq!(promo_queen.promotion_piece(), Some(Piece::Queen));

    let promo_knight = Move::new(Sq::B7, Sq::B8, MoveFlags::PromoKnight);
    assert_eq!(promo_knight.promotion_piece(), Some(Piece::Knight));

    // Promotion with capture
    let promo_capture_rook = Move::new(Sq::C7, Sq::D8, MoveFlags::PromoCaptureRook);
    assert!(promo_capture_rook.is_capture()); // PromoCaptureRook is 0b1110 (bit 2 is 1)
    assert!(promo_capture_rook.is_promotion());
    assert_eq!(promo_capture_rook.promotion_piece(), Some(Piece::Rook));
}

#[test]
fn test_move_none() {
    assert_eq!(Move::NONE, Move::default());
    assert!(Move::NONE.is_none());
    assert!(!Move::new(Sq::A2, Sq::A3, MoveFlags::Quiet).is_none());
    assert_eq!(Move::NONE.from(), Sq::A1);
    assert_eq!(Move::NONE.to(), Sq::A1);
    assert_eq!(Move::NONE.flags(), MoveFlags::Quiet);
}

#[test]
fn test_move_is_castle() {
    assert!(Move::new(Sq::E1, Sq::G1, MoveFlags::CastleKing).is_castle());
    assert!(Move::new(Sq::E1, Sq::C1, MoveFlags::CastleQueen).is_castle());

    assert!(!Move::new(Sq::E2, Sq::E4, MoveFlags::Quiet).is_castle());
    assert!(!Move::new(Sq::E4, Sq::D5, MoveFlags::Capture).is_castle());
    assert!(!Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn).is_castle());
    assert!(!Move::new(Sq::E5, Sq::D6, MoveFlags::EnPassant).is_castle());
    // Promotions must not be confused with castling (PromoRook = 0b1010 has bit-1 set)
    assert!(!Move::new(Sq::A7, Sq::A8, MoveFlags::PromoRook).is_castle());
    assert!(!Move::new(Sq::A7, Sq::A8, MoveFlags::PromoQueen).is_castle());
    assert!(!Move::new(Sq::A7, Sq::A8, MoveFlags::PromoKnight).is_castle());
    assert!(!Move::new(Sq::A7, Sq::A8, MoveFlags::PromoBishop).is_castle());
    assert!(!Move::new(Sq::A7, Sq::B8, MoveFlags::PromoCaptureRook).is_castle());
    assert!(!Move::new(Sq::A7, Sq::B8, MoveFlags::PromoCaptureQueen).is_castle());
}

#[test]
fn test_move_en_passant_and_double_pawn_flags() {
    let ep = Move::new(Sq::E5, Sq::D6, MoveFlags::EnPassant);
    assert_eq!(ep.flags(), MoveFlags::EnPassant);
    assert!(ep.is_capture());
    assert!(!ep.is_promotion());
    assert!(!ep.is_castle());

    let dp = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
    assert_eq!(dp.flags(), MoveFlags::DoublePawn);
    assert!(!dp.is_capture());
    assert!(!dp.is_promotion());
    assert!(!dp.is_castle());
}
