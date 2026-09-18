use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::search::search;
use chess_engine::time::TimeManager;
use chess_engine::transposition::TranspositionTable;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use uci_parser::messages::UciSearchOptions;

#[test]
fn test_time_manager_from_depth() {
    let tm = TimeManager::from_depth(5);
    assert_eq!(tm.limits.max_depth, 5);
    assert!(tm.limits.optimum_time.is_none());
    assert!(tm.limits.max_time.is_none());
    assert!(!tm.limits.infinite);
}

#[test]
fn test_time_manager_clock_allocation_white_and_black() {
    let opts = UciSearchOptions {
        wtime: Some(Duration::from_millis(60_000)),
        winc: Some(Duration::from_millis(1_000)),
        btime: Some(Duration::from_millis(30_000)),
        binc: Some(Duration::from_millis(500)),
        ..Default::default()
    };

    let tm_w = TimeManager::from_uci_options(
        &opts,
        Color::White,
        chess_engine::time::DEFAULT_MOVE_OVERHEAD_MS,
    );
    let tm_b = TimeManager::from_uci_options(
        &opts,
        Color::Black,
        chess_engine::time::DEFAULT_MOVE_OVERHEAD_MS,
    );

    assert!(
        tm_w.limits.optimum_time.unwrap().as_millis()
            > tm_b.limits.optimum_time.unwrap().as_millis()
    );
    assert!(tm_w.limits.max_time.unwrap() > tm_w.limits.optimum_time.unwrap());
}

#[test]
fn test_time_manager_movestogo() {
    let opts = UciSearchOptions {
        wtime: Some(Duration::from_millis(60_000)),
        movestogo: Some(10),
        ..Default::default()
    };

    let tm = TimeManager::from_uci_options(
        &opts,
        Color::White,
        chess_engine::time::DEFAULT_MOVE_OVERHEAD_MS,
    );
    let opt_ms = tm.limits.optimum_time.unwrap().as_millis();

    // 60s / 10 moves ≈ 6s per move
    assert!((5_000..=6_500).contains(&opt_ms));
}

#[test]
fn test_time_manager_panic_mode_low_time() {
    let opts = UciSearchOptions {
        wtime: Some(Duration::from_millis(50)), // 50ms left
        winc: Some(Duration::from_millis(0)),
        ..Default::default()
    };

    let tm = TimeManager::from_uci_options(
        &opts,
        Color::White,
        chess_engine::time::DEFAULT_MOVE_OVERHEAD_MS,
    );
    assert!(tm.limits.optimum_time.is_some());
    assert!(tm.limits.max_time.is_some());
    assert!(tm.limits.optimum_time.unwrap() <= Duration::from_millis(50));
    assert!(tm.limits.max_time.unwrap() <= Duration::from_millis(50));
}

#[test]
fn test_time_manager_move_overhead() {
    let opts = UciSearchOptions {
        wtime: Some(Duration::from_millis(10_000)),
        movestogo: Some(20),
        ..Default::default()
    };

    let tm_low_overhead = TimeManager::from_uci_options(&opts, Color::White, 10);
    let tm_high_overhead = TimeManager::from_uci_options(&opts, Color::White, 500);

    assert!(
        tm_low_overhead.limits.optimum_time.unwrap()
            > tm_high_overhead.limits.optimum_time.unwrap()
    );
    assert!(tm_low_overhead.limits.max_time.unwrap() > tm_high_overhead.limits.max_time.unwrap());

    // movetime with overhead
    let movetime_opts = UciSearchOptions {
        movetime: Some(Duration::from_millis(100)),
        ..Default::default()
    };
    let tm_movetime = TimeManager::from_uci_options(&movetime_opts, Color::White, 30);
    assert_eq!(
        tm_movetime.limits.optimum_time,
        Some(Duration::from_millis(70))
    );
    assert_eq!(tm_movetime.limits.max_time, Some(Duration::from_millis(70)));
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_search_movetime_limit() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let tm = TimeManager::from_movetime(Duration::from_millis(50));
    let start = chess_engine::time::Instant::now();
    let best_move = search(board.clone(), &[], tm, stop_requested, &tt, |_| {});

    let elapsed = start.elapsed();
    assert!(!best_move.is_none());
    assert!(board.legal(best_move));
    assert!(elapsed <= Duration::from_millis(250)); // Well bounded
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_tt_not_polluted_when_stopped() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(true)); // Pre-stopped
    let tt = TranspositionTable::with_buckets(16);

    let tm = TimeManager::from_depth(10);
    let _ = search(board, &[], tm, stop_requested, &tt, |_| {});

    // Ensure TT is not written to when stopped
    assert_eq!(tt.hashfull(), 0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_search_nodes_limit() {
    let board = Board::start_pos();
    let stop_requested = Arc::new(AtomicBool::new(false));
    let tt = TranspositionTable::with_buckets(16);

    let tm = TimeManager::from_nodes(5000);
    let best_move = search(board.clone(), &[], tm, stop_requested, &tt, |_| {});

    assert!(!best_move.is_none());
    assert!(board.legal(best_move));
}
