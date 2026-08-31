use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::search::{HistoryTable, search};
use chess_engine::time::TimeManager;
use chess_engine::transposition::{TTEntry, TTFlag, TranspositionTable};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[test]
fn test_history_table_operations() {
    let mut history = HistoryTable::default();

    // Verify initial values are zero
    assert_eq!(history.get(Color::White, Sq::E2, Sq::E4), 0);
    assert_eq!(history.get(Color::Black, Sq::E7, Sq::E5), 0);

    // Apply updates
    history.update(Color::White, Sq::E2, Sq::E4, 3);
    let val1 = history.get(Color::White, Sq::E2, Sq::E4);
    assert!(val1 > 0);

    // Apply multiple updates and verify gravity damping (stays bounded <= 10_000)
    for _ in 0..100 {
        history.update(Color::White, Sq::E2, Sq::E4, 8);
    }
    let bounded_val = history.get(Color::White, Sq::E2, Sq::E4);
    assert!(bounded_val <= 10_000);
    assert!(bounded_val > 0);

    // Verify clear resets everything
    history.clear();
    assert_eq!(history.get(Color::White, Sq::E2, Sq::E4), 0);
}

#[test]
fn test_search_start_pos_depth_1_and_2() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board.clone(),
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert!(!best_move.is_none());
    assert!(board.legal(best_move));
    assert_eq!(info_lines.len(), 2);
    assert!(info_lines[0].starts_with("info depth 1"));
    assert!(info_lines[1].starts_with("info depth 2"));
    assert!(info_lines[1].contains("score cp"));
}

#[test]
fn test_search_mate_in_1_white_scholars() {
    // Scholar's mate: Qxf7#
    let board =
        Board::from_fen("r1bqkb1r/pppp1ppp/2n5/4p3/2B1n3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 0 1")
            .unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board.clone(),
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert_eq!(best_move.from(), Sq::F3);
    assert_eq!(best_move.to(), Sq::F7);
    assert!(best_move.is_capture());
    assert!(info_lines.iter().any(|line| line.contains("score mate 1")));
}

#[test]
fn test_search_mate_in_1_black_fools() {
    // Black plays Qh4#
    let board =
        Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board.clone(),
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert_eq!(best_move.from(), Sq::D8);
    assert_eq!(best_move.to(), Sq::H4);
    assert!(info_lines.iter().any(|line| line.contains("score mate 1")));
}

#[test]
fn test_search_already_checkmated_terminal() {
    // White is checkmated by Qh4# (0 legal moves and in check)
    let board =
        Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 2").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert!(best_move.is_none());
    assert!(info_lines.iter().any(|line| line.contains("score mate")));
}

#[test]
fn test_search_stalemate_terminal() {
    // Black King on a8, White Queen on b6, White King on a1. Black to move, 0 legal moves, not in check.
    let board = Board::from_fen("k7/8/1Q6/8/8/8/8/K7 b - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert!(best_move.is_none());
    assert!(info_lines.iter().any(|line| line.contains("score cp 0")));
}

#[test]
fn test_search_draw_fifty_move_rule() {
    // 50-move rule triggered (halfmove clock = 100)
    let board = Board::from_fen("k7/8/8/8/8/8/8/K6R w - - 100 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let _best_move = search(
        board,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert!(info_lines.iter().any(|line| line.contains("score cp 0")));
}

#[test]
fn test_search_draw_insufficient_material() {
    // King vs King
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let _best_move = search(
        board,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert!(info_lines.iter().any(|line| line.contains("score cp 0")));
}

#[test]
fn test_search_stop_requested_before_search() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(true)); // Already stopped
    let tt = TranspositionTable::with_buckets(16);

    let mut count = 0;
    let _ = search(
        board,
        TimeManager::from_depth(5),
        stop_requested,
        &tt,
        |_| count += 1,
    );

    // Since stop was set immediately, 0 depths should complete
    assert_eq!(count, 0);
}

#[test]
fn test_search_stop_requested_during_search() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let stop_clone = Arc::clone(&stop_requested);
    let completed_depths = Arc::new(AtomicUsize::new(0));
    let completed_clone = Arc::clone(&completed_depths);

    let _ = search(
        board,
        TimeManager::from_depth(10),
        stop_requested,
        &tt,
        move |_line| {
            completed_clone.fetch_add(1, Ordering::Relaxed);
            // Abort after depth 1
            stop_clone.store(true, Ordering::Relaxed);
        },
    );

    assert_eq!(completed_depths.load(Ordering::Relaxed), 1);
}

#[test]
fn test_search_transposition_table_reuse_and_cutoff() {
    let board = Board::from_fen("4k3/8/8/8/8/2N5/1Q6/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    // First search populates TT
    let mov1 = search(
        board.clone(),
        TimeManager::from_depth(2),
        Arc::clone(&stop_requested),
        &tt,
        |_| {},
    );
    assert!(!mov1.is_none());

    // Second search on the same position reuses TT and hits cutoffs
    let mut lines = Vec::new();
    let mov2 = search(
        board,
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |info| lines.push(info),
    );
    assert_eq!(mov1, mov2);
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_search_transposition_table_manual_exact_cutoff() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    // Pre-seed an exact entry with depth 4
    let best_m = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
    let entry = TTEntry::new(best_m, 120, 10, 4, TTFlag::Exact);
    tt.store(board.hash, entry, 0);

    let mut lines = Vec::new();
    let chosen = search(
        board,
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |info| lines.push(info),
    );
    // Since depth 4 exact entry is present, depth 1 and 2 probe and cutoff immediately
    assert_eq!(chosen, best_m);
}

#[test]
fn test_search_null_move_pruning_and_zugzwang_skip() {
    // 1. Position with non-pawn material (Queen + Knight vs lone King, NMP triggers)
    let board_nmp = Board::from_fen("4k3/8/8/8/8/2N5/1Q6/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov = search(
        board_nmp,
        TimeManager::from_depth(3),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov.is_none());

    // 2. Pawn endgame (only pawns, has_non_pawn_material is false -> NMP skipped to avoid zugzwang UB/bugs)
    let board_pawn = Board::from_fen("8/8/8/4k3/4P3/8/4K3/8 w - - 0 1").unwrap();
    let stop_requested2 = Arc::new(AtomicBool::new(false));
    let tt2 = TranspositionTable::with_buckets(16);
    let mov2 = search(
        board_pawn,
        TimeManager::from_depth(2),
        stop_requested2,
        &tt2,
        |_| {},
    );
    assert!(!mov2.is_none());
}

#[test]
fn test_search_reverse_futility_pruning() {
    // White is up a massive amount of material; non-PV nodes will trigger RFP
    let board = Board::from_fen("8/8/8/8/8/5k2/7Q/4K2R w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov = search(
        board,
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov.is_none());
}

#[test]
fn test_search_pvs_and_killer_moves() {
    // Compact tactical position with 5 pieces: White Rook + Knight vs Black Queen
    let board = Board::from_fen("4k3/8/8/3q4/8/2N5/1R6/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov = search(
        board,
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov.is_none());
}

#[test]
fn test_search_quiescence_captures_and_promotions() {
    // 1. Capture chain in qsearch
    let board_caps = Board::from_fen("6k1/8/8/3q4/4R3/8/8/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov = search(
        board_caps,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov.is_none());

    // 2. Promotion in qsearch
    let board_promo = Board::from_fen("8/4P3/8/8/8/8/k7/4K3 w - - 0 1").unwrap();
    let stop_requested2 = Arc::new(AtomicBool::new(false));
    let tt2 = TranspositionTable::with_buckets(16);
    let promo_mov = search(
        board_promo,
        TimeManager::from_depth(1),
        stop_requested2,
        &tt2,
        |_| {},
    );
    assert_eq!(promo_mov.from(), Sq::E7);
    assert_eq!(promo_mov.to(), Sq::E8);
    assert!(promo_mov.is_promotion());
}

#[test]
fn test_search_quiescence_in_check_evasions() {
    // White is in check in qsearch -> stand-pat disabled, must find evasion
    let board_in_check = Board::from_fen("4k3/8/8/8/7q/5P2/4P3/4K3 w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov = search(
        board_in_check,
        TimeManager::from_depth(1),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov.is_none());
}

#[test]
fn test_search_aspiration_window_depth_5() {
    // Search at depth 5 activates aspiration windows (ASPIRATION_MIN_DEPTH = 5)
    // using a minimal branching mate-in-2 position for lightning fast execution
    let board = Board::from_fen("k7/8/K7/8/8/8/7p/7R w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board,
        TimeManager::from_depth(5),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert_eq!(best_move.from(), Sq::H1);
    assert_eq!(best_move.to(), Sq::H2);
    assert_eq!(info_lines.len(), 5);
    assert!(info_lines[4].starts_with("info depth 5"));
}

#[test]
fn test_search_aspiration_fail_high_and_low_recovery() {
    // Mate-in-2 position where mate score fluctuations trigger aspiration window widening
    let board = Board::from_fen("k7/8/K7/8/8/8/7p/7R w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let mut info_lines = Vec::new();
    let best_move = search(
        board,
        TimeManager::from_depth(5),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );

    assert_eq!(best_move.from(), Sq::H1);
    assert_eq!(best_move.to(), Sq::H2);
    assert_eq!(info_lines.len(), 5);
}

#[test]
fn test_search_non_zero_root_ply() {
    // Board with high ply count to verify relative ply arithmetic: `self.board.ply - self.root_ply`
    let board = Board::from_fen("4k3/8/8/8/4P3/5N2/8/4K3 w - - 4 5").unwrap();
    assert_eq!(board.ply, 8);

    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let best_move = search(
        board.clone(),
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!best_move.is_none());
    assert!(board.legal(best_move));
}

#[test]
fn test_search_black_to_move_endgame() {
    // Black to move with a passed pawn
    let board = Board::from_fen("8/4k3/8/8/8/8/4p3/4K3 b - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let best_move = search(
        board.clone(),
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!best_move.is_none());
    assert!(board.legal(best_move));
}

#[test]
fn test_search_castling_and_en_passant() {
    // 1. Castling availability during search
    let board_castle = Board::from_fen("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);
    let mov1 = search(
        board_castle.clone(),
        TimeManager::from_depth(2),
        stop_requested,
        &tt,
        |_| {},
    );
    assert!(!mov1.is_none());
    assert!(board_castle.legal(mov1));

    // 2. En passant capture availability during search
    let board_ep = Board::from_fen("4k3/8/8/3Pp3/8/8/8/4K3 w - e6 0 1").unwrap();
    let stop_requested2 = Arc::new(AtomicBool::new(false));
    let tt2 = TranspositionTable::with_buckets(16);
    let mov2 = search(
        board_ep.clone(),
        TimeManager::from_depth(2),
        stop_requested2,
        &tt2,
        |_| {},
    );
    assert!(!mov2.is_none());
    assert!(board_ep.legal(mov2));
}

#[test]
fn test_search_aspiration_fail_low_widening() {
    // Force fail-low in aspiration search by pre-seeding high score at root
    let board = Board::from_fen("k7/8/K7/8/8/8/7p/7R w - - 0 1").unwrap();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    // Seed depth 4 with artificially high score (+2000) so depth 5 fails low initially
    let entry = TTEntry::new(Move::NONE, 2000, 100, 4, TTFlag::LowerBound);
    tt.store(board.hash, entry, 0);

    let mut info_lines = Vec::new();
    let best_move = search(
        board,
        TimeManager::from_depth(5),
        stop_requested,
        &tt,
        |info| info_lines.push(info),
    );
    assert_eq!(best_move.from(), Sq::H1);
    assert_eq!(best_move.to(), Sq::H2);
}
