use crate::board::Board;
use crate::search::search;
use crate::time::{Instant, TimeManager};
use crate::transposition::TranspositionTable;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Standard benchmark positions covering openings, middlegames, endgames, and tactical lines.
pub const BENCH_POSITIONS: &[&str] = &[
    // 1. Standard start position
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    // 2. Kiwipete (tactical complex with king in center, pins, open files)
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    // 3. Silver Suite / Active endgame
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    // 4. Perft Pos 4 (promotions, king attack, checks)
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    // 5. Perft Pos 5 (pinned king, knight discovery)
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    // 6. Perft Pos 6 (complex middlegame with bishops)
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    // 7. French Defense middlegame with knight attacks
    "4rrk1/pp1n3p/3q2pQ/2p1pb2/2PP4/2P3N1/P2B2PP/4RRK1 b - - 7 19",
    // 8. Kingside attack with queen and minor pieces
    "r1bq1r1k/1pp1n1pp/1p1p4/4p2Q/4Pp2/1BNP4/PPP2PPP/3R1RK1 w - - 2 14",
    // 9. Asymmetric castling with queens active
    "r3r1k1/2p2ppp/p1p1bn2/8/1q2P3/2NPQN2/PPP3PP/R4RK1 b - - 2 15",
    // 10. Sharp Sicilian defense with centralized knights
    "2rqkb1r/ppp2p2/2npb1p1/1N1Nn2p/2P1PP2/8/PP2B1PP/R1BQK2R b KQ - 0 11",
    // 11. Classical Ruy Lopez middlegame
    "r1bq1r1k/b1p1npp1/p2p3p/1p6/3PP3/1B2NN2/PP3PPP/R2Q1RK1 w - - 1 16",
    // 12. French structure with locked center
    "r1q2rk1/2p1bppp/2Pp4/p6b/Q1PNp3/4B3/PP1R1PPP/2K4R w - - 2 18",
    // 13. Double rook endgame
    "4k2r/1pb2ppp/1p2p3/1R1p4/3P4/2r1PN2/P4PPP/1R4K1 b - - 3 22",
    // 14. Queen & bishop diagonal pressure
    "3q2k1/pb3p1p/4pbp1/2r5/PpN2N2/1P2P2P/5PP1/Q2R2K1 b - - 4 26",
    // 15. Knight vs knight endgame
    "6k1/6p1/6Pp/ppp5/3pn2P/1P3K2/1PP2P2/3N4 b - - 0 1",
    // 16. Pure pawn endgame with pawn breaks
    "8/2p5/8/2kPKp1p/2p4P/2P5/3P4/8 w - - 0 1",
    // 17. Pawn race endgame
    "8/1p3pp1/7p/5P1P/2k3P1/8/2K2P2/8 w - - 0 1",
    // 18. Double rook endgame with passed pawns
    "8/pp2r1k1/2p1p3/3pP2p/1P1P1P1P/P5KR/8/8 w - - 0 1",
    // 19. Opposite-colored bishops endgame
    "8/3b4/p1bk3p/Pp6/1Kp1PpPp/2P2P1P/2P5/5B2 b - - 0 1",
    // 20. Tactical rook & pawn endgame
    "5k2/7R/4P2p/5K2/p1r2P1p/8/8/8 b - - 0 1",
    // 21. Passed pawn knight defense
    "6k1/6p1/P6p/r1N5/5p2/7P/1b3PP1/4R1K1 w - - 0 1",
    // 22. Queen & minor pieces tactical scramble
    "1r3k2/4q3/2Pp3b/3Bp3/2Q2p2/1p1P2P1/1P2KP2/3N4 w - - 0 1",
    // 23. Minor pieces & rook endgame
    "6k1/4pp1p/3p2p1/P1pPb3/R7/1r2P1PP/3B1P2/6K1 w - - 0 1",
    // 24. Bishop endgame with passed a-pawn
    "8/3p3B/5p2/5P2/p7/PP5b/k7/6K1 w - - 0 1",
    // 25. Closed King's Indian defense
    "r1bq1rk1/ppp1npbp/3p2p1/3Pp3/2P1P3/2N1B3/PP2BPPP/R2Q1RK1 w - - 0 11",
    // 26. Caro-Kann advance variation
    "r2qkbnr/pp1b1ppp/2n1p3/1B1pP3/3P4/5N2/PP3PPP/RNBQK2R b KQkq - 0 7",
    // 27. Queen vs two rooks
    "3r1rk1/p5pp/bpp1pp2/8/q1PP1P2/b3P3/P2NQRPP/1R2B1K1 b - - 6 22",
    // 28. Queen endgame
    "8/6pk/1p6/8/PP3p1p/5P2/4KP1q/3Q4 w - - 0 1",
    // 29. Bishop pair attacking kingside
    "4r1k1/r1q2ppp/ppp2n2/4P3/5Rb1/1N1BQ3/PPP3PP/R5K1 w - - 1 17",
    // 30. Knight outpost vs bad bishop
    "r1bbk1nr/pp3p1p/2n5/1N4p1/2Np1B2/8/PPP2PPP/2KR1B1R w kq - 0 13",
    // 31. Queenless middlegame with queenside play
    "r3k2r/3nnpbp/q2pp1p1/p7/Pp1PPPP1/4BNN1/1P5P/R2Q1RK1 w kq - 0 16",
    // 32. Multi-piece center pin
    "4rrk1/1p1nq3/p7/2p1P1pp/3P2bp/3Q1Bn1/PPPB4/1K2R1NR w - - 40 21",
];

pub fn run_bench(depth: u8, tt_mb: usize, mut output_cb: impl FnMut(String)) -> u64 {
    let tt = TranspositionTable::new(tt_mb);
    let stop_requested = Arc::new(AtomicBool::new(false));

    let mut total_nodes = 0u64;
    let start_total = Instant::now();

    for (i, &fen) in BENCH_POSITIONS.iter().enumerate() {
        let Some(board) = Board::from_fen(fen) else {
            continue;
        };

        tt.clear();
        let tm = TimeManager::from_depth(depth);
        let pos_start = Instant::now();

        let mut nodes = 0u64;
        let _best = search(board, tm, stop_requested.clone(), &tt, |line| {
            if let Some(idx) = line.find("nodes ") {
                if let Some(token) = line[idx + 6..].split_whitespace().next() {
                    if let Ok(n) = token.parse::<u64>() {
                        nodes = n;
                    }
                }
            }
        });

        let pos_elapsed = pos_start.elapsed().as_millis().max(1);
        let pos_nps = (nodes as u128 * 1000 / pos_elapsed) as u64;
        total_nodes += nodes;

        output_cb(format!(
            "Position {:2}/{}: {:8} nodes  {:5} ms  {:8} nps",
            i + 1,
            BENCH_POSITIONS.len(),
            nodes,
            pos_elapsed,
            pos_nps
        ));
    }

    let total_time_ms = start_total.elapsed().as_millis().max(1);
    let nps = (total_nodes as u128 * 1000 / total_time_ms) as u64;

    output_cb("===========================".to_string());
    output_cb(format!("Total time (ms) : {total_time_ms}"));
    output_cb(format!("Nodes searched  : {total_nodes}"));
    output_cb(format!("Nodes/second    : {nps}"));

    total_nodes
}
