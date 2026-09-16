use chess_engine::board::Board;
use chess_engine::perft::perft;

#[test]
fn test_perft_start_position() {
    let mut board = Board::start_pos();
    let depth = if cfg!(miri) { 2 } else { 5 };
    let expected = [20, 400, 8902, 197281, 4865609];
    assert_eq!(perft(&mut board, depth as u8), expected[depth - 1]);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_kiwipete() {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .expect("Valid Kiwipete FEN");
    assert_eq!(perft(&mut board, 4), 4_085_603);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_3() {
    let mut board =
        Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("Valid Position 3 FEN");
    assert_eq!(perft(&mut board, 5), 674_624);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_4() {
    let mut board =
        Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
            .expect("Valid Position 4 FEN");
    assert_eq!(perft(&mut board, 4), 422_333);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_4_mirrored() {
    let mut board =
        Board::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
            .expect("Valid Position 4 Mirrored FEN");
    assert_eq!(perft(&mut board, 4), 422_333);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_5() {
    let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
        .expect("Valid Position 5 FEN");
    assert_eq!(perft(&mut board, 4), 2_103_487);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_position_6() {
    let mut board =
        Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .expect("Valid Position 6 FEN");
    assert_eq!(perft(&mut board, 4), 3_894_594);
}
