use chess_engine::board::Board;
use chess_engine::perft::perft;
use std::time::Instant;

pub struct PerftSuiteEntry {
    pub name: &'static str,
    pub fen: &'static str,
    pub default_depth: u8,
    pub expected: &'static [u64],
}

pub const PERFT_SUITE: &[PerftSuiteEntry] = &[
    PerftSuiteEntry {
        name: "Start Position",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        default_depth: 5,
        expected: &[
            20,
            400,
            8902,
            197281,
            4865609,
            119060324,
            3195901860,
            84998978956,
        ],
    },
    PerftSuiteEntry {
        name: "Kiwipete (Position 2)",
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        default_depth: 4,
        expected: &[48, 2039, 97862, 4085603, 193690690, 8031647685],
    },
    PerftSuiteEntry {
        name: "Position 3",
        fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        default_depth: 5,
        expected: &[
            14, 191, 2812, 43238, 674624, 11030083, 178633661, 3009794393,
        ],
    },
    PerftSuiteEntry {
        name: "Position 4",
        fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        default_depth: 4,
        expected: &[6, 264, 9467, 422333, 15833292, 706045033],
    },
    PerftSuiteEntry {
        name: "Position 4 (Mirrored)",
        fen: "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        default_depth: 4,
        expected: &[6, 264, 9467, 422333, 15833292, 706045033],
    },
    PerftSuiteEntry {
        name: "Position 5",
        fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        default_depth: 4,
        expected: &[44, 1486, 62379, 2103487, 89941194],
    },
    PerftSuiteEntry {
        name: "Position 6",
        fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        default_depth: 4,
        expected: &[
            46,
            2079,
            89890,
            3894594,
            164075551,
            6923051137,
            287188994746,
        ],
    },
];

pub fn run_perft_suite(target_depth: Option<u8>) {
    let suite_start = Instant::now();
    let mut total_nodes = 0u64;
    let mut all_passed = true;

    println!("============================================================");
    println!(" Lucky Chess — Perft Verification Suite");
    println!("============================================================");

    for entry in PERFT_SUITE {
        let max_d = target_depth
            .unwrap_or(entry.default_depth)
            .min(entry.expected.len() as u8);
        println!("\n--- {} ---", entry.name);
        println!("FEN: {}", entry.fen);

        let mut board = Board::from_fen(entry.fen).expect("Valid FEN in test suite");

        for d in 1..=max_d {
            let start = Instant::now();
            let nodes = perft(&mut board, d);
            let elapsed = start.elapsed();
            let expected_nodes = entry.expected[d as usize - 1];

            let elapsed_ms = elapsed.as_millis().max(1);
            let nps = (nodes as u128 * 1000 / elapsed_ms) as u64;

            let ok = nodes == expected_nodes;
            if !ok {
                all_passed = false;
            }

            let status = if ok { "OK" } else { "FAIL" };
            println!(
                "Depth {:2}: {:12} nodes  {:6.2?}  {:8.1} MNPS  [{}]",
                d,
                nodes,
                elapsed,
                nps as f64 / 1_000_000.0,
                status
            );

            if !ok {
                eprintln!("  Expected: {expected_nodes}, got: {nodes}");
                break;
            }

            if d == max_d {
                total_nodes += nodes;
            }
        }
    }

    let suite_elapsed = suite_start.elapsed();
    let total_ms = suite_elapsed.as_millis().max(1);
    let total_nps = (total_nodes as u128 * 1000 / total_ms) as u64;

    println!("\n============================================================");
    println!("Total Time  : {:?}", suite_elapsed);
    println!("Leaf Nodes  : {total_nodes}");
    println!("Throughput  : {:.2} MNPS", total_nps as f64 / 1_000_000.0);
    println!(
        "Suite Status: {}",
        if all_passed { "PASSED" } else { "FAILED" }
    );
    println!("============================================================");
}
