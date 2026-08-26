use chess_base::prelude::*;

use crate::{
    board::Board,
    move_gen::{Black, Evasions, MoveList, NonEvasions, White, generate_moves},
};

pub fn perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let us = board.to_play;

    let in_check = board.checkers != 0;
    let mut moves = MoveList::default();
    let ptr = match (us, in_check) {
        (Color::White, true) => generate_moves::<White, Evasions>(board, moves.as_ptr()),
        (Color::White, false) => generate_moves::<White, NonEvasions>(board, moves.as_ptr()),
        (Color::Black, true) => generate_moves::<Black, Evasions>(board, moves.as_ptr()),
        (Color::Black, false) => generate_moves::<Black, NonEvasions>(board, moves.as_ptr()),
    };
    moves.update_size(ptr);

    let mut nodes = 0;
    for mov in moves.as_slice() {
        if !board.legal(*mov) {
            continue;
        }

        if depth == 1 {
            nodes += 1;
            continue;
        }

        let undo_info = board.make_move(*mov);
        nodes += perft(board, depth - 1);
        board.undo_move(*mov, undo_info);
    }

    nodes
}

pub fn run_perft(name: &str, fen: &str, expected: &[u64]) {
    use std::time::Instant;
    println!("--- {} ---", name);
    // Use start_pos() if fen is empty for the standard start position
    let mut board = if fen.is_empty() {
        Board::start_pos()
    } else {
        Board::from_fen(fen).unwrap()
    };

    for (i, &expected_nodes) in expected.iter().enumerate() {
        let depth = (i + 1) as u8;
        let instant = Instant::now();
        let nodes = perft(&mut board, depth);
        let elapsed = instant.elapsed();

        println!("Depth {}: {} nodes - {:?}", depth, nodes, elapsed);
        assert_eq!(nodes, expected_nodes, "Perft failed at depth {}", depth);
    }
    println!();
}

pub fn perft_start(max_depth: usize) {
    run_perft(
        "Start Position",
        "",
        &[
            20,
            400,
            8902,
            197281,
            4865609,
            119060324,
            3195901860,
            84998978956,
            2439530234167,
            69352859712417,
            2097651003696806,
            62854969236701747,
            1981066775000396239,
        ][..max_depth],
    );
}

pub fn perft_kiwipete(max_depth: usize) {
    run_perft(
        "Kiwipete (Position 2)",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        &[48, 2039, 97862, 4085603, 193690690, 8031647685][..max_depth],
    );
}

pub fn perft_pos3(max_depth: usize) {
    run_perft(
        "Position 3",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[
            14, 191, 2812, 43238, 674624, 11030083, 178633661, 3009794393,
        ][..max_depth],
    );
}

pub fn perft_pos4(max_depth: usize) {
    run_perft(
        "Position 4",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9467, 422333, 15833292, 706045033][..max_depth],
    );
}

pub fn perft_pos4_mirrored(max_depth: usize) {
    run_perft(
        "Position 4 (Mirrored)",
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        &[6, 264, 9467, 422333, 15833292, 706045033][..max_depth],
    );
}

pub fn perft_pos5(max_depth: usize) {
    run_perft(
        "Position 5",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1486, 62379, 2103487, 89941194][..max_depth],
    );
}

pub fn perft_pos6(max_depth: usize) {
    run_perft(
        "Position 6",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        &[
            46,
            2079,
            89890,
            3894594,
            164075551,
            6923051137,
            287188994746,
            11923589843526,
            490154852788714,
        ][..max_depth],
    );
}
