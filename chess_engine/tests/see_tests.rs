use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::scoring::see_ge;

#[test]
fn test_see_hanging_piece() {
    // White Knight captures undefended Black Queen on d5
    let board = Board::from_fen("4k3/8/8/3q4/8/2N5/8/4K3 w - - 0 1").unwrap();
    let mov = Move::new(Sq::C3, Sq::D5, MoveFlags::Capture);

    assert!(see_ge(mov, &board, 0));
    assert!(see_ge(mov, &board, 900));
    assert!(!see_ge(mov, &board, 901));
}

#[test]
fn test_see_equal_exchange() {
    // White Bishop captures Black Knight on f6 defended by g7 Pawn
    let board =
        Board::from_fen("rnbqkb1r/pppppppp/5n2/4B3/8/8/PPPPPPPP/RN1QKBNR w KQkq - 0 1").unwrap();
    let mov = Move::new(Sq::E5, Sq::F6, MoveFlags::Capture);

    // Bishop for Knight is 300 - 300 = 0
    assert!(see_ge(mov, &board, -100));
    assert!(see_ge(mov, &board, 0));
    assert!(!see_ge(mov, &board, 1));
    assert!(!see_ge(mov, &board, 100));
}

#[test]
fn test_see_losing_sacrifice() {
    // White Queen captures defended Pawn on f7
    let board = Board::from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5Q2/PPPP1PPP/RNB1KBNR w KQkq - 0 1")
        .unwrap();
    let mov = Move::new(Sq::F3, Sq::F7, MoveFlags::Capture);

    // Qxf7+ loses Queen for Pawn: 100 - 900 = -800
    assert!(!see_ge(mov, &board, 0));
    assert!(!see_ge(mov, &board, -799));
    assert!(see_ge(mov, &board, -800));
    assert!(see_ge(mov, &board, -900));
}

#[test]
fn test_see_rook_takes_undefended_pawn() {
    // 1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1
    // Pawn on e5 is undefended because Black Rook is on d8
    let board = Board::from_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1").unwrap();
    let mov = Move::new(Sq::E1, Sq::E5, MoveFlags::Capture);

    assert!(see_ge(mov, &board, 0));
    assert!(see_ge(mov, &board, 100));
    assert!(!see_ge(mov, &board, 101));
}

#[test]
fn test_see_rook_takes_defended_pawn() {
    // 1k2r3/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1
    // Pawn on e5 is defended by Black Rook on e8. White loses 500 - 100 = 400
    let board = Board::from_fen("1k2r3/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1").unwrap();
    let mov = Move::new(Sq::E1, Sq::E5, MoveFlags::Capture);

    assert!(!see_ge(mov, &board, 0));
    assert!(!see_ge(mov, &board, -399));
    assert!(see_ge(mov, &board, -400));
}

#[test]
fn test_see_complex_xray_battery() {
    // Classical CPW position: 1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1
    // White plays Nxe5
    let board =
        Board::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1").unwrap();
    let mov = Move::new(Sq::D3, Sq::E5, MoveFlags::Capture);

    // Nxe5 loses material overall due to battery defenses
    assert!(!see_ge(mov, &board, 0));
    assert!(!see_ge(mov, &board, -100));
}

#[test]
fn test_see_en_passant() {
    // White pawn on d5, Black pawn just played e7-e5 -> e6 is ep square
    let board = Board::from_fen("4k3/8/8/3Pp3/8/8/8/4K3 w - e6 0 1").unwrap();
    let mov = Move::new(Sq::D5, Sq::E6, MoveFlags::EnPassant);

    // En passant captures black pawn (value 100), square e6 is undefended
    assert!(see_ge(mov, &board, 0));
    assert!(see_ge(mov, &board, 100));
    assert!(!see_ge(mov, &board, 101));
}

#[test]
fn test_see_promotion_capture() {
    // White pawn on e7 captures Black Rook on f8 promoting to Queen
    let board = Board::from_fen("5r2/4P3/8/8/8/8/8/4K2k w - - 0 1").unwrap();
    let mov = Move::new(Sq::E7, Sq::F8, MoveFlags::PromoCaptureQueen);

    // Gains Rook (500) + Promo bonus (900 - 100 = 800) = 1300
    assert!(see_ge(mov, &board, 1300));
    assert!(!see_ge(mov, &board, 1301));
}

#[test]
fn test_see_quiet_promotion() {
    // White pawn on e7 advances to e8 promoting to Queen (undefended, Black King on g7)
    let board = Board::from_fen("8/4P1k1/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let mov = Move::new(Sq::E7, Sq::E8, MoveFlags::PromoQueen);

    // Promo bonus: 900 - 100 = 800
    assert!(see_ge(mov, &board, 800));
    assert!(!see_ge(mov, &board, 801));
}

#[test]
fn test_see_king_cannot_capture_defended_piece() {
    // White King on e2, Black Rook on e3 defended by Black Rook on e8
    let board = Board::from_fen("4r1k1/8/8/8/8/4r3/4K3/8 w - - 0 1").unwrap();
    let mov = Move::new(Sq::E2, Sq::E3, MoveFlags::Capture);

    // King cannot capture defended rook (would be in check)
    assert!(!see_ge(mov, &board, 0));
}

#[test]
fn test_see_quiet_move_to_attacked_square() {
    // White Knight on c3 moves to d5, which is attacked by Black Pawn on e6
    let board = Board::from_fen("4k3/8/4p3/8/8/2N5/8/4K3 w - - 0 1").unwrap();
    let bad_quiet = Move::new(Sq::C3, Sq::D5, MoveFlags::Quiet);
    let good_quiet = Move::new(Sq::C3, Sq::B5, MoveFlags::Quiet);

    // Nd5 hangs the knight to the pawn: loses 300
    assert!(!see_ge(bad_quiet, &board, 0));
    assert!(see_ge(bad_quiet, &board, -300));

    // Nb5 is safe: gains 0
    assert!(see_ge(good_quiet, &board, 0));
    assert!(!see_ge(good_quiet, &board, 1));
}
