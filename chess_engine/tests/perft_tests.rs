use chess_engine::board::Board;
use chess_engine::perft::perft;

const PERFT_SUITE: &[(&str, &[u64])] = &[
    (
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &[20, 400, 8902, 197281, 4865609],
    ),
    (
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        &[48, 2039, 97862, 4085603],
    ),
    (
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[14, 191, 2812, 43238, 674624],
    ),
    (
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9467, 422333],
    ),
    (
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        &[6, 264, 9467, 422333],
    ),
    (
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1486, 62379, 2103487],
    ),
    (
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        &[46, 2079, 89890, 3894594],
    ),
];

#[test]
#[cfg_attr(miri, ignore)]
fn test_perft_suite() {
    for (fen, expected) in PERFT_SUITE {
        let mut board = Board::from_fen(fen).unwrap();
        let depth = expected.len() as u8;
        let nodes = perft(&mut board, depth);
        assert_eq!(
            nodes,
            expected[depth as usize - 1],
            "perft({depth}) failed for {fen}: got {nodes}, expected {}",
            expected[depth as usize - 1]
        );
    }
}
